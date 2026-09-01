use anyhow::{bail, Context, Result};
use std::{
    collections::VecDeque,
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::{Duration, Instant},
};
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    http::StatusCode,
    Message, WebSocket,
};

const POLL_DELAY: Duration = Duration::from_millis(10);
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(2);

pub struct CommandRequest {
    pub command: String,
    pub deadline: Instant,
}

pub enum CommandDelivery {
    Sent,
    Unavailable(String),
}

pub struct CommandBridge {
    pub requests: Sender<CommandRequest>,
    pub deliveries: Receiver<CommandDelivery>,
}

pub fn spawn(endpoint: &str, token: &str) -> Result<CommandBridge> {
    let address = endpoint
        .parse::<SocketAddr>()
        .context("HLAE bridge endpoint is invalid")?;
    if !address.ip().is_loopback() {
        bail!("HLAE bridge endpoint is not loopback-only");
    }
    if token.len() < 16 || !token.chars().all(|character| character.is_ascii_hexdigit()) {
        bail!("HLAE bridge token is invalid");
    }

    // Bind before returning so the Director never advertises a control worker
    // whose socket failed to start. HLAE is the WebSocket client and retries
    // its outward loopback connection automatically.
    let listener = TcpListener::bind(address)
        .with_context(|| format!("could not listen for HLAE at {address}"))?;
    listener.set_nonblocking(true)?;

    let (request_sender, request_receiver) = mpsc::channel::<CommandRequest>();
    let (delivery_sender, delivery_receiver) = mpsc::channel::<CommandDelivery>();
    let expected_path = format!("/{token}");
    thread::spawn(move || run(listener, expected_path, request_receiver, delivery_sender));
    Ok(CommandBridge {
        requests: request_sender,
        deliveries: delivery_receiver,
    })
}

fn run(
    listener: TcpListener,
    expected_path: String,
    requests: Receiver<CommandRequest>,
    deliveries: Sender<CommandDelivery>,
) {
    let mut socket = None::<WebSocket<TcpStream>>;
    let mut pending = VecDeque::<CommandRequest>::new();
    let mut requests_open = true;

    while requests_open || !pending.is_empty() {
        loop {
            match requests.try_recv() {
                Ok(request) => pending.push_back(request),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    requests_open = false;
                    break;
                }
            }
        }

        while pending
            .front()
            .is_some_and(|request| Instant::now() >= request.deadline)
        {
            pending.pop_front();
            if deliveries
                .send(CommandDelivery::Unavailable(
                    "HLAE did not connect to its command bridge before the timeout".into(),
                ))
                .is_err()
            {
                return;
            }
        }

        if socket.is_none() {
            match listener.accept() {
                Ok((stream, peer)) if peer.ip().is_loopback() => {
                    socket = accept_authenticated(stream, &expected_path).ok();
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => {
                    fail_pending(
                        &mut pending,
                        &deliveries,
                        format!("bridge accept failed: {error}"),
                    );
                    return;
                }
            }
        }

        let mut disconnected = false;
        if let Some(websocket) = socket.as_mut() {
            if let Some(request) = pending.front() {
                let message = Message::Binary(exec_message(&request.command).into());
                // `write` accepts the message into tungstenite's buffer. A
                // later `flush` drives non-blocking socket progress without
                // ever re-queuing the same keyframe edit after WouldBlock.
                match websocket.write(message) {
                    Ok(()) => {
                        pending.pop_front();
                        if deliveries.send(CommandDelivery::Sent).is_err() {
                            return;
                        }
                    }
                    Err(tungstenite::Error::WriteBufferFull(_)) => {}
                    Err(tungstenite::Error::Io(error))
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) =>
                    {
                        // Tungstenite retains the frame after a transient
                        // stream write failure; retrying the message would
                        // execute a destructive edit twice.
                        pending.pop_front();
                        if deliveries.send(CommandDelivery::Sent).is_err() {
                            return;
                        }
                    }
                    Err(_) => disconnected = true,
                }
            }

            if !disconnected {
                match websocket.flush() {
                    Ok(()) => {}
                    Err(tungstenite::Error::Io(error))
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) => {}
                    Err(_) => disconnected = true,
                }
            }

            if !disconnected {
                match websocket.read() {
                    Ok(message) if message.is_close() => disconnected = true,
                    Ok(_) => {
                        let _ = websocket.flush();
                    }
                    Err(tungstenite::Error::Io(error))
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                        ) => {}
                    Err(
                        tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed,
                    ) => disconnected = true,
                    Err(_) => disconnected = true,
                }
            }
        }
        if disconnected {
            socket = None;
        }
        thread::sleep(POLL_DELAY);
    }
}

fn accept_authenticated(stream: TcpStream, expected_path: &str) -> Result<WebSocket<TcpStream>> {
    stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT))?;
    stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT))?;
    let expected_path = expected_path.to_owned();
    let callback = move |request: &Request, response: Response| {
        if request.uri().path() == expected_path {
            Ok(response)
        } else {
            Err(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Some("invalid Director session token".into()))
                .expect("static WebSocket rejection response"))
        }
    };
    let mut websocket = accept_hdr(stream, callback).map_err(|error| anyhow::anyhow!("{error}"))?;
    websocket.get_mut().set_read_timeout(None)?;
    websocket.get_mut().set_write_timeout(None)?;
    websocket.get_mut().set_nonblocking(true)?;
    Ok(websocket)
}

fn exec_message(command: &str) -> Vec<u8> {
    let mut message = Vec::with_capacity(command.len() + 6);
    message.extend_from_slice(b"exec\0");
    message.extend_from_slice(command.as_bytes());
    message.push(0);
    message
}

fn fail_pending(
    pending: &mut VecDeque<CommandRequest>,
    deliveries: &Sender<CommandDelivery>,
    reason: String,
) {
    while pending.pop_front().is_some() {
        if deliveries
            .send(CommandDelivery::Unavailable(reason.clone()))
            .is_err()
        {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn hlae_exec_message_matches_the_documented_binary_protocol() {
        assert_eq!(
            exec_message("mirv_campath print"),
            b"exec\0mirv_campath print\0"
        );
    }

    #[test]
    fn bridge_rejects_non_loopback_endpoints_and_short_tokens() {
        assert!(spawn("192.168.1.10:32145", "0123456789abcdef").is_err());
        assert!(spawn("127.0.0.1:32145", "short").is_err());
    }

    #[test]
    fn authenticated_hlae_client_receives_exec_messages() {
        let probe = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let endpoint = probe.local_addr().unwrap();
        drop(probe);
        let token = "0123456789abcdef0123456789abcdef";
        let bridge = spawn(&endpoint.to_string(), token).unwrap();

        let unauthenticated = TcpStream::connect(endpoint).unwrap();
        assert!(tungstenite::client(
            format!("ws://{endpoint}/ffffffffffffffffffffffffffffffff"),
            unauthenticated,
        )
        .is_err());

        let stream = TcpStream::connect(endpoint).unwrap();
        let url = format!("ws://{endpoint}/{token}");
        let (mut hlae, _) = tungstenite::client(url, stream).unwrap();
        bridge
            .requests
            .send(CommandRequest {
                command: "mirv_campath print".into(),
                deadline: Instant::now() + Duration::from_secs(2),
            })
            .unwrap();

        assert!(matches!(
            bridge.deliveries.recv_timeout(Duration::from_secs(2)),
            Ok(CommandDelivery::Sent)
        ));
        assert_eq!(
            hlae.read().unwrap().into_data().as_ref(),
            b"exec\0mirv_campath print\0"
        );
    }
}
