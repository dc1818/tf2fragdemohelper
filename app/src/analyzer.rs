use crate::models::{Candidate, DemoContext};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

const TICKS_PER_SECOND: f64 = 66.666_666_7;
const SEQUENCE_GAP: i64 = (TICKS_PER_SECOND * 4.0) as i64;
const PRE_ROLL: i64 = (TICKS_PER_SECOND * 5.0) as i64;
const POST_ROLL: i64 = (TICKS_PER_SECOND * 3.0) as i64;
const BOOKMARK_SCORE: f64 = 30.0;

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
    start_event: String,
    end_event: String,
}

#[derive(Clone, Debug, Default)]
struct Death {
    event_tick: i64,
    demo_tick: i64,
    packet_sequence: i64,
    event_index: i64,
    attacker: i64,
    victim: i64,
    round_index: i64,
    weapon: String,
    weapon_id: i64,
    custom_kill: i64,
    crit_type: i64,
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
}

#[derive(Clone, Debug, Default)]
struct StateScan {
    at_death: HashMap<i64, HashMap<i64, Map<String, Value>>>,
    roster_samples: Vec<(i64, HashMap<String, HashMap<String, usize>>)>,
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
    pub candidate_workers_used: usize,
    pub stage_seconds: BTreeMap<String, f64>,
    pub total_seconds: f64,
}

pub fn analyze_export(export: &Path, item_schema: Option<&Path>) -> Result<Vec<Candidate>> {
    let total_started = Instant::now();
    let mut profile = AnalysisProfile {
        format: "tf2-frag-analysis-profile-rust".into(),
        format_version: 2,
        candidate_workers_used: 1,
        ..AnalysisProfile::default()
    };

    let started = Instant::now();
    let events = read_ndjson::<EventRecord>(&export.join("events.ndjson"))?;
    profile.stage_seconds.insert("read_events".into(), started.elapsed().as_secs_f64());

    let header = read_json(&export.join("header.json"));
    let manifest = read_json(&export.join("manifest.json"));
    let source_demo = manifest.get("source_demo").and_then(Value::as_str).unwrap_or_default().to_owned();
    let map_name = header.get("map").and_then(Value::as_str).unwrap_or_default().to_owned();
    let mut context = capture_context(&events, &header, &manifest);
    profile.capture_type = context.capture_type.clone();
    profile.analysis_scope = context.analysis_scope.clone();

    let started = Instant::now();
    let rounds = build_rounds(&events, header.get("ticks").and_then(Value::as_i64).unwrap_or_default());
    let mut deaths = normalized_deaths(&events, &rounds, &context);
    profile.total_player_death_events = events.iter().filter(|event| event.event_type == "player_death").count();
    profile.accepted_live_scope_kills = deaths.len();
    profile.stage_seconds.insert("early_death_gating".into(), started.elapsed().as_secs_f64());

    let started = Instant::now();
    let scan = scan_state_stream(&export.join("state_samples.ndjson"), &deaths, &rounds)?;
    attach_state(&mut deaths, &scan);
    profile.stage_seconds.insert("stream_state_enrichment".into(), started.elapsed().as_secs_f64());

    let started = Instant::now();
    let mode = classify_mode(&header, &events, &scan.roster_samples);
    context.mode = mode.0;
    context.mode_label = mode.1;
    context.mode_confidence = mode.2;
    context.mode_evidence = mode.3;
    profile.mode = context.mode.clone();
    profile.mode_confidence = context.mode_confidence.clone();
    profile.stage_seconds.insert("mode_classification".into(), started.elapsed().as_secs_f64());

    let started = Instant::now();
    let mut candidates = build_candidates(&mut deaths, &rounds, &context, &source_demo, &map_name);
    profile.candidate_group_jobs = candidates.len();
    append_bookmarks(export, &source_demo, &context, &mut candidates)?;
    candidates.sort_by(|left, right| right.overall_score.total_cmp(&left.overall_score).then(left.clip_start_tick.cmp(&right.clip_start_tick)));
    profile.stage_seconds.insert("candidate_grouping_and_scoring".into(), started.elapsed().as_secs_f64());

    let started = Instant::now();
    write_candidates(export, &candidates, &context, &source_demo, &map_name, item_schema)?;
    profile.stage_seconds.insert("write_outputs".into(), started.elapsed().as_secs_f64());
    profile.total_seconds = total_started.elapsed().as_secs_f64();
    fs::write(export.join("analysis_profile.json"), serde_json::to_vec_pretty(&profile)?)?;
    Ok(candidates)
}

fn read_ndjson<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    let source = File::open(path).with_context(|| format!("missing {}", path.display()))?;
    BufReader::new(source)
        .lines()
        .filter_map(|line| match line {
            Ok(line) if !line.trim().is_empty() => Some(serde_json::from_str(&line).map_err(anyhow::Error::from)),
            Ok(_) => None,
            Err(error) => Some(Err(error.into())),
        })
        .collect()
}

fn read_json(path: &Path) -> Value {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(Value::Null)
}

fn capture_context(events: &[EventRecord], header: &Value, manifest: &Value) -> DemoContext {
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
    let pov = (capture_type == "pov" && name_matches.len() == 1).then(|| *name_matches.iter().next().unwrap());
    DemoContext {
        capture_type: capture_type.clone(),
        capture_confidence: capture.get("confidence").and_then(Value::as_str).unwrap_or("unknown").into(),
        analysis_scope: if pov.is_some() { "pov_player_only".into() } else { "all_players".into() },
        pov_player_user_id: pov,
        ..DemoContext::default()
    }
}

fn build_rounds(events: &[EventRecord], header_ticks: i64) -> Vec<Round> {
    let activation = ["teamplay_round_start", "teamplay_waiting_ends", "teamplay_round_active", "round_start"];
    let endings = ["teamplay_round_win", "teamplay_round_stalemate", "teamplay_game_over", "tf_game_over", "round_end"];
    let mut rounds = Vec::new();
    let mut current: Option<(i64, String)> = None;
    let mut waiting = true;
    for event in events {
        let name = event.event_type.as_str();
        let tick = event.analysis_tick();
        if name == "teamplay_waiting_begins" {
            waiting = true;
            current = None;
            continue;
        }
        if activation.contains(&name) {
            // round_active at demo bootstrap is not sufficient by itself.
            if name != "teamplay_round_active" || !rounds.is_empty() || events.iter().any(|event| event.event_type == "teamplay_waiting_ends") {
                waiting = false;
                current.get_or_insert((tick, name.to_owned()));
            }
        }
        if endings.contains(&name) {
            if let Some((start, start_event)) = current.take() {
                if tick > start {
                    rounds.push(Round { index: rounds.len() as i64 + 1, start, end: tick, start_event, end_event: name.to_owned() });
                }
            }
            waiting = true;
        }
    }
    if let Some((start, start_event)) = current {
        let end = header_ticks.max(events.last().map(EventRecord::analysis_tick).unwrap_or(start));
        if end > start && !waiting {
            rounds.push(Round { index: rounds.len() as i64 + 1, start, end, start_event, end_event: "demo_end_while_live".into() });
        }
    }
    rounds
}

fn normalized_deaths(events: &[EventRecord], rounds: &[Round], context: &DemoContext) -> Vec<Death> {
    let mut classes: HashMap<i64, String> = HashMap::new();
    let mut teams: HashMap<i64, String> = HashMap::new();
    let mut deaths = Vec::new();
    for event in events {
        if matches!(event.event_type.as_str(), "player_changeclass" | "player_spawn") {
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
        let Some(round) = rounds.iter().find(|round| tick >= round.start && tick <= round.end) else { continue };
        deaths.push(Death {
            event_tick: tick,
            demo_tick: if event.demo_tick > 0 { event.demo_tick } else { event.tick },
            packet_sequence: event.packet_sequence,
            event_index: event.event_index_in_packet,
            attacker,
            victim,
            round_index: round.index,
            weapon: event.text(&["weapon", "weapon_logclassname"]),
            weapon_id: event.int(&["weapon_id"]),
            custom_kill: event.int(&["custom_kill", "customkill"]),
            crit_type: event.int(&["crit_type"]),
            attacker_class: classes.get(&attacker).cloned().unwrap_or_default(),
            attacker_team: teams.get(&attacker).cloned().unwrap_or_default(),
            victim_class: classes.get(&victim).cloned().unwrap_or_default(),
            victim_team: teams.get(&victim).cloned().unwrap_or_default(),
            ..Death::default()
        });
    }
    deaths
}

fn scan_state_stream(path: &Path, deaths: &[Death], rounds: &[Round]) -> Result<StateScan> {
    if !path.is_file() {
        return Ok(StateScan::default());
    }
    let mut targets: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
    for (index, death) in deaths.iter().enumerate() {
        targets.entry(death.event_tick).or_default().push(index);
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
    let mut result = StateScan::default();
    let mut pending_targets = targets.into_iter().peekable();
    let mut pending_rosters = roster_ticks.into_iter().peekable();
    for line in source.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: Value = serde_json::from_str(&line)?;
        let tick = record.get("server_tick").and_then(Value::as_i64).or_else(|| record.get("demo_tick").and_then(Value::as_i64)).unwrap_or_default();
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
            }
        }
        if let Some(removed) = record.get("removed_players").and_then(Value::as_array) {
            for entity in removed.iter().filter_map(Value::as_i64) {
                if let Some(user) = entity_users.remove(&entity) {
                    current.remove(&user);
                }
            }
        }
        while pending_targets.peek().is_some_and(|(target, _)| *target <= tick) {
            let (target, indexes) = pending_targets.next().unwrap();
            let mut snapshot = HashMap::new();
            for index in indexes {
                let death = &deaths[index];
                for user in [death.attacker, death.victim] {
                    if let Some(state) = current.get(&user) {
                        snapshot.insert(user, state.clone());
                    }
                }
            }
            result.at_death.insert(target, snapshot);
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
        if pending_targets.peek().is_none() && pending_rosters.peek().is_none() {
            break;
        }
    }
    Ok(result)
}

fn attach_state(deaths: &mut [Death], scan: &StateScan) {
    for death in deaths {
        let Some(snapshot) = scan.at_death.get(&death.event_tick) else { continue };
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
    }
}

fn classify_mode(header: &Value, events: &[EventRecord], samples: &[(i64, HashMap<String, HashMap<String, usize>>)]) -> (String, String, String, Vec<String>) {
    let server = header.get("server").and_then(Value::as_str).unwrap_or_default().to_lowercase();
    let text = events
        .iter()
        .filter(|event| matches!(event.event_type.as_str(), "player_say" | "say_text" | "server_message" | "server_cvar"))
        .fold(server.clone(), |mut text, event| {
            text.push_str(&serde_json::to_string(&event.event).unwrap_or_default().to_lowercase());
            text
        });
    let rgl = text.contains("rgl");
    let valve = ["valve", "matchmaking", "casual"].iter().any(|token| text.contains(token));
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
    if highlander >= 3 && ratio(highlander) >= 0.40 {
        return (
            if rgl { "rgl_highlander" } else { "highlander" }.into(),
            if rgl { "RGL Highlander" } else { "Highlander Competitive" }.into(),
            if ratio(highlander) >= 0.70 { "high" } else { "medium" }.into(),
            vec![format!("{highlander} of {observed} round-wide roster samples matched tolerant Highlander")],
        );
    }
    if sixes >= 3 && ratio(sixes) >= 0.40 {
        return (
            if rgl { "rgl_6v6" } else { "6v6" }.into(),
            if rgl { "RGL 6v6" } else { "6v6 Competitive" }.into(),
            if ratio(sixes) >= 0.70 { "high" } else { "medium" }.into(),
            vec![format!("{sixes} of {observed} round-wide roster samples matched tolerant 6v6")],
        );
    }
    if rgl {
        return ("rgl_competitive".into(), "RGL Competitive".into(), "medium".into(), vec!["RGL signature recorded; format uncertain".into()]);
    }
    if small >= 3 && ratio(small) >= 0.60 {
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

fn build_candidates(deaths: &mut [Death], rounds: &[Round], context: &DemoContext, source_demo: &str, map_name: &str) -> Vec<Candidate> {
    deaths.sort_by_key(|death| (death.round_index, death.attacker, death.event_tick, death.packet_sequence, death.event_index));
    let mut buckets: BTreeMap<(i64, i64), Vec<&Death>> = BTreeMap::new();
    for death in deaths.iter() {
        buckets.entry((death.round_index, death.attacker)).or_default().push(death);
    }
    let mut candidates = Vec::new();
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
            let (score, tags, breakdown) = score_group(&group);
            if group.len() == 1 && score < 25.0 {
                continue;
            }
            let first = group.first().unwrap();
            let last = group.last().unwrap();
            let round = rounds.iter().find(|round| round.index == round_index).unwrap();
            let kill_values: Vec<Value> = group.iter().map(|kill| death_json(kill)).collect();
            candidates.push(Candidate {
                candidate_id: format!("r{round_index}-p{attacker}-t{}", first.event_tick),
                source_demo: source_demo.into(),
                map_name: map_name.into(),
                round_index,
                overall_score: score,
                attacker_user_id: attacker,
                attacker_class: first.attacker_class.clone(),
                attacker_team: first.attacker_team.clone(),
                clip_start_tick: (first.demo_tick - PRE_ROLL).max(0),
                clip_end_tick: last.demo_tick + POST_ROLL,
                point_of_kill_ticks: group.iter().map(|kill| kill.demo_tick).collect(),
                tags,
                metrics: json!({"kills": group.len(), "live_round": true}),
                kills: kill_values,
                score_breakdown: breakdown,
                demo_context: context.clone(),
                extra: Map::from_iter([
                    ("live_round".into(), Value::Bool(true)),
                    ("round_state".into(), json!({"classification":"live","start_tick":round.start,"start_event":round.start_event,"end_tick":round.end,"end_event":round.end_event})),
                ]),
                ..Candidate::default()
            });
        }
    }
    candidates
}

fn score_group(group: &[&Death]) -> (f64, Vec<String>, Vec<Value>) {
    let mut score = if group.len() > 1 { 25.0 + (group.len() - 1) as f64 * 12.0 } else { 0.0 };
    let mut tags = Vec::new();
    let mut breakdown = Vec::new();
    if group.len() > 1 {
        tags.push("multi_kill".into());
        breakdown.push(json!({"reason":"multi_kill","points":25.0}));
        if group.len() > 2 {
            breakdown.push(json!({"reason":"additional_kills","points":(group.len()-2) as f64*12.0,"count":group.len()-2}));
        }
    }
    for kill in group {
        let weapon = kill.weapon.to_lowercase();
        let victim_airborne = kill.state.victim.get("blast_jumping").and_then(Value::as_bool).unwrap_or(false)
            || kill.state.victim.get("on_ground").and_then(Value::as_bool) == Some(false);
        let projectile = ["rocketlauncher", "directhit", "grenadelauncher", "loch_n_load", "iron_bomber", "loose_cannon", "flaregun", "huntsman"]
            .iter().any(|name| weapon.contains(name));
        if projectile && victim_airborne {
            score += 18.0;
            tags.push("airshot".into());
            breakdown.push(json!({"reason":"confirmed_airshot","points":18.0,"event_tick":kill.event_tick}));
        }
        let victim_charge = kill.state.victim.get("medic_charge").and_then(Value::as_f64).unwrap_or_default();
        if kill.victim_class == "medic" && victim_charge >= 95.0 {
            score += 25.0;
            tags.push("uber_drop".into());
            breakdown.push(json!({"reason":"confirmed_uber_drop","points":25.0,"event_tick":kill.event_tick}));
        }
        if weapon.contains("market_gardener") && kill.state.attacker.get("blast_jumping").and_then(Value::as_bool).unwrap_or(false) {
            score += 30.0;
            tags.push("market_garden".into());
            breakdown.push(json!({"reason":"confirmed_market_garden","points":30.0,"event_tick":kill.event_tick}));
        }
        if kill.custom_kill == 2 {
            score += 22.0;
            tags.push("backstab".into());
            breakdown.push(json!({"reason":"backstab","points":22.0,"event_tick":kill.event_tick}));
        }
        if matches!(kill.custom_kill, 1 | 3) || weapon.contains("headshot") {
            score += 20.0;
            tags.push("headshot".into());
            breakdown.push(json!({"reason":"headshot","points":20.0,"event_tick":kill.event_tick}));
        }
        if kill.crit_type == 2 && !weapon.contains("market_gardener") {
            score += 8.0;
            tags.push("critical_kill".into());
            breakdown.push(json!({"reason":"critical_kill","points":8.0,"event_tick":kill.event_tick}));
        }
    }
    tags.sort();
    tags.dedup();
    (score, tags, breakdown)
}

fn death_json(death: &Death) -> Value {
    json!({
        "tick": death.demo_tick,
        "event_tick": death.event_tick,
        "attacker_user_id": death.attacker,
        "victim_user_id": death.victim,
        "attacker_class": death.attacker_class,
        "attacker_team": death.attacker_team,
        "victim_class": death.victim_class,
        "victim_team": death.victim_team,
        "weapon": death.weapon,
        "weapon_id": death.weapon_id,
        "custom_kill": death.custom_kill,
        "crit_type": death.crit_type,
        "state_evidence": {"attacker":death.state.attacker,"victim":death.state.victim,"state_available":!death.state.attacker.is_empty() || !death.state.victim.is_empty()},
    })
}

fn append_bookmarks(export: &Path, source_demo: &str, context: &DemoContext, candidates: &mut Vec<Candidate>) -> Result<()> {
    let mut paths = vec![export.join("bookmarks.json")];
    if !source_demo.is_empty() {
        let demo = PathBuf::from(source_demo);
        paths.push(demo.with_extension("json"));
        paths.push(PathBuf::from(format!("{}.json", demo.display())));
    }
    let Some(path) = paths.into_iter().find(|path| path.is_file()) else { return Ok(()) };
    let value = read_json(&path);
    let bookmarks = value.get("bookmarks").and_then(Value::as_array).cloned().unwrap_or_default();
    for (index, bookmark) in bookmarks.into_iter().enumerate() {
        let tick = bookmark.get("tick").and_then(Value::as_i64).unwrap_or_default();
        if tick <= 0 {
            continue;
        }
        let comment = bookmark.get("comment").and_then(Value::as_str).unwrap_or_default().to_owned();
        let linked = candidates
            .iter()
            .filter(|candidate| tick >= candidate.clip_start_tick - (TICKS_PER_SECOND * 8.0) as i64 && tick <= candidate.clip_end_tick + (TICKS_PER_SECOND * 2.0) as i64)
            .min_by_key(|candidate| candidate.point_of_kill_ticks.iter().map(|kill| (kill - tick).abs()).min().unwrap_or(i64::MAX))
            .cloned();
        let mut candidate = linked.unwrap_or_else(|| Candidate {
            candidate_id: format!("bookmark-{tick}-{index}"),
            source_demo: source_demo.into(),
            attacker_class: "bookmark".into(),
            clip_start_tick: (tick - PRE_ROLL).max(0),
            clip_end_tick: tick + POST_ROLL,
            point_of_kill_ticks: vec![tick],
            metrics: json!({"kills":0}),
            demo_context: context.clone(),
            ..Candidate::default()
        });
        candidate.candidate_id = format!("bookmark-{tick}-{index}");
        candidate.overall_score += BOOKMARK_SCORE;
        candidate.tags.push("bookmark".into());
        candidate.tags.sort();
        candidate.tags.dedup();
        candidate.bookmark_comment = comment;
        candidate.bookmark_tick = Some(tick);
        candidate.score_breakdown.push(json!({"reason":"bookmark","points":BOOKMARK_SCORE,"event_tick":tick}));
        candidates.push(candidate);
    }
    Ok(())
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
        "format_version":2,
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

    #[test]
    fn tolerant_sixes_allows_one_offclass() {
        let roster = HashMap::from([
            ("scout".into(), 1), ("sniper".into(), 1), ("soldier".into(), 2),
            ("demoman".into(), 1), ("medic".into(), 1),
        ]);
        assert!(sixes_fit(&roster));
    }
}
