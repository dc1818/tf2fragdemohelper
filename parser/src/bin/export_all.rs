use bitbuffer::BitRead;
use main_error::MainError;
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tf_demo_parser::demo::header::Header;
use tf_demo_parser::demo::message::Message;
use tf_demo_parser::demo::packet::Packet;
use tf_demo_parser::demo::data::game_state::{GameState, Player, PlayerClassData, PlayerCondition, Projectile};
use tf_demo_parser::demo::parser::gamestateanalyser::GameStateAnalyser;
use tf_demo_parser::demo::parser::{DemoHandler, RawPacketStream};
use tf_demo_parser::Demo;

#[derive(Debug, Clone, Serialize, PartialEq)]
struct PlayerStateSample {
    entity_id: u32,
    user_id: Option<u16>,
    name: Option<String>,
    team: String,
    class: String,
    position: [f32; 3],
    velocity: [f32; 3],
    flags: Option<u32>,
    on_ground: Option<bool>,
    scoped: bool,
    shield_charging: bool,
    health: u16,
    max_health: u16,
    life_state: String,
    in_pvs: bool,
    weapon_handles: [i64; 3],
    medic_charge: Option<u8>,
    medigun: Option<String>,
}

impl PlayerStateSample {
    fn from_player(player: &Player) -> Self {
        let (medic_charge, medigun) = match &player.class_data {
            PlayerClassData::Medic { charge, medigun, .. } => {
                (Some(*charge), Some(format!("{medigun:?}").to_ascii_lowercase()))
            }
            _ => (None, None),
        };
        Self {
            entity_id: u32::from(player.entity),
            user_id: player.info.as_ref().map(|info| u16::from(info.user_id)),
            name: player.info.as_ref().map(|info| info.name.clone()),
            team: player.team.to_string(),
            class: player.class.to_string(),
            position: player.position.into(),
            velocity: player.velocity.into(),
            flags: player.flags_known.then_some(player.flags),
            on_ground: player.flags_known.then_some(player.flags & 1 != 0),
            scoped: player.has_condition(PlayerCondition::Zoomed),
            shield_charging: player.has_condition(PlayerCondition::ShieldCharge),
            health: player.health,
            max_health: player.max_health,
            life_state: format!("{:?}", player.state).to_ascii_lowercase(),
            in_pvs: player.in_pvs,
            weapon_handles: [
                player.weapons[0].0,
                player.weapons[1].0,
                player.weapons[2].0,
            ],
            medic_charge,
            medigun,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ProjectileStateSample {
    entity_id: u32,
    team: String,
    projectile_type: String,
    position: [f32; 3],
    initial_velocity: [f32; 3],
    launcher_handle: i64,
    critical: bool,
}

impl ProjectileStateSample {
    fn from_projectile(projectile: &Projectile) -> Self {
        Self {
            entity_id: u32::from(projectile.id),
            team: projectile.team.to_string(),
            projectile_type: format!("{:?}", projectile.ty).to_ascii_lowercase(),
            position: projectile.position.into(),
            initial_velocity: projectile.initial_speed.into(),
            launcher_handle: projectile.launcher.0,
            critical: projectile.critical,
        }
    }
}

#[derive(Default)]
struct StateDeltaWriter {
    players: BTreeMap<u32, PlayerStateSample>,
    projectiles: BTreeMap<u32, ProjectileStateSample>,
    sample_count: u64,
}

impl StateDeltaWriter {
    fn write(
        &mut self,
        state: &GameState,
        demo_tick: u32,
        server_tick: Option<u32>,
        packet_sequence: u64,
        out: &mut BufWriter<File>,
    ) -> Result<(), MainError> {
        let players: BTreeMap<u32, PlayerStateSample> = state
            .players
            .iter()
            .map(|player| {
                let sample = PlayerStateSample::from_player(player);
                (sample.entity_id, sample)
            })
            .collect();
        let projectiles: BTreeMap<u32, ProjectileStateSample> = state
            .projectiles
            .iter()
            .map(|(_, projectile)| {
                let sample = ProjectileStateSample::from_projectile(projectile);
                (sample.entity_id, sample)
            })
            .collect();
        let changed_players: Vec<&PlayerStateSample> = players
            .iter()
            .filter_map(|(id, sample)| (self.players.get(id) != Some(sample)).then_some(sample))
            .collect();
        let changed_projectiles: Vec<&ProjectileStateSample> = projectiles
            .iter()
            .filter_map(|(id, sample)| (self.projectiles.get(id) != Some(sample)).then_some(sample))
            .collect();
        let current_player_ids: BTreeSet<u32> = players.keys().copied().collect();
        let current_projectile_ids: BTreeSet<u32> = projectiles.keys().copied().collect();
        let removed_players: Vec<u32> = self
            .players
            .keys()
            .filter(|id| !current_player_ids.contains(id))
            .copied()
            .collect();
        let removed_projectiles: Vec<u32> = self
            .projectiles
            .keys()
            .filter(|id| !current_projectile_ids.contains(id))
            .copied()
            .collect();

        if changed_players.is_empty()
            && changed_projectiles.is_empty()
            && removed_players.is_empty()
            && removed_projectiles.is_empty()
        {
            return Ok(());
        }

        serde_json::to_writer(
            &mut *out,
            &json!({
                "demo_tick": demo_tick,
                "server_tick": server_tick,
                "tick_namespace": if server_tick.is_some() { "server" } else { "demo" },
                "packet_sequence": packet_sequence,
                "players": changed_players,
                "removed_players": removed_players,
                "projectiles": changed_projectiles,
                "removed_projectiles": removed_projectiles,
            }),
        )?;
        out.write_all(b"\n")?;
        self.players = players;
        self.projectiles = projectiles;
        self.sample_count += 1;
        Ok(())
    }
}

fn write_game_events(
    packet: &Packet<'_>,
    packet_sequence: u64,
    server_tick: Option<u32>,
    out: &mut BufWriter<File>,
) -> Result<(), MainError> {
    let messages = match packet {
        Packet::Signon(message_packet) | Packet::Message(message_packet) => &message_packet.messages,
        _ => return Ok(()),
    };

    for (event_index_in_packet, message) in messages.iter().enumerate() {
        if let Message::GameEvent(game_event) = message {
            serde_json::to_writer(
                &mut *out,
                &json!({
                    "tick": packet.tick(),
                    "demo_tick": packet.tick(),
                    "server_tick": server_tick,
                    "tick_namespace": if server_tick.is_some() { "server" } else { "demo" },
                    "packet_sequence": packet_sequence,
                    "event_index_in_packet": event_index_in_packet,
                    "event_type": game_event.event_type.as_str(),
                    "event": &game_event.event,
                }),
            )?;
            out.write_all(b"\n")?;
        }
    }
    Ok(())
}

fn packet_server_tick(packet: &Packet<'_>) -> Option<u32> {
    let messages = match packet {
        Packet::Signon(message_packet) | Packet::Message(message_packet) => &message_packet.messages,
        _ => return None,
    };
    messages.iter().rev().find_map(|message| match message {
        Message::NetTick(net_tick) => Some(u32::from(net_tick.tick)),
        _ => None,
    })
}

fn capture_metadata(header: &Header, usercmd_packet_count: u64) -> serde_json::Value {
    let nick = header.nick.trim();
    let normalized_nick = nick.to_ascii_lowercase();
    if normalized_nick.contains("sourcetv") || normalized_nick.contains("source tv") {
        json!({
            "classification": "stv",
            "confidence": "high",
            "evidence": ["Demo header nickname identifies a SourceTV recorder."],
            "header_nick": nick,
            "usercmd_packet_count": usercmd_packet_count,
        })
    } else if usercmd_packet_count > 0 {
        json!({
            "classification": "pov",
            "confidence": "medium",
            "evidence": ["Demo contains dem_usercmd packets, which record a client player's input."],
            "header_nick": nick,
            "usercmd_packet_count": usercmd_packet_count,
        })
    } else {
        json!({
            "classification": "unknown",
            "confidence": "low",
            "evidence": ["Header is not explicitly SourceTV and no dem_usercmd packets were found."],
            "header_nick": nick,
            "usercmd_packet_count": usercmd_packet_count,
        })
    }
}

fn usage(program: &str) {
    eprintln!(
        "Usage:\n  {program} <input.dem> <output_directory>\n\n\
         Example:\n  {program} \"C:\\Demos\\match.dem\" \"C:\\Demos\\match_export\""
    );
}

fn main() -> Result<(), MainError> {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "export_all".to_string());

    let Some(input_arg) = args.next() else {
        usage(&program);
        std::process::exit(2);
    };
    let Some(output_arg) = args.next() else {
        usage(&program);
        std::process::exit(2);
    };
    if args.next().is_some() {
        usage(&program);
        std::process::exit(2);
    }

    let input_path = PathBuf::from(input_arg);
    let output_dir = PathBuf::from(output_arg);

    if !input_path.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Demo file not found: {}", input_path.display()),
        )
        .into());
    }

    fs::create_dir_all(&output_dir)?;

    let bytes = fs::read(&input_path)?;
    let demo = Demo::new(&bytes);
    let mut stream = demo.get_stream();

    // The header is outside the normal packet stream.
    let header = Header::read(&mut stream)?;

    let header_file = File::create(output_dir.join("header.json"))?;
    serde_json::to_writer_pretty(BufWriter::new(header_file), &header)?;

    // DemoHandler::default() configures the parser to decode all supported
    // message types while maintaining send-table, baseline, string-table,
    // event-definition, and entity-class state for subsequent packets.
    // POV demos may omit player_connect events. Keep the decoded userinfo
    // roster so header.nick can still be resolved to a user ID.
    let mut handler = DemoHandler::with_analyser(GameStateAnalyser::new());
    handler.handle_header(&header);

    let mut packet_stream = RawPacketStream::new(stream);
    let mut packets_out = BufWriter::new(File::create(output_dir.join("packets.ndjson"))?);
    let mut index_out = BufWriter::new(File::create(output_dir.join("packet_index.ndjson"))?);
    let mut events_out = BufWriter::new(File::create(output_dir.join("events.ndjson"))?);
    let mut states_out = BufWriter::new(File::create(output_dir.join("state_samples.ndjson"))?);
    let mut state_delta_writer = StateDeltaWriter::default();
    let mut packet_count: u64 = 0;
    let mut usercmd_packet_count: u64 = 0;

    loop {
        // RawPacketStream positions are bit positions in the original demo stream.
        let start_bit = packet_stream.pos();

        let Some(packet) = packet_stream.next(&handler.state_handler)? else {
            break;
        };

        let end_bit = packet_stream.pos();
        let tick = packet.tick();
        let packet_server_tick = packet_server_tick(&packet).or_else(|| {
            let current = u32::from(handler.server_tick);
            if current == 0 { None } else { Some(current) }
        });
        let packet_type = packet.packet_type().as_lowercase_str();
        if matches!(&packet, Packet::UserCmd(_)) {
            usercmd_packet_count += 1;
        }

        // Write the complete decoded packet as one independent JSON line.
        // This streams to disk and does not keep the full match in memory.
        serde_json::to_writer(&mut packets_out, &packet)?;
        packets_out.write_all(b"\n")?;

        // Write a small seek/index record separately.
        serde_json::to_writer(
            &mut index_out,
            &json!({
                "sequence": packet_count,
                "tick": tick,
                "packet_type": packet_type,
                "start_bit": start_bit,
                "end_bit": end_bit,
                "encoded_bit_length": end_bit.saturating_sub(start_bit)
            }),
        )?;
        index_out.write_all(b"\n")?;

        // A compact event stream avoids repeated scans through deeply nested raw
        // packets during highlight analysis. packets.ndjson stays authoritative
        // for later position, projectile, and airshot reconstruction.
        write_game_events(&packet, packet_count, packet_server_tick, &mut events_out)?;

        // Apply this packet to parser state so later delta-compressed packets,
        // baselines, event definitions, and string-table updates decode correctly.
        handler.handle_packet(packet)?;
        state_delta_writer.write(
            handler.borrow_output(),
            u32::from(tick),
            packet_server_tick,
            packet_count,
            &mut states_out,
        )?;

        packet_count += 1;
    }

    packets_out.flush()?;
    index_out.flush()?;
    events_out.flush()?;
    states_out.flush()?;
    let match_state = handler.into_output();
    let players: BTreeMap<u16, _> = match_state
        .players
        .iter()
        .filter_map(|player| {
            player
                .info
                .as_ref()
                .map(|info| (u16::from(info.user_id), info))
        })
        .collect();
    let players_file = File::create(output_dir.join("players.json"))?;
    serde_json::to_writer_pretty(BufWriter::new(players_file), &players)?;
    let manifest = json!({
        "format": "tf-demo-parser-decoded-packet-stream",
        "format_version": 1,
        "source_demo": input_path.to_string_lossy(),
        "packet_count": packet_count,
        "state_sample_count": state_delta_writer.sample_count,
        "demo_capture": capture_metadata(&header, usercmd_packet_count),
        "parser_reported_incomplete": packet_stream.incomplete,
        "files": {
            "header": "header.json",
            "packets": "packets.ndjson",
            "packet_index": "packet_index.ndjson",
            "events": "events.ndjson",
            "state_samples": "state_samples.ndjson",
            "players": "players.json",
            "frag_candidates": "frag_candidates.ndjson",
            "frag_summary": "frag_summary.json"
        },
        "notes": [
            "packets.ndjson contains one complete parser-decoded top-level demo packet per line",
            "packet_index.ndjson contains packet order, tick, type, and original stream bit ranges",
            "events.ndjson contains normalized decoded game events for highlight analysis",
            "state_samples.ndjson contains parser-reconstructed player and projectile state deltas",
            "frag_candidates.ndjson and frag_summary.json are written by analyze_frags.py after parsing",
            "keep the original .dem file as the bit-exact source archive"
        ]
    });

    let manifest_file = File::create(output_dir.join("manifest.json"))?;
    serde_json::to_writer_pretty(BufWriter::new(manifest_file), &manifest)?;

    if packet_stream.incomplete {
        eprintln!(
            "Export finished, but the parser reported that the demo ended with incomplete data."
        );
    }

    println!(
        "Exported {packet_count} packets to {}",
        output_dir.display()
    );

    Ok(())
}
