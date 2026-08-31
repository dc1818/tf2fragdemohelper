use anyhow::{bail, Context, Result};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

const SERVERDATA_RESPONSE_VALUE: i32 = 0;
const SERVERDATA_EXECCOMMAND: i32 = 2;
const SERVERDATA_AUTH_RESPONSE: i32 = 2;
const SERVERDATA_AUTH: i32 = 3;
const MAX_PACKET_BYTES: usize = 4 * 1024 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_millis(750);
const IO_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Debug, PartialEq)]
struct Packet {
    id: i32,
    kind: i32,
    body: String,
}

#[derive(Debug, PartialEq)]
pub enum CommandDelivery {
    Confirmed(String),
    SentUnconfirmed(String),
}

pub fn execute_once(endpoint: &str, password: &str, command: &str) -> Result<CommandDelivery> {
    if command.is_empty() || command.len() > 16 * 1024 {
        bail!("Director command length is invalid");
    }
    if command.as_bytes().contains(&0) {
        bail!("Director command contains a null byte");
    }
    let address = endpoint
        .parse::<SocketAddr>()
        .context("Director RCON endpoint is invalid")?;
    if !address.ip().is_loopback() {
        bail!("Director RCON endpoint is not loopback-only");
    }

    let mut stream = TcpStream::connect_timeout(&address, CONNECT_TIMEOUT)
        .with_context(|| format!("could not connect to TF2 at {address}"))?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    authenticate(&mut stream, password)?;

    let request_id = 2;
    if let Err(error) = write_packet(&mut stream, request_id, SERVERDATA_EXECCOMMAND, command) {
        return Ok(CommandDelivery::SentUnconfirmed(format!(
            "connection failed while sending: {error}"
        )));
    }
    match read_response(&mut stream, request_id, SERVERDATA_RESPONSE_VALUE) {
        Ok(response) => Ok(CommandDelivery::Confirmed(response)),
        Err(error) => Ok(CommandDelivery::SentUnconfirmed(format!(
            "TF2 did not confirm the command: {error}"
        ))),
    }
}

fn authenticate(stream: &mut TcpStream, password: &str) -> Result<()> {
    if password.is_empty() || password.as_bytes().contains(&0) {
        bail!("Director RCON password is invalid");
    }
    let request_id = 1;
    write_packet(stream, request_id, SERVERDATA_AUTH, password)
        .context("could not send TF2 RCON authentication")?;
    let response = read_response(stream, request_id, SERVERDATA_AUTH_RESPONSE)
        .context("TF2 RCON authentication did not complete")?;
    if response == "__AUTH_FAILED__" {
        bail!("TF2 rejected the Director RCON password");
    }
    Ok(())
}

fn read_response(stream: &mut TcpStream, request_id: i32, expected_kind: i32) -> Result<String> {
    for _ in 0..16 {
        let packet = read_packet(stream)?;
        if packet.kind == expected_kind {
            if packet.id == -1 {
                return Ok("__AUTH_FAILED__".into());
            }
            if packet.id == request_id {
                return Ok(packet.body);
            }
        }
    }
    bail!("TF2 returned too many unrelated RCON packets")
}

fn write_packet(
    output: &mut impl Write,
    id: i32,
    kind: i32,
    body: &str,
) -> std::io::Result<()> {
    let packet_size = body.len().checked_add(10).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "RCON packet is too large")
    })?;
    let packet_size = i32::try_from(packet_size).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "RCON packet is too large")
    })?;
    output.write_all(&packet_size.to_le_bytes())?;
    output.write_all(&id.to_le_bytes())?;
    output.write_all(&kind.to_le_bytes())?;
    output.write_all(body.as_bytes())?;
    output.write_all(&[0, 0])?;
    output.flush()
}

fn read_packet(input: &mut impl Read) -> Result<Packet> {
    let mut size = [0_u8; 4];
    input.read_exact(&mut size)?;
    let size = i32::from_le_bytes(size);
    if size < 10 || usize::try_from(size).unwrap_or(usize::MAX) > MAX_PACKET_BYTES {
        bail!("TF2 returned an invalid RCON packet size");
    }
    let mut payload = vec![0_u8; size as usize];
    input.read_exact(&mut payload)?;
    if payload[payload.len() - 2..] != [0, 0] {
        bail!("TF2 returned a malformed RCON packet");
    }
    let id = i32::from_le_bytes(payload[0..4].try_into().unwrap());
    let kind = i32::from_le_bytes(payload[4..8].try_into().unwrap());
    let body = String::from_utf8_lossy(&payload[8..payload.len() - 2]).into_owned();
    Ok(Packet { id, kind, body })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Cursor, net::TcpListener, thread};

    #[test]
    fn source_rcon_packets_round_trip() {
        let mut bytes = Vec::new();
        write_packet(&mut bytes, 7, SERVERDATA_EXECCOMMAND, "mirv_campath print").unwrap();
        let packet = read_packet(&mut Cursor::new(bytes)).unwrap();
        assert_eq!(
            packet,
            Packet {
                id: 7,
                kind: SERVERDATA_EXECCOMMAND,
                body: "mirv_campath print".into(),
            }
        );
    }

    #[test]
    fn executes_one_command_over_a_fresh_loopback_connection() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let endpoint = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let auth = read_packet(&mut stream).unwrap();
            assert_eq!(auth.kind, SERVERDATA_AUTH);
            assert_eq!(auth.body, "0123456789abcdef");
            write_packet(&mut stream, auth.id, SERVERDATA_RESPONSE_VALUE, "").unwrap();
            write_packet(&mut stream, auth.id, SERVERDATA_AUTH_RESPONSE, "").unwrap();

            let command = read_packet(&mut stream).unwrap();
            assert_eq!(command.kind, SERVERDATA_EXECCOMMAND);
            assert_eq!(command.body, "mirv_campath print");
            write_packet(&mut stream, command.id, SERVERDATA_RESPONSE_VALUE, "ok").unwrap();
        });

        let delivery = execute_once(
            &endpoint.to_string(),
            "0123456789abcdef",
            "mirv_campath print",
        )
        .unwrap();
        assert_eq!(delivery, CommandDelivery::Confirmed("ok".into()));
        server.join().unwrap();
    }
}
