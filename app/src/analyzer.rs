use crate::{
    models::{Candidate, DemoContext, TickTagGroup},
    scheduler::RuntimeGovernor,
};
use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

const TICKS_PER_SECOND: f64 = 66.666_666_7;
const SEQUENCE_GAP: i64 = 267;
const PRE_ROLL: i64 = 333;
const POST_ROLL: i64 = 200;
const BOOKMARK_SCORE: f64 = 30.0;
const OBJECTIVE_CONVERSION_TICKS: i64 = 533;
const CAPTURE_DENIAL_TICKS: i64 = 133;
const ROUND_CLINCH_TICKS: i64 = 200;
const SACK_RECOVERY_TICKS: i64 = 667;
const MEDIC_FORCE_FOLLOWUP_TICKS: i64 = 267;
const MEDIC_FORCE_PRESSURE_TICKS: i64 = 133;
const KRITZKRIEG_DURATION_TICKS: i64 = 533;
const DOUBLE_DONK_WINDOW_TICKS: i64 = 33;
const AIRSHOT_PRE_IMPACT_WINDOW_TICKS: i64 = 24;
const AIRSHOT_GROUNDED_LOOKBACK_TICKS: i64 = 133;
const AIRSHOT_MIN_AIRBORNE_TICKS: i64 = 4;
const AIRSHOT_MIN_VERTICAL_DISPLACEMENT: f64 = 12.0;
const AIRSHOT_MIN_VERTICAL_SPEED: f64 = 80.0;
const AIRSHOT_STANDARD_IMPACT_GUARD_TICKS: i64 = 2;
const AIRSHOT_LOOSE_CANNON_IMPACT_GUARD_TICKS: i64 = 6;
const PLAYER_SWING_MIN_WINDOW_TICKS: i64 = 267;
const CHARGE_MELEE_FOLLOWUP_TICKS: i64 = 57;
const DUPLICATE_DEATH_TICKS: i64 = 133;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct EventRecord {
    #[serde(default)]
    tick: i64,
    #[serde(default)]
    demo_tick: i64,
    #[serde(default)]
    server_tick: Option<i64>,
    #[serde(default)]
    packet_sequence: i64,
    #[serde(default)]
    event_index_in_packet: i64,
    #[serde(default)]
    event_type: String,
    #[serde(default)]
    event: Map<String, Value>,
}

impl EventRecord {
    fn analysis_tick(&self) -> i64 {
        self.server_tick.filter(|tick| *tick > 0).unwrap_or(self.tick.max(self.demo_tick))
    }

    fn int(&self, names: &[&str]) -> i64 {
        names.iter().find_map(|name| int_value(self.event.get(*name))).unwrap_or_default()
    }

    fn text(&self, names: &[&str]) -> String {
        names.iter().find_map(|name| text_value(self.event.get(*name))).unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
struct Round {
    index: i64,
    start: i64,
    end: i64,
    round_active_tick: i64,
    start_event: String,
    end_event: String,
    winning_team: i64,
    ready_up: bool,
    red_ready_tick: Option<i64>,
    blu_ready_tick: Option<i64>,
    ready_restart_tick: Option<i64>,
    countdown_tick: Option<i64>,
    setup_finished_tick: Option<i64>,
    activation_trigger: String,
}

#[derive(Clone, Debug, Default)]
struct Death {
    event_tick: i64,
    demo_tick: i64,
    packet_sequence: i64,
    event_index: i64,
    attacker: i64,
    victim: i64,
    assister: i64,
    round_index: i64,
    weapon: String,
    weapon_id: i64,
    weapon_def_index: i64,
    weapon_slot: String,
    custom_kill: i64,
    crit_type: i64,
    kill_streak_total: i64,
    rocket_jump_victim: bool,
    attacker_class: String,
    attacker_team: String,
    victim_class: String,
    victim_team: String,
    state: StateEvidence,
}

#[derive(Clone, Debug, Default)]
struct StateEvidence {
    attacker: Map<String, Value>,
    victim: Map<String, Value>,
    friendly_alive_before: usize,
    enemy_alive_before: usize,
    friendly_state_roster: usize,
    enemy_state_roster: usize,
    friendly_medic_charge: Option<f64>,
    enemy_medic_charge: Option<f64>,
    recent_friendly_deaths: usize,
    recent_friendly_death_ticks: Vec<i64>,
    player_disadvantage_before: usize,
    enemy_uber_advantage_before: bool,
    victim_next_respawn_tick: Option<i64>,
    friendly_pending_respawn_ticks: Vec<i64>,
    projectile: Option<Value>,
    attacker_recent_shield_charge_tick: Option<i64>,
    confirmed_double_donk: bool,
    victim_airborne_before_projectile_impact: bool,
    projectile_impact_check_tick: Option<i64>,
    confirmed_kritzkrieg_boost: bool,
    confirmed_uber_drop: bool,
    enemy_medic_force_followups: Vec<Value>,
}

#[derive(Clone, Debug, Default)]
struct StateScan {
    at_death: HashMap<i64, HashMap<i64, Map<String, Value>>>,
    all_at_death: HashMap<i64, HashMap<i64, Map<String, Value>>>,
    roster_samples: Vec<(i64, HashMap<String, HashMap<String, usize>>)>,
    projectile_tracks: HashMap<i64, Vec<(i64, Map<String, Value>)>>,
    projectile_removals: HashMap<i64, Vec<i64>>,
    player_history: HashMap<i64, Vec<(i64, Map<String, Value>)>>,
}

#[derive(Clone, Debug)]
struct BuildingEvent {
    tick: i64,
    attacker: i64,
    object_type: String,
}

#[derive(Clone, Debug)]
struct ObjectiveEvent {
    tick: i64,
    team: i64,
    actor_user_id: i64,
    kind: String,
    data: Value,
}

#[derive(Clone, Debug)]
struct ItemInfo {
    slot: String,
    log_name: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct AnalysisProfile {
    pub format: String,
    pub format_version: u32,
    pub capture_type: String,
    pub analysis_scope: String,
    pub mode: String,
    pub mode_confidence: String,
    pub total_player_death_events: usize,
    pub accepted_live_scope_kills: usize,
    pub candidate_group_jobs: usize,
    pub candidate_workers_requested: usize,
    pub candidate_workers_used: usize,
    pub stage_seconds: BTreeMap<String, f64>,
    pub total_seconds: f64,
}

pub fn analyze_export(export: &Path, item_schema: Option<&Path>) -> Result<Vec<Candidate>> {
    let workers = std::thread::available_parallelism().map(|value| value.get()).unwrap_or(1);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .thread_name(|index| format!("tf2-analysis-{index}"))
        .build()?;
    pool.install(|| analyze_export_internal(export, item_schema, workers, None, None))
}

pub fn analyze_export_in_current_pool(
    export: &Path,
    item_schema: Option<&Path>,
    cancelled: &AtomicBool,
    governor: &RuntimeGovernor,
) -> Result<Vec<Candidate>> {
    analyze_export_internal(export, item_schema, rayon::current_num_threads().max(1), Some(cancelled), Some(governor))
}

fn analyze_export_internal(
    export: &Path,
    item_schema: Option<&Path>,
    candidate_workers: usize,
    cancelled: Option<&AtomicBool>,
    governor: Option<&RuntimeGovernor>,
) -> Result<Vec<Candidate>> {
    let total_started = Instant::now();
    let mut profile = AnalysisProfile {
        format: "tf2-frag-analysis-profile-rust".into(),
        format_version: 3,
        candidate_workers_requested: candidate_workers.max(1),
        ..AnalysisProfile::default()
    };

    let started = Instant::now();
    let events = read_ndjson::<EventRecord>(&export.join("events.ndjson"), cancelled, governor)?;
    check_runtime(cancelled, governor)?;
    profile.stage_seconds.insert("read_events".into(), started.elapsed().as_secs_f64());

    let header = read_json(&export.join("header.json"));
    let manifest = read_json(&export.join("manifest.json"));
    let source_demo = manifest.get("source_demo").and_then(Value::as_str).unwrap_or_default().to_owned();
    let map_name = header.get("map").and_then(Value::as_str).unwrap_or_default().to_owned();
    let players = read_json(&export.join("players.json"));
    let mut context = capture_context(&events, &header, &manifest, &players);
    profile.capture_type = context.capture_type.clone();
    profile.analysis_scope = context.analysis_scope.clone();

    let started = Instant::now();
    let rounds = build_rounds(&events, header.get("ticks").and_then(Value::as_i64).unwrap_or_default());
    let item_map = item_schema.and_then(|path| parse_item_schema(path).ok()).unwrap_or_default();
    let mut deaths = normalized_deaths(&events, &rounds, &context, &item_map);
    profile.total_player_death_events = events.iter().filter(|event| event.event_type == "player_death").count();
    profile.accepted_live_scope_kills = deaths.len();
    profile.stage_seconds.insert("early_death_gating".into(), started.elapsed().as_secs_f64());

    let started = Instant::now();
    let scan = scan_state_stream(&export.join("state_samples.ndjson"), &deaths, &rounds, cancelled, governor)?;
    check_runtime(cancelled, governor)?;
    attach_state(&mut deaths, &scan, cancelled, governor)?;
    attach_event_evidence(&events, &mut deaths, &scan, cancelled, governor)?;
    profile.stage_seconds.insert("stream_state_enrichment".into(), started.elapsed().as_secs_f64());

    let started = Instant::now();
    let mode = classify_mode(&header, &manifest, &events, &scan.roster_samples);
    context.mode = mode.0;
    context.mode_label = mode.1;
    context.mode_confidence = mode.2;
    context.mode_evidence = mode.3;
    profile.mode = context.mode.clone();
    profile.mode_confidence = context.mode_confidence.clone();
    profile.stage_seconds.insert("mode_classification".into(), started.elapsed().as_secs_f64());

    let started = Instant::now();
    let buildings = normalized_buildings(&events, &rounds);
    let objectives = normalized_objectives(&events, &rounds);
    let (mut candidates, group_jobs, workers_used) = build_candidates(
        &mut deaths, &rounds, &context, &source_demo, &map_name, &buildings, &objectives, cancelled, governor,
    )?;
    profile.candidate_group_jobs = group_jobs;
    profile.candidate_workers_used = workers_used;
    append_bookmarks(export, &source_demo, &context, &mut candidates)?;
    // Keep low-scoring single kills available long enough for a nearby
    // bookmark to inherit their player/class/weapon/state identifiers. Normal
    // low-scoring singles are removed only after bookmark candidates exist.
    candidates.retain(|candidate| {
        candidate.bookmark_tick.is_some() || candidate.kill_count() != 1 || candidate.overall_score >= 25.0
    });
    candidates.sort_by(|left, right| right.overall_score.total_cmp(&left.overall_score).then(left.clip_start_tick.cmp(&right.clip_start_tick)));
    profile.stage_seconds.insert("candidate_grouping_and_scoring".into(), started.elapsed().as_secs_f64());

    let started = Instant::now();
    write_candidates(export, &candidates, &context, &source_demo, &map_name, item_schema)?;
    profile.stage_seconds.insert("write_outputs".into(), started.elapsed().as_secs_f64());
    profile.total_seconds = total_started.elapsed().as_secs_f64();
    fs::write(export.join("analysis_profile.json"), serde_json::to_vec_pretty(&profile)?)?;
    Ok(candidates)
}

fn check_cancelled(cancelled: Option<&AtomicBool>) -> Result<()> {
    if cancelled.is_some_and(|cancelled| cancelled.load(Ordering::Relaxed)) { anyhow::bail!("cancelled"); }
    Ok(())
}

fn check_runtime(cancelled: Option<&AtomicBool>, governor: Option<&RuntimeGovernor>) -> Result<()> {
    check_cancelled(cancelled)?;
    if let Some(governor) = governor { governor.checkpoint()?; }
    Ok(())
}

fn read_ndjson<T: for<'de> Deserialize<'de>>(
    path: &Path,
    cancelled: Option<&AtomicBool>,
    governor: Option<&RuntimeGovernor>,
) -> Result<Vec<T>> {
    let source = File::open(path).with_context(|| format!("missing {}", path.display()))?;
    BufReader::new(source)
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if index % 4096 == 0 {
                if let Err(error) = check_runtime(cancelled, governor) {
                    return Some(Err(error));
                }
            }
            match line {
            Ok(line) if !line.trim().is_empty() => Some(serde_json::from_str(&line).map_err(anyhow::Error::from)),
            Ok(_) => None,
            Err(error) => Some(Err(error.into())),
        }})
        .collect()
}

fn read_json(path: &Path) -> Value {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(Value::Null)
}

fn capture_context(events: &[EventRecord], header: &Value, manifest: &Value, players: &Value) -> DemoContext {
    let empty_capture = Value::Null;
    let capture = manifest.get("demo_capture").unwrap_or(&empty_capture);
    let capture_type = capture.get("classification").and_then(Value::as_str).unwrap_or("unknown").to_lowercase();
    let nickname = header.get("nick").and_then(Value::as_str).unwrap_or_default();
    let mut name_matches = HashSet::new();
    for event in events.iter().filter(|event| matches!(event.event_type.as_str(), "player_connect" | "player_connect_client" | "player_changename")) {
        let name = event.text(if event.event_type == "player_changename" { &["newname"] } else { &["name"] });
        if name.eq_ignore_ascii_case(nickname) {
            let user_id = event.int(&["user_id", "userid"]);
            if user_id > 0 {
                name_matches.insert(user_id);
            }
        }
    }
    if capture_type == "pov" && name_matches.is_empty() {
        if let Some(roster) = players.as_object() {
            for (key, player) in roster {
                if player.get("name").and_then(Value::as_str).is_some_and(|name| name.eq_ignore_ascii_case(nickname)) {
                    let user = int_value(player.get("user_id")).or_else(|| key.parse().ok()).unwrap_or_default();
                    if user > 0 { name_matches.insert(user); }
                }
            }
        }
    }
    let pov = (capture_type == "pov" && name_matches.len() == 1).then(|| *name_matches.iter().next().unwrap());
    let analysis_scope = if capture_type == "pov" && pov.is_some() { "pov_player_only" } else { "all_players" };
    let scope_reason = if capture_type == "pov" && pov.is_some() {
        "POV recorder matched to player events or the decoded userinfo roster."
    } else if capture_type == "pov" {
        "POV recording detected, but the recorded player could not be matched safely; all players were retained."
    } else {
        "STV and unknown demos retain candidates from every player."
    };
    DemoContext {
        capture_type: capture_type.clone(),
        capture_confidence: capture.get("confidence").and_then(Value::as_str).unwrap_or("unknown").into(),
        capture_evidence: capture.get("evidence").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(ToOwned::to_owned).collect(),
        header_nick: (!nickname.is_empty()).then(|| nickname.to_owned()),
        // A POV classification alone does not identify the recorder. If the
        // header nickname cannot be resolved uniquely, retaining all players
        // is safer than silently producing zero candidates.
        analysis_scope: analysis_scope.into(),
        pov_player_user_id: pov,
        roster_match_available: players.as_object().is_some_and(|players| !players.is_empty()),
        scope_reason: scope_reason.into(),
        ..DemoContext::default()
    }
}

fn build_rounds(events: &[EventRecord], header_ticks: i64) -> Vec<Round> {
    let activation = ["teamplay_round_start", "teamplay_restart_round", "teamplay_ready_restart", "teamplay_round_restart_seconds", "teamplay_waiting_ends", "round_start"];
    let endings = ["teamplay_round_win", "teamplay_round_stalemate", "teamplay_game_over", "tf_game_over", "game_end", "round_end"];
    let mut rounds = Vec::new();
    let mut current: Option<Round> = None;
    let mut pending_activation: Option<(i64, String)> = None;
    let mut red_ready = false;
    let mut blu_ready = false;
    let mut red_ready_tick: Option<i64> = None;
    let mut blu_ready_tick: Option<i64> = None;
    let mut ready_restart_tick: Option<i64> = None;
    let mut countdown_tick: Option<i64> = None;
    for event in events {
        let name = event.event_type.as_str();
        let tick = event.analysis_tick();
        if name == "teamplay_team_ready" {
            let ready = event.event.get("ready").and_then(Value::as_bool).unwrap_or(true);
            match event.int(&["team"]) {
                2 => { red_ready = ready; red_ready_tick = ready.then_some(tick); }
                3 => { blu_ready = ready; blu_ready_tick = ready.then_some(tick); }
                _ => {}
            }
        } else if activation.contains(&name) {
            if let Some(mut active) = current.take() {
                active.end = tick;
                active.end_event = name.into();
                if active.end > active.start { rounds.push(active); }
            }
            if name == "teamplay_ready_restart" { ready_restart_tick = Some(tick); }
            if name == "teamplay_round_restart_seconds" { countdown_tick = Some(tick); }
            pending_activation = Some((tick, name.into()));
        } else if name == "teamplay_round_active" {
            let Some((trigger_tick, trigger)) = pending_activation.take() else { continue };
            if let Some(mut active) = current.take() {
                active.end = tick;
                active.end_event = "superseded_by_round_active".into();
                if active.end > active.start { rounds.push(active); }
            }
            current = Some(Round {
                index: rounds.len() as i64 + 1,
                start: tick,
                end: 0,
                round_active_tick: tick,
                start_event: "teamplay_round_active".into(),
                end_event: String::new(),
                winning_team: 0,
                ready_up: red_ready && blu_ready,
                red_ready_tick,
                blu_ready_tick,
                ready_restart_tick,
                countdown_tick,
                setup_finished_tick: None,
                activation_trigger: format!("{trigger}@{trigger_tick}"),
            });
            red_ready = false;
            blu_ready = false;
            red_ready_tick = None;
            blu_ready_tick = None;
            ready_restart_tick = None;
            countdown_tick = None;
        } else if name == "teamplay_setup_finished" {
            if let Some(active) = current.as_mut() {
                active.start = tick;
                active.start_event = name.into();
                active.setup_finished_tick = Some(tick);
            }
        } else if endings.contains(&name) {
            if let Some(mut active) = current.take() {
                active.end = tick;
                active.end_event = name.into();
                active.winning_team = event.int(&["team", "winning_team"]);
                if active.end > active.start { rounds.push(active); }
            }
            pending_activation = None;
            red_ready = false;
            blu_ready = false;
            red_ready_tick = None;
            blu_ready_tick = None;
            ready_restart_tick = None;
            countdown_tick = None;
        } else if name == "teamplay_waiting_begins" {
            if let Some(mut active) = current.take() {
                active.end = tick;
                active.end_event = name.into();
                if active.end > active.start { rounds.push(active); }
            }
            pending_activation = None;
            red_ready = false;
            blu_ready = false;
            red_ready_tick = None;
            blu_ready_tick = None;
            ready_restart_tick = None;
            countdown_tick = None;
        }
    }
    if let Some(mut active) = current {
        active.end = header_ticks.max(events.last().map(EventRecord::analysis_tick).unwrap_or(active.start) + 1);
        active.end_event = "demo_end_while_event_confirmed_round_active".into();
        if active.end > active.start { rounds.push(active); }
    }
    if rounds.is_empty() && !events.iter().any(|event| event.event_type == "teamplay_round_active") {
        let mut teams = HashMap::<i64, String>::new();
        let mut first: Option<i64> = None;
        for event in events {
            if event.event_type == "player_team" {
                let user = event.int(&["user_id", "userid"]);
                let team = canonical_team_value(event.event.get("team"));
                if user > 0 && !team.is_empty() { teams.insert(user, team); }
            } else if event.event_type == "player_death" {
                let attacker = event.int(&["attacker"]);
                let victim = event.int(&["user_id", "userid"]);
                if attacker > 0 && victim > 0 && attacker != victim
                    && teams.get(&attacker).is_some_and(|team| matches!(team.as_str(), "red" | "blu"))
                    && teams.get(&victim).is_some_and(|team| matches!(team.as_str(), "red" | "blu"))
                    && teams.get(&attacker) != teams.get(&victim) {
                    first = Some(event.analysis_tick());
                    break;
                }
            }
        }
        if let Some(start) = first {
            let end = header_ticks.max(events.last().map(EventRecord::analysis_tick).unwrap_or(start) + 1);
            rounds.push(Round { index: 1, start, end, round_active_tick: start, start_event: "in_progress_public_server".into(), end_event: "demo_end_while_public_play_active".into(), winning_team: 0, ready_up: false, red_ready_tick: None, blu_ready_tick: None, ready_restart_tick: None, countdown_tick: None, setup_finished_tick: None, activation_trigger: format!("state_confirmed_opposing_team_death@{start}") });
        }
    }
    rounds
}

fn normalized_deaths(events: &[EventRecord], rounds: &[Round], context: &DemoContext, items: &HashMap<i64, ItemInfo>) -> Vec<Death> {
    let mut classes: HashMap<i64, String> = HashMap::new();
    let mut teams: HashMap<i64, String> = HashMap::new();
    let mut last_death_by_victim: HashMap<i64, i64> = HashMap::new();
    let mut deaths = Vec::new();
    for event in events {
        if matches!(event.event_type.as_str(), "player_changeclass" | "player_class" | "player_spawn") {
            let user = event.int(&["user_id", "userid"]);
            let class = event.text(&["class", "class_name"]);
            if user > 0 && !class.is_empty() {
                classes.insert(user, canonical_class(&class));
            }
        }
        if matches!(event.event_type.as_str(), "player_team" | "player_spawn") {
            let user = event.int(&["user_id", "userid"]);
            let team = canonical_team_value(event.event.get("team"));
            if user > 0 && !team.is_empty() {
                teams.insert(user, team);
            }
        }
        if event.event_type != "player_death" {
            continue;
        }
        let attacker = event.int(&["attacker"]);
        let victim = event.int(&["user_id", "userid"]);
        if attacker <= 0 || victim <= 0 || attacker == victim {
            continue;
        }
        if context.analysis_scope == "pov_player_only" && context.pov_player_user_id != Some(attacker) {
            continue;
        }
        let tick = event.analysis_tick();
        let Some(round) = rounds.iter().find(|round| tick >= round.start && tick < round.end) else { continue };
        if last_death_by_victim.get(&victim).is_some_and(|previous| tick >= *previous && tick - *previous <= DUPLICATE_DEATH_TICKS) {
            continue;
        }
        last_death_by_victim.insert(victim, tick);
        let weapon_def_index = event.int(&["weapon_def_index", "weapon_defindex", "defindex"]);
        let item = items.get(&weapon_def_index);
        let mut weapon = event.text(&["weapon", "weapon_logclassname"]).to_ascii_lowercase();
        if weapon.is_empty() { weapon = item.map(|item| item.log_name.clone()).unwrap_or_default(); }
        let mut weapon_slot = event.text(&["weapon_slot"]).to_lowercase();
        if weapon_slot.is_empty() { weapon_slot = item.map(|item| item.slot.clone()).unwrap_or_default(); }
        deaths.push(Death {
            event_tick: tick,
            demo_tick: if event.demo_tick > 0 { event.demo_tick } else { event.tick },
            packet_sequence: event.packet_sequence,
            event_index: event.event_index_in_packet,
            attacker,
            victim,
            assister: event.int(&["assister"]),
            round_index: round.index,
            weapon,
            weapon_id: event.int(&["weapon_id"]),
            weapon_def_index,
            weapon_slot,
            custom_kill: event.int(&["custom_kill", "customkill"]),
            crit_type: event.int(&["crit_type"]),
            kill_streak_total: event.int(&["kill_streak_total"]),
            rocket_jump_victim: event.event.get("rocket_jump").and_then(Value::as_bool).unwrap_or(false),
            attacker_class: classes.get(&attacker).cloned().unwrap_or_default(),
            attacker_team: teams.get(&attacker).cloned().unwrap_or_default(),
            victim_class: classes.get(&victim).cloned().unwrap_or_default(),
            victim_team: teams.get(&victim).cloned().unwrap_or_default(),
            ..Death::default()
        });
    }
    deaths
}

fn scan_state_stream(
    path: &Path,
    deaths: &[Death],
    rounds: &[Round],
    cancelled: Option<&AtomicBool>,
    governor: Option<&RuntimeGovernor>,
) -> Result<StateScan> {
    if !path.is_file() {
        return Ok(StateScan::default());
    }
    let mut roster_ticks = BTreeSet::new();
    for round in rounds {
        let mut tick = round.start + (TICKS_PER_SECOND * 15.0) as i64;
        while tick < round.end {
            roster_ticks.insert(tick);
            tick += (TICKS_PER_SECOND * 30.0) as i64;
        }
    }
    let source = BufReader::new(File::open(path)?);
    let mut current: HashMap<i64, Map<String, Value>> = HashMap::new();
    let mut entity_users: HashMap<i64, i64> = HashMap::new();
    let mut current_projectiles: HashMap<i64, Map<String, Value>> = HashMap::new();
    let mut result = StateScan::default();
    let mut pending_rosters = roster_ticks.into_iter().peekable();
    let mut death_users = BTreeMap::<i64, BTreeSet<i64>>::new();
    for death in deaths {
        death_users.entry(death.event_tick).or_default().extend([death.attacker, death.victim]);
    }
    let death_ticks = death_users.keys().copied().collect::<Vec<_>>();
    let mut death_index = 0usize;
    let mut lines = source.lines();
    loop {
        check_runtime(cancelled, governor)?;
        let mut batch = Vec::with_capacity(1024);
        let mut exhausted = false;
        for _ in 0..1024 {
            let Some(line) = lines.next() else { exhausted = true; break };
            let line = line?;
            if !line.trim().is_empty() { batch.push(line); }
        }
        if batch.is_empty() {
            if exhausted { break; }
            continue;
        }
        let records = batch.par_iter().map(|line| serde_json::from_str::<Value>(line)).collect::<serde_json::Result<Vec<_>>>()?;
        for record in records {
            let tick = record.get("server_tick").and_then(Value::as_i64).or_else(|| record.get("demo_tick").and_then(Value::as_i64)).unwrap_or_default();
            // State deltas describe the packet after it was applied. Capture
            // each frag from the state immediately before its event tick so a
            // killed player still appears alive with the pre-frag charge.
            while death_index < death_ticks.len() && death_ticks[death_index] <= tick {
                let target = death_ticks[death_index];
                capture_death_snapshot(target, death_users.get(&target), &current, &mut result);
                death_index += 1;
            }
            if let Some(players) = record.get("players").and_then(Value::as_array) {
                for player in players {
                    let user = player.get("user_id").and_then(Value::as_i64).unwrap_or_default();
                    if user <= 0 {
                        continue;
                    }
                    if let Some(entity) = player.get("entity_id").and_then(Value::as_i64) {
                        entity_users.insert(entity, user);
                    }
                    let state = current.entry(user).or_default();
                    if let Some(update) = player.as_object() {
                        for (key, value) in update {
                            state.insert(key.clone(), value.clone());
                        }
                    }
                    let compact = compact_player_state(state);
                    let history = result.player_history.entry(user).or_default();
                    if history.last().is_none_or(|(_, previous)| previous != &compact) {
                        history.push((tick, compact));
                    }
                }
            }
            if let Some(removed) = record.get("removed_players").and_then(Value::as_array) {
                for entity in removed.iter().filter_map(Value::as_i64) {
                    if let Some(user) = entity_users.remove(&entity) {
                        current.remove(&user);
                    }
                }
            }
            if let Some(projectiles) = record.get("projectiles").and_then(Value::as_array) {
                for projectile in projectiles {
                    let entity = int_value(projectile.get("entity_id")).unwrap_or_default();
                    if entity <= 0 { continue; }
                    let state = current_projectiles.entry(entity).or_default();
                    if let Some(update) = projectile.as_object() {
                        for (key, value) in update { state.insert(key.clone(), value.clone()); }
                    }
                    state.insert("state_tick".into(), json!(tick));
                    result.projectile_tracks.entry(entity).or_default().push((tick, compact_projectile_state(state)));
                }
            }
            if let Some(removed) = record.get("removed_projectiles").and_then(Value::as_array) {
                for entity in removed.iter().filter_map(Value::as_i64) {
                    current_projectiles.remove(&entity);
                    result.projectile_removals.entry(entity).or_default().push(tick);
                }
            }
            while pending_rosters.peek().is_some_and(|target| *target <= tick) {
                let target = pending_rosters.next().unwrap();
                let mut roster: HashMap<String, HashMap<String, usize>> = HashMap::new();
                for state in current.values() {
                    let team = canonical_team_value(state.get("team"));
                    let class = state.get("class").and_then(Value::as_str).map(canonical_class).unwrap_or_default();
                    if matches!(team.as_str(), "red" | "blu") && !class.is_empty() {
                        *roster.entry(team).or_default().entry(class).or_default() += 1;
                    }
                }
                result.roster_samples.push((target, roster));
            }
        }
        if exhausted { break; }
    }
    while death_index < death_ticks.len() {
        let target = death_ticks[death_index];
        capture_death_snapshot(target, death_users.get(&target), &current, &mut result);
        death_index += 1;
    }
    Ok(result)
}

fn capture_death_snapshot(
    tick: i64,
    users: Option<&BTreeSet<i64>>,
    current: &HashMap<i64, Map<String, Value>>,
    result: &mut StateScan,
) {
    let selected = users.into_iter().flatten().filter_map(|user| {
        current.get(user).cloned().map(|state| (*user, state))
    }).collect();
    let all = current.iter().map(|(user, state)| (*user, compact_player_state(state))).collect();
    result.at_death.insert(tick, selected);
    result.all_at_death.insert(tick, all);
}

fn compact_player_state(state: &Map<String, Value>) -> Map<String, Value> {
    const KEYS: &[&str] = &[
        "team", "class", "health", "life_state", "medic_charge", "shield_charging", "medigun",
        "position", "velocity", "on_ground", "blast_jumping",
    ];
    KEYS.iter().filter_map(|key| state.get(*key).cloned().map(|value| ((*key).into(), value))).collect()
}

fn compact_projectile_state(state: &Map<String, Value>) -> Map<String, Value> {
    const KEYS: &[&str] = &["launcher_handle", "projectile_type", "position", "state_tick"];
    KEYS.iter().filter_map(|key| state.get(*key).cloned().map(|value| ((*key).into(), value))).collect()
}

fn player_state_before(scan: &StateScan, user: i64, tick: i64) -> Option<&Map<String, Value>> {
    scan.player_history.get(&user)?.iter().rev().find(|(sample_tick, _)| *sample_tick < tick).map(|(_, state)| state)
}

fn last_player_flag_tick(scan: &StateScan, user: i64, tick: i64, key: &str, window_ticks: i64) -> Option<i64> {
    scan.player_history.get(&user)?.iter().rev().find_map(|(sample_tick, state)| {
        (*sample_tick <= tick && *sample_tick >= tick - window_ticks
            && state.get(key).and_then(Value::as_bool).unwrap_or(false)).then_some(*sample_tick)
    })
}

/// Airshots require a recent, observed grounded-to-airborne transition followed
/// by sustained vertical motion. Absolute Z height is deliberately irrelevant:
/// standing or walking on a high ledge is not an airshot. `impact_guard_ticks`
/// moves the evidence cutoff earlier than the reported hurt tick so projectile
/// knockback (especially the Loose Cannon collision) cannot certify itself.
fn player_airborne_before_impact(
    scan: &StateScan,
    user: i64,
    impact_tick: i64,
    impact_guard_ticks: i64,
) -> bool {
    let Some(history) = scan.player_history.get(&user) else { return false };
    let evidence_tick = impact_tick.saturating_sub(impact_guard_ticks.max(0));
    let grounded_tick = history.iter().rev().find_map(|(tick, state)| {
        (*tick < evidence_tick
            && *tick >= evidence_tick - AIRSHOT_GROUNDED_LOOKBACK_TICKS
            && state.get("on_ground").and_then(Value::as_bool) == Some(true))
            .then_some(*tick)
    });
    let Some(grounded_tick) = grounded_tick else { return false };
    let samples = history
        .iter()
        .filter(|(tick, _)| {
            *tick > grounded_tick
                && *tick < evidence_tick
                && *tick >= evidence_tick - AIRSHOT_PRE_IMPACT_WINDOW_TICKS
        })
        .collect::<Vec<_>>();
    let Some((latest_tick, latest)) = samples.last() else { return false };
    if latest.get("on_ground").and_then(Value::as_bool) == Some(true) {
        return false;
    }
    if *latest_tick - grounded_tick < AIRSHOT_MIN_AIRBORNE_TICKS {
        return false;
    }
    let airborne_samples = samples
        .iter()
        .filter(|(_, state)| state.get("on_ground").and_then(Value::as_bool) == Some(false))
        .count();
    if airborne_samples < 3 {
        return false;
    }
    let vertical_speed = samples.iter()
        .filter_map(|(_, state)| vector3(state.get("velocity")).map(|velocity| velocity[2].abs()))
        .fold(0.0, f64::max);
    let vertical_displacement = samples
        .iter()
        .filter_map(|(_, state)| vector3(state.get("position")).map(|position| position[2]))
        .fold(None::<(f64, f64)>, |range, z| {
            Some(match range {
                Some((minimum, maximum)) => (minimum.min(z), maximum.max(z)),
                None => (z, z),
            })
        })
        .map(|(minimum, maximum)| maximum - minimum)
        .unwrap_or_default();
    vertical_speed >= AIRSHOT_MIN_VERTICAL_SPEED
        || vertical_displacement >= AIRSHOT_MIN_VERTICAL_DISPLACEMENT
}

fn attach_state(
    deaths: &mut [Death],
    scan: &StateScan,
    cancelled: Option<&AtomicBool>,
    governor: Option<&RuntimeGovernor>,
) -> Result<()> {
    deaths.par_chunks_mut(128).try_for_each(|chunk| -> Result<()> {
        check_runtime(cancelled, governor)?;
        chunk.iter_mut().for_each(|death| {
        let Some(snapshot) = scan.at_death.get(&death.event_tick) else { return };
        if let Some(state) = snapshot.get(&death.attacker) {
            death.state.attacker = state.clone();
            death.attacker_class = state.get("class").and_then(Value::as_str).map(canonical_class).unwrap_or_else(|| death.attacker_class.clone());
            death.attacker_team = canonical_team_value(state.get("team"));
        }
        if let Some(state) = snapshot.get(&death.victim) {
            death.state.victim = state.clone();
            death.victim_class = state.get("class").and_then(Value::as_str).map(canonical_class).unwrap_or_else(|| death.victim_class.clone());
            death.victim_team = canonical_team_value(state.get("team"));
        }
        if let Some(players) = scan.all_at_death.get(&death.event_tick) {
            let attacker_team = death.attacker_team.clone();
            let victim_team = death.victim_team.clone();
            for state in players.values() {
                let team = canonical_team_value(state.get("team"));
                if team == attacker_team { death.state.friendly_state_roster += 1; }
                if team == victim_team { death.state.enemy_state_roster += 1; }
                if player_alive(state) {
                    if team == attacker_team { death.state.friendly_alive_before += 1; }
                    if team == victim_team { death.state.enemy_alive_before += 1; }
                    if state.get("class").and_then(Value::as_str).map(canonical_class).as_deref() == Some("medic") {
                        let charge = state.get("medic_charge").and_then(Value::as_f64);
                        if team == attacker_team { death.state.friendly_medic_charge = max_optional(death.state.friendly_medic_charge, charge); }
                        if team == victim_team { death.state.enemy_medic_charge = max_optional(death.state.enemy_medic_charge, charge); }
                    }
                }
            }
            death.state.player_disadvantage_before = death.state.enemy_alive_before.saturating_sub(death.state.friendly_alive_before);
            let friendly_charge = death.state.friendly_medic_charge;
            let enemy_charge = death.state.enemy_medic_charge;
            death.state.enemy_uber_advantage_before = enemy_charge.is_some_and(|enemy| {
                (enemy >= 95.0 && friendly_charge.is_none_or(|friendly| friendly < 95.0))
                    || (enemy >= 75.0 && friendly_charge.is_none_or(|friendly| enemy - friendly >= 25.0))
            });
            death.state.friendly_pending_respawn_ticks = players.iter().filter_map(|(user, state)| {
                (canonical_team_value(state.get("team")) == attacker_team && !player_alive(state))
                    .then(|| next_alive_tick(scan, *user, death.event_tick))
                    .flatten()
            }).collect();
            death.state.friendly_pending_respawn_ticks.sort_unstable();
        }
        death.state.victim_next_respawn_tick = next_alive_tick(scan, death.victim, death.event_tick);
        death.state.confirmed_uber_drop = death.victim_class == "medic"
            && death.state.victim.get("medic_charge").and_then(Value::as_f64).unwrap_or_default() >= 95.0;
        death.state.projectile = matching_projectile(death, scan);
        });
        Ok(())
    })
}

fn next_alive_tick(scan: &StateScan, user: i64, tick: i64) -> Option<i64> {
    scan.player_history.get(&user)?.iter().find_map(|(state_tick, state)| {
        (*state_tick > tick && player_alive(state)).then_some(*state_tick)
    })
}

fn player_alive(state: &Map<String, Value>) -> bool {
    state.get("life_state").and_then(Value::as_str).is_none_or(|value| value.contains("alive"))
        && state.get("health").and_then(Value::as_i64).unwrap_or(1) > 0
}

fn max_optional(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) { (Some(a), Some(b)) => Some(a.max(b)), (Some(a), None) => Some(a), (None, value) => value }
}

fn vector3(value: Option<&Value>) -> Option<[f64; 3]> {
    let values = value?.as_array()?;
    (values.len() >= 3).then(|| [values[0].as_f64().unwrap_or_default(), values[1].as_f64().unwrap_or_default(), values[2].as_f64().unwrap_or_default()])
}

/// Returns victim height relative to the attacker, horizontal separation, and
/// the upward elevation angle required to aim from attacker origin to victim
/// origin. World-space Z is never scored by itself.
fn airshot_geometry(
    attacker: &Map<String, Value>,
    victim: &Map<String, Value>,
) -> Option<(f64, f64, f64)> {
    let attacker = vector3(attacker.get("position"))?;
    let victim = vector3(victim.get("position"))?;
    let delta_x = victim[0] - attacker[0];
    let delta_y = victim[1] - attacker[1];
    let relative_height = victim[2] - attacker[2];
    let horizontal_distance = (delta_x.powi(2) + delta_y.powi(2)).sqrt();
    let elevation_degrees = relative_height.atan2(horizontal_distance.max(0.001)).to_degrees();
    Some((relative_height, horizontal_distance, elevation_degrees))
}

/// Rewards visually difficult skyward airshots only after airborne eligibility
/// has already been proven. Requiring both meaningful relative height and a
/// steep upward angle prevents either measurement from dominating by itself.
fn airshot_style_bonus(relative_height: f64, elevation_degrees: f64) -> f64 {
    if relative_height <= 0.0 || elevation_degrees <= 0.0 {
        return 0.0;
    }
    let height_points: f64 = match relative_height {
        value if value >= 256.0 => 10.0,
        value if value >= 192.0 => 7.0,
        value if value >= 128.0 => 4.0,
        value if value >= 64.0 => 2.0,
        _ => 0.0,
    };
    let angle_points: f64 = match elevation_degrees {
        value if value >= 50.0 => 10.0,
        value if value >= 35.0 => 7.0,
        value if value >= 20.0 => 4.0,
        value if value >= 10.0 => 2.0,
        _ => 0.0,
    };
    (height_points + angle_points).min(20.0)
}

fn matching_projectile(death: &Death, scan: &StateScan) -> Option<Value> {
    let victim = vector3(death.state.victim.get("position"))?;
    let handles = death.state.attacker.get("weapon_handles").and_then(Value::as_array)?
        .iter().filter_map(Value::as_i64).collect::<HashSet<_>>();
    if handles.is_empty() { return None; }
    let weapon = death.weapon.to_ascii_lowercase();
    let mut best: Option<(f64, Value)> = None;
    for (entity, track) in &scan.projectile_tracks {
        let Some((state_tick, state)) = track.iter().min_by_key(|(tick, _)| (tick - death.event_tick).abs()) else { continue };
        if (state_tick - death.event_tick).abs() > 6 { continue; }
        if !handles.contains(&int_value(state.get("launcher_handle")).unwrap_or_default()) { continue; }
        let projectile_type = state.get("projectile_type").and_then(Value::as_str).unwrap_or_default().to_ascii_lowercase();
        if !projectile_matches_weapon(&projectile_type, &weapon) { continue; }
        let position = vector3(state.get("position"))?;
        let distance = ((position[0]-victim[0]).powi(2)+(position[1]-victim[1]).powi(2)+(position[2]-victim[2]).powi(2)).sqrt();
        if distance > 220.0 { continue; }
        let usable = track.iter().filter(|(tick, _)| *tick <= death.event_tick + 5).collect::<Vec<_>>();
        let launch_tick = usable.first().map(|(tick, _)| *tick).unwrap_or(*state_tick);
        let impact_tick = scan.projectile_removals.get(entity).and_then(|ticks| ticks.iter().copied().filter(|tick| *tick >= launch_tick).min()).unwrap_or(death.event_tick);
        let mut path_distance = 0.0;
        let mut previous: Option<[f64; 3]> = None;
        let mut vertical_range = (f64::MAX, f64::MIN);
        for (_, sample) in &usable {
            if let Some(current) = vector3(sample.get("position")) {
                vertical_range.0 = vertical_range.0.min(current[2]);
                vertical_range.1 = vertical_range.1.max(current[2]);
                if let Some(prior) = previous { path_distance += ((current[0]-prior[0]).powi(2)+(current[1]-prior[1]).powi(2)+(current[2]-prior[2]).powi(2)).sqrt(); }
                previous = Some(current);
            }
        }
        let in_flight = projectile_type != "pipe" || path_distance >= 2.0 || vertical_range.1 - vertical_range.0 >= 6.0;
        let evidence = json!({
            "entity_id":entity, "projectile_type":projectile_type, "launcher_handle":state.get("launcher_handle"),
            "distance_to_victim":(distance*100.0).round()/100.0, "impact_proximity":if distance <= 64.0 {"direct"} else {"splash"},
            "launch_tick":launch_tick, "impact_tick":impact_tick, "flight_seconds":((impact_tick-launch_tick).max(0) as f64/TICKS_PER_SECOND*1000.0).round()/1000.0,
            "tracked_path_distance":(path_distance*100.0).round()/100.0, "airshot_eligible":in_flight,
        });
        if best.as_ref().is_none_or(|(prior, _)| distance < *prior) { best = Some((distance, evidence)); }
    }
    best.map(|(_, value)| value)
}

fn projectile_matches_weapon(projectile: &str, weapon: &str) -> bool {
    (projectile.contains("rocket") && ["rocket", "directhit", "blackbox", "liberty", "airstrike"].iter().any(|name| weapon.contains(name)))
        || (projectile.contains("pipe") && ["grenade", "loch", "iron_bomber"].iter().any(|name| weapon.contains(name)))
        || (projectile.contains("cannon") && weapon.contains("loose_cannon"))
        || (projectile.contains("flare") && weapon.contains("flare"))
        || ((projectile.contains("arrow") || projectile.contains("unknown")) && ["huntsman", "crossbow"].iter().any(|name| weapon.contains(name)))
}

fn attach_event_evidence(
    events: &[EventRecord],
    deaths: &mut [Death],
    scan: &StateScan,
    cancelled: Option<&AtomicBool>,
    governor: Option<&RuntimeGovernor>,
) -> Result<()> {
    let death_history = deaths.iter().map(|death| (death.event_tick, death.victim, death.victim_team.clone())).collect::<Vec<_>>();
    let deploys = events.iter().filter(|event| matches!(event.event_type.as_str(), "player_chargedeployed" | "medic_deployed"))
        .map(|event| (event.analysis_tick(), event.int(&["user_id", "userid"]), event.int(&["targetid", "target_user_id"]))).collect::<Vec<_>>();
    let charged_deaths = events.iter().filter(|event| event.event_type == "medic_death" && event.event.get("charged").and_then(Value::as_bool).unwrap_or(false))
        .map(|event| (event.analysis_tick(), event.int(&["user_id", "userid"]))).collect::<HashSet<_>>();
    let hurts = events.iter().filter(|event| event.event_type == "player_hurt").map(|event| (
        event.analysis_tick(), event.int(&["attacker"]), event.int(&["user_id", "userid"]), event.int(&["weapon_id"]),
        event.event.get("mini_crit").or_else(|| event.event.get("minicrit")).and_then(Value::as_bool).unwrap_or(false)
    )).collect::<Vec<_>>();
    deaths.par_chunks_mut(128).try_for_each(|chunk| -> Result<()> {
        check_runtime(cancelled, governor)?;
        chunk.iter_mut().for_each(|death| {
        let recent_losses = death_history.iter().filter(|(tick, _, team)| {
            *tick < death.event_tick && *tick >= death.event_tick - SACK_RECOVERY_TICKS && *team == death.attacker_team
        }).collect::<Vec<_>>();
        death.state.recent_friendly_deaths = recent_losses.iter().map(|(_, victim, _)| *victim).collect::<HashSet<_>>().len();
        death.state.recent_friendly_death_ticks = recent_losses.iter().map(|(tick, _, _)| *tick).collect();
        death.state.recent_friendly_death_ticks.sort_unstable();
        death.state.recent_friendly_death_ticks.dedup();
        if charged_deaths.contains(&(death.event_tick, death.victim)) { death.state.confirmed_uber_drop = true; }
        let recently_deployed = deploys.iter().any(|(tick, medic, _)| *medic == death.victim && *tick <= death.event_tick && death.event_tick - *tick <= CAPTURE_DENIAL_TICKS);
        if recently_deployed { death.state.confirmed_uber_drop = false; }
        let attacker_boosted = death.state.attacker.get("kritz_boosted").and_then(Value::as_bool).unwrap_or(false);
        death.state.confirmed_kritzkrieg_boost = attacker_boosted && deploys.iter().any(|(tick, medic, target)| {
            *target == death.attacker && *tick <= death.event_tick && death.event_tick - *tick <= KRITZKRIEG_DURATION_TICKS
                && player_state_before(scan, *medic, *tick).is_some_and(|state| {
                    state.get("class").and_then(Value::as_str).map(canonical_class).as_deref() == Some("medic")
                        && canonical_team_value(state.get("team")) == death.attacker_team
                        && state.get("medigun").and_then(Value::as_str).is_some_and(|value| value.to_ascii_lowercase().contains("kritz"))
                })
        });
        if death.weapon.contains("loose_cannon") {
            let matching = hurts.iter().filter(|(tick, attacker, victim, weapon_id, _)| {
                *attacker == death.attacker && *victim == death.victim && *tick <= death.event_tick && death.event_tick - *tick <= DOUBLE_DONK_WINDOW_TICKS
                    && (*weapon_id == 0 || death.weapon_id == 0 || *weapon_id == death.weapon_id)
            }).collect::<Vec<_>>();
            death.state.confirmed_double_donk = matching.iter().any(|impact| !impact.4 && matching.iter().any(|explosion| {
                explosion.4 && explosion.0 >= impact.0 && explosion.0 - impact.0 <= DOUBLE_DONK_WINDOW_TICKS
                    && death.event_tick - explosion.0 <= 1
            }));
            let impact_tick = matching
                .iter()
                .map(|event| event.0)
                .min()
                .unwrap_or(death.event_tick);
            death.state.projectile_impact_check_tick = Some(impact_tick);
            death.state.victim_airborne_before_projectile_impact =
                player_airborne_before_impact(
                    scan,
                    death.victim,
                    impact_tick,
                    AIRSHOT_LOOSE_CANNON_IMPACT_GUARD_TICKS,
                );
        } else {
            death.state.projectile_impact_check_tick = Some(death.event_tick);
            death.state.victim_airborne_before_projectile_impact =
                player_airborne_before_impact(
                    scan,
                    death.victim,
                    death.event_tick,
                    AIRSHOT_STANDARD_IMPACT_GUARD_TICKS,
                );
        }
        death.state.attacker_recent_shield_charge_tick = last_player_flag_tick(scan, death.attacker, death.event_tick, "shield_charging", CHARGE_MELEE_FOLLOWUP_TICKS);
        if let Some(players) = scan.all_at_death.get(&death.event_tick) {
            for (tick, medic, target) in &deploys {
                if *tick < death.event_tick || *tick > death.event_tick + MEDIC_FORCE_FOLLOWUP_TICKS { continue; }
                let Some(medic_state) = player_state_before(scan, *medic, *tick).or_else(|| players.get(medic)) else { continue };
                if canonical_team_value(medic_state.get("team")) == death.victim_team
                    && medic_state.get("class").and_then(Value::as_str).map(canonical_class).as_deref() == Some("medic")
                    && hurts.iter().any(|(hurt_tick, attacker, victim, _, _)| *attacker == death.attacker && (*victim == *medic || *victim == *target)
                        && *hurt_tick <= *tick && *tick - *hurt_tick <= MEDIC_FORCE_PRESSURE_TICKS) {
                    death.state.enemy_medic_force_followups.push(json!({"event_tick":tick,"medic_user_id":medic,"target_user_id":target,"seconds_after_kill":(*tick-death.event_tick) as f64/TICKS_PER_SECOND,"direct_candidate_pressure":true}));
                }
            }
        }
        });
        Ok(())
    })
}

fn classify_mode(header: &Value, manifest: &Value, events: &[EventRecord], samples: &[(i64, HashMap<String, HashMap<String, usize>>)]) -> (String, String, String, Vec<String>) {
    let server = header.get("server").and_then(Value::as_str).unwrap_or_default().to_lowercase();
    let mut text = manifest.get("mode_signals").and_then(Value::as_array).into_iter().flatten()
        .filter_map(Value::as_str).fold(server.clone(), |mut text, value| { text.push(' '); text.push_str(&value.to_ascii_lowercase()); text });
    text = events
        .iter()
        .filter(|event| matches!(event.event_type.as_str(), "player_say" | "say_text" | "server_message" | "server_cvar" | "teamplay_broadcast_audio"))
        .fold(text, |mut text, event| {
            text.push(' ');
            text.push_str(&serde_json::to_string(&event.event).unwrap_or_default().to_lowercase());
            text
        });
    let rgl = ["rgl_", "rgl.gg", "rgl cfg", "rgl config", "rgl_whitelist"].iter().any(|token| text.contains(token));
    let tournament = rgl || ["mp_tournament", "tf_tournament", "whitelist", "etf2l", "ugc_", "ozfortress", "competitive"].iter().any(|token| text.contains(token));
    let explicit_highlander = text.contains("highlander") || text.contains("rgl_hl") || text.contains("rgl-hl");
    let explicit_sixes = text.contains("6v6") || text.contains("sixes") || text.contains("rgl_6s") || text.contains("rgl-6s");
    let valve = ["valve", "matchmaking", "casual", "quickplay"].iter().any(|token| text.contains(token))
        || header.get("server").and_then(Value::as_str).is_some_and(|server| server.to_ascii_lowercase().contains("valve"));
    let stv = manifest.pointer("/demo_capture/classification").and_then(Value::as_str).is_some_and(|value| value.eq_ignore_ascii_case("stv"));
    let mut sixes = 0usize;
    let mut highlander = 0usize;
    let mut small = 0usize;
    let mut observed = 0usize;
    for (_, teams) in samples {
        let Some(red) = teams.get("red") else { continue };
        let Some(blu) = teams.get("blu") else { continue };
        observed += 1;
        let sizes = [red.values().sum::<usize>(), blu.values().sum::<usize>()];
        if sizes.iter().all(|size| (5..=10).contains(size)) {
            small += 1;
        }
        if [red, blu].iter().all(|roster| sixes_fit(roster)) {
            sixes += 1;
        }
        if [red, blu].iter().all(|roster| roster.values().sum::<usize>() == 9 && roster.len() >= 8) {
            highlander += 1;
        }
    }
    let ratio = |count| if observed == 0 { 0.0 } else { count as f64 / observed as f64 };
    let minimum_samples = if observed < 3 { 1 } else { 3 };
    if explicit_highlander || (highlander >= minimum_samples && ratio(highlander) >= 0.35) {
        return (
            if rgl { "rgl_highlander" } else { "highlander" }.into(),
            if rgl { "RGL Highlander" } else { "Highlander Competitive" }.into(),
            if explicit_highlander || (tournament && ratio(highlander) >= 0.60) { "high" } else { "medium" }.into(),
            vec![format!("{highlander} of {observed} sustained roster samples matched Highlander"), format!("explicit Highlander config evidence: {explicit_highlander}"), format!("competitive config evidence: {tournament}"), format!("SourceTV: {stv}")],
        );
    }
    if explicit_sixes || (sixes >= minimum_samples && ratio(sixes) >= 0.35) {
        return (
            if rgl { "rgl_6v6" } else { "6v6" }.into(),
            if rgl { "RGL 6v6" } else { "6v6 Competitive" }.into(),
            if explicit_sixes || (tournament && ratio(sixes) >= 0.60) { "high" } else { "medium" }.into(),
            vec![format!("{sixes} of {observed} sustained roster samples matched tolerant 6v6"), format!("explicit 6v6 config evidence: {explicit_sixes}"), format!("competitive config evidence: {tournament}"), format!("SourceTV: {stv}")],
        );
    }
    if rgl {
        return ("rgl_competitive".into(), "RGL Competitive".into(), "medium".into(), vec!["RGL signature recorded; format uncertain".into()]);
    }
    if tournament || (small >= minimum_samples && ratio(small) >= 0.60) {
        return ("competitive_uncertain".into(), "Competitive — Format Uncertain".into(), "medium".into(), vec![format!("{small} of {observed} samples had stable small-team rosters")]);
    }
    if valve {
        return ("valve_public".into(), "Valve Public".into(), "medium".into(), vec!["Valve/matchmaking signature recorded".into()]);
    }
    if !server.is_empty() {
        return ("community_public".into(), "Community Public".into(), "medium".into(), vec!["Community server without sustained competitive roster evidence".into()]);
    }
    ("unknown".into(), "Unknown / Mixed".into(), "low".into(), Vec::new())
}

fn sixes_fit(roster: &HashMap<String, usize>) -> bool {
    if roster.values().sum::<usize>() != 6 {
        return false;
    }
    let expected = [("scout", 2usize), ("soldier", 2), ("demoman", 1), ("medic", 1)];
    let distance = expected.iter().map(|(class, count)| roster.get(*class).copied().unwrap_or_default().abs_diff(*count)).sum::<usize>();
    let offclasses = roster.iter().filter(|(class, _)| !expected.iter().any(|(wanted, _)| wanted == &class.as_str())).map(|(_, count)| count).sum::<usize>();
    distance + offclasses <= 2
}

fn normalized_buildings(events: &[EventRecord], rounds: &[Round]) -> Vec<BuildingEvent> {
    events.iter().filter(|event| matches!(event.event_type.as_str(), "object_destroyed" | "building_destroyed" | "building_destruction"))
        .filter_map(|event| {
            let tick = event.analysis_tick();
            rounds.iter().any(|round| tick >= round.start && tick < round.end).then(|| BuildingEvent {
                tick,
                attacker: event.int(&["attacker", "attacker_user_id"]),
                object_type: event.text(&["objecttype", "object_type", "object"]).to_ascii_lowercase(),
            })
        }).collect()
}

fn normalized_objectives(events: &[EventRecord], rounds: &[Round]) -> Vec<ObjectiveEvent> {
    let mut teams = HashMap::<i64, i64>::new();
    let mut objectives = Vec::new();
    for event in events {
        if matches!(event.event_type.as_str(), "player_team" | "player_spawn") {
            let user = event.int(&["user_id", "userid"]);
            let team = event.int(&["team"]);
            if user > 0 && matches!(team, 2 | 3) { teams.insert(user, team); }
        }
        let kind = match event.event_type.as_str() {
            "teamplay_point_captured" => "point_capture",
            "payload_pushed" => "payload_progress",
            "teamplay_capture_blocked" => "capture_denial",
            _ => continue,
        };
        let tick = event.analysis_tick();
        if !rounds.iter().any(|round| tick >= round.start && tick < round.end) { continue; }
        let actor = if kind == "payload_progress" { event.int(&["pusher"]) } else if kind == "capture_denial" { event.int(&["blocker"]) } else { 0 };
        let team = if actor > 0 { teams.get(&actor).copied().unwrap_or_default() } else { event.int(&["team", "team_id"]) };
        objectives.push(ObjectiveEvent { tick, team, actor_user_id: actor, kind: kind.into(), data: Value::Object(event.event.clone()) });
    }
    objectives
}

fn parse_item_schema(path: &Path) -> Result<HashMap<i64, ItemInfo>> {
    let text = fs::read_to_string(path).with_context(|| format!("could not read item schema {}", path.display()))?;
    let mut result = HashMap::new();
    let mut current: Option<i64> = None;
    let mut depth = 0i32;
    let mut slot = String::new();
    let mut log_name = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if current.is_none() {
            if let Some(value) = quoted_values(line).first().and_then(|value| value.parse::<i64>().ok()) {
                current = Some(value); slot.clear(); log_name.clear(); depth = 0;
            }
        }
        if current.is_some() {
            depth += line.matches('{').count() as i32;
            let values = quoted_values(line);
            if values.len() >= 2 {
                match values[0].as_str() {
                    "item_slot" => slot = values[1].to_ascii_lowercase(),
                    "item_logname" | "name" if log_name.is_empty() => log_name = values[1].trim_start_matches('#').to_ascii_lowercase(),
                    _ => {}
                }
            }
            depth -= line.matches('}').count() as i32;
            if depth <= 0 && line.contains('}') {
                let index = current.take().unwrap();
                result.insert(index, ItemInfo { slot: slot.clone(), log_name: log_name.clone() });
            }
        }
    }
    Ok(result)
}

fn quoted_values(line: &str) -> Vec<String> {
    line.split('"').enumerate().filter_map(|(index, value)| (index % 2 == 1).then(|| value.to_owned())).collect()
}

fn build_candidates(
    deaths: &mut [Death], rounds: &[Round], context: &DemoContext, source_demo: &str, map_name: &str,
    buildings: &[BuildingEvent], objectives: &[ObjectiveEvent], cancelled: Option<&AtomicBool>,
    governor: Option<&RuntimeGovernor>,
) -> Result<(Vec<Candidate>, usize, usize)> {
    deaths.sort_by_key(|death| (death.round_index, death.attacker, death.event_tick, death.packet_sequence, death.event_index));
    let mut buckets: BTreeMap<(i64, i64), Vec<&Death>> = BTreeMap::new();
    for death in deaths.iter() {
        buckets.entry((death.round_index, death.attacker)).or_default().push(death);
    }
    let mut jobs = Vec::<(i64, i64, Vec<Death>, Round)>::new();
    for ((round_index, attacker), kills) in buckets {
        let mut groups: Vec<Vec<&Death>> = Vec::new();
        for kill in kills {
            if groups.last().and_then(|group| group.first()).is_some_and(|first| kill.event_tick - first.event_tick <= SEQUENCE_GAP) {
                groups.last_mut().unwrap().push(kill);
            } else {
                groups.push(vec![kill]);
            }
        }
        for group in groups {
            if let Some(round) = rounds.iter().find(|round| round.index == round_index) {
                jobs.push((round_index, attacker, group.into_iter().cloned().collect(), round.clone()));
            }
        }
    }
    let workers = rayon::current_num_threads().min(jobs.len().max(1)).max(1);
    let score_job = |job: &(i64, i64, Vec<Death>, Round)| -> Result<ScoredGroup> {
        check_runtime(cancelled, governor)?;
        Ok(score_group(&job.2, &job.3, buildings, objectives))
    };
    let scored = if workers > 1 && jobs.len() >= 4 {
        jobs.par_iter().map(score_job).collect::<Result<Vec<_>>>()?
    } else {
        jobs.iter().map(score_job).collect::<Result<Vec<_>>>()?
    };
    let mut candidates = Vec::new();
    for (job, scored) in jobs.iter().zip(scored) {
            let (round_index, attacker, group, round) = job;
            let first = group.first().unwrap();
            let last = group.last().unwrap();
            let kill_values: Vec<Value> = group.iter().map(death_json).collect();
            let point_events = group.iter().map(|kill| json!({
                "tick":kill.demo_tick, "demo_tick":kill.demo_tick, "server_tick":kill.event_tick,
                "packet_sequence":kill.packet_sequence, "event_index_in_packet":kill.event_index,
            })).collect::<Vec<_>>();
            let state_pass = json!({
                "status":if group.iter().any(|kill| !kill.state.attacker.is_empty() && !kill.state.victim.is_empty()) {"complete"} else {"unavailable"},
                "confirmed_airshots":scored.metrics.get("confirmed_airshots"),
                "confirmed_uber_drops":group.iter().filter(|kill| kill.state.confirmed_uber_drop).count(),
                "enemy_alive_before":scored.metrics.get("enemy_alive_before"),
                "enemy_alive_after_sequence":scored.metrics.get("enemy_alive_after_sequence"),
                "medic_force":scored.metrics.get("medic_force"),
                "player_count_swing":scored.metrics.get("player_count_swing"),
                "sack_uber_recovery":scored.metrics.get("sack_uber_recovery"),
            });
            let tick_tags = tick_tag_groups(group);
            let tick_tag_set = tick_tags
                .iter()
                .flat_map(|group| group.tags.iter().cloned())
                .collect::<HashSet<_>>();
            let sequence_tags = scored
                .tags
                .iter()
                .filter(|tag| !tick_tag_set.contains(*tag))
                .cloned()
                .collect::<Vec<_>>();
            let mut candidate = Candidate {
                candidate_id: format!("r{round_index}-p{attacker}-t{}", first.event_tick),
                source_demo: source_demo.into(),
                map_name: map_name.into(),
                round_index: *round_index,
                overall_score: scored.score,
                attacker_user_id: *attacker,
                attacker_class: first.attacker_class.clone(),
                attacker_team: first.attacker_team.clone(),
                clip_start_tick: (first.demo_tick - PRE_ROLL).max(0),
                clip_end_tick: last.demo_tick + POST_ROLL,
                point_of_kill_ticks: group.iter().map(|kill| kill.demo_tick).collect(),
                tags: scored.tags,
                tick_tags,
                sequence_tags,
                metrics: scored.metrics,
                kills: kill_values,
                score_breakdown: scored.breakdown,
                demo_context: context.clone(),
                extra: Map::from_iter([
                    ("live_round".into(), Value::Bool(true)),
                    ("attacker_name".into(), first.state.attacker.get("name").cloned().unwrap_or(Value::Null)),
                    ("attacker_steam_id".into(), first.state.attacker.get("steam_id").cloned().unwrap_or(Value::Null)),
                    ("start_tick".into(), json!((first.demo_tick - PRE_ROLL).max(0))),
                    ("end_tick".into(), json!(last.demo_tick + POST_ROLL)),
                    ("first_kill_tick".into(), json!(first.demo_tick)),
                    ("last_kill_tick".into(), json!(last.demo_tick)),
                    ("first_kill_server_tick".into(), json!(first.event_tick)),
                    ("last_kill_server_tick".into(), json!(last.event_tick)),
                    ("point_of_kill_server_ticks".into(), json!(group.iter().map(|kill| kill.event_tick).collect::<Vec<_>>())),
                    ("point_of_kill_events".into(), Value::Array(point_events)),
                    ("round_state".into(), json!({
                        "classification":"live","start_tick":round.start,"start_event":round.start_event,
                        "round_active_tick":round.round_active_tick,"setup_finished_tick":round.setup_finished_tick,
                        "activation_trigger":round.activation_trigger,
                        "ready_up":{
                            "red_ready_tick":round.red_ready_tick,"blu_ready_tick":round.blu_ready_tick,
                            "both_teams_ready":round.ready_up,
                            "both_teams_ready_tick":match (round.red_ready_tick,round.blu_ready_tick) { (Some(red),Some(blu))=>Some(red.max(blu)), _=>None },
                            "ready_restart_tick":round.ready_restart_tick,"countdown_tick":round.countdown_tick,
                        },
                        "end_tick":round.end,"end_event":round.end_event,"winning_team":round.winning_team
                    })),
                    ("objective_followups".into(), Value::Array(scored.objective_followups)),
                    ("building_destructions".into(), Value::Array(scored.building_followups)),
                    ("state_pass".into(), state_pass),
                ]),
                ..Candidate::default()
            };
            candidate.primary_tag = candidate.inferred_primary_tag();
            candidates.push(candidate);
    }
    Ok((candidates, jobs.len(), if workers > 1 && jobs.len() >= 4 { workers } else { 1 }))
}

struct ScoredGroup { score: f64, tags: Vec<String>, metrics: Value, breakdown: Vec<Value>, objective_followups: Vec<Value>, building_followups: Vec<Value> }

fn tick_tag_groups(group: &[Death]) -> Vec<TickTagGroup> {
    let mut grouped = BTreeMap::<i64, (BTreeSet<i64>, BTreeSet<String>)>::new();
    for kill in group {
        let entry = grouped.entry(kill.demo_tick).or_default();
        entry.0.insert(kill.event_tick);
        entry.1.extend(kill_tags(kill));
    }
    grouped
        .into_iter()
        .map(|(demo_tick, (server_ticks, tags))| TickTagGroup {
            demo_tick,
            server_ticks: server_ticks.into_iter().collect(),
            tags: tags.into_iter().collect(),
        })
        .collect()
}

fn kill_tags(kill: &Death) -> BTreeSet<String> {
    let mut tags = BTreeSet::new();
    let weapon = kill.weapon.to_ascii_lowercase();
    if matches!(weapon.as_str(), "rocketlauncher" | "directhit" | "blackbox" | "liberty_launcher" | "airstrike" | "grenadelauncher" | "loch_n_load" | "iron_bomber" | "stickybomb_launcher" | "quickiebomb_launcher" | "flaregun" | "detonator" | "scorch_shot" | "compound_bow" | "crusaders_crossbow" | "syringegun_medic" | "rescue_ranger" | "righteous_bison" | "loose_cannon" | "loose_cannon_impact" | "loose_cannon_explosion") {
        tags.insert("projectile_kill".into());
    }
    if matches!(weapon.as_str(), "grenadelauncher" | "loch_n_load" | "iron_bomber") { tags.insert("pipe".into()); }
    if matches!(weapon.as_str(), "rocketlauncher" | "directhit" | "blackbox" | "liberty_launcher" | "airstrike") { tags.insert("rocket".into()); }
    if matches!(weapon.as_str(), "compound_bow" | "huntsman") { tags.insert("huntsman".into()); }
    if matches!(weapon.as_str(), "crusaders_crossbow" | "crossbow") { tags.insert("crossbow".into()); }
    if let Some(tag) = match weapon.as_str() {
        "market_gardener" => Some("market_gardener"), "axtinguisher" => Some("axtinguisher"),
        "backburner" => Some("backburner"), "ambassador" => Some("ambassador"), "kunai" => Some("kunai"),
        "eternal_reward" => Some("eternal_reward"), "tribalkukri" => Some("tribalman's_shiv"), _ => None,
    } { tags.insert(tag.into()); }

    if kill.state.confirmed_kritzkrieg_boost && kill.crit_type > 0 { tags.insert("kritzkrieg_kill".into()); }
    let market = weapon == "market_gardener"
        && kill.crit_type > 0
        && kill.state.attacker.get("blast_jumping").and_then(Value::as_bool).unwrap_or(false);
    if market { tags.insert("market_garden".into()); }
    if kill.state.confirmed_double_donk { tags.insert("double_donk".into()); }
    let velocity_z = vector3(kill.state.attacker.get("velocity")).map(|value| value[2]).unwrap_or_default();
    if kill.attacker_class == "sniper"
        && matches!(weapon.as_str(), "sniperrifle" | "sniperrifle_classic" | "sniperrifle_decap")
        && kill.state.attacker.get("scoped").and_then(Value::as_bool).unwrap_or(false)
        && kill.state.attacker.get("on_ground").and_then(Value::as_bool) == Some(false)
        && velocity_z < -20.0
    {
        tags.insert("sniper_dropshot".into());
    }
    let shield_bash = kill.custom_kill == 23;
    let charge_melee = !shield_bash
        && kill.attacker_class == "demoman"
        && is_melee(kill)
        && kill.state.attacker_recent_shield_charge_tick.is_some();
    if shield_bash {
        tags.extend(["demoknight".into(), "shield_bash_kill".into()]);
    } else if charge_melee {
        tags.extend(["demoknight".into(), "charge_melee_kill".into()]);
    }
    let backstab = kill.custom_kill == 2;
    let taunt = taunt_kill_name(kill.custom_kill);
    if is_melee(kill) && !backstab && taunt.is_none() && !shield_bash && !charge_melee && !market {
        tags.insert("melee_kill".into());
    }
    if backstab { tags.insert("backstab".into()); }
    if taunt.is_some() { tags.insert("taunt_kill".into()); }
    if kill.victim_class == "medic" {
        tags.insert("medic_pick".into());
        if kill.state.confirmed_uber_drop { tags.insert("uber_drop".into()); }
    }
    if kill.victim_class == "demoman" { tags.insert("demoman_pick".into()); }

    let victim_airborne = kill.state.victim_airborne_before_projectile_impact;
    let projectile_weapon = ["rocket", "directhit", "blackbox", "grenade", "loch", "iron_bomber", "loose_cannon", "flare", "huntsman", "crossbow"]
        .iter().any(|name| weapon.contains(name));
    let confirmed_airshot = victim_airborne
        && kill.state.projectile.as_ref().and_then(|value| value.get("airshot_eligible")).and_then(Value::as_bool).unwrap_or(false);
    if confirmed_airshot {
        tags.insert("confirmed_airshot".into());
        if let Some(projectile) = kill.state.projectile.as_ref() {
            if projectile.get("impact_proximity").and_then(Value::as_str) == Some("direct") { tags.insert("direct_airshot".into()); }
            if projectile.get("flight_seconds").and_then(Value::as_f64).unwrap_or_default() >= 0.5 { tags.insert("long_flight_airshot".into()); }
        }
        if let Some((relative_height, _, elevation_degrees)) = airshot_geometry(&kill.state.attacker, &kill.state.victim) {
            if airshot_style_bonus(relative_height, elevation_degrees) > 0.0 {
                if relative_height >= 128.0 { tags.insert("high_airshot".into()); }
                if elevation_degrees >= 35.0 { tags.insert("skyward_airshot".into()); }
                if relative_height >= 256.0 && elevation_degrees >= 50.0 { tags.insert("extreme_airshot".into()); }
            }
        }
    } else if victim_airborne && projectile_weapon {
        tags.insert("airborne_projectile_kill".into());
    } else if kill.rocket_jump_victim {
        tags.insert("rocket_jump_victim".into());
    }
    if kill.kill_streak_total >= 10 { tags.insert("streak_10_plus".into()); }
    if kill.crit_type == 2
        && !charge_melee
        && !kill.state.confirmed_kritzkrieg_boost
        && weapon != "market_gardener"
        && !backstab
    {
        tags.insert("random_full_crit".into());
    }
    tags
}

fn score_group(group: &[Death], round: &Round, buildings: &[BuildingEvent], objectives: &[ObjectiveEvent]) -> ScoredGroup {
    let mut score = 10.0;
    let mut tags = HashSet::<String>::new();
    let mut breakdown = vec![json!({"reason":"candidate_base","points":10.0})];
    let mut confirmed_airshots = 0usize;
    let mut medic_kills = 0usize;
    let mut demoman_kills = 0usize;
    for kill in group {
        let weapon = kill.weapon.to_lowercase();
        if matches!(weapon.as_str(), "rocketlauncher" | "directhit" | "blackbox" | "liberty_launcher" | "airstrike" | "grenadelauncher" | "loch_n_load" | "iron_bomber" | "stickybomb_launcher" | "quickiebomb_launcher" | "flaregun" | "detonator" | "scorch_shot" | "compound_bow" | "crusaders_crossbow" | "syringegun_medic" | "rescue_ranger" | "righteous_bison" | "loose_cannon" | "loose_cannon_impact" | "loose_cannon_explosion") {
            tags.insert("projectile_kill".into());
        }
        if matches!(weapon.as_str(), "grenadelauncher" | "loch_n_load" | "iron_bomber") { tags.insert("pipe".into()); }
        if matches!(weapon.as_str(), "rocketlauncher" | "directhit" | "blackbox" | "liberty_launcher" | "airstrike") { tags.insert("rocket".into()); }
        if matches!(weapon.as_str(), "compound_bow" | "huntsman") { tags.insert("huntsman".into()); }
        if matches!(weapon.as_str(), "crusaders_crossbow" | "crossbow") { tags.insert("crossbow".into()); }
        if let Some(tag) = match weapon.as_str() {
            "market_gardener" => Some("market_gardener"), "axtinguisher" => Some("axtinguisher"),
            "backburner" => Some("backburner"), "ambassador" => Some("ambassador"), "kunai" => Some("kunai"),
            "eternal_reward" => Some("eternal_reward"), "tribalkukri" => Some("tribalman's_shiv"), _ => None,
        } { tags.insert(tag.into()); }
        let victim_airborne = kill.state.victim_airborne_before_projectile_impact;
        let projectile_weapon = ["rocket", "directhit", "blackbox", "grenade", "loch", "iron_bomber", "loose_cannon", "flare", "huntsman", "crossbow"]
            .iter().any(|name| weapon.contains(name));
        if kill.state.confirmed_kritzkrieg_boost && kill.crit_type > 0 {
            score += 8.0; tags.insert("kritzkrieg_kill".into()); breakdown.push(json!({"reason":"confirmed_kritzkrieg_boosted_kill","points":8.0,"event_tick":kill.event_tick}));
        }
        let market = weapon == "market_gardener" && kill.crit_type > 0 && kill.state.attacker.get("blast_jumping").and_then(Value::as_bool).unwrap_or(false);
        if market {
            score += 20.0; tags.insert("market_garden".into()); breakdown.push(json!({"reason":"confirmed_market_garden","points":20.0,"event_tick":kill.event_tick}));
        }
        if kill.state.confirmed_double_donk {
            score += 18.0; tags.insert("double_donk".into()); breakdown.push(json!({"reason":"confirmed_loose_cannon_double_donk","points":18.0,"event_tick":kill.event_tick}));
        }
        let velocity_z = vector3(kill.state.attacker.get("velocity")).map(|value| value[2]).unwrap_or_default();
        if kill.attacker_class == "sniper" && matches!(weapon.as_str(), "sniperrifle" | "sniperrifle_classic" | "sniperrifle_decap") && kill.state.attacker.get("scoped").and_then(Value::as_bool).unwrap_or(false)
            && kill.state.attacker.get("on_ground").and_then(Value::as_bool) == Some(false) && velocity_z < -20.0 {
            score += 18.0; tags.insert("sniper_dropshot".into()); breakdown.push(json!({"reason":"confirmed_sniper_dropshot","points":18.0,"event_tick":kill.event_tick}));
        }
        let shield_bash = kill.custom_kill == 23;
        let charge_melee = !shield_bash && kill.attacker_class == "demoman" && is_melee(kill) && kill.state.attacker_recent_shield_charge_tick.is_some();
        if shield_bash {
            score += 22.0; tags.extend(["demoknight".into(), "shield_bash_kill".into()]); breakdown.push(json!({"reason":"confirmed_shield_bash_kill","points":22.0,"event_tick":kill.event_tick}));
        } else if charge_melee {
            score += 16.0; tags.extend(["demoknight".into(), "charge_melee_kill".into()]); breakdown.push(json!({"reason":"shield_charge_followed_by_melee_kill","points":16.0,"event_tick":kill.event_tick,"weapon":kill.weapon,"weapon_def_index":kill.weapon_def_index}));
        }
        let backstab = kill.custom_kill == 2;
        let taunt = taunt_kill_name(kill.custom_kill);
        if is_melee(kill) && !backstab && taunt.is_none() && !shield_bash && !charge_melee && !market {
            score += 15.0; tags.insert("melee_kill".into()); breakdown.push(json!({"reason":"player_melee_kill","points":15.0,"event_tick":kill.event_tick,"weapon":kill.weapon,"weapon_def_index":kill.weapon_def_index}));
        }
        if backstab {
            score += 20.0; tags.insert("backstab".into()); breakdown.push(json!({"reason":"confirmed_spy_backstab","points":20.0,"event_tick":kill.event_tick}));
        }
        if let Some(taunt) = taunt {
            score += 25.0; tags.insert("taunt_kill".into()); breakdown.push(json!({"reason":"confirmed_taunt_kill","points":25.0,"event_tick":kill.event_tick,"taunt":taunt}));
        }
        if kill.victim_class == "medic" {
            medic_kills += 1; score += 18.0; tags.insert("medic_pick".into()); breakdown.push(json!({"reason":"medic_pick","points":18.0,"event_tick":kill.event_tick}));
            if kill.state.confirmed_uber_drop { score += 20.0; tags.insert("uber_drop".into()); breakdown.push(json!({"reason":"confirmed_uber_drop","points":20.0,"event_tick":kill.event_tick,"charge":kill.state.victim.get("medic_charge")})); }
        }
        if kill.victim_class == "demoman" { demoman_kills += 1; score += 10.0; tags.insert("demoman_pick".into()); breakdown.push(json!({"reason":"demoman_pick","points":10.0,"event_tick":kill.event_tick})); }
        let confirmed_airshot = victim_airborne && kill.state.projectile.as_ref().and_then(|value| value.get("airshot_eligible")).and_then(Value::as_bool).unwrap_or(false);
        if confirmed_airshot {
            confirmed_airshots += 1; score += 20.0; tags.insert("confirmed_airshot".into()); breakdown.push(json!({"reason":"state_confirmed_airshot","points":20.0,"event_tick":kill.event_tick,"projectile":kill.state.projectile}));
            let projectile = kill.state.projectile.as_ref().unwrap();
            if projectile.get("impact_proximity").and_then(Value::as_str) == Some("direct") { score += 6.0; tags.insert("direct_airshot".into()); breakdown.push(json!({"reason":"direct_airshot_proximity","points":6.0,"event_tick":kill.event_tick,"distance":projectile.get("distance_to_victim")})); }
            if projectile.get("flight_seconds").and_then(Value::as_f64).unwrap_or_default() >= 0.5 { score += 5.0; tags.insert("long_flight_airshot".into()); breakdown.push(json!({"reason":"long_flight_airshot","points":5.0,"event_tick":kill.event_tick,"flight_seconds":projectile.get("flight_seconds")})); }
            if let Some((relative_height, horizontal_distance, elevation_degrees)) =
                airshot_geometry(&kill.state.attacker, &kill.state.victim)
            {
                let style_points = airshot_style_bonus(relative_height, elevation_degrees);
                if style_points > 0.0 {
                    score += style_points;
                    if relative_height >= 128.0 { tags.insert("high_airshot".into()); }
                    if elevation_degrees >= 35.0 { tags.insert("skyward_airshot".into()); }
                    if relative_height >= 256.0 && elevation_degrees >= 50.0 {
                        tags.insert("extreme_airshot".into());
                    }
                    breakdown.push(json!({
                        "reason":"relative_height_airshot_style",
                        "points":style_points,
                        "event_tick":kill.event_tick,
                        "relative_height_units":(relative_height*100.0).round()/100.0,
                        "horizontal_distance_units":(horizontal_distance*100.0).round()/100.0,
                        "upward_elevation_degrees":(elevation_degrees*100.0).round()/100.0
                    }));
                }
            }
        } else if victim_airborne && projectile_weapon {
            score += 8.0; tags.insert("airborne_projectile_kill".into()); breakdown.push(json!({"reason":"state_confirmed_airborne_victim","points":8.0,"event_tick":kill.event_tick}));
        } else if kill.rocket_jump_victim {
            score += 10.0; tags.insert("rocket_jump_victim".into()); breakdown.push(json!({"reason":"rocket_jump_victim","points":10.0,"event_tick":kill.event_tick}));
        }
        if kill.kill_streak_total >= 10 { score += 5.0; tags.insert("streak_10_plus".into()); breakdown.push(json!({"reason":"streak_10_plus","points":5.0,"event_tick":kill.event_tick})); }
        if kill.crit_type == 2 && !charge_melee && !kill.state.confirmed_kritzkrieg_boost && weapon != "market_gardener" && !backstab {
            score -= 12.0; tags.insert("random_full_crit".into()); breakdown.push(json!({"reason":"random_full_crit","points":-12.0,"event_tick":kill.event_tick}));
        }
    }
    let unique_victims = group.iter().map(|kill| kill.victim).collect::<HashSet<_>>().len();
    if unique_victims > 1 { let points = 18.0 * (unique_victims-1) as f64; score += points; tags.insert("multi_kill".into()); breakdown.push(json!({"reason":"additional_kills","points":points,"count":unique_victims-1})); }
    if unique_victims >= 3 { score += 15.0; tags.insert("three_kill".into()); breakdown.push(json!({"reason":"three_kill","points":15.0})); }
    if unique_victims >= 4 { score += 25.0; tags.insert("four_kill_plus".into()); breakdown.push(json!({"reason":"four_kill_plus","points":25.0})); }
    if confirmed_airshots >= 2 { score += 15.0; tags.insert("double_airshot_sequence".into()); breakdown.push(json!({"reason":"multiple_confirmed_airshots","points":15.0,"count":confirmed_airshots})); }
    let first = group.first().unwrap();
    let last = group.last().unwrap();
    let enemy_after = first.state.enemy_alive_before.saturating_sub(unique_victims);
    if first.state.enemy_state_roster >= 4 && first.state.enemy_alive_before > 0 && unique_victims >= first.state.enemy_alive_before {
        score += 18.0; tags.insert("team_wipe".into()); if first.state.enemy_alive_before == 1 { tags.insert("last_enemy_alive".into()); } breakdown.push(json!({"reason":"sequence_finished_enemy_team","points":18.0,"enemy_alive_before":first.state.enemy_alive_before}));
    }
    let mut force_followups = BTreeMap::<(i64, i64), Value>::new();
    for followup in group.iter().flat_map(|kill| &kill.state.enemy_medic_force_followups) {
        let tick = followup.get("event_tick").and_then(Value::as_i64).unwrap_or_default();
        let medic = followup.get("medic_user_id").and_then(Value::as_i64).unwrap_or_default();
        if tick >= last.event_tick && tick <= last.event_tick + MEDIC_FORCE_FOLLOWUP_TICKS {
            force_followups.insert((tick, medic), followup.clone());
        }
    }
    let medic_force = !force_followups.is_empty();
    let force_followups = force_followups.into_values().collect::<Vec<_>>();
    if medic_force { score += 16.0; tags.insert("medic_force".into()); breakdown.push(json!({"reason":"enemy_medic_forced_uber_after_sequence","points":16.0,"force_events":force_followups.clone()})); }
    let earliest_enemy_respawn_tick = group.iter().filter_map(|kill| kill.state.victim_next_respawn_tick)
        .filter(|tick| *tick > last.event_tick).min().unwrap_or(round.end);
    let player_advantage_window_ticks = earliest_enemy_respawn_tick.saturating_sub(last.event_tick);
    let friendly_respawns_before_enemy = last.state.friendly_pending_respawn_ticks.iter().copied()
        .filter(|tick| *tick > last.event_tick && *tick <= earliest_enemy_respawn_tick).collect::<BTreeSet<_>>();
    let erased_player_disadvantage = first.state.friendly_alive_before > 0
        && first.state.enemy_alive_before >= first.state.friendly_alive_before + 2
        && enemy_after <= first.state.friendly_alive_before;
    let player_swing = erased_player_disadvantage && !medic_force && player_advantage_window_ticks >= PLAYER_SWING_MIN_WINDOW_TICKS;
    if player_swing { score += 16.0; tags.insert("player_count_swing".into()); breakdown.push(json!({"reason":"sequence_created_player_count_window","points":16.0,"friendly_alive_before":first.state.friendly_alive_before,"enemy_alive_before":first.state.enemy_alive_before,"enemy_alive_after":enemy_after,"window_seconds":player_advantage_window_ticks as f64/TICKS_PER_SECOND,"earliest_enemy_respawn_tick":earliest_enemy_respawn_tick,"friendly_respawns_before_enemy":friendly_respawns_before_enemy.clone()})); }
    let enemy_uber_advantage = first.state.enemy_uber_advantage_before;
    let sack = first.state.recent_friendly_deaths >= 2 && enemy_uber_advantage && (player_swing || medic_kills > 0);
    if sack { score += 16.0; tags.insert("sack_uber_recovery".into()); breakdown.push(json!({"reason":"sack_uber_recovery_after_losses","points":16.0,"recent_friendly_deaths":first.state.recent_friendly_deaths,"player_disadvantage_before":first.state.player_disadvantage_before,"window_seconds":10.0,"death_ticks":first.state.recent_friendly_death_ticks,"friendly_medic_charge":first.state.friendly_medic_charge,"enemy_medic_charge":first.state.enemy_medic_charge})); if medic_kills > 0 { score += 12.0; tags.insert("sack_uber_medic_equalizer".into()); breakdown.push(json!({"reason":"sack_uber_medic_equalizer","points":12.0})); } }
    let duration = (last.event_tick-first.event_tick).max(0) as f64/TICKS_PER_SECOND;
    if unique_victims >= 2 && duration <= 2.0 { score += 12.0; tags.insert("rapid_sequence".into()); breakdown.push(json!({"reason":"rapid_sequence","points":12.0})); }
    if group.iter().any(|kill| ["rocket", "grenade", "flare", "huntsman", "crossbow", "loose_cannon"].iter().any(|name| kill.weapon.contains(name))) { score += 8.0; breakdown.push(json!({"reason":"projectile_sequence","points":8.0})); }
    if round.end-last.event_tick <= (TICKS_PER_SECOND*8.0) as i64 { score += 8.0; tags.insert("late_round".into()); breakdown.push(json!({"reason":"late_round","points":8.0})); }
    let attacker_team_id = if first.attacker_team == "red" { 2 } else if first.attacker_team == "blu" { 3 } else { 0 };
    if attacker_team_id > 0 && attacker_team_id == round.winning_team && round.end-last.event_tick <= ROUND_CLINCH_TICKS { score += 12.0; tags.insert("round_clinch".into()); breakdown.push(json!({"reason":"team_won_immediately_after_sequence","points":12.0,"event_tick":round.end})); }
    let building_followups = buildings.iter().filter(|building| building.attacker == first.attacker && building.tick <= first.event_tick && building.tick >= first.event_tick-(TICKS_PER_SECOND*2.0) as i64)
        .map(|building| json!({"event_tick":building.tick,"attacker_user_id":building.attacker,"object_type":building.object_type})).collect::<Vec<_>>();
    if !building_followups.is_empty() { score += 6.0; tags.insert("building_to_kill_sequence".into()); breakdown.push(json!({"reason":"building_destruction_led_to_kills","points":6.0,"count":building_followups.len()})); }
    let objective_followups = objectives.iter().filter(|objective| objective.tick >= last.event_tick && objective.tick <= last.event_tick+OBJECTIVE_CONVERSION_TICKS && objective.team == attacker_team_id)
        .map(|objective| json!({"event_tick":objective.tick,"kind":objective.kind,"actor_user_id":objective.actor_user_id,"data":objective.data})).collect::<Vec<_>>();
    let point = objective_followups.iter().find(|item| item.get("kind").and_then(Value::as_str) == Some("point_capture"));
    let denial = objective_followups.iter().find(|item| item.get("kind").and_then(Value::as_str) == Some("capture_denial")
        && item.get("actor_user_id").and_then(Value::as_i64) == Some(first.attacker)
        && item.get("event_tick").and_then(Value::as_i64).unwrap_or_default()-last.event_tick <= CAPTURE_DENIAL_TICKS);
    let payload = objective_followups.iter().find(|item| item.get("kind").and_then(Value::as_str) == Some("payload_progress"));
    let mut objective_kind = "";
    let mut objective_score = 0.0;
    if let Some(item) = point { objective_kind="kills_to_secure_cap"; objective_score=24.0; score+=objective_score; tags.insert("kills_to_secure_cap".into()); breakdown.push(json!({"reason":"kills_to_secure_cap","points":objective_score,"event_tick":item.get("event_tick"),"evidence":item})); }
    else if let Some(item) = denial { objective_kind="capture_denial"; objective_score=20.0; score+=objective_score; tags.insert("capture_denial_followup".into()); breakdown.push(json!({"reason":"kill_sequence_blocked_capture","points":objective_score,"event_tick":item.get("event_tick"),"evidence":item})); }
    else if let Some(item) = payload {
        objective_kind="payload_progress";
        let pusher = item.get("actor_user_id").and_then(Value::as_i64).unwrap_or_default();
        objective_score=if pusher == first.attacker {16.0} else {12.0};
        score+=objective_score; tags.insert("payload_progress_followup".into());
        if pusher == first.attacker { tags.insert("payload_pusher".into()); }
        breakdown.push(json!({"reason":"kill_sequence_led_to_payload_progress","points":objective_score,"event_tick":item.get("event_tick"),"pusher_user_id":pusher,"evidence":item}));
    }
    let mut tags = tags.into_iter().collect::<Vec<_>>(); tags.sort();
    let raw_score = score;
    let score = (score.max(0.0)*100.0).round()/100.0;
    let unique_weapons = group.iter().filter_map(|kill| (!kill.weapon.is_empty()).then(|| kill.weapon.clone())).collect::<BTreeSet<_>>();
    let projectile_kills = group.iter().filter(|kill| ["rocket", "grenade", "loch", "iron_bomber", "loose_cannon", "flare", "huntsman", "crossbow"].iter().any(|name| kill.weapon.contains(name))).count();
    let metrics = json!({
        "kills":group.len(), "unique_victims":unique_victims, "duration_seconds":duration,
        "unique_weapons":unique_weapons, "projectile_kills":projectile_kills,
        "melee_kills":group.iter().filter(|kill| is_melee(kill) && kill.custom_kill != 2 && taunt_kill_name(kill.custom_kill).is_none() && kill.custom_kill != 23).count(),
        "backstab_kills":group.iter().filter(|kill| kill.custom_kill == 2).count(),
        "taunt_kills":group.iter().filter(|kill| taunt_kill_name(kill.custom_kill).is_some()).count(),
        "shield_bash_kills":group.iter().filter(|kill| kill.custom_kill == 23).count(),
        "charge_melee_kills":group.iter().filter(|kill| kill.custom_kill != 23 && kill.attacker_class == "demoman" && is_melee(kill) && kill.state.attacker_recent_shield_charge_tick.is_some()).count(),
        "medic_kills":medic_kills, "demoman_kills":demoman_kills,
        "full_crit_kills":group.iter().filter(|kill| kill.crit_type == 2).count(),
        "kritzkrieg_kills":group.iter().filter(|kill| kill.state.confirmed_kritzkrieg_boost && kill.crit_type > 0).count(),
        "market_gardens":group.iter().filter(|kill| kill.weapon == "market_gardener" && kill.crit_type > 0 && kill.state.attacker.get("blast_jumping").and_then(Value::as_bool).unwrap_or(false)).count(),
        "double_donks":group.iter().filter(|kill| kill.state.confirmed_double_donk).count(),
        "confirmed_airshots":confirmed_airshots,
        "direct_airshots":group.iter().filter(|kill| {
            kill.state.victim_airborne_before_projectile_impact
                && kill.state.projectile.as_ref().and_then(|value| value.get("airshot_eligible")).and_then(Value::as_bool).unwrap_or(false)
                && kill.state.projectile.as_ref().and_then(|value| value.get("impact_proximity")).and_then(Value::as_str) == Some("direct")
        }).count(),
        "airborne_projectile_kills":group.iter().filter(|kill| {
            kill.state.victim_airborne_before_projectile_impact
                && matches!(kill.weapon.as_str(), "rocketlauncher" | "directhit" | "blackbox" | "liberty_launcher" | "airstrike" | "grenadelauncher" | "loch_n_load" | "iron_bomber" | "loose_cannon" | "loose_cannon_impact" | "loose_cannon_explosion" | "flaregun" | "detonator" | "scorch_shot" | "compound_bow" | "huntsman")
        }).count(),
        "confirmed_uber_drops":group.iter().filter(|kill| kill.state.confirmed_uber_drop).count(),
        "friendly_alive_before":first.state.friendly_alive_before,"enemy_alive_before":first.state.enemy_alive_before,"enemy_alive_after_sequence":enemy_after,
        "friendly_state_roster":first.state.friendly_state_roster,"enemy_state_roster":first.state.enemy_state_roster,
        "player_advantage_window_seconds":player_advantage_window_ticks as f64/TICKS_PER_SECOND,
        "earliest_enemy_respawn_tick":earliest_enemy_respawn_tick,
        "friendly_respawns_before_enemy":friendly_respawns_before_enemy,
        "medic_force":medic_force, "medic_force_followups":force_followups,
        "recent_friendly_deaths_before":first.state.recent_friendly_deaths,
        "player_disadvantage_before":first.state.player_disadvantage_before,
        "enemy_uber_advantage_before":enemy_uber_advantage,
        "player_count_swing":player_swing,"sack_uber_recovery":sack,"sack_uber_medic_equalizer":sack && medic_kills > 0,
        "first_kill_tick":first.event_tick,"last_kill_tick":last.event_tick,
        "score_before_floor":raw_score,"score_floor_applied":raw_score < 0.0,
        "linked_building_destructions":building_followups.len(),
        "objective_followups":objective_followups.len(),
        "point_capture_followups":objective_followups.iter().filter(|item| item.get("kind").and_then(Value::as_str) == Some("point_capture")).count(),
        "payload_progress_followups":objective_followups.iter().filter(|item| item.get("kind").and_then(Value::as_str) == Some("payload_progress")).count(),
        "capture_denial_followups":objective_followups.iter().filter(|item| item.get("kind").and_then(Value::as_str) == Some("capture_denial")).count(),
        "objective_followup_evidence":objective_followups,
        "objective_conversion_kind":objective_kind,"objective_score":objective_score,"live_round":true
    });
    ScoredGroup { score, tags, metrics, breakdown, objective_followups, building_followups }
}

fn is_melee(kill: &Death) -> bool {
    if !kill.weapon_slot.is_empty() { return kill.weapon_slot.eq_ignore_ascii_case("melee"); }
    if matches!(kill.weapon_id, 1..=11 | 64 | 72 | 74 | 82 | 96 | 104 | 106) { return true; }
    matches!(kill.weapon.as_str(),
        "bottle" | "sword" | "eyelander" | "headtaker" | "golfclub" | "scotsmans_skullcutter" | "skullcutter" |
        "paintrain" | "pain_train" | "ullapool_caber" | "battleaxe" | "claidheamh_mor" | "claidheamohmor" |
        "half_zatoichi" | "katana" | "persian_persuader" | "fryingpan" | "golden_fryingpan" | "saxxy" |
        "conscientious_objector" | "freedom_staff" | "ham_shank" | "memory_maker" | "necro_smasher" |
        "crossing_guard" | "prinny_machete" | "bat" | "bat_wood" | "sandman" | "wrap_assassin" | "atomizer" |
        "fan_o_war" | "holy_mackerel" | "boston_basher" | "three_rune_blade" | "sun_on_a_stick" | "candy_cane" |
        "fish" | "fists" | "gloves" | "holiday_punch" | "kgb" | "warrior_spirit" | "eviction_notice" | "fireaxe" |
        "back_scratcher" | "powerjack" | "homewrecker" | "maul" | "neon_annihilator" | "thirddegree" |
        "volcano_fragment" | "axtinguisher" | "postal_pummeler" | "shovel" | "equalizer" | "escape_plan" |
        "market_gardener" | "disciplinary_action" | "wrench" | "gunslinger" | "southern_hospitality" | "jag" |
        "eureka_effect" | "bonesaw" | "ubersaw" | "vita_saw" | "amputator" | "solemn_vow" | "knife" | "kunai" |
        "eternal_reward" | "wanga_prick" | "big_earner" | "spy_cicle" | "black_rose" | "sharp_dresser" |
        "tribalkukri" | "shahanshah" | "bushwacka" | "kukri")
}

fn taunt_kill_name(custom: i64) -> Option<&'static str> {
    Some(match custom {
        7=>"hadouken", 9=>"high_noon", 10=>"grand_slam", 13=>"fencing", 15=>"arrow_stab",
        21=>"grenade_taunt", 24=>"barbarian_swing", 29=>"uberslice", 33=>"engineer_guitar_smash",
        38=>"engineer_arm_impale", 52=>"armageddon", 60=>"allclass_guitar_riff", 80=>"gas_blast",
        _=>return None,
    })
}

fn death_json(death: &Death) -> Value {
    json!({
        "tick": death.demo_tick,
        "demo_tick": death.demo_tick,
        "server_tick": death.event_tick,
        "event_tick": death.event_tick,
        "packet_sequence": death.packet_sequence,
        "event_index_in_packet": death.event_index,
        "attacker_user_id": death.attacker,
        "attacker_name": death.state.attacker.get("name"),
        "attacker_steam_id": death.state.attacker.get("steam_id"),
        "victim_user_id": death.victim,
        "victim_name": death.state.victim.get("name"),
        "victim_steam_id": death.state.victim.get("steam_id"),
        "attacker_class": death.attacker_class,
        "attacker_team": death.attacker_team,
        "victim_class": death.victim_class,
        "victim_team": death.victim_team,
        "weapon": death.weapon,
        "weapon_id": death.weapon_id,
        "weapon_def_index": death.weapon_def_index,
        "weapon_slot": death.weapon_slot,
        "custom_kill": death.custom_kill,
        "crit_type": death.crit_type,
        "assister_user_id": death.assister,
        "kill_streak_total": death.kill_streak_total,
        "rocket_jump_victim": death.rocket_jump_victim,
        "state_evidence": {
            "attacker":death.state.attacker,
            "victim":death.state.victim,
            "state_available":!death.state.attacker.is_empty() || !death.state.victim.is_empty(),
            "friendly_alive_before":death.state.friendly_alive_before,
            "enemy_alive_before":death.state.enemy_alive_before,
            "friendly_state_roster":death.state.friendly_state_roster,
            "enemy_state_roster":death.state.enemy_state_roster,
            "recent_friendly_death_ticks":death.state.recent_friendly_death_ticks,
            "recent_friendly_death_count":death.state.recent_friendly_deaths,
            "sack_recovery_window_seconds":10.0,
            "player_disadvantage_before":death.state.player_disadvantage_before,
            "friendly_medic_charge":death.state.friendly_medic_charge,
            "enemy_medic_charge":death.state.enemy_medic_charge,
            "enemy_uber_advantage_before":death.state.enemy_uber_advantage_before,
            "victim_next_respawn_tick":death.state.victim_next_respawn_tick,
            "victim_respawn_seconds":death.state.victim_next_respawn_tick.map(|tick| (tick-death.event_tick) as f64/TICKS_PER_SECOND),
            "friendly_pending_respawn_ticks":death.state.friendly_pending_respawn_ticks,
            "attacker_recent_shield_charge_tick":death.state.attacker_recent_shield_charge_tick,
            "confirmed_double_donk":death.state.confirmed_double_donk,
            "victim_airborne_before_projectile_impact":death.state.victim_airborne_before_projectile_impact,
            "projectile_impact_check_tick":death.state.projectile_impact_check_tick,
            "confirmed_kritzkrieg_boost":death.state.confirmed_kritzkrieg_boost,
            "confirmed_uber_drop":death.state.confirmed_uber_drop,
            "enemy_medic_force_followups":death.state.enemy_medic_force_followups,
            "projectile":death.state.projectile,
        },
    })
}

fn bookmark_entries(value: &Value) -> Vec<(i64, String)> {
    let mut entries = Vec::new();

    if let Some(bookmarks) = value.get("bookmarks").and_then(Value::as_array) {
        for bookmark in bookmarks {
            let tick = int_value(bookmark.get("tick")).unwrap_or_default();
            if tick <= 0 {
                continue;
            }
            let comment = ["comment", "value", "title", "description"]
                .into_iter()
                .find_map(|key| bookmark.get(key).and_then(Value::as_str))
                .unwrap_or_default()
                .to_owned();
            entries.push((tick, comment));
        }
    }

    // TF2 Demo Support does not normally embed ds_mark in the .dem stream.
    // It writes a same-name JSON sidecar with records shaped as
    // {"events":[{"tick":...,"name":"bookmark","value":"comment"}]}.
    if let Some(events) = value.get("events").and_then(Value::as_array) {
        for event in events {
            let event_type = ["name", "type", "event", "event_type"]
                .into_iter()
                .find_map(|key| event.get(key).and_then(Value::as_str))
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase();
            if !event_type.contains("bookmark") && event_type != "mark" && event_type != "ds_mark" {
                continue;
            }
            let tick = int_value(event.get("tick")).unwrap_or_default();
            if tick <= 0 {
                continue;
            }
            let comment = ["value", "comment", "title", "description"]
                .into_iter()
                .find_map(|key| event.get(key).and_then(Value::as_str))
                .unwrap_or_default()
                .to_owned();
            entries.push((tick, comment));
        }
    }

    entries
}

fn append_bookmarks(export: &Path, source_demo: &str, context: &DemoContext, candidates: &mut Vec<Candidate>) -> Result<()> {
    let mut paths = BTreeSet::from([export.join("bookmarks.json")]);
    if !source_demo.is_empty() {
        let demo = PathBuf::from(source_demo);
        paths.insert(demo.with_extension("json"));
        paths.insert(PathBuf::from(format!("{}.json", demo.display())));
    }

    // Merge all available sources. The parser-generated bookmarks file can be
    // valid but empty, so it must never prevent the TF2 sidecar from loading.
    let mut bookmarks = Vec::new();
    let mut seen = HashSet::new();
    for path in paths.into_iter().filter(|path| path.is_file()) {
        for (tick, comment) in bookmark_entries(&read_json(&path)) {
            let identity = (tick, comment.trim().to_ascii_lowercase());
            if seen.insert(identity) {
                bookmarks.push((tick, comment));
            }
        }
    }
    bookmarks.sort_by_key(|(tick, _)| *tick);

    for (index, (tick, comment)) in bookmarks.into_iter().enumerate() {
        let linked_index = candidates
            .iter()
            .enumerate()
            // Only a real frag candidate can absorb a bookmark. A standalone
            // bookmark uses its own tick as a synthetic point-of-kill and must
            // not swallow another nearby bookmark.
            .filter(|(_, candidate)| {
                candidate.kill_count() > 0 && candidate.attacker_class != "bookmark"
            })
            .filter(|(_, candidate)| {
                tick >= candidate.clip_start_tick - OBJECTIVE_CONVERSION_TICKS
                    && tick <= candidate.clip_end_tick + CAPTURE_DENIAL_TICKS
            })
            .min_by_key(|(_, candidate)| {
                candidate
                    .point_of_kill_ticks
                    .iter()
                    .map(|kill| (kill - tick).abs())
                    .min()
                    .unwrap_or(i64::MAX)
            })
            .map(|(candidate_index, _)| candidate_index);

        if let Some(candidate_index) = linked_index {
            add_bookmark_to_candidate(&mut candidates[candidate_index], tick, comment);
            continue;
        }

        let mut candidate = Candidate {
            candidate_id: format!("bookmark-{tick}-{index}"),
            source_demo: source_demo.into(),
            attacker_class: "bookmark".into(),
            clip_start_tick: (tick - PRE_ROLL).max(0),
            clip_end_tick: tick + POST_ROLL,
            point_of_kill_ticks: vec![tick],
            metrics: json!({"kills":0}),
            demo_context: context.clone(),
            ..Candidate::default()
        };
        add_bookmark_to_candidate(&mut candidate, tick, comment);
        candidates.push(candidate);
    }
    Ok(())
}

fn add_bookmark_to_candidate(candidate: &mut Candidate, tick: i64, comment: String) {
    let previous_bookmarks = candidate
        .metrics
        .get("bookmarks")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let previous_bookmark_score = candidate
        .metrics
        .get("bookmark_score")
        .and_then(Value::as_f64)
        .unwrap_or_default();
    candidate.overall_score += BOOKMARK_SCORE;
    candidate.tags.push("bookmark".into());
    candidate.tags.sort();
    candidate.tags.dedup();
    candidate.sequence_tags.push("bookmark".into());
    candidate.sequence_tags.sort();
    candidate.sequence_tags.dedup();
    candidate.bookmark_comment = comment;
    candidate.bookmark_tick = Some(tick);
    if !candidate.metrics.is_object() {
        candidate.metrics = json!({});
    }
    candidate.metrics["bookmarks"] = json!(previous_bookmarks + 1);
    candidate.metrics["bookmark_score"] = json!(previous_bookmark_score + BOOKMARK_SCORE);
    candidate
        .score_breakdown
        .push(json!({"reason":"bookmark","points":BOOKMARK_SCORE,"event_tick":tick}));
    candidate.primary_tag.clear();
    candidate.primary_tag = candidate.inferred_primary_tag();
}

fn write_candidates(export: &Path, candidates: &[Candidate], context: &DemoContext, source_demo: &str, map_name: &str, item_schema: Option<&Path>) -> Result<()> {
    let output = File::create(export.join("frag_candidates.ndjson"))?;
    let mut output = BufWriter::new(output);
    for candidate in candidates {
        serde_json::to_writer(&mut output, candidate)?;
        output.write_all(b"\n")?;
    }
    output.flush()?;
    let summary = json!({
        "format":"tf2-frag-candidates-rust",
        "format_version":3,
        "source_demo":source_demo,
        "map_name":map_name,
        "candidate_count":candidates.len(),
        "demo_context":context,
        "item_schema":item_schema.map(|path| path.display().to_string()),
    });
    fs::write(export.join("frag_summary.json"), serde_json::to_vec_pretty(&summary)?)?;
    Ok(())
}

fn canonical_team_value(value: Option<&Value>) -> String {
    match value.and_then(|value| {
        value.as_str().map(ToOwned::to_owned).or_else(|| value.as_i64().map(|value| value.to_string()))
    }).unwrap_or_default().to_lowercase().as_str() {
        "2" | "red" => "red".into(),
        "3" | "blue" | "blu" => "blu".into(),
        _ => String::new(),
    }
}

fn canonical_class(class: &str) -> String {
    match class.trim().to_lowercase().as_str() {
        "1" => "scout".into(), "2" => "sniper".into(), "3" => "soldier".into(),
        "4" => "demoman".into(), "5" => "medic".into(), "6" => "heavy".into(),
        "7" => "pyro".into(), "8" => "spy".into(), "9" => "engineer".into(),
        value => value.into(),
    }
}

fn int_value(value: Option<&Value>) -> Option<i64> {
    value.and_then(|value| value.as_i64().or_else(|| value.as_u64().map(|value| value as i64)).or_else(|| value.as_str()?.parse().ok()))
}

fn text_value(value: Option<&Value>) -> Option<String> {
    value.and_then(|value| value.as_str().map(ToOwned::to_owned).or_else(|| value.as_i64().map(|value| value.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn movement_state(on_ground: bool, z: f64, velocity_z: f64) -> Map<String, Value> {
        Map::from_iter([
            ("on_ground".into(), json!(on_ground)),
            ("position".into(), json!([0.0, 0.0, z])),
            ("velocity".into(), json!([0.0, 0.0, velocity_z])),
            ("blast_jumping".into(), json!(false)),
        ])
    }

    fn position_state(x: f64, y: f64, z: f64) -> Map<String, Value> {
        Map::from_iter([("position".into(), json!([x, y, z]))])
    }

    fn bookmark_test_directory(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "tf2-frag-helper-bookmark-{name}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn tolerant_sixes_allows_one_offclass() {
        let roster = HashMap::from([
            ("scout".into(), 1), ("sniper".into(), 1), ("soldier".into(), 2),
            ("demoman".into(), 1), ("medic".into(), 1),
        ]);
        assert!(sixes_fit(&roster));
    }

    #[test]
    fn elevated_stationary_player_is_not_an_airshot() {
        let mut scan = StateScan::default();
        scan.player_history.insert(42, vec![
            (980, movement_state(false, 256.0, 0.0)),
            (990, movement_state(false, 256.0, 0.0)),
            (999, movement_state(false, 256.0, 0.0)),
        ]);
        assert!(!player_airborne_before_impact(&scan, 42, 1_000, 0));
    }

    #[test]
    fn player_moving_up_a_high_ledge_without_takeoff_is_not_an_airshot() {
        let mut scan = StateScan::default();
        scan.player_history.insert(42, vec![
            (960, movement_state(false, 240.0, 0.0)),
            (980, movement_state(false, 246.0, 90.0)),
            (990, movement_state(false, 256.0, 90.0)),
            (999, movement_state(false, 268.0, 90.0)),
        ]);
        assert!(!player_airborne_before_impact(&scan, 42, 1_000, 0));
    }

    #[test]
    fn moving_victim_already_airborne_before_impact_is_an_airshot() {
        let mut scan = StateScan::default();
        scan.player_history.insert(42, vec![
            (965, movement_state(true, 192.0, 0.0)),
            (975, movement_state(false, 198.0, 220.0)),
            (980, movement_state(false, 208.0, 180.0)),
            (990, movement_state(false, 228.0, 120.0)),
            (999, movement_state(false, 240.0, 70.0)),
        ]);
        assert!(player_airborne_before_impact(&scan, 42, 1_000, 0));
    }

    #[test]
    fn loose_cannon_launch_after_collision_cannot_create_its_own_airshot() {
        let mut scan = StateScan::default();
        scan.player_history.insert(42, vec![
            (980, movement_state(true, 128.0, 0.0)),
            (993, movement_state(true, 128.0, 0.0)),
            (995, movement_state(false, 130.0, 260.0)),
            (999, movement_state(false, 145.0, 220.0)),
            (1_000, movement_state(false, 150.0, 200.0)),
            (1_010, movement_state(false, 170.0, 180.0)),
        ]);
        assert!(!player_airborne_before_impact(
            &scan,
            42,
            1_000,
            AIRSHOT_LOOSE_CANNON_IMPACT_GUARD_TICKS,
        ));
    }

    #[test]
    fn loose_cannon_can_still_confirm_victim_airborne_well_before_collision() {
        let mut scan = StateScan::default();
        scan.player_history.insert(42, vec![
            (940, movement_state(true, 128.0, 0.0)),
            (950, movement_state(false, 136.0, 240.0)),
            (965, movement_state(false, 175.0, 170.0)),
            (972, movement_state(false, 193.0, 135.0)),
            (980, movement_state(false, 205.0, 110.0)),
            (990, movement_state(false, 216.0, 70.0)),
            (999, movement_state(false, 220.0, 260.0)),
        ]);
        assert!(player_airborne_before_impact(
            &scan,
            42,
            1_000,
            AIRSHOT_LOOSE_CANNON_IMPACT_GUARD_TICKS,
        ));
    }

    #[test]
    fn airshot_geometry_uses_height_relative_to_attacker_not_world_height() {
        let attacker = position_state(0.0, 0.0, 1_000.0);
        let same_high_ledge = position_state(100.0, 0.0, 1_000.0);
        let victim_above = position_state(100.0, 0.0, 1_200.0);
        let (ledge_height, _, ledge_angle) = airshot_geometry(&attacker, &same_high_ledge).unwrap();
        let (air_height, _, air_angle) = airshot_geometry(&attacker, &victim_above).unwrap();
        assert_eq!(ledge_height, 0.0);
        assert_eq!(airshot_style_bonus(ledge_height, ledge_angle), 0.0);
        assert!(air_height >= 192.0);
        assert!(air_angle > 50.0);
        assert_eq!(airshot_style_bonus(air_height, air_angle), 17.0);
    }

    #[test]
    fn steeper_higher_airshot_receives_more_style_points() {
        let low = airshot_style_bonus(80.0, 15.0);
        let medium = airshot_style_bonus(150.0, 30.0);
        let extreme = airshot_style_bonus(300.0, 55.0);
        assert!(low < medium);
        assert!(medium < extreme);
        assert_eq!(extreme, 20.0);
    }

    #[test]
    fn tf2_demo_support_events_are_read_as_bookmarks() {
        let value = json!({
            "events": [
                {"tick": 1_234, "name": "bookmark", "value": "nice frag"},
                {"tick": 1_300, "name": "killstreak", "value": "3"}
            ]
        });
        assert_eq!(bookmark_entries(&value), vec![(1_234, "nice frag".into())]);
    }

    #[test]
    fn empty_parser_bookmarks_do_not_hide_tf2_sidecar_bookmark() {
        let root = bookmark_test_directory("sidecar");
        let export = root.join("export");
        fs::create_dir_all(&export).unwrap();
        fs::write(export.join("bookmarks.json"), br#"{"bookmarks":[]}"#).unwrap();
        let demo = root.join("marked.dem");
        fs::write(
            demo.with_extension("json"),
            br#"{"events":[{"tick":1234,"name":"bookmark","value":"saved play"}]}"#,
        )
        .unwrap();

        let mut candidates = Vec::new();
        append_bookmarks(
            &export,
            demo.to_str().unwrap(),
            &DemoContext::default(),
            &mut candidates,
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].bookmark_tick, Some(1_234));
        assert_eq!(candidates[0].bookmark_comment, "saved play");
        assert_eq!(candidates[0].overall_score, BOOKMARK_SCORE);
        assert!(candidates[0].tags.iter().any(|tag| tag == "bookmark"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bookmark_candidate_inherits_nearby_frag_tags_and_score() {
        let root = bookmark_test_directory("linked");
        let export = root.join("export");
        fs::create_dir_all(&export).unwrap();
        let demo = root.join("marked.dem");
        fs::write(
            demo.with_extension("json"),
            br#"{"events":[{"tick":1000,"name":"Bookmark","value":"airshot"}]}"#,
        )
        .unwrap();
        let original = Candidate {
            candidate_id: "frag".into(),
            source_demo: demo.display().to_string(),
            overall_score: 20.0,
            clip_start_tick: 900,
            clip_end_tick: 1_100,
            point_of_kill_ticks: vec![1_010],
            tags: vec!["airshot".into()],
            metrics: json!({"kills":1}),
            ..Candidate::default()
        };
        let mut candidates = vec![original];

        append_bookmarks(
            &export,
            demo.to_str().unwrap(),
            &DemoContext::default(),
            &mut candidates,
        )
        .unwrap();

        assert_eq!(candidates.len(), 1);
        let bookmarked = &candidates[0];
        assert_eq!(bookmarked.candidate_id, "frag");
        assert_eq!(bookmarked.bookmark_tick, Some(1_000));
        assert_eq!(bookmarked.overall_score, 20.0 + BOOKMARK_SCORE);
        assert!(bookmarked.tags.iter().any(|tag| tag == "airshot"));
        assert!(bookmarked.tags.iter().any(|tag| tag == "bookmark"));
        assert_eq!(bookmarked.metrics["bookmark_score"], json!(BOOKMARK_SCORE));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn separate_unlinked_bookmarks_remain_separate_candidates() {
        let root = bookmark_test_directory("unlinked");
        let export = root.join("export");
        fs::create_dir_all(&export).unwrap();
        let demo = root.join("marked.dem");
        fs::write(
            demo.with_extension("json"),
            br#"{"events":[{"tick":1000,"name":"Bookmark","value":"first"},{"tick":1100,"name":"Bookmark","value":"second"}]}"#,
        )
        .unwrap();
        let mut candidates = Vec::new();

        append_bookmarks(
            &export,
            demo.to_str().unwrap(),
            &DemoContext::default(),
            &mut candidates,
        )
        .unwrap();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].candidate_id, "bookmark-1000-0");
        assert_eq!(candidates[1].candidate_id, "bookmark-1100-1");
        fs::remove_dir_all(root).unwrap();
    }
}
