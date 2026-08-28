#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![recursion_limit = "256"]

mod analyzer;
mod batch;
mod camera;
mod models;
mod preflight;
mod recording;
mod scheduler;

use crate::{
    batch::{BatchController, ProgressEvent},
    models::{AppSettings, Candidate},
    recording::{
        estimate_recording_space, latest_recording_session, launch_hlae_batch, preview_candidate,
        recover_interrupted_profile, recover_recording_sessions, shutdown_active_recording,
        validate_cinematic_batch, RecordingIndex, RecordingProgress, RecordingProgressSink,
    },
    scheduler::PerformanceProfile,
};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use slint::winit_030::{winit, EventResult, WinitWindowAccessor};
use slint::{Model, ModelRc, SharedString, VecModel, Weak};
use std::{
    any::Any,
    backtrace::Backtrace,
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    panic::{self, AssertUnwindSafe},
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    thread,
    time::Duration,
};

slint::include_modules!();

#[derive(Clone, Debug, Default)]
struct CandidateUiFilters {
    tag_query: String,
    maps: BTreeSet<String>,
    classes: BTreeSet<String>,
    server_types: BTreeSet<String>,
    recorded: Option<bool>,
    map_search: String,
    server_type_search: String,
}

impl CandidateUiFilters {
    fn matches(&self, candidate: &Candidate, recorded: bool) -> bool {
        if self.recorded.is_some_and(|wanted| wanted != recorded) {
            return false;
        }
        if !self.maps.is_empty()
            && !self
                .maps
                .iter()
                .any(|map| candidate.map_name.eq_ignore_ascii_case(map))
        {
            return false;
        }
        if !self.classes.is_empty() {
            let actual = canonical_candidate_class(&candidate.attacker_class);
            if !self
                .classes
                .iter()
                .any(|class| actual == canonical_candidate_class(class))
            {
                return false;
            }
        }
        let server_type = candidate_server_type(candidate);
        if !self.server_types.is_empty()
            && !self
                .server_types
                .iter()
                .any(|wanted| server_type.eq_ignore_ascii_case(wanted))
        {
            return false;
        }
        let tag_query = normalize_tag_search(&self.tag_query);
        if !tag_query.is_empty() {
            let tags = normalize_tag_search(&candidate.tags.join(" "));
            if !tag_query
                .split_whitespace()
                .all(|keyword| tags.contains(keyword))
            {
                return false;
            }
        }
        true
    }
}

fn canonical_candidate_class(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" => "scout".into(),
        "2" => "sniper".into(),
        "3" => "soldier".into(),
        "4" | "demo" => "demoman".into(),
        "5" => "medic".into(),
        "6" | "heavyweapons" | "heavyweaponsguy" => "heavy".into(),
        "7" => "pyro".into(),
        "8" => "spy".into(),
        "9" | "engi" | "engie" => "engineer".into(),
        value => value.replace([' ', '_', '-'], ""),
    }
}

fn normalize_tag_search(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace(['_', '-'], " ")
}

fn candidate_server_type(candidate: &Candidate) -> &str {
    if candidate.demo_context.mode_label.is_empty() {
        "Unknown / Mixed"
    } else {
        &candidate.demo_context.mode_label
    }
}

struct CandidateDetailText {
    player: String,
    player_meta: String,
    map: String,
    map_meta: String,
    score: String,
    score_meta: String,
    summary: String,
    kills: String,
    score_breakdown: String,
    tags: String,
}

fn nonempty_json_string(value: &serde_json::Value) -> Option<&str> {
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn humanize_identifier(value: &str) -> String {
    let spaced = value.trim().replace(['_', '-'], " ");
    let mut characters = spaced.chars();
    match characters.next() {
        Some(first) => format!(
            "{}{}",
            first.to_uppercase().collect::<String>(),
            characters.as_str()
        ),
        None => String::new(),
    }
}

fn humanize_weapon(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "grenadelauncher" | "tf_projectile_pipe" => "Grenade Launcher".into(),
        "stickybomb_launcher" | "tf_projectile_pipe_remote" => "Stickybomb Launcher".into(),
        "quickiebomb_launcher" => "Quickiebomb Launcher".into(),
        "loch_n_load" => "Loch-n-Load".into(),
        "iron_bomber" => "Iron Bomber".into(),
        "loose_cannon" | "loose_cannon_impact" | "loose_cannon_explosion" => "Loose Cannon".into(),
        "rocketlauncher" => "Rocket Launcher".into(),
        "directhit" => "Direct Hit".into(),
        "blackbox" => "Black Box".into(),
        "liberty_launcher" => "Liberty Launcher".into(),
        "airstrike" => "Air Strike".into(),
        "market_gardener" => "Market Gardener".into(),
        "sniperrifle" => "Sniper Rifle".into(),
        "compound_bow" | "huntsman" => "Huntsman".into(),
        "crusaders_crossbow" | "crossbow" => "Crusader's Crossbow".into(),
        "knife" => "Knife".into(),
        "backburner" => "Backburner".into(),
        "flaregun" => "Flare Gun".into(),
        "detonator" => "Detonator".into(),
        "scorch_shot" => "Scorch Shot".into(),
        "" => "Unknown weapon".into(),
        other => humanize_identifier(other),
    }
}

fn friendly_tag(value: &str) -> String {
    match value {
        "confirmed_airshot" => "confirmed airshot".into(),
        "direct_airshot" => "direct-hit airshot".into(),
        "long_flight_airshot" => "long-flight airshot".into(),
        "high_airshot" => "high relative airshot".into(),
        "skyward_airshot" => "steep skyward airshot".into(),
        "extreme_airshot" => "extreme skyward airshot".into(),
        "double_airshot_sequence" => "double-airshot sequence".into(),
        "airborne_projectile_kill" => "airborne projectile kill".into(),
        "rocket_jump_victim" => "rocket-jumping opponent kill".into(),
        "double_donk" => "double donk".into(),
        "market_garden" => "Market Garden".into(),
        "medic_pick" => "Medic pick".into(),
        "uber_drop" => "Über drop".into(),
        "medic_force" => "forced enemy Über".into(),
        "demoman_pick" => "Demoman pick".into(),
        "backstab" => "backstab".into(),
        "taunt_kill" => "taunt kill".into(),
        "shield_bash_kill" => "shield-bash kill".into(),
        "charge_melee_kill" => "charge melee kill".into(),
        "multi_kill" => "multi-kill".into(),
        "three_kill" => "three-kill sequence".into(),
        "four_kill_plus" => "four-or-more-kill sequence".into(),
        "rapid_sequence" => "rapid sequence".into(),
        "team_wipe" => "team wipe".into(),
        "player_count_swing" => "player-count swing".into(),
        "kills_to_secure_cap" => "kills that secured a capture".into(),
        "capture_denial_followup" => "capture denial".into(),
        "payload_progress_followup" => "payload progress after the frags".into(),
        "round_clinch" => "round-clinching sequence".into(),
        "late_round" => "late-round frags".into(),
        "random_full_crit" => "random full critical hit".into(),
        other => humanize_identifier(other).to_ascii_lowercase(),
    }
}

fn friendly_score_reason(value: &str) -> String {
    match value {
        "candidate_base" => "Base candidate value".into(),
        "confirmed_kritzkrieg_boosted_kill" => "Kritzkrieg-boosted kill".into(),
        "confirmed_market_garden" => "Confirmed Market Garden".into(),
        "confirmed_loose_cannon_double_donk" => "Confirmed Loose Cannon double donk".into(),
        "confirmed_sniper_dropshot" => "Confirmed airborne Sniper dropshot".into(),
        "confirmed_shield_bash_kill" => "Confirmed shield-bash kill".into(),
        "shield_charge_followed_by_melee_kill" => "Shield charge followed by a melee kill".into(),
        "player_melee_kill" => "Melee kill".into(),
        "confirmed_spy_backstab" => "Confirmed backstab".into(),
        "confirmed_taunt_kill" => "Confirmed taunt kill".into(),
        "medic_pick" => "Medic pick".into(),
        "confirmed_uber_drop" => "Medic killed with Über ready".into(),
        "demoman_pick" => "Demoman pick".into(),
        "state_confirmed_airshot" => "Confirmed airshot from player and projectile state".into(),
        "direct_airshot_proximity" => "Direct-hit airshot".into(),
        "long_flight_airshot" => "Long-flight airshot".into(),
        "relative_height_airshot_style" => "Relative height and upward aim bonus".into(),
        "state_confirmed_airborne_victim" => "Projectile kill on an airborne opponent".into(),
        "rocket_jump_victim" => "Killed a rocket-jumping opponent".into(),
        "streak_10_plus" => "Extended kill streak".into(),
        "random_full_crit" => "Random full-crit penalty".into(),
        "additional_kills" => "Additional unique victims".into(),
        "three_kill" => "Three-kill sequence bonus".into(),
        "four_kill_plus" => "Four-or-more-kill sequence bonus".into(),
        "multiple_confirmed_airshots" => "Multiple confirmed airshots".into(),
        "sequence_finished_enemy_team" => "Sequence eliminated every remaining enemy".into(),
        "enemy_medic_forced_uber_after_sequence" => {
            "Sequence forced the enemy Medic to Über".into()
        }
        "sequence_created_player_count_window" => "Sequence created a player advantage".into(),
        "sack_uber_recovery_after_losses" => {
            "Recovered an Über disadvantage after team losses".into()
        }
        "sack_uber_medic_equalizer" => "Medic pick equalized the Über situation".into(),
        "rapid_sequence" => "Rapid multi-kill sequence".into(),
        "projectile_sequence" => "Projectile frag sequence".into(),
        "late_round" => "Late-round impact".into(),
        "team_won_immediately_after_sequence" => "Team won immediately after the sequence".into(),
        "building_destruction_led_to_kills" => "Building destruction led into the frags".into(),
        "kills_to_secure_cap" => "Frags directly secured a capture".into(),
        "kill_sequence_blocked_capture" => "Frag sequence denied a capture".into(),
        "kill_sequence_led_to_payload_progress" => "Frag sequence enabled payload progress".into(),
        "bookmark" => "User-bookmarked moment".into(),
        other => humanize_identifier(other),
    }
}

fn human_join(values: &[String]) -> String {
    match values {
        [] => String::new(),
        [only] => only.clone(),
        [first, second] => format!("{first} and {second}"),
        _ => format!(
            "{}, and {}",
            values[..values.len() - 1].join(", "),
            values.last().unwrap()
        ),
    }
}

fn candidate_player_name(candidate: &Candidate) -> String {
    candidate
        .extra
        .get("attacker_name")
        .and_then(nonempty_json_string)
        .or_else(|| {
            candidate
                .kills
                .first()
                .and_then(|kill| kill.get("attacker_name"))
                .and_then(nonempty_json_string)
        })
        .or_else(|| {
            candidate
                .kills
                .first()
                .and_then(|kill| kill.get("state_evidence"))
                .and_then(|state| state.get("attacker"))
                .and_then(|attacker| attacker.get("name"))
                .and_then(nonempty_json_string)
        })
        .or_else(|| {
            (candidate.demo_context.pov_player_user_id == Some(candidate.attacker_user_id))
                .then_some(candidate.demo_context.header_nick.as_deref())
                .flatten()
        })
        .map(str::to_owned)
        .unwrap_or_else(|| format!("Player #{}", candidate.attacker_user_id))
}

fn kill_notes(kill: &serde_json::Value) -> Vec<String> {
    let mut notes = Vec::new();
    let state = kill.get("state_evidence");
    let projectile = state.and_then(|value| value.get("projectile"));
    let airborne = state
        .and_then(|value| value.get("victim_airborne_before_projectile_impact"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let airshot_eligible = projectile
        .and_then(|value| value.get("airshot_eligible"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if airborne && airshot_eligible {
        notes.push("confirmed airshot".into());
    }
    if airborne
        && projectile
            .and_then(|value| value.get("impact_proximity"))
            .and_then(serde_json::Value::as_str)
            == Some("direct")
    {
        notes.push("direct hit".into());
    }
    if state
        .and_then(|value| value.get("confirmed_double_donk"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        notes.push("double donk".into());
    }
    if state
        .and_then(|value| value.get("confirmed_kritzkrieg_boost"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        notes.push("Kritzkrieg boosted".into());
    }
    if state
        .and_then(|value| value.get("confirmed_uber_drop"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        notes.push("Über drop".into());
    }
    if kill
        .get("rocket_jump_victim")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
        && !notes.iter().any(|note| note == "confirmed airshot")
    {
        notes.push("rocket-jumping opponent".into());
    }
    notes
}

fn candidate_detail_text(candidate: &Candidate) -> CandidateDetailText {
    let player = candidate_player_name(candidate);
    let class = if candidate.attacker_class.is_empty() {
        "Unknown class".into()
    } else {
        humanize_identifier(&candidate.attacker_class)
    };
    let team = if candidate.attacker_team.is_empty() {
        "Unknown team".into()
    } else {
        candidate.attacker_team.to_ascii_uppercase()
    };
    let map = if candidate.map_name.is_empty() {
        "Unknown map".into()
    } else {
        candidate.map_name.clone()
    };
    let demo_name = Path::new(&candidate.source_demo)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(candidate.source_demo.as_str());
    let round = if candidate.round_index > 0 {
        format!(" • Round {}", candidate.round_index)
    } else {
        String::new()
    };
    let map_meta = if demo_name.is_empty() {
        format!("{}{}", candidate_server_type(candidate), round)
    } else {
        format!(
            "{}{} • {}",
            candidate_server_type(candidate),
            round,
            demo_name
        )
    };
    let kill_count = candidate.kill_count();
    let kill_word = if kill_count == 1 { "kill" } else { "kills" };

    let mut victims = Vec::new();
    let mut weapons = BTreeSet::new();
    let mut kill_lines = Vec::new();
    for (index, kill) in candidate.kills.iter().enumerate() {
        let victim_id = kill
            .get("victim_user_id")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or_default();
        let victim_name = kill
            .get("victim_name")
            .and_then(nonempty_json_string)
            .or_else(|| {
                kill.get("state_evidence")
                    .and_then(|state| state.get("victim"))
                    .and_then(|victim| victim.get("name"))
                    .and_then(nonempty_json_string)
            })
            .map(str::to_owned)
            .unwrap_or_else(|| format!("Player #{victim_id}"));
        let victim_class = kill
            .get("victim_class")
            .and_then(nonempty_json_string)
            .map(humanize_identifier)
            .unwrap_or_default();
        let victim_display = if victim_class.is_empty() {
            victim_name
        } else {
            format!("{victim_name} ({victim_class})")
        };
        if !victims.contains(&victim_display) {
            victims.push(victim_display.clone());
        }
        let weapon = humanize_weapon(
            kill.get("weapon")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        if weapon != "Unknown weapon" {
            weapons.insert(weapon.clone());
        }
        let tick = kill
            .get("demo_tick")
            .or_else(|| kill.get("tick"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or_default();
        let notes = kill_notes(kill);
        let note = if notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", notes.join(", "))
        };
        kill_lines.push(format!(
            "{}. Tick {} — {} with {}{}",
            index + 1,
            tick,
            victim_display,
            weapon,
            note
        ));
    }

    let sequence_seconds = candidate
        .metrics
        .get("duration_seconds")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or_else(|| {
            let first = candidate
                .kills
                .first()
                .and_then(|kill| kill.get("server_tick"))
                .and_then(serde_json::Value::as_i64)
                .unwrap_or_default();
            let last = candidate
                .kills
                .last()
                .and_then(|kill| kill.get("server_tick"))
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(first);
            (last - first).max(0) as f64 / 66.666_666_7
        });
    let mut summary = if kill_count == 0 {
        if candidate.bookmark_comment.trim().is_empty() {
            format!("{player} has a bookmarked moment on {map}.")
        } else {
            format!(
                "{player} has a bookmarked moment on {map}: {}",
                candidate.bookmark_comment.trim()
            )
        }
    } else if kill_count == 1 {
        format!("{player}, playing {class} for {team}, gets a kill on {map}")
    } else {
        format!("{player}, playing {class} for {team}, gets a {kill_count}-kill sequence on {map}")
    };
    if kill_count > 1 && sequence_seconds > 0.0 {
        summary.push_str(&format!(" over {sequence_seconds:.1} seconds"));
    }
    if !victims.is_empty() {
        summary.push_str(&format!(", eliminating {}", human_join(&victims)));
    }
    let weapons = weapons.into_iter().collect::<Vec<_>>();
    if !weapons.is_empty() {
        summary.push_str(&format!(" with {}", human_join(&weapons)));
    }
    if !summary.ends_with('.') {
        summary.push('.');
    }

    let highlight_priority = [
        "team_wipe",
        "kills_to_secure_cap",
        "capture_denial_followup",
        "round_clinch",
        "uber_drop",
        "medic_force",
        "double_airshot_sequence",
        "confirmed_airshot",
        "direct_airshot",
        "double_donk",
        "market_garden",
        "backstab",
        "taunt_kill",
        "shield_bash_kill",
        "charge_melee_kill",
        "three_kill",
        "four_kill_plus",
        "rapid_sequence",
        "player_count_swing",
        "medic_pick",
        "demoman_pick",
        "late_round",
    ];
    let highlights = highlight_priority
        .iter()
        .filter(|wanted| candidate.tags.iter().any(|tag| tag.as_str() == **wanted))
        .take(5)
        .map(|tag| friendly_tag(tag))
        .collect::<Vec<_>>();
    if !highlights.is_empty() {
        summary.push_str(&format!(" Highlights include {}.", human_join(&highlights)));
    }

    let score_breakdown = if candidate.score_breakdown.is_empty() {
        "No itemized score evidence is available for this imported candidate.".into()
    } else {
        candidate
            .score_breakdown
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let reason = friendly_score_reason(
                    item.get("reason")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("score adjustment"),
                );
                let points = item
                    .get("points")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or_default();
                let tick = item
                    .get("event_tick")
                    .and_then(serde_json::Value::as_i64)
                    .filter(|tick| *tick > 0)
                    .map(|tick| format!(" at server tick {tick}"))
                    .unwrap_or_default();
                let count = item
                    .get("count")
                    .and_then(serde_json::Value::as_u64)
                    .filter(|count| *count > 0)
                    .map(|count| format!(" ({count} counted)"))
                    .unwrap_or_default();
                format!("{}. {reason}: {points:+.1} points{tick}{count}", index + 1)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    CandidateDetailText {
        player,
        player_meta: format!("{class} • {team} • User ID #{}", candidate.attacker_user_id),
        map,
        map_meta,
        score: format!("{:.1} POINTS", candidate.overall_score),
        score_meta: format!(
            "{kill_count} {kill_word} • Clip ticks {}–{}",
            candidate.clip_start_tick, candidate.clip_end_tick
        ),
        summary,
        kills: if kill_lines.is_empty() {
            "No parsed kill events are attached to this bookmarked candidate.".into()
        } else {
            kill_lines.join("\n")
        },
        score_breakdown,
        tags: if candidate.tags.is_empty() {
            "No frag tags were assigned.".into()
        } else {
            candidate
                .tags
                .iter()
                .map(|tag| friendly_tag(tag))
                .collect::<Vec<_>>()
                .join("  •  ")
        },
    }
}

fn filter_summary(selected: &BTreeSet<String>, plural: &str) -> String {
    match selected.len() {
        0 => format!("All {plural}"),
        1 => selected.iter().next().cloned().unwrap_or_default(),
        count => format!("{count} {plural} selected"),
    }
}

fn filter_dropdown_width(values: &BTreeSet<String>, minimum: i32, supporting_text: &[&str]) -> i32 {
    let longest = values
        .iter()
        .map(|value| value.chars().count())
        .chain(supporting_text.iter().map(|value| value.chars().count()))
        .max()
        .unwrap_or_default();
    let estimated = longest.saturating_mul(8).saturating_add(52);
    minimum.max(estimated.min(i32::MAX as usize) as i32)
}

fn candidate_tags_width(rows: &[CandidateRow]) -> i32 {
    let longest = rows
        .iter()
        .map(|row| row.tags.as_str().chars().count())
        .max()
        .unwrap_or_default();
    let estimated = longest.saturating_mul(8).saturating_add(24);
    160.max(estimated.min(i32::MAX as usize) as i32)
}

fn selectable_options(
    values: BTreeSet<String>,
    selected: &BTreeSet<String>,
    search: &str,
) -> Vec<FilterOption> {
    let search = search.trim().to_ascii_lowercase();
    values
        .into_iter()
        .filter(|value| search.is_empty() || value.to_ascii_lowercase().contains(&search))
        .map(|value| FilterOption {
            selected: selected.contains(&value),
            value: value.into(),
        })
        .collect()
}

fn sync_candidate_filter_controls(ui: &AppWindow, state: &State) {
    let maps = state
        .candidates
        .iter()
        .filter_map(|candidate| {
            (!candidate.map_name.is_empty()).then(|| candidate.map_name.clone())
        })
        .collect::<BTreeSet<_>>();
    let server_types = state
        .candidates
        .iter()
        .map(|candidate| candidate_server_type(candidate).to_owned())
        .collect::<BTreeSet<_>>();
    let filters = &state.candidate_filters;

    ui.set_map_filter_width(filter_dropdown_width(
        &maps,
        180,
        &["All maps", "Search maps...", "maps selected"],
    ));
    ui.set_server_type_filter_width(filter_dropdown_width(
        &server_types,
        220,
        &[
            "All server types",
            "Search server types...",
            "server types selected",
        ],
    ));

    ui.set_map_filter_options(ModelRc::new(VecModel::from(selectable_options(
        maps,
        &filters.maps,
        &filters.map_search,
    ))));
    ui.set_server_type_filter_options(ModelRc::new(VecModel::from(selectable_options(
        server_types,
        &filters.server_types,
        &filters.server_type_search,
    ))));
    ui.set_map_filter_summary(filter_summary(&filters.maps, "maps").into());
    ui.set_class_filter_summary(filter_summary(&filters.classes, "classes").into());
    ui.set_server_type_filter_summary(filter_summary(&filters.server_types, "server types").into());
    ui.set_tag_filter(filters.tag_query.clone().into());
    ui.set_map_filter_search(filters.map_search.clone().into());
    ui.set_server_type_filter_search(filters.server_type_search.clone().into());
    ui.set_recorded_filter(
        match filters.recorded {
            Some(true) => "Yes",
            Some(false) => "No",
            None => "Any",
        }
        .into(),
    );

    let has_class = |class: &str| {
        filters
            .classes
            .iter()
            .any(|selected| selected.eq_ignore_ascii_case(class))
    };
    ui.set_selected_scout(has_class("Scout"));
    ui.set_selected_soldier(has_class("Soldier"));
    ui.set_selected_pyro(has_class("Pyro"));
    ui.set_selected_demoman(has_class("Demoman"));
    ui.set_selected_heavy(has_class("Heavy"));
    ui.set_selected_engineer(has_class("Engineer"));
    ui.set_selected_medic(has_class("Medic"));
    ui.set_selected_sniper(has_class("Sniper"));
    ui.set_selected_spy(has_class("Spy"));
}

struct State {
    demos: Vec<PathBuf>,
    export_root: Option<PathBuf>,
    candidates: Vec<Candidate>,
    recorded: Vec<bool>,
    visible: Vec<usize>,
    selected: Vec<bool>,
    candidate_filters: CandidateUiFilters,
    settings: AppSettings,
    controller: Option<BatchController>,
    recording_index: RecordingIndex,
    last_recording_session: Option<PathBuf>,
    recording_active: bool,
    recovery_active: bool,
}

fn crash_log_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("TF2FragDemoHelper")
        .join("crash.log")
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|value| (*value).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-text Rust panic".into())
}

fn install_panic_logger() {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let path = crash_log_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut output) = OpenOptions::new().create(true).append(true).open(&path) {
            let message = info
                .payload()
                .downcast_ref::<&str>()
                .map(|value| (*value).to_owned())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "non-text Rust panic".into());
            let location = info
                .location()
                .map(|location| {
                    format!(
                        "{}:{}:{}",
                        location.file(),
                        location.line(),
                        location.column()
                    )
                })
                .unwrap_or_else(|| "unknown location".into());
            let _ = writeln!(
                output,
                "\n[{}] PANIC: {}\nLocation: {}\nBacktrace:\n{}\n",
                chrono::Local::now().to_rfc3339(),
                message,
                location,
                Backtrace::force_capture(),
            );
        }
        previous(info);
    }));
}

fn main() -> Result<()> {
    install_panic_logger();
    if let Some(command) = std::env::args().nth(1) {
        if command == "--analyze-export" {
            let export = PathBuf::from(std::env::args().nth(2).context("missing export path")?);
            analyzer::analyze_export(&export, None)?;
            return Ok(());
        }
    }

    let recovered_recording_profile = recover_interrupted_profile()?;
    let ui = AppWindow::new()?;
    let settings = AppSettings::load();
    // Ensure first-run defaults and any normalized legacy values exist on
    // disk immediately. All later Recording Settings changes remain autosaved.
    let _ = settings.save();
    ui.set_export_directory(settings.output_directory.display().to_string().into());
    ui.set_item_schema(settings.item_schema.display().to_string().into());
    ui.set_tf2_path(settings.tf2_executable.display().to_string().into());
    ui.set_hlae_path(settings.hlae_executable.display().to_string().into());
    ui.set_ffmpeg_path(settings.ffmpeg_executable.display().to_string().into());
    ui.set_recording_directory(
        settings
            .recording_output_directory
            .display()
            .to_string()
            .into(),
    );
    ui.set_lead_seconds(settings.lead_seconds.min(60) as i32);
    ui.set_outro_seconds(settings.outro_seconds.min(60) as i32);
    ui.set_capture_fps(settings.capture_fps.to_string().into());
    ui.set_jpg_quality(settings.jpg_quality as i32);
    ui.set_recording_format(settings.recording_format.clone().into());
    ui.set_camera_mode(settings.camera_mode.clone().into());
    ui.set_mp4_compatibility(settings.mp4_compatibility.clone().into());
    ui.set_mp4_video_codec(settings.mp4_video_codec.clone().into());
    ui.set_mp4_pixel_format(settings.mp4_pixel_format.clone().into());
    ui.set_mp4_h264_profile(settings.mp4_h264_profile.clone().into());
    ui.set_mp4_crf(settings.mp4_crf as i32);
    ui.set_mp4_encoder_preset(settings.mp4_encoder_preset.clone().into());
    ui.set_mp4_audio_codec(settings.mp4_audio_codec.clone().into());
    ui.set_mp4_audio_bitrate(settings.mp4_audio_bitrate_kbps.to_string().into());
    ui.set_avi_video_codec(settings.avi_video_codec.clone().into());
    ui.set_avi_pixel_format(settings.avi_pixel_format.clone().into());
    ui.set_dnxhr_profile(settings.dnxhr_profile.clone().into());
    ui.set_performance_profile(settings.performance_profile.clone().into());
    ui.set_resolution(settings.resolution.clone().into());
    ui.set_dx_level(settings.dx_level.clone().into());
    ui.set_skybox(settings.skybox.clone().into());
    ui.set_hud(settings.hud.clone().into());
    ui.set_viewmodels(settings.viewmodels.clone().into());
    ui.set_viewmodel_fov(settings.viewmodel_fov as i32);
    ui.set_maximum_graphics(settings.maximum_graphics);
    ui.set_motion_blur(settings.motion_blur);
    ui.set_disable_hit_sounds(settings.disable_hit_sounds);
    ui.set_disable_voice_chat(settings.disable_voice_chat);
    ui.set_minimal_hud(settings.minimal_hud);
    ui.set_disable_combat_text(settings.disable_combat_text);
    ui.set_disable_crosshair(settings.disable_crosshair);
    ui.set_disable_crosshair_switching(settings.disable_crosshair_switching);
    ui.set_hud_player_model(settings.hud_player_model);
    ui.set_isolate_custom_resources(settings.isolate_custom_resources);
    ui.set_disable_announcer(settings.disable_announcer_voices);
    ui.set_disable_applause(settings.disable_applause_sounds);
    ui.set_disable_domination(settings.disable_domination_sounds);
    ui.set_custom_resources(
        settings
            .custom_resources
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("; ")
            .into(),
    );
    update_output_description(&ui);
    if recovered_recording_profile {
        ui.set_status_text("Recovered TF2 files from an interrupted recording session".into());
    }

    let state = Arc::new(Mutex::new(State {
        demos: Vec::new(),
        export_root: None,
        candidates: Vec::new(),
        recorded: Vec::new(),
        visible: Vec::new(),
        selected: Vec::new(),
        candidate_filters: CandidateUiFilters::default(),
        settings,
        controller: None,
        recording_index: RecordingIndex::empty(),
        last_recording_session: None,
        recording_active: false,
        recovery_active: true,
    }));
    bind_file_callbacks(&ui, &state);
    bind_batch_callbacks(&ui, &state);
    bind_candidate_callbacks(&ui, &state);
    bind_settings_callbacks(&ui, &state);
    bind_demo_file_drop(&ui, &state);
    start_recording_recovery(&ui, &state);
    center_window_on_startup(&ui);
    ui.run()?;
    if let Err(error) = shutdown_active_recording() {
        rfd::MessageDialog::new()
            .set_title("TF2 Recording Restore Warning")
            .set_description(format!("TF2 recording files could not be fully restored. Your backups were retained.\n\n{error}"))
            .set_level(rfd::MessageLevel::Error)
            .show();
        return Err(error);
    }
    Ok(())
}

/// Position the window once the native Slint/winit window is available.
///
/// This deliberately runs on the event loop rather than at construction time:
/// winit only exposes the monitor and the decorated outer size after the window
/// has been created. The window is horizontally centered and placed in the
/// upper half of the display selected by the operating system at launch.
fn center_window_on_startup(ui: &AppWindow) {
    let weak = ui.as_weak();
    slint::Timer::single_shot(Duration::ZERO, move || {
        let Some(ui) = weak.upgrade() else { return };
        let _ = ui.window().with_winit_window(|window| {
            let size = window.inner_size();
            update_recording_layout_for_size(&ui, size.width, size.height, window.scale_factor());
            let Some(monitor) = window
                .current_monitor()
                .or_else(|| window.available_monitors().next())
            else {
                return;
            };

            let monitor_position = monitor.position();
            let monitor_size = monitor.size();
            let window_size = window.outer_size();
            let horizontal_margin = monitor_size.width.saturating_sub(window_size.width);
            let vertical_margin = monitor_size.height.saturating_sub(window_size.height);
            let x = monitor_position.x + (horizontal_margin / 2) as i32;
            // Keep more open space below the app than above it, while never
            // placing the title bar outside of the active display.
            let y = monitor_position.y + (vertical_margin / 4) as i32;

            window.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
        });
    });
}

/// Report the exact native and logical client sizes in the header and use the
/// accordion when the narrow client area drops below the last height where the
/// normal settings layout can keep its action buttons visible. Converting with
/// winit's live scale factor makes the result independent of Windows display
/// scaling.
fn update_recording_layout_for_size(
    ui: &AppWindow,
    physical_width: u32,
    physical_height: u32,
    scale: f64,
) {
    let logical_width = (physical_width as f64 / scale).round() as u32;
    let logical_height = (physical_height as f64 / scale).round() as u32;
    // 900x680 is the final normal-layout size; one pixel below it switches to
    // the five-button accordion before Save Settings can be covered.
    let accordion = logical_width <= 940 && logical_height < 680;
    ui.set_minimal_recording_settings(accordion);
    ui.set_frame_debug(format!("WINDOW {} x {}", logical_width, logical_height).into());
}

fn start_recording_recovery(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    // Do not enumerate the Recording Sessions directory or load the recording
    // index on the UI thread. Slint's event loop must start immediately so the
    // window can paint, move, and close even when retained data is damaged.
    let settings = state.lock().settings.clone();
    let idle_status = ui.get_status_text().to_string();
    ui.set_status_text("Checking retained HLAE recording sessions in the background...".into());
    set_background_process(ui, "CHECKING RECORDING RECOVERY", true);
    let weak = ui.as_weak();
    let recovery_state = state.clone();
    thread::spawn(move || {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let report = recover_recording_sessions(&settings);
            let recording_index = RecordingIndex::load();
            let latest_session = latest_recording_session();
            (report, recording_index, latest_session)
        }));
        let (status, recording_index, latest_session) = match result {
            Ok((report, recording_index, latest_session)) => {
                let had_recovery_work = report.scanned_sessions > 0
                    || report.deferred_sessions > 0
                    || report.disabled_sessions > 0
                    || !report.errors.is_empty();
                let status = if had_recovery_work {
                    report.summary()
                } else {
                    idle_status
                };
                (status, recording_index, latest_session)
            }
            Err(payload) => (
                format!(
                    "Recording-session recovery stopped safely: {}",
                    panic_payload_message(payload.as_ref())
                ),
                RecordingIndex::empty(),
                None,
            ),
        };
        let _ = slint::invoke_from_event_loop(move || {
            let Some(ui) = weak.upgrade() else { return };
            {
                let mut current = recovery_state.lock();
                current.recovery_active = false;
                current.recording_index = recording_index;
                current.last_recording_session = latest_session;
            }
            recompute_recorded_status(&recovery_state);
            let filter = ui.get_filter_text().to_string();
            let score = ui.get_minimum_score();
            refresh_candidates(&ui, &recovery_state, &filter, score);
            ui.set_status_text(status.into());
            set_background_process(&ui, "READY", false);
        });
    });
}

fn set_background_process(ui: &AppWindow, status: &str, active: bool) {
    ui.set_background_process_status(status.into());
    ui.set_background_process_active(active);
    ui.set_recording_ready(!active);
}

fn recording_background_label(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("consolidating recorded candidate") {
        message.to_ascii_uppercase()
    } else if lower.contains("consolidating completed") {
        "CONSOLIDATING COMPLETED RECORDINGS".into()
    } else if lower.contains("restoring") {
        "RESTORING TF2 FILES".into()
    } else if lower.contains("archiving") || lower.contains("finaliz") {
        "FINALIZING OUTPUTS".into()
    } else if lower.contains("hlae started") || lower.contains("recording") {
        "HLAE RECORDING ACTIVE".into()
    } else {
        "RECORDING WORK IN PROGRESS".into()
    }
}

fn bind_demo_file_drop(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let weak = ui.as_weak();
    let drop_state = state.clone();
    ui.window().on_winit_window_event(move |window, event| {
        if let Some(ui) = weak.upgrade() {
            match event {
                winit::event::WindowEvent::Resized(size) => {
                    update_recording_layout_for_size(
                        &ui,
                        size.width,
                        size.height,
                        f64::from(window.scale_factor()),
                    );
                }
                winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    let size = window.size();
                    update_recording_layout_for_size(
                        &ui,
                        size.width,
                        size.height,
                        *scale_factor,
                    );
                }
                _ => {}
            }
        }
        if matches!(event, winit::event::WindowEvent::CloseRequested) && drop_state.lock().recording_active {
            let choice = rfd::MessageDialog::new()
                .set_title("Recording Finalization Is Still Active")
                .set_description("TF2 recording, output finalization, or interrupted-session recovery is still running. Closing now can interrupt muxing or file transfers.\n\nYes: close anyway and preserve the session for recovery on the next launch\nNo: keep the helper open until finalization finishes")
                .set_buttons(rfd::MessageButtons::YesNo)
                .set_level(rfd::MessageLevel::Warning)
                .show();
            return if choice == rfd::MessageDialogResult::Yes {
                EventResult::Propagate
            } else {
                EventResult::PreventDefault
            };
        }
        match event {
            winit::event::WindowEvent::HoveredFile(path) => {
                if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("dem")) {
                    if let Some(ui) = weak.upgrade() { ui.set_demo_drop_active(true); }
                } else {
                    if let Some(ui) = weak.upgrade() { ui.set_demo_drop_active(false); }
                }
                return EventResult::PreventDefault;
            }
            winit::event::WindowEvent::HoveredFileCancelled => {
                if let Some(ui) = weak.upgrade() { ui.set_demo_drop_active(false); }
                return EventResult::PreventDefault;
            }
            _ => {}
        }
        let winit::event::WindowEvent::DroppedFile(path) = event else { return EventResult::Propagate };
        if let Some(ui) = weak.upgrade() { ui.set_demo_drop_active(false); }
        if !path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("dem")) {
            if let Some(ui) = weak.upgrade() { ui.set_status_text("Only TF2 .dem files can be dropped here".into()); }
            return EventResult::PreventDefault;
        }
        let additions = vec![path.clone()];
        let (files, tf2, schema) = {
            let mut current = drop_state.lock();
            for addition in additions {
                if !current.demos.contains(&addition) { current.demos.push(addition); }
            }
            current.demos.sort();
            let files = current.demos.clone();
            let tf2 = current.settings.tf2_executable.is_file().then(|| current.settings.tf2_executable.clone())
                .or_else(|| discover_tf2_executable(&files));
            if let Some(path) = &tf2 { current.settings.tf2_executable = path.clone(); }
            let schema = current.settings.item_schema.is_file().then(|| current.settings.item_schema.clone())
                .or_else(|| discover_item_schema(&files, tf2.as_deref()));
            if let Some(path) = &schema { current.settings.item_schema = path.clone(); }
            let _ = current.settings.save();
            (files, tf2, schema)
        };
        if let Some(ui) = weak.upgrade() {
            set_imported_demos(&ui, &files);
            if let Some(path) = tf2 { ui.set_tf2_path(path.display().to_string().into()); }
            if let Some(path) = schema { ui.set_item_schema(path.display().to_string().into()); }
            ui.set_status_text(format!("Loaded {} dropped demo{}", files.len(), if files.len() == 1 { "" } else { "s" }).into());
            update_batch_preflight(&ui, &drop_state);
        }
        EventResult::PreventDefault
    });
}

fn bind_file_callbacks(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let weak = ui.as_weak();
    let demos_state = state.clone();
    ui.on_choose_demos(move || {
        let files = rfd::FileDialog::new()
            .add_filter("TF2 demos", &["dem"])
            .pick_files()
            .unwrap_or_default();
        if files.is_empty() {
            return;
        }
        let (tf2, schema) = {
            let mut current = demos_state.lock();
            current.demos = files.clone();
            let tf2 = current
                .settings
                .tf2_executable
                .is_file()
                .then(|| current.settings.tf2_executable.clone())
                .or_else(|| discover_tf2_executable(&files));
            if let Some(path) = &tf2 {
                current.settings.tf2_executable = path.clone();
            }
            let schema = current
                .settings
                .item_schema
                .is_file()
                .then(|| current.settings.item_schema.clone())
                .or_else(|| discover_item_schema(&files, tf2.as_deref()));
            if let Some(path) = &schema {
                current.settings.item_schema = path.clone();
            }
            let _ = current.settings.save();
            (tf2, schema)
        };
        if let Some(ui) = weak.upgrade() {
            set_imported_demos(&ui, &files);
            if let Some(path) = tf2 {
                ui.set_tf2_path(path.display().to_string().into());
            }
            if let Some(path) = schema {
                ui.set_item_schema(path.display().to_string().into());
            }
            update_batch_preflight(&ui, &demos_state);
        }
    });
    let weak = ui.as_weak();
    let output_state = state.clone();
    ui.on_choose_export_directory(move || {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            if let Some(ui) = weak.upgrade() {
                ui.set_export_directory(path.display().to_string().into());
                update_batch_preflight(&ui, &output_state);
            }
        }
    });
    let weak = ui.as_weak();
    ui.on_choose_item_schema(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("TF2 item schema", &["txt"])
            .pick_file()
        {
            if let Some(ui) = weak.upgrade() {
                ui.set_item_schema(path.display().to_string().into());
            }
        }
    });
}

fn set_imported_demos(ui: &AppWindow, demos: &[PathBuf]) {
    let names = demos
        .iter()
        .map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string())
                .into()
        })
        .collect::<Vec<SharedString>>();
    ui.set_imported_demo_count(demos.len().min(i32::MAX as usize) as i32);
    ui.set_imported_demos(ModelRc::new(VecModel::from(names)));
}

fn bind_batch_callbacks(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let weak = ui.as_weak();
    let state_for_start = state.clone();
    ui.on_start_batch(move || {
        let Some(ui) = weak.upgrade() else { return };
        let demos = state_for_start.lock().demos.clone();
        if demos.is_empty() {
            ui.set_status_text("Choose one or more .dem files".into());
            return;
        }
        let output = PathBuf::from(ui.get_export_directory().to_string());
        let schema_text = ui.get_item_schema().to_string();
        let mut schema = (!schema_text.trim().is_empty()).then(|| PathBuf::from(schema_text)).filter(|path| path.is_file());
        if schema.is_none() {
            let tf2 = PathBuf::from(ui.get_tf2_path().to_string());
            schema = discover_item_schema(&demos, tf2.is_file().then_some(tf2.as_path()));
            if let Some(path) = &schema { ui.set_item_schema(path.display().to_string().into()); }
        }
        let performance_profile = PerformanceProfile::from_setting(&ui.get_performance_profile().to_string());
        let preflight = match batch::estimate_batch_preflight(&demos, &output, performance_profile) {
            Ok(estimate) => estimate,
            Err(error) => {
                ui.set_batch_estimate(format!("Pre-flight failed: {error}").into());
                ui.set_status_text(format!("Cannot start: {error}").into());
                return;
            }
        };
        ui.set_batch_estimate(preflight.summary().into());
        if !preflight.has_enough_space() {
            rfd::MessageDialog::new()
                .set_title("Insufficient Export Space")
                .set_description(preflight.summary())
                .set_level(rfd::MessageLevel::Error)
                .show();
            ui.set_status_text("Parsing blocked because the export drive does not have enough free space".into());
            return;
        }
        let confirmation = rfd::MessageDialog::new()
            .set_title("Parse and Analyze Pre-flight")
            .set_description(format!("{}\n\nStart parsing and analysis now?", preflight.summary()))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        if confirmation != rfd::MessageDialogResult::Yes {
            ui.set_status_text("Parsing cancelled after pre-flight review".into());
            return;
        }
        let controller = BatchController::new();
        state_for_start.lock().controller = Some(controller.clone());
        ui.set_busy(true);
        ui.set_has_export(false);
        ui.set_selected_count(0);
        ui.set_all_visible_selected(false);
        ui.set_selected_page(0);
        ui.set_progress_value(0.0);
        ui.set_log_text("".into());
        ui.set_status_text("Preparing Rust resource plan...".into());
        let weak_for_thread = weak.clone();
        let state_for_thread = state_for_start.clone();
        thread::spawn(move || {
            let progress_weak = weak_for_thread.clone();
            let sink: batch::ProgressSink = Arc::new(move |event| {
                let weak = progress_weak.clone();
                let _ = slint::invoke_from_event_loop(move || update_progress(&weak, event));
            });
            let failure_sink = sink.clone();
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                batch::run_batch(demos, output, schema, performance_profile, controller, sink)
            }));
            let result = match result {
                Ok(result) => result,
                Err(payload) => {
                    let message = panic_payload_message(payload.as_ref());
                    let log_path = crash_log_path();
                    failure_sink(ProgressEvent::Failed(format!(
                        "Analyzer crashed: {message}. Crash report: {}",
                        log_path.display(),
                    )));
                    Err(anyhow::anyhow!("analyzer crashed: {message}"))
                }
            };
            if let Err(error) = &result {
                if error.to_string() != "cancelled" && !error.to_string().starts_with("analyzer crashed:") {
                    failure_sink(ProgressEvent::Failed(error.to_string()));
                }
            }
            if let Ok(root) = result {
                match load_candidates(&root.join("frag_candidates.ndjson")) {
                    Ok(candidates) => {
                        let selected = vec![false; candidates.len()];
                        let mut state = state_for_thread.lock();
                        let recorded = candidates.iter().map(|candidate| state.recording_index.is_recorded_indexed(candidate)).collect::<Vec<_>>();
                        let filters = CandidateUiFilters::default();
                        let (visible, rows) = build_candidate_rows(&candidates, &recorded, &selected, &filters, 0);
                        state.export_root = Some(root.clone());
                        state.selected = selected;
                        state.recorded = recorded;
                        state.visible = visible;
                        state.candidates = candidates;
                        state.candidate_filters = filters;
                        drop(state);
                        let state = state_for_thread.clone();
                        let weak = weak_for_thread.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak.upgrade() {
                                let count = rows.len();
                                ui.set_candidate_summary(format!("{count} of {count} ranked candidates").into());
                                ui.set_candidate_tags_width(candidate_tags_width(&rows));
                                ui.set_candidate_rows(ModelRc::new(VecModel::from(rows)));
                                ui.set_selected_count(0);
                                ui.set_all_visible_selected(false);
                                {
                                    let current = state.lock();
                                    sync_candidate_filter_controls(&ui, &current);
                                }
                                update_recording_estimate(&ui, &state);
                                ui.set_has_export(true);
                                ui.set_selected_page(1);
                            }
                        });
                    }
                    Err(error) => {
                        let weak = weak_for_thread.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak.upgrade() {
                                ui.set_status_text(format!("Analysis completed but candidates could not be loaded: {error}").into());
                            }
                        });
                    }
                }
            }
        });
    });

    let state_for_cancel = state.clone();
    ui.on_cancel_batch(move || {
        if let Some(controller) = &state_for_cancel.lock().controller {
            controller.cancel();
        }
    });

    let weak = ui.as_weak();
    let state_for_load = state.clone();
    ui.on_load_export(move || {
        let Some(root) = rfd::FileDialog::new().pick_folder() else { return };
        let path = if root.join("frag_candidates.ndjson").is_file() { root.join("frag_candidates.ndjson") } else { return };
        if let Some(ui) = weak.upgrade() {
            ui.set_busy(true);
            ui.set_status_text("Loading parsed export: reading candidates…".into());
        }
        let weak_for_thread = weak.clone();
        let state_for_thread = state_for_load.clone();
        thread::spawn(move || {
            let result = load_candidates(&path).map_err(|error| error.to_string());
            if let Ok(candidates) = &result {
                let count = candidates.len();
                let weak = weak_for_thread.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak.upgrade() {
                        ui.set_status_text(format!("Loaded {count} candidates: preparing table and recorded status…").into());
                    }
                });
            }
            let result = result.map(|candidates| {
                let index = state_for_thread.lock().recording_index.clone();
                let recorded = candidates.iter().map(|candidate| index.is_recorded_indexed(candidate)).collect::<Vec<_>>();
                let selected = vec![false; candidates.len()];
                let filters = CandidateUiFilters::default();
                let (visible, rows) = build_candidate_rows(&candidates, &recorded, &selected, &filters, 0);
                (candidates, recorded, selected, visible, rows, filters)
            });
            let _ = slint::invoke_from_event_loop(move || {
                let Some(ui) = weak_for_thread.upgrade() else { return };
                match result {
                    Ok((candidates, recorded, selected, visible, rows, filters)) => {
                        let count = candidates.len();
                        {
                            let mut state = state_for_thread.lock();
                            state.export_root = Some(root.clone());
                            state.selected = selected;
                            state.recorded = recorded;
                            state.visible = visible;
                            state.candidates = candidates;
                            state.candidate_filters = filters;
                        }
                        ui.set_candidate_summary(format!("{count} of {count} ranked candidates").into());
                        ui.set_candidate_tags_width(candidate_tags_width(&rows));
                        ui.set_candidate_rows(ModelRc::new(VecModel::from(rows)));
                        ui.set_selected_count(0);
                        ui.set_all_visible_selected(false);
                        {
                            let current = state_for_thread.lock();
                            sync_candidate_filter_controls(&ui, &current);
                        }
                        update_recording_estimate(&ui, &state_for_thread);
                        ui.set_has_export(true);
                        ui.set_selected_page(1);
                        ui.set_busy(false);
                        if reconcile_recorded_outputs(ui.as_weak(), state_for_thread.clone(), root) {
                            ui.set_status_text(format!("Loaded {count} candidates — checking saved recordings in the background").into());
                        } else {
                            ui.set_status_text(format!("Loaded {count} candidates").into());
                        }
                    }
                    Err(error) => {
                        ui.set_busy(false);
                        ui.set_status_text(format!("Could not load parsed export: {error}").into());
                    }
                }
            });
        });
    });

    let weak = ui.as_weak();
    let state_for_open = state.clone();
    ui.on_open_export(move || {
        if let Some(path) = state_for_open.lock().export_root.clone() {
            if let Err(error) = open_path(&path) {
                if let Some(ui) = weak.upgrade() {
                    ui.set_status_text(error.to_string().into());
                }
            }
        }
    });
}

fn bind_candidate_callbacks(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let weak = ui.as_weak();
    let state_for_filter = state.clone();
    ui.on_refresh_filter(move |filter, score| {
        if let Some(ui) = weak.upgrade() {
            refresh_candidates(&ui, &state_for_filter, &filter, score);
        }
    });
    let weak = ui.as_weak();
    let state_for_tags = state.clone();
    ui.on_set_tag_filter(move |query| {
        let Some(ui) = weak.upgrade() else { return };
        state_for_tags.lock().candidate_filters.tag_query = query.to_string();
        refresh_candidates(&ui, &state_for_tags, "", ui.get_minimum_score());
    });
    let weak = ui.as_weak();
    let state_for_classes = state.clone();
    ui.on_toggle_class_filter(move |class| {
        let Some(ui) = weak.upgrade() else { return };
        {
            let mut state = state_for_classes.lock();
            let class = class.to_string();
            if class.eq_ignore_ascii_case("all") {
                state.candidate_filters.classes.clear();
            } else if !state.candidate_filters.classes.remove(&class) {
                state.candidate_filters.classes.insert(class);
            }
            sync_candidate_filter_controls(&ui, &state);
        }
        refresh_candidates(&ui, &state_for_classes, "", ui.get_minimum_score());
    });
    let weak = ui.as_weak();
    let state_for_maps = state.clone();
    ui.on_toggle_map_filter(move |map| {
        let Some(ui) = weak.upgrade() else { return };
        {
            let mut state = state_for_maps.lock();
            let map = map.to_string();
            if !state.candidate_filters.maps.remove(&map) {
                state.candidate_filters.maps.insert(map);
            }
            sync_candidate_filter_controls(&ui, &state);
        }
        refresh_candidates(&ui, &state_for_maps, "", ui.get_minimum_score());
    });
    let weak = ui.as_weak();
    let state_for_map_search = state.clone();
    ui.on_search_map_filters(move |query| {
        let Some(ui) = weak.upgrade() else { return };
        let mut state = state_for_map_search.lock();
        state.candidate_filters.map_search = query.to_string();
        sync_candidate_filter_controls(&ui, &state);
    });
    let weak = ui.as_weak();
    let state_for_servers = state.clone();
    ui.on_toggle_server_type_filter(move |server_type| {
        let Some(ui) = weak.upgrade() else { return };
        {
            let mut state = state_for_servers.lock();
            let server_type = server_type.to_string();
            if !state.candidate_filters.server_types.remove(&server_type) {
                state.candidate_filters.server_types.insert(server_type);
            }
            sync_candidate_filter_controls(&ui, &state);
        }
        refresh_candidates(&ui, &state_for_servers, "", ui.get_minimum_score());
    });
    let weak = ui.as_weak();
    let state_for_server_search = state.clone();
    ui.on_search_server_type_filters(move |query| {
        let Some(ui) = weak.upgrade() else { return };
        let mut state = state_for_server_search.lock();
        state.candidate_filters.server_type_search = query.to_string();
        sync_candidate_filter_controls(&ui, &state);
    });
    let weak = ui.as_weak();
    let state_for_recorded = state.clone();
    ui.on_set_recorded_filter(move |value| {
        let Some(ui) = weak.upgrade() else { return };
        state_for_recorded.lock().candidate_filters.recorded = match value.as_str() {
            "Yes" => Some(true),
            "No" => Some(false),
            _ => None,
        };
        refresh_candidates(&ui, &state_for_recorded, "", ui.get_minimum_score());
    });
    let weak = ui.as_weak();
    let state_for_drag = state.clone();
    ui.on_drag_select_candidates(move |start_index, delta_pixels, selecting, additive| {
        let Some(ui) = weak.upgrade() else { return };
        let mut state = state_for_drag.lock();
        if state.visible.is_empty() {
            return;
        }
        let last = state.visible.len().saturating_sub(1) as i32;
        let start = start_index.clamp(0, last);
        let target = (start + (delta_pixels / 30.0).round() as i32).clamp(0, last);
        let first_row = start.min(target) as usize;
        let last_row = start.max(target) as usize;
        if !additive {
            state.selected.fill(false);
        }
        let model = ui.get_candidate_rows();
        for visible_row in 0..state.visible.len() {
            if let Some(candidate_index) = state.visible.get(visible_row).copied() {
                let in_drag_range = (first_row..=last_row).contains(&visible_row);
                if in_drag_range {
                    state.selected[candidate_index] = selecting;
                }
                if let Some(mut row) = model.row_data(visible_row) {
                    let selected = state
                        .selected
                        .get(candidate_index)
                        .copied()
                        .unwrap_or(false);
                    if row.selected != selected {
                        row.selected = selected;
                        model.set_row_data(visible_row, row);
                    }
                }
            }
        }
        ui.set_selected_count(
            state
                .selected
                .iter()
                .filter(|selected| **selected)
                .count()
                .min(i32::MAX as usize) as i32,
        );
        ui.set_all_visible_selected(
            !state.visible.is_empty()
                && state
                    .visible
                    .iter()
                    .all(|index| state.selected.get(*index).copied().unwrap_or(false)),
        );
    });
    let weak = ui.as_weak();
    let selection_state = state.clone();
    ui.on_selection_changed(move || {
        if let Some(ui) = weak.upgrade() {
            update_recording_estimate(&ui, &selection_state);
        }
    });
    let weak = ui.as_weak();
    let state_for_all = state.clone();
    ui.on_select_all_visible(move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut state = state_for_all.lock();
        let visible = state.visible.clone();
        let deselect = !visible.is_empty()
            && visible
                .iter()
                .all(|index| state.selected.get(*index).copied().unwrap_or(false));
        for candidate_index in &visible {
            state.selected[*candidate_index] = !deselect;
        }
        let selected_count = state
            .selected
            .iter()
            .filter(|selected| **selected)
            .count()
            .min(i32::MAX as usize) as i32;
        drop(state);
        let model = ui.get_candidate_rows();
        let mut rows = Vec::with_capacity(model.row_count());
        for row_index in 0..model.row_count() {
            if let Some(mut row) = model.row_data(row_index) {
                row.selected = !deselect;
                rows.push(row);
            }
        }
        ui.set_candidate_rows(ModelRc::new(VecModel::from(rows)));
        ui.set_selected_count(selected_count);
        ui.set_all_visible_selected(!deselect);
        update_recording_estimate(&ui, &state_for_all);
    });
    let weak = ui.as_weak();
    let state_for_preview = state.clone();
    ui.on_preview_selected(move || {
        let Some(ui) = weak.upgrade() else { return };
        let (candidate, mut settings) = {
            let state = state_for_preview.lock();
            let selected = selected_candidates(&state)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            if selected.len() != 1 {
                ui.set_status_text("Select exactly one candidate to preview".into());
                return;
            }
            let candidate = selected[0].clone();
            (candidate, state.settings.clone())
        };

        let entered_path = PathBuf::from(ui.get_tf2_path().to_string());
        if entered_path.is_file() {
            settings.tf2_executable = entered_path;
        }
        if !settings.tf2_executable.is_file() {
            settings.tf2_executable =
                discover_tf2_executable(&[PathBuf::from(&candidate.source_demo)])
                    .unwrap_or_default();
            if settings.tf2_executable.is_file() {
                ui.set_tf2_path(settings.tf2_executable.display().to_string().into());
            }
        }
        if !settings.tf2_executable.is_file() {
            let mut dialog =
                rfd::FileDialog::new().set_title("Select the Team Fortress 2 Executable");
            if cfg!(target_os = "windows") {
                dialog = dialog.add_filter("Team Fortress 2 executable", &["exe"]);
            }
            let Some(path) = dialog.pick_file() else {
                ui.set_status_text("TF2 preview cancelled; no executable was selected".into());
                return;
            };
            settings.tf2_executable = path.clone();
            ui.set_tf2_path(path.display().to_string().into());
            let mut state = state_for_preview.lock();
            state.settings.tf2_executable = path;
            let _ = state.settings.save();
        }

        settings.lead_seconds = ui.get_lead_seconds().clamp(0, 60) as u32;

        let result = preview_candidate(&candidate, &settings);
        ui.set_status_text(
            result
                .map(|tick| format!("TF2 preview launched at demo tick {tick}"))
                .unwrap_or_else(|error| error.to_string())
                .into(),
        );
    });
    let weak = ui.as_weak();
    let state_for_record = state.clone();
    ui.on_record_selected(move || {
        let Some(ui) = weak.upgrade() else { return };
        let recording_or_recovery_active = {
            let state = state_for_record.lock();
            state.recording_active || state.recovery_active
        };
        if recording_or_recovery_active {
            ui.set_status_text("Wait for the active recording or output-recovery process to finish before starting another batch".into());
            return;
        }
        let (mut selected, mut settings, recorded_count) = {
            let state = state_for_record.lock();
            let recorded_count = state.selected.iter().zip(&state.recorded).filter(|(selected, recorded)| **selected && **recorded).count();
            (selected_candidates(&state).into_iter().cloned().collect::<Vec<_>>(), state.settings.clone(), recorded_count)
        };
        let mut replace_existing = false;
        sync_settings_from_ui(&ui, &mut settings);
        if !settings.tf2_executable.is_file() {
            let demos = selected.iter().map(|candidate| PathBuf::from(&candidate.source_demo)).collect::<Vec<_>>();
            settings.tf2_executable = discover_tf2_executable(&demos).unwrap_or_default();
        }
        if !settings.hlae_executable.is_file() { settings.hlae_executable = discover_named_executable("HLAE.exe").unwrap_or_default(); }
        if !settings.ffmpeg_executable.is_file() { settings.ffmpeg_executable = discover_named_executable(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" }).unwrap_or_default(); }
        for (kind, title, required) in [
            ("tf2", "Select the Team Fortress 2 Executable", true),
            ("hlae", "Select the HLAE Executable", true),
            ("ffmpeg", "Select the FFmpeg Executable", !settings.recording_format.contains("Image")),
        ] {
            let current = match kind { "tf2" => &settings.tf2_executable, "hlae" => &settings.hlae_executable, _ => &settings.ffmpeg_executable };
            if !required || current.is_file() { continue; }
            let Some(path) = rfd::FileDialog::new().set_title(title).pick_file() else {
                ui.set_status_text(format!("Recording cancelled; {title} was not selected").into());
                return;
            };
            match kind {
                "tf2" => { settings.tf2_executable = path.clone(); ui.set_tf2_path(path.display().to_string().into()); }
                "hlae" => { settings.hlae_executable = path.clone(); ui.set_hlae_path(path.display().to_string().into()); }
                _ => { settings.ffmpeg_executable = path.clone(); ui.set_ffmpeg_path(path.display().to_string().into()); }
            }
        }
        if recorded_count > 0 {
            let choice = rfd::MessageDialog::new()
                .set_title("Previously Recorded Candidates")
                .set_description(format!("{recorded_count} selected candidate(s) already have a matching recording.\n\nYes: record them again and replace the old output after each replacement finishes successfully\nNo: skip the previously recorded candidates and record only new selections\nCancel: do not start\n\nThe loaded candidates and their selections remain available after every batch."))
                .set_buttons(rfd::MessageButtons::YesNoCancel)
                .show();
            match choice {
                rfd::MessageDialogResult::Yes => {
                    replace_existing = true;
                }
                rfd::MessageDialogResult::No => {
                    let state = state_for_record.lock();
                    selected = state.candidates.iter().zip(&state.selected).zip(&state.recorded)
                        .filter_map(|((candidate, selected), recorded)| (*selected && !*recorded).then(|| candidate.clone()))
                        .collect();
                }
                _ => {
                    ui.set_status_text("Recording cancelled".into());
                    return;
                }
            }
        }
        if selected.is_empty() {
            ui.set_status_text("All selected candidates already have matching recordings".into());
            return;
        }
        if let Err(error) = validate_cinematic_batch(&selected, &settings) {
            let choice = rfd::MessageDialog::new()
                .set_title("Cinematic Angle Unavailable")
                .set_description(format!(
                    "{error}\n\nYes: record the selected candidates with their original camera\nNo: cancel without recording"
                ))
                .set_buttons(rfd::MessageButtons::YesNo)
                .set_level(rfd::MessageLevel::Warning)
                .show();
            if choice != rfd::MessageDialogResult::Yes {
                ui.set_status_text("Recording cancelled because the cinematic camera was unavailable".into());
                return;
            }
            settings.camera_mode = "Original Camera".into();
            ui.set_recording_settings_syncing(true);
            ui.set_camera_mode("Original Camera".into());
            ui.set_recording_settings_syncing(false);
            ui.set_status_text("Cinematic camera unavailable; recording will use the original camera".into());
        }
        let recording_preflight = match estimate_recording_space(&selected, &settings) {
            Ok(estimate) => estimate,
            Err(error) => {
                ui.set_recording_estimate(format!("Recording pre-flight failed: {error}").into());
                ui.set_status_text(format!("Recording blocked: {error}").into());
                return;
            }
        };
        ui.set_recording_estimate(recording_preflight.summary().into());
        if !recording_preflight.has_enough_space() {
            rfd::MessageDialog::new()
                .set_title("Insufficient Recording Space")
                .set_description(recording_preflight.summary())
                .set_level(rfd::MessageLevel::Error)
                .show();
            ui.set_status_text("Recording blocked because the output drive does not have enough free space".into());
            return;
        }
        let confirmation = rfd::MessageDialog::new()
            .set_title("HLAE Recording Pre-flight")
            .set_description(format!("{}\n\nStart the offline recording batch now?", recording_preflight.summary()))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        if confirmation != rfd::MessageDialogResult::Yes {
            ui.set_status_text("Recording cancelled after pre-flight review".into());
            return;
        }
        {
            let mut state = state_for_record.lock();
            state.settings = settings.clone();
            state.recording_active = true;
            let _ = state.settings.save();
        }
        set_background_process(&ui, "PREPARING HLAE RECORDING", true);
        let progress_weak = weak.clone();
        let progress_state = state_for_record.clone();
        let progress: RecordingProgressSink = Arc::new(move |event| {
            let weak = progress_weak.clone();
            let state = progress_state.clone();
            let _ = slint::invoke_from_event_loop(move || {
                let Some(ui) = weak.upgrade() else { return };
                match event {
                    RecordingProgress::Status(message) => {
                        let background = recording_background_label(&message);
                        set_background_process(&ui, &background, true);
                        ui.set_status_text(message.into());
                    }
                    RecordingProgress::ClipStarted { candidate_id, current, total } => {
                        let progress = format!("CANDIDATE {current} / {total}: {candidate_id}");
                        set_background_process(&ui, &progress, true);
                        ui.set_status_text(format!("Recording candidate {current} of {total}: {candidate_id}").into());
                    }
                    RecordingProgress::ClipCompleted { candidate_id, output_path } => {
                        set_background_process(&ui, "FINALIZING OUTPUTS", true);
                        state.lock().recording_index = RecordingIndex::load();
                        recompute_recorded_status(&state);
                        let filter = ui.get_filter_text().to_string();
                        let score = ui.get_minimum_score();
                        refresh_candidates(&ui, &state, &filter, score);
                        ui.set_status_text(format!("Recorded {candidate_id}: {}", output_path.display()).into());
                    }
                    RecordingProgress::Finished { completed, failed, session } => {
                        {
                            let mut current = state.lock();
                            current.recording_active = false;
                            current.recording_index = RecordingIndex::load();
                            current.last_recording_session = session.clone();
                        }
                        recompute_recorded_status(&state);
                        let filter = ui.get_filter_text().to_string();
                        let score = ui.get_minimum_score();
                        refresh_candidates(&ui, &state, &filter, score);
                        let status = match session {
                            Some(session) => format!("Recording finished: {completed} completed, {failed} failed. Logs: {}", session.display()),
                            None => format!("Recording finished: {completed} completed, {failed} failed. Temporary session data was cleaned up."),
                        };
                        ui.set_status_text(status.into());
                        set_background_process(&ui, "READY", false);
                    }
                }
            });
        });
        match launch_hlae_batch(&selected, &settings, replace_existing, Some(progress)) {
            Ok(path) => {
                state_for_record.lock().last_recording_session = Some(path.clone());
                ui.set_status_text(format!("Offline HLAE recording launched: {}", path.display()).into());
            }
            Err(error) => {
                state_for_record.lock().recording_active = false;
                set_background_process(&ui, "READY", false);
                let message = format!("HLAE recording could not start:\n\n{error}");
                rfd::MessageDialog::new()
                    .set_title("HLAE Launch Failed")
                    .set_description(&message)
                    .set_level(rfd::MessageLevel::Error)
                    .show();
                ui.set_status_text(message.into());
            }
        }
    });

    let weak = ui.as_weak();
    let details_state = state.clone();
    ui.on_view_selected_details(move || {
        let Some(ui) = weak.upgrade() else { return };
        let state = details_state.lock();
        let selected = selected_candidates(&state);
        if selected.len() != 1 {
            ui.set_status_text("Select exactly one candidate to view details".into());
            return;
        }
        let details = candidate_detail_text(selected[0]);
        ui.set_candidate_detail_player(details.player.into());
        ui.set_candidate_detail_player_meta(details.player_meta.into());
        ui.set_candidate_detail_map(details.map.into());
        ui.set_candidate_detail_map_meta(details.map_meta.into());
        ui.set_candidate_detail_score(details.score.into());
        ui.set_candidate_detail_score_meta(details.score_meta.into());
        ui.set_candidate_detail_summary(details.summary.into());
        ui.set_candidate_detail_kills(details.kills.into());
        ui.set_candidate_detail_score_breakdown(details.score_breakdown.into());
        ui.set_candidate_detail_tags(details.tags.into());
        ui.set_selected_page(4);
    });

    let weak = ui.as_weak();
    let logs_state = state.clone();
    ui.on_view_recording_logs(move || {
        let Some(ui) = weak.upgrade() else { return };
        let path = {
            let state = logs_state.lock();
            state
                .last_recording_session
                .clone()
                .filter(|path| path.is_dir())
                .or_else(latest_recording_session)
        };
        match path {
            Some(path) => match open_path(&path) {
                Ok(()) => ui.set_status_text(
                    format!("Opened HLAE recording logs: {}", path.display()).into(),
                ),
                Err(error) => ui.set_status_text(error.to_string().into()),
            },
            None => ui.set_status_text("No HLAE recording log session has been created yet".into()),
        }
    });

    let weak = ui.as_weak();
    let estimate_state = state.clone();
    ui.on_update_recording_estimate(move || {
        if let Some(ui) = weak.upgrade() {
            update_output_description(&ui);
            update_recording_estimate(&ui, &estimate_state);
        }
    });
}

fn bind_settings_callbacks(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let weak = ui.as_weak();
    let path_state = state.clone();
    ui.on_choose_setting_path(move |kind| {
        let kind = kind.to_string();
        let path = if kind == "recording-output" {
            rfd::FileDialog::new().pick_folder()
        } else {
            rfd::FileDialog::new().pick_file()
        };
        let Some(path) = path else { return };
        if let Some(ui) = weak.upgrade() {
            let value: SharedString = path.display().to_string().into();
            match kind.as_str() {
                "tf2" => ui.set_tf2_path(value),
                "hlae" => ui.set_hlae_path(value),
                "ffmpeg" => ui.set_ffmpeg_path(value),
                "recording-output" => ui.set_recording_directory(value),
                _ => {}
            }
            update_recording_estimate(&ui, &path_state);
        }
    });
    let weak = ui.as_weak();
    ui.on_choose_custom_resources(move |kind| {
        let Some(ui) = weak.upgrade() else { return };
        let additions = if kind.to_string() == "folder" {
            rfd::FileDialog::new()
                .pick_folder()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            rfd::FileDialog::new()
                .add_filter("TF2 custom resources", &["vpk", "zip"])
                .pick_files()
                .unwrap_or_default()
        };
        if additions.is_empty() {
            return;
        }
        let mut values = split_paths(&ui.get_custom_resources().to_string());
        for path in additions {
            if !values.contains(&path) {
                values.push(path);
            }
        }
        ui.set_custom_resources(
            values
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("; ")
                .into(),
        );
    });
    let weak = ui.as_weak();
    let autosave_state = state.clone();
    ui.on_recording_settings_changed(move || {
        let Some(ui) = weak.upgrade() else { return };
        persist_recording_settings(&ui, &autosave_state, false);
    });
}

fn update_progress(weak: &Weak<AppWindow>, event: ProgressEvent) {
    let Some(ui) = weak.upgrade() else { return };
    match event {
        ProgressEvent::Plan(plan) => {
            ui.set_resource_plan_text(format!(
                "{} performance | {} logical CPUs | parser jobs {} | analyzer threads {} / resident demos {} | live CPU target {:.0}% | available RAM {:.1} GB",
                plan.performance_profile.label(),
                plan.logical_processors,
                plan.parse_worker_ceiling,
                plan.analysis_worker_ceiling,
                plan.analysis_job_ceiling,
                plan.performance_profile.target_cpu_percent(),
                plan.available_memory_bytes as f64 / 1_073_741_824.0,
            ).into());
        }
        ProgressEvent::Log(line) => {
            let mut text = ui.get_log_text().to_string();
            if text.len() > 200_000 {
                text.drain(..100_000);
            }
            text.push_str(&line);
            text.push('\n');
            ui.set_log_text(text.into());
        }
        ProgressEvent::Phase {
            phase,
            completed,
            total,
            fraction,
            eta_seconds,
            active_workers,
            worker_limit,
        } => {
            ui.set_progress_value(fraction);
            let eta = eta_seconds
                .map(format_duration)
                .unwrap_or_else(|| "estimating…".into());
            ui.set_status_text(format!("Phase {phase} of 2: {completed}/{total} | active workers {active_workers}/{worker_limit} | ETA {eta}").into());
        }
        ProgressEvent::Complete {
            export_root,
            candidates,
        } => {
            ui.set_busy(false);
            ui.set_progress_value(1.0);
            ui.set_status_text(
                format!(
                    "Complete: {candidates} candidates — {}",
                    export_root.display()
                )
                .into(),
            );
        }
        ProgressEvent::Failed(error) => {
            ui.set_busy(false);
            ui.set_status_text(format!("Failed: {error}").into());
        }
        ProgressEvent::Cancelled => {
            ui.set_busy(false);
            ui.set_status_text("Cancelled; completed exports were retained".into());
        }
    }
}

fn refresh_candidates(
    ui: &AppWindow,
    state: &Arc<Mutex<State>>,
    _filter: &str,
    minimum_score: i32,
) {
    let state_ref = state;
    let mut state = state.lock();
    let (visible, rows) = build_candidate_rows(
        &state.candidates,
        &state.recorded,
        &state.selected,
        &state.candidate_filters,
        minimum_score,
    );
    state.visible = visible;
    ui.set_candidate_summary(
        format!(
            "{} of {} ranked candidates",
            rows.len(),
            state.candidates.len()
        )
        .into(),
    );
    ui.set_candidate_tags_width(candidate_tags_width(&rows));
    ui.set_candidate_rows(ModelRc::new(VecModel::from(rows)));
    ui.set_selected_count(
        state
            .selected
            .iter()
            .filter(|selected| **selected)
            .count()
            .min(i32::MAX as usize) as i32,
    );
    ui.set_all_visible_selected(
        !state.visible.is_empty()
            && state
                .visible
                .iter()
                .all(|index| state.selected.get(*index).copied().unwrap_or(false)),
    );
    drop(state);
    update_recording_estimate(ui, state_ref);
}

fn build_candidate_rows(
    candidates: &[Candidate],
    recorded: &[bool],
    selected: &[bool],
    filters: &CandidateUiFilters,
    minimum_score: i32,
) -> (Vec<usize>, Vec<CandidateRow>) {
    let mut visible = Vec::new();
    let mut rows = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let is_recorded = recorded.get(index).copied().unwrap_or(false);
        if candidate.overall_score < minimum_score as f64
            || !filters.matches(candidate, is_recorded)
        {
            continue;
        }
        visible.push(index);
        rows.push(CandidateRow {
            rank: (index + 1) as i32,
            score: format!("{:.1}", candidate.overall_score).into(),
            kills: candidate.kill_count().to_string().into(),
            attacker: format!("#{}", candidate.attacker_user_id).into(),
            class_name: candidate.attacker_class.clone().into(),
            team: candidate.attacker_team.to_ascii_uppercase().into(),
            demo: Path::new(&candidate.source_demo)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&candidate.source_demo)
                .into(),
            map_name: candidate.map_name.clone().into(),
            mode: candidate_server_type(candidate).into(),
            demo_type: candidate.demo_context.capture_type.to_uppercase().into(),
            recorded: if is_recorded { "Recorded" } else { "" }.into(),
            ticks: candidate
                .point_of_kill_ticks
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
                .into(),
            tags: candidate.tags.join(", ").into(),
            selected: selected.get(index).copied().unwrap_or(false),
        });
    }
    (visible, rows)
}

fn selected_candidates(state: &State) -> Vec<&Candidate> {
    state
        .candidates
        .iter()
        .zip(&state.selected)
        .filter_map(|(candidate, selected)| selected.then_some(candidate))
        .collect()
}

fn load_candidates(path: &Path) -> Result<Vec<Candidate>> {
    BufReader::new(File::open(path).with_context(|| format!("missing {}", path.display()))?)
        .lines()
        .filter_map(|line| match line {
            Ok(line) if !line.trim().is_empty() => {
                Some(serde_json::from_str(&line).map_err(anyhow::Error::from))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error.into())),
        })
        .collect()
}

fn reconcile_recorded_outputs(
    weak: Weak<AppWindow>,
    state: Arc<Mutex<State>>,
    export_root: PathBuf,
) -> bool {
    let (candidates, output_root) = {
        let state = state.lock();
        (
            state.candidates.clone(),
            state.settings.recording_output_directory.clone(),
        )
    };
    if candidates.is_empty() || !output_root.is_dir() {
        return false;
    }
    thread::spawn(move || {
        let mut reconciled_index = { state.lock().recording_index.clone() };
        let added = reconciled_index.reconcile_output_root(&candidates, &output_root);
        let _ = slint::invoke_from_event_loop(move || {
            let Some(ui) = weak.upgrade() else { return };
            {
                let mut current = state.lock();
                if current.export_root.as_ref() != Some(&export_root) {
                    return;
                }
                current
                    .recording_index
                    .merge_missing_entries(&reconciled_index);
                let index = current.recording_index.clone();
                current.recorded = current
                    .candidates
                    .iter()
                    .map(|candidate| index.is_recorded_indexed(candidate))
                    .collect();
            }
            let filter = ui.get_filter_text().to_string();
            let minimum_score = ui.get_minimum_score();
            refresh_candidates(&ui, &state, &filter, minimum_score);
            if added > 0 {
                ui.set_status_text(
                    format!("Loaded export — found {added} additional saved recording(s)").into(),
                );
            }
        });
    });
    true
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

fn split_paths(value: &str) -> Vec<PathBuf> {
    value
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn sync_settings_from_ui(ui: &AppWindow, settings: &mut AppSettings) {
    settings.tf2_executable = PathBuf::from(ui.get_tf2_path().to_string());
    settings.hlae_executable = PathBuf::from(ui.get_hlae_path().to_string());
    settings.ffmpeg_executable = PathBuf::from(ui.get_ffmpeg_path().to_string());
    settings.recording_output_directory = PathBuf::from(ui.get_recording_directory().to_string());
    settings.lead_seconds = ui.get_lead_seconds().clamp(0, 60) as u32;
    settings.outro_seconds = ui.get_outro_seconds().clamp(0, 60) as u32;
    settings.capture_fps = ui.get_capture_fps().parse().unwrap_or(120);
    settings.jpg_quality = ui.get_jpg_quality().clamp(1, 100) as u8;
    settings.recording_format = ui.get_recording_format().to_string();
    settings.camera_mode = ui.get_camera_mode().to_string();
    settings.mp4_compatibility = ui.get_mp4_compatibility().to_string();
    settings.mp4_video_codec = ui.get_mp4_video_codec().to_string();
    settings.mp4_pixel_format = ui.get_mp4_pixel_format().to_string();
    settings.mp4_h264_profile = ui.get_mp4_h264_profile().to_string();
    settings.mp4_crf = ui.get_mp4_crf().clamp(0, 35) as u8;
    settings.mp4_encoder_preset = ui.get_mp4_encoder_preset().to_string();
    settings.mp4_audio_codec = ui.get_mp4_audio_codec().to_string();
    settings.mp4_audio_bitrate_kbps = ui.get_mp4_audio_bitrate().parse().unwrap_or(192);
    settings.avi_video_codec = ui.get_avi_video_codec().to_string();
    settings.avi_pixel_format = ui.get_avi_pixel_format().to_string();
    settings.dnxhr_profile = ui.get_dnxhr_profile().to_string();
    settings.performance_profile = ui.get_performance_profile().to_string();
    settings.resolution = ui.get_resolution().to_string();
    settings.dx_level = ui.get_dx_level().to_string();
    settings.skybox = ui.get_skybox().to_string();
    settings.hud = ui.get_hud().to_string();
    settings.viewmodels = ui.get_viewmodels().to_string();
    settings.viewmodel_fov = ui.get_viewmodel_fov().clamp(1, 179) as u32;
    settings.maximum_graphics = ui.get_maximum_graphics();
    settings.motion_blur = ui.get_motion_blur();
    settings.disable_hit_sounds = ui.get_disable_hit_sounds();
    settings.disable_voice_chat = ui.get_disable_voice_chat();
    settings.minimal_hud = ui.get_minimal_hud();
    settings.disable_combat_text = ui.get_disable_combat_text();
    settings.disable_crosshair = ui.get_disable_crosshair();
    settings.disable_crosshair_switching = ui.get_disable_crosshair_switching();
    settings.hud_player_model = ui.get_hud_player_model();
    settings.isolate_custom_resources = ui.get_isolate_custom_resources();
    settings.disable_announcer_voices = ui.get_disable_announcer();
    settings.disable_applause_sounds = ui.get_disable_applause();
    settings.disable_domination_sounds = ui.get_disable_domination();
    settings.custom_resources = split_paths(&ui.get_custom_resources().to_string());
    settings.normalize_encoding_options();
    settings.normalize_recording_options();
}

fn apply_normalized_recording_settings(ui: &AppWindow, settings: &AppSettings) {
    ui.set_recording_settings_syncing(true);
    ui.set_mp4_h264_profile(settings.mp4_h264_profile.clone().into());
    ui.set_mp4_audio_bitrate(settings.mp4_audio_bitrate_kbps.to_string().into());
    ui.set_avi_video_codec(settings.avi_video_codec.clone().into());
    ui.set_avi_pixel_format(settings.avi_pixel_format.clone().into());
    ui.set_dnxhr_profile(settings.dnxhr_profile.clone().into());
    ui.set_camera_mode(settings.camera_mode.clone().into());
    ui.set_dx_level(settings.dx_level.clone().into());
    ui.set_skybox(settings.skybox.clone().into());
    ui.set_hud(settings.hud.clone().into());
    ui.set_viewmodels(settings.viewmodels.clone().into());
    ui.set_recording_settings_syncing(false);
}

fn persist_recording_settings(ui: &AppWindow, state: &Arc<Mutex<State>>, report_manual_save: bool) {
    let (save_result, normalized) = {
        let mut state = state.lock();
        state.settings.output_directory = PathBuf::from(ui.get_export_directory().to_string());
        state.settings.item_schema = PathBuf::from(ui.get_item_schema().to_string());
        sync_settings_from_ui(ui, &mut state.settings);
        (state.settings.save(), state.settings.clone())
    };

    apply_normalized_recording_settings(ui, &normalized);
    match save_result {
        Ok(()) => {
            ui.set_recording_save_status("SETTINGS SAVED".into());
            if report_manual_save {
                ui.set_status_text("Settings saved".into());
            }
        }
        Err(error) => {
            let message = format!("SETTINGS SAVE FAILED: {error}");
            ui.set_recording_save_status(message.clone().into());
            ui.set_status_text(message.into());
        }
    }
    update_output_description(ui);
    update_recording_estimate(ui, state);
}

fn update_batch_preflight(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let demos = state.lock().demos.clone();
    if demos.is_empty() {
        ui.set_batch_estimate("Pre-flight estimate: choose one or more demos.".into());
        return;
    }
    let output = PathBuf::from(ui.get_export_directory().to_string());
    let profile = PerformanceProfile::from_setting(&ui.get_performance_profile().to_string());
    match batch::estimate_batch_preflight(&demos, &output, profile) {
        Ok(estimate) => ui.set_batch_estimate(estimate.summary().into()),
        Err(error) => {
            ui.set_batch_estimate(format!("Pre-flight estimate unavailable: {error}").into())
        }
    }
}

fn update_output_description(ui: &AppWindow) {
    let format = ui.get_recording_format().to_string();
    let description = if format == "JPG Image Sequence" {
        "JPG Image Sequence → Image Sequences/<candidate>/Frames/frame00000.jpg… plus Audio/ (quality 100 is highest; 90 is default)."
    } else if format == "TGA Image Sequence" {
        "TGA Image Sequence → Image Sequences/<candidate>/Frames/frame00000.tga… plus Audio/."
    } else if format == "AVI - Raw" {
        "AVI → Videos/<candidate>.avi using the selected advanced Raw, FFV1, or HuffYUV codec with PCM audio."
    } else if format == "MOV - DNxHR" {
        "DNxHR → Videos/<candidate>.mov using the selected LB/SQ/HQ/HQX/444 editing profile with PCM audio."
    } else if format == "MP4 - Standard"
        && ui.get_mp4_compatibility().to_string() == "DaVinci Resolve / Universal"
    {
        "MP4 Standard (default) → H.264 High / yuv420p with AAC for DaVinci Resolve and common editors."
    } else {
        "MP4 → Videos/<candidate>.mp4 with verified audio muxing (requires HLAE and FFmpeg)."
    };
    ui.set_recording_output_description(description.into());
}

fn update_recording_estimate(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let (candidates, mut settings) = {
        let state = state.lock();
        (
            selected_candidates(&state)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
            state.settings.clone(),
        )
    };
    if candidates.is_empty() {
        ui.set_recording_estimate("Recording pre-flight: select one or more candidates.".into());
        return;
    }
    sync_settings_from_ui(ui, &mut settings);
    match estimate_recording_space(&candidates, &settings) {
        Ok(estimate) => ui.set_recording_estimate(estimate.summary().into()),
        Err(error) => {
            ui.set_recording_estimate(format!("Recording pre-flight unavailable: {error}").into())
        }
    }
}

fn recompute_recorded_status(state: &Arc<Mutex<State>>) {
    let mut state = state.lock();
    let index = state.recording_index.clone();
    state.recorded = state
        .candidates
        .iter()
        .map(|candidate| index.is_recorded_indexed(candidate))
        .collect();
}

fn discover_named_executable(name: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|directory| directory.join(name)));
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(directory) = executable.parent() {
            candidates.extend([
                directory.join(name),
                directory.join("HLAE").join(name),
                directory.join("ffmpeg/bin").join(name),
            ]);
        }
    }
    candidates.into_iter().find(|path| path.is_file())
}

fn discover_tf2_executable(demos: &[PathBuf]) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    for demo in demos {
        for root in demo.ancestors() {
            candidates.extend([
                root.join("tf_win64.exe"),
                root.join("tf.exe"),
                root.join("win64/tf_win64.exe"),
                root.join("tf/win64/tf_win64.exe"),
                root.join("tf/tf.exe"),
            ]);
        }
    }
    for variable in [
        "ProgramFiles(x86)",
        "ProgramFiles",
        "STEAM_COMPAT_CLIENT_INSTALL_PATH",
    ] {
        if let Some(root) = std::env::var_os(variable).map(PathBuf::from) {
            for steam in [root.join("Steam"), root] {
                let game = steam.join("steamapps/common/Team Fortress 2");
                candidates.extend([
                    game.join("tf_win64.exe"),
                    game.join("tf/win64/tf_win64.exe"),
                    game.join("tf.exe"),
                    game.join("tf/tf.exe"),
                ]);
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        for steam in [
            home.join(".steam/steam"),
            home.join(".local/share/Steam"),
            home.join("Library/Application Support/Steam"),
        ] {
            let game = steam.join("steamapps/common/Team Fortress 2");
            candidates.extend([
                game.join("tf_win64.exe"),
                game.join("tf/win64/tf_win64.exe"),
                game.join("tf_linux64"),
                game.join("tf_osx"),
            ]);
        }
    }
    candidates.into_iter().find(|path| path.is_file())
}

fn discover_item_schema(demos: &[PathBuf], tf2: Option<&Path>) -> Option<PathBuf> {
    let mut roots = demos
        .iter()
        .flat_map(|demo| demo.ancestors().map(Path::to_path_buf))
        .collect::<Vec<_>>();
    if let Some(tf2) = tf2 {
        roots.extend(tf2.ancestors().map(Path::to_path_buf));
    }
    roots
        .into_iter()
        .flat_map(|root| {
            [
                root.join("scripts/items/items_game.txt"),
                root.join("tf/scripts/items/items_game.txt"),
            ]
        })
        .find(|path| path.is_file())
}

fn open_path(path: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    Command::new("explorer.exe").arg(path).spawn()?;
    #[cfg(target_os = "macos")]
    Command::new("open").arg(path).spawn()?;
    #[cfg(all(unix, not(target_os = "macos")))]
    Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}
