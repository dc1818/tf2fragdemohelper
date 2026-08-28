use crate::models::Candidate;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const FALLBACK_TICK_INTERVAL: f64 = 1.0 / 66.666_666_7;
const POST_KILL_SECONDS: f64 = 0.70;
const VICTIM_HOLD_SECONDS: f64 = 0.32;
const TRACK_KEY_INTERVAL_SECONDS: f64 = 0.20;
const MAX_TRACK_GAP_SECONDS: f64 = 0.22;
const MIN_CINEMATIC_PRE_KILL_TICKS: i64 = 3;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    fn from_array(value: [f32; 3]) -> Self {
        Self { x: value[0], y: value[1], z: value[2] }
    }
    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
    fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
    fn scale(self, value: f32) -> Self {
        Self { x: self.x * value, y: self.y * value, z: self.z * value }
    }
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    fn normalized(self) -> Option<Self> {
        let length = self.length();
        (length.is_finite() && length > 0.001).then(|| self.scale(1.0 / length))
    }
    fn lerp(self, other: Self, amount: f32) -> Self {
        self.scale(1.0 - amount).add(other.scale(amount))
    }
    fn finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
struct PlayerDelta {
    entity_id: u32,
    user_id: Option<i64>,
    position: [f32; 3],
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    in_pvs: bool,
    life_state: String,
    spawn_generation: Option<u32>,
    simulation_tick: Option<u64>,
    fresh: Option<bool>,
}

impl Default for PlayerDelta {
    fn default() -> Self {
        Self {
            entity_id: 0,
            user_id: None,
            position: [0.0; 3],
            bounds_min: [-24.0, -24.0, 0.0],
            bounds_max: [24.0, 24.0, 82.0],
            in_pvs: false,
            life_state: String::new(),
            spawn_generation: None,
            simulation_tick: None,
            fresh: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct StateDeltaLine {
    demo_tick: i64,
    players: Vec<PlayerDelta>,
    removed_players: Vec<u32>,
}

#[derive(Clone, Debug)]
struct TrackPoint {
    tick: i64,
    entity_id: u32,
    generation: u32,
    position: Vec3,
    eye_height: f32,
    valid: bool,
}

impl TrackPoint {
    fn eye_position(&self) -> Vec3 {
        self.position.add(Vec3 { x: 0.0, y: 0.0, z: self.eye_height })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CameraKeyframe {
    pub time_seconds: f64,
    pub demo_tick: i64,
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub fov: f32,
    pub target: Vec3,
}

#[derive(Clone, Debug, Serialize)]
pub struct CinematicPlan {
    pub format: &'static str,
    pub format_version: u32,
    pub candidate_id: String,
    pub capture_type: String,
    pub primary_kill_event_tick: i64,
    pub primary_kill_demo_tick: i64,
    pub victim_user_id: i64,
    pub attacker_user_id: i64,
    pub activation_demo_tick: i64,
    pub interval_per_tick: f64,
    pub pre_kill_seconds: f64,
    pub collision_map: String,
    pub collision_coverage: &'static str,
    pub track_source: String,
    pub kill_shots: Vec<KillShotPlan>,
    pub keyframes: Vec<CameraKeyframe>,
}

#[derive(Clone, Debug, Serialize)]
pub struct KillShotPlan {
    pub demo_tick: i64,
    pub event_ticks: Vec<i64>,
    pub victim_user_ids: Vec<i64>,
    pub start_demo_tick: i64,
    pub end_demo_tick: i64,
    pub pre_kill_seconds: f64,
}

impl CinematicPlan {
    pub fn xml(&self) -> String {
        let mut output = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<campath hold=\"\">\n  <points>\n");
        for key in &self.keyframes {
            output.push_str(&format!(
                "    <p t=\"{:.6}\" x=\"{:.6}\" y=\"{:.6}\" z=\"{:.6}\" fov=\"{:.6}\" rx=\"0.000000\" ry=\"{:.6}\" rz=\"{:.6}\"/>\n",
                key.time_seconds,
                key.position.x,
                key.position.y,
                key.position.z,
                key.fov,
                key.pitch,
                key.yaw,
            ));
        }
        output.push_str("  </points>\n</campath>\n");
        output
    }

    pub fn setup_commands(&self, xml_path: &Path) -> String {
        let path = xml_path.display().to_string().replace('\\', "/").replace('"', "");
        let source_setup = if self.capture_type.eq_ignore_ascii_case("pov") {
            // Advancedfx documents this exact order for detached POV cameras.
            "sv_cheats 1; thirdperson; mirv_input camera"
        } else {
            "spec_autodirector 0; spec_mode 7; mirv_input camera"
        };
        format!(
            "mirv_campath enabled 0; mirv_campath clear; {source_setup}; mirv_campath load \"{path}\"; mirv_input end; mirv_campath hold 1; mirv_campath offset current#0; mirv_campath enabled 1; r_drawviewmodel 0"
        )
    }
}

#[derive(Clone, Debug)]
struct KillSpec {
    event_tick: i64,
    demo_tick: i64,
    victim_user_id: i64,
}

#[derive(Clone, Debug)]
struct KillGroup {
    demo_tick: i64,
    event_ticks: Vec<i64>,
    victim_user_ids: Vec<i64>,
}

#[derive(Clone, Debug)]
struct ShotSubjects {
    attacker: Vec3,
    victims: Vec<Vec3>,
}

#[derive(Clone, Copy, Debug)]
struct TrackWindow {
    start_tick: i64,
    end_tick: i64,
}

impl TrackWindow {
    fn contains(self, tick: i64) -> bool {
        tick >= self.start_tick && tick <= self.end_tick
    }
}

pub fn plan_candidate(candidate: &Candidate, game_directory: &Path) -> Result<CinematicPlan> {
    plan_candidates(std::slice::from_ref(candidate), game_directory)?
        .into_iter()
        .next()
        .context("cinematic planner returned no plan")
}

pub fn plan_candidates(candidates: &[Candidate], _game_directory: &Path) -> Result<Vec<CinematicPlan>> {
    let mut groups = BTreeMap::<PathBuf, Vec<usize>>::new();
    for (index, candidate) in candidates.iter().enumerate() {
        groups
            .entry(camera_state_path(candidate).with_context(|| {
                format!("candidate {} has no usable camera export", candidate.candidate_id)
            })?)
            .or_default()
            .push(index);
    }
    let mut output = vec![None; candidates.len()];
    for (state_path, indices) in groups {
        let mut windows = BTreeMap::<i64, Vec<TrackWindow>>::new();
        for &index in &indices {
            let candidate = &candidates[index];
            let kill_groups = kill_groups(candidate);
            let last_kill = kill_groups
                .last()
                .context("cinematic angle unavailable: a selected candidate has no usable kill")?;
            let interval = camera_interval(candidate);
            let gap_ticks = (MAX_TRACK_GAP_SECONDS / interval).ceil() as i64;
            let track_start = candidate.clip_start_tick.saturating_add(2).max(0);
            windows.entry(candidate.attacker_user_id).or_default().push(TrackWindow {
                start_tick: track_start,
                end_tick: last_kill.demo_tick.saturating_add(gap_ticks),
            });
            for kill_group in &kill_groups {
                let victim_window = TrackWindow {
                    start_tick: track_start,
                    end_tick: kill_group.demo_tick.saturating_add(gap_ticks),
                };
                for &victim_user_id in &kill_group.victim_user_ids {
                    windows.entry(victim_user_id).or_default().push(victim_window);
                }
            }
        }
        coalesce_track_windows(&mut windows);
        let tracks = read_tracks(&state_path, &windows)?;
        for index in indices {
            let candidate = &candidates[index];
            output[index] = Some(
                plan_candidate_from_tracks(candidate, &state_path, &tracks).with_context(|| {
                    format!("candidate {} cannot use Cinematic Kill Shot", candidate.candidate_id)
                })?,
            );
        }
    }
    output
        .into_iter()
        .map(|plan| plan.context("cinematic planner did not produce a plan for every candidate"))
        .collect()
}

fn coalesce_track_windows(windows: &mut BTreeMap<i64, Vec<TrackWindow>>) {
    for user_windows in windows.values_mut() {
        user_windows.sort_by_key(|window| window.start_tick);
        let mut merged = Vec::<TrackWindow>::new();
        for window in user_windows.drain(..) {
            if let Some(previous) = merged.last_mut() {
                if window.start_tick <= previous.end_tick.saturating_add(1) {
                    previous.end_tick = previous.end_tick.max(window.end_tick);
                    continue;
                }
            }
            merged.push(window);
        }
        *user_windows = merged;
    }
}

fn plan_candidate_from_tracks(
    candidate: &Candidate,
    state_path: &Path,
    tracks: &HashMap<i64, Vec<TrackPoint>>,
) -> Result<CinematicPlan> {
    let kill_groups = kill_groups(candidate);
    let primary_group = kill_groups
        .last()
        .context("cinematic angle unavailable: the candidate has no kill with a victim ID and demo tick")?;
    let primary_event_tick = *primary_group.event_ticks.last().unwrap_or(&primary_group.demo_tick);
    let primary_victim_user_id = *primary_group
        .victim_user_ids
        .last()
        .context("cinematic angle unavailable: the final kill has no victim")?;
    let interval = camera_interval(candidate);
    let attacker = tracks.get(&candidate.attacker_user_id).with_context(|| {
        format!("cinematic angle unavailable: attacker {} has no camera track", candidate.attacker_user_id)
    })?;
    let hold_ticks = (VICTIM_HOLD_SECONDS / interval).round().max(1.0) as i64;
    let post_ticks = (POST_KILL_SECONDS / interval).round().max(1.0) as i64;
    let clip_start = candidate.clip_start_tick.saturating_add(2).max(0);

    // Each distinct kill tick owns a segment. The first segment starts with the
    // candidate; later segments start immediately after the previous victim's
    // hold, or earlier when the next kill is too close to preserve three keys.
    let mut shot_starts = Vec::with_capacity(kill_groups.len());
    for (index, group) in kill_groups.iter().enumerate() {
        let requested_start = if index == 0 {
            clip_start
        } else {
            let previous_impact = kill_groups[index - 1].demo_tick;
            let desired_after_hold = previous_impact.saturating_add(hold_ticks);
            let latest_usable = group.demo_tick.saturating_sub(MIN_CINEMATIC_PRE_KILL_TICKS);
            desired_after_hold.min(latest_usable).max(previous_impact.saturating_add(1))
        };
        let mut start_tick = continuous_track_start(
            attacker,
            requested_start,
            group.demo_tick,
            interval,
            "attacker",
        )?
        .max(requested_start);
        for &victim_user_id in &group.victim_user_ids {
            let victim = tracks.get(&victim_user_id).with_context(|| {
                format!("cinematic angle unavailable: victim {victim_user_id} at demo tick {} has no camera track", group.demo_tick)
            })?;
            start_tick = start_tick.max(continuous_track_start(
                victim,
                requested_start,
                group.demo_tick,
                interval,
                "victim",
            )?);
        }
        let available_ticks = group.demo_tick - start_tick;
        if index == 0 && available_ticks < MIN_CINEMATIC_PRE_KILL_TICKS {
            bail!(
                "cinematic angle unavailable: kill at demo tick {} has only {available_ticks} continuous pre-kill ticks; at least {MIN_CINEMATIC_PRE_KILL_TICKS} are required",
                group.demo_tick,
            );
        }
        shot_starts.push(start_tick);
    }

    let activation_demo_tick = *shot_starts.first().context("cinematic angle unavailable: no shot start")?;
    let mut keyframes = Vec::new();
    let mut kill_shots = Vec::new();
    for (index, group) in kill_groups.iter().enumerate() {
        let start_tick = shot_starts[index];
        let impact_tick = group.demo_tick;
        let span = impact_tick - start_tick;
        let regular_step = (TRACK_KEY_INTERVAL_SECONDS / interval).round().max(1.0) as i64;
        let sample_step = regular_step.min((span / 3).max(1));
        let mut sample_ticks = Vec::new();
        let mut sample_tick = start_tick;
        while sample_tick < impact_tick {
            sample_ticks.push(sample_tick);
            sample_tick = sample_tick.saturating_add(sample_step);
        }
        sample_ticks.push(impact_tick);
        let next_start = shot_starts.get(index + 1).copied();
        let hold_end = next_start
            .map(|next| impact_tick.saturating_add(hold_ticks).min(next.saturating_sub(1)))
            .unwrap_or_else(|| impact_tick.saturating_add(hold_ticks));
        if hold_end > impact_tick {
            sample_ticks.push(hold_end);
        }
        if index + 1 == kill_groups.len() {
            sample_ticks.push(impact_tick.saturating_add(post_ticks));
        }
        sample_ticks.sort_unstable();
        sample_ticks.dedup();

        let mut victim_tracks = Vec::with_capacity(group.victim_user_ids.len());
        let mut impact_victims = Vec::with_capacity(group.victim_user_ids.len());
        for &victim_user_id in &group.victim_user_ids {
            let victim = tracks.get(&victim_user_id).with_context(|| {
                format!("cinematic angle unavailable: victim {victim_user_id} at demo tick {impact_tick} has no camera track")
            })?;
            impact_victims.push(interpolate_track(victim, impact_tick, interval, "victim")?);
            victim_tracks.push(victim);
        }
        let mut subjects = Vec::with_capacity(sample_ticks.len());
        for &tick in &sample_ticks {
            let attacker_point = interpolate_track(attacker, tick.min(impact_tick), interval, "attacker")?;
            let mut victims = Vec::with_capacity(victim_tracks.len());
            for (victim_index, victim) in victim_tracks.iter().enumerate() {
                let point = if tick <= impact_tick {
                    interpolate_track(victim, tick, interval, "victim")?
                } else {
                    impact_victims[victim_index].clone()
                };
                victims.push(point.eye_position());
            }
            subjects.push(ShotSubjects {
                attacker: attacker_point.eye_position(),
                victims,
            });
        }

        let mut best: Option<(f32, Vec<CameraKeyframe>)> = None;
        for side in [-1.0_f32, 1.0] {
            for distance in [260.0_f32, 330.0, 410.0] {
                for height in [90.0_f32, 140.0, 190.0] {
                    for along in [-90.0_f32, 0.0, 90.0] {
                        if let Some((score, keys)) = build_path_candidate(
                            &sample_ticks,
                            activation_demo_tick,
                            interval,
                            &subjects,
                            side,
                            distance,
                            height,
                            along,
                        ) {
                            if best.as_ref().is_none_or(|(best_score, _)| score < *best_score) {
                                best = Some((score, keys));
                            }
                        }
                    }
                }
            }
        }
        let (_, mut shot_keys) = best.with_context(|| {
            format!("cinematic angle unavailable: no generated path could frame every victim at demo tick {impact_tick}")
        })?;
        keyframes.append(&mut shot_keys);
        kill_shots.push(KillShotPlan {
            demo_tick: impact_tick,
            event_ticks: group.event_ticks.clone(),
            victim_user_ids: group.victim_user_ids.clone(),
            start_demo_tick: start_tick,
            end_demo_tick: *sample_ticks.last().unwrap_or(&impact_tick),
            pre_kill_seconds: span as f64 * interval,
        });
    }
    keyframes.sort_by(|left, right| left.time_seconds.total_cmp(&right.time_seconds));
    Ok(CinematicPlan {
        format: "tf2-frag-cinematic-camera-plan",
        format_version: 2,
        candidate_id: candidate.candidate_id.clone(),
        capture_type: candidate.demo_context.capture_type.clone(),
        primary_kill_event_tick: primary_event_tick,
        primary_kill_demo_tick: primary_group.demo_tick,
        victim_user_id: primary_victim_user_id,
        attacker_user_id: candidate.attacker_user_id,
        activation_demo_tick,
        interval_per_tick: interval,
        pre_kill_seconds: kill_shots.last().map(|shot| shot.pre_kill_seconds).unwrap_or_default(),
        collision_map: "disabled".into(),
        collision_coverage: "disabled for victim-tracking test build",
        track_source: state_path.display().to_string(),
        kill_shots,
        keyframes,
    })
}

pub fn write_artifacts(plan: &CinematicPlan, directory: &Path, base: &str) -> Result<PathBuf> {
    fs::create_dir_all(directory)?;
    let xml_path = directory.join(format!("{base}_camera_path.xml"));
    let json_path = directory.join(format!("{base}_camera_plan.json"));
    let cfg_path = directory.join(format!("{base}_camera_setup.cfg"));
    fs::write(&xml_path, plan.xml())?;
    fs::write(&json_path, serde_json::to_vec_pretty(plan)?)?;
    fs::write(&cfg_path, format!("{}\n", plan.setup_commands(&xml_path)))?;
    Ok(xml_path)
}

fn kill_spec(value: &Value) -> Option<KillSpec> {
    let event_tick = value.get("event_tick")?.as_i64()?;
    let demo_tick = value.get("demo_tick").and_then(Value::as_i64).unwrap_or(event_tick);
    let victim_user_id = value.get("victim_user_id")?.as_i64()?;
    (event_tick > 0 && demo_tick > 0 && victim_user_id > 0).then_some(KillSpec {
        event_tick,
        demo_tick,
        victim_user_id,
    })
}

fn kill_groups(candidate: &Candidate) -> Vec<KillGroup> {
    let mut groups = BTreeMap::<i64, KillGroup>::new();
    for kill in candidate.kills.iter().filter_map(kill_spec) {
        let group = groups.entry(kill.demo_tick).or_insert_with(|| KillGroup {
            demo_tick: kill.demo_tick,
            event_ticks: Vec::new(),
            victim_user_ids: Vec::new(),
        });
        if !group.event_ticks.contains(&kill.event_tick) {
            group.event_ticks.push(kill.event_tick);
        }
        if !group.victim_user_ids.contains(&kill.victim_user_id) {
            group.victim_user_ids.push(kill.victim_user_id);
        }
    }
    groups.into_values().collect()
}

fn camera_interval(candidate: &Candidate) -> f64 {
    candidate
        .extra
        .get("camera_context")
        .and_then(|value| value.get("interval_per_tick"))
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value >= 0.001 && *value <= 0.1)
        .unwrap_or(FALLBACK_TICK_INTERVAL)
}

fn camera_state_path(candidate: &Candidate) -> Result<PathBuf> {
    let direct = candidate
        .extra
        .get("camera_context")
        .and_then(|value| value.get("state_samples"))
        .and_then(Value::as_str)
        .map(PathBuf::from);
    let batch = candidate
        .extra
        .get("batch_context")
        .and_then(|value| value.get("export_directory"))
        .and_then(Value::as_str)
        .map(|value| PathBuf::from(value).join("state_samples.ndjson"));
    direct
        .into_iter()
        .chain(batch)
        .find(|path| path.is_file())
        .context("cinematic angle unavailable: state_samples.ndjson is missing; re-parse this demo with the cinematic build")
}

fn read_tracks(
    path: &Path,
    windows: &BTreeMap<i64, Vec<TrackWindow>>,
) -> Result<HashMap<i64, Vec<TrackPoint>>> {
    let input = BufReader::new(File::open(path)?);
    let mut tracks = HashMap::<i64, Vec<TrackPoint>>::new();
    let mut entity_users = HashMap::<u32, i64>::new();
    let mut saw_camera_fields = false;
    for (line_index, line) in input.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let delta: StateDeltaLine = serde_json::from_str(&line)
            .with_context(|| format!("invalid camera state line {} in {}", line_index + 1, path.display()))?;
        for player in delta.players {
            let Some(user_id) = player.user_id else { continue };
            entity_users.insert(player.entity_id, user_id);
            let Some(user_windows) = windows.get(&user_id) else {
                continue;
            };
            if !user_windows.iter().any(|window| window.contains(delta.demo_tick)) {
                continue;
            }
            let Some(generation) = player.spawn_generation else {
                continue;
            };
            if player.simulation_tick.is_none() || player.fresh != Some(true) {
                continue;
            }
            saw_camera_fields = true;
            let position = Vec3::from_array(player.position);
            let eye_height = (player.bounds_max[2] - 10.0).clamp(48.0, 80.0);
            let alive = player.life_state.eq_ignore_ascii_case("alive")
                || player.life_state.eq_ignore_ascii_case("dying");
            tracks.entry(user_id).or_default().push(TrackPoint {
                tick: delta.demo_tick,
                entity_id: player.entity_id,
                generation,
                position,
                eye_height,
                valid: player.in_pvs && alive && position.finite(),
            });
        }
        for entity_id in delta.removed_players {
            if let Some(user_id) = entity_users.remove(&entity_id) {
                if windows
                    .get(&user_id)
                    .is_some_and(|user_windows| user_windows.iter().any(|window| window.contains(delta.demo_tick)))
                {
                    let generation = tracks
                        .get(&user_id)
                        .and_then(|points| points.last())
                        .map(|point| point.generation)
                        .unwrap_or_default();
                    tracks.entry(user_id).or_default().push(TrackPoint {
                        tick: delta.demo_tick,
                        entity_id,
                        generation,
                        position: Vec3::default(),
                        eye_height: 64.0,
                        valid: false,
                    });
                }
            }
        }
    }
    if !saw_camera_fields {
        bail!("cinematic angle unavailable: this export predates generation-safe camera tracks; re-parse the demo");
    }
    for points in tracks.values_mut() {
        points.sort_by_key(|point| point.tick);
        points.dedup_by_key(|point| point.tick);
    }
    Ok(tracks)
}

fn continuous_track_start(
    points: &[TrackPoint],
    requested_start: i64,
    end: i64,
    interval: f64,
    label: &str,
) -> Result<i64> {
    let max_gap = (MAX_TRACK_GAP_SECONDS / interval).ceil().max(2.0) as i64;
    let end_index = points
        .partition_point(|point| point.tick <= end)
        .checked_sub(1)
        .context("cinematic angle unavailable: no player samples exist before the kill")?;
    let anchor = &points[end_index];
    if !anchor.valid || end - anchor.tick > max_gap {
        let missing_seconds = (end - anchor.tick).max(0) as f64 * interval;
        bail!(
            "cinematic angle unavailable: the {label} was not continuously networked at the kill (last valid coverage is more than {missing_seconds:.3} seconds away)"
        );
    }

    let generation = anchor.generation;
    let entity_id = anchor.entity_id;
    let mut earliest_tick = anchor.tick;
    let mut right = anchor;
    for left in points[..end_index].iter().rev() {
        let continuous = left.valid
            && right.valid
            && left.generation == generation
            && right.generation == generation
            && left.entity_id == entity_id
            && right.entity_id == entity_id
            && right.tick - left.tick <= max_gap;
        if !continuous {
            break;
        }
        earliest_tick = left.tick;
        right = left;
        if earliest_tick <= requested_start {
            break;
        }
    }
    Ok(earliest_tick.max(requested_start))
}

fn interpolate_track(points: &[TrackPoint], tick: i64, interval: f64, label: &str) -> Result<TrackPoint> {
    let max_gap = (MAX_TRACK_GAP_SECONDS / interval).ceil().max(2.0) as i64;
    let upper = points.partition_point(|point| point.tick < tick);
    let before = upper.checked_sub(1).and_then(|index| points.get(index));
    let after = points.get(upper);
    match (before, after) {
        (Some(left), Some(right))
            if left.valid
                && right.valid
                && left.entity_id == right.entity_id
                && left.generation == right.generation
                && tick - left.tick <= max_gap
                && right.tick - tick <= max_gap =>
        {
            let span = (right.tick - left.tick).max(1) as f32;
            let amount = (tick - left.tick) as f32 / span;
            let mut result = left.clone();
            result.tick = tick;
            result.position = left.position.lerp(right.position, amount);
            result.eye_height = left.eye_height * (1.0 - amount) + right.eye_height * amount;
            Ok(result)
        }
        (Some(point), _) if point.valid && (tick - point.tick).abs() <= max_gap => Ok(point.clone()),
        (_, Some(point)) if point.valid && (point.tick - tick).abs() <= max_gap => Ok(point.clone()),
        _ => bail!("cinematic angle unavailable: no continuous {label} position exists at demo tick {tick}"),
    }
}

fn build_path_candidate(
    ticks: &[i64],
    activation_tick: i64,
    interval: f64,
    subjects: &[ShotSubjects],
    side: f32,
    distance: f32,
    height: f32,
    along: f32,
) -> Option<(f32, Vec<CameraKeyframe>)> {
    let mut keys = Vec::with_capacity(ticks.len());
    let mut score = 0.0_f32;
    let last_index = ticks.len().saturating_sub(1).max(1) as f32;
    for (index, (&tick, subjects)) in ticks.iter().zip(subjects).enumerate() {
        let victim = centroid(&subjects.victims)?;
        let attacker = subjects.attacker;
        let horizontal = Vec3 { x: victim.x - attacker.x, y: victim.y - attacker.y, z: 0.0 }
            .normalized()?;
        let perpendicular = Vec3 { x: -horizontal.y * side, y: horizontal.x * side, z: 0.0 };
        let midpoint = attacker.lerp(victim, 0.55);
        let progress = index as f32 / last_index;
        let phase = 1.12 - progress.min(0.75) * 0.34;
        let camera = midpoint
            .add(perpendicular.scale(distance * phase))
            .add(horizontal.scale(along * (1.0 - index as f32 * 0.04)))
            .add(Vec3 { x: 0.0, y: 0.0, z: height * (0.92 + index as f32 * 0.025) });
        // The victim centroid is the look target. The attacker is only used to
        // choose an angle that keeps the actual kill readable in the frame.
        let target = victim;
        let attacker_ray = attacker.sub(camera).normalized()?;
        let mut framing_angle = 0.0_f32;
        for victim in &subjects.victims {
            let victim_ray = victim.sub(camera).normalized()?;
            framing_angle = framing_angle.max(attacker_ray.dot(victim_ray).clamp(-1.0, 1.0).acos().to_degrees());
        }
        if framing_angle > 62.0 {
            return None;
        }
        let (pitch, yaw) = look_angles(camera, target)?;
        let fov = 75.0 - progress.min(0.75) * 23.0 + (progress - 0.75).max(0.0) * 28.0;
        score += framing_angle * 0.25 + (camera.sub(target).length() - 330.0).abs() * 0.01;
        keys.push(CameraKeyframe {
            time_seconds: (tick - activation_tick) as f64 * interval,
            demo_tick: tick,
            position: camera,
            pitch,
            yaw,
            roll: 0.0,
            fov,
            target,
        });
    }
    for pair in keys.windows(2) {
        let duration = (pair[1].time_seconds - pair[0].time_seconds).max(0.001) as f32;
        let speed = pair[1].position.sub(pair[0].position).length() / duration;
        if speed > 1_200.0 {
            return None;
        }
        score += speed * 0.004;
    }
    Some((score, keys))
}

fn centroid(points: &[Vec3]) -> Option<Vec3> {
    if points.is_empty() {
        return None;
    }
    let sum = points.iter().copied().fold(Vec3::default(), Vec3::add);
    Some(sum.scale(1.0 / points.len() as f32))
}

fn look_angles(camera: Vec3, target: Vec3) -> Option<(f32, f32)> {
    let delta = target.sub(camera);
    let horizontal = (delta.x * delta.x + delta.y * delta.y).sqrt();
    if !horizontal.is_finite() || horizontal < 0.001 {
        return None;
    }
    let yaw = delta.y.atan2(delta.x).to_degrees();
    let pitch = -delta.z.atan2(horizontal).to_degrees();
    Some((pitch, yaw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_at_points_camera_toward_target() {
        let (pitch, yaw) = look_angles(
            Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            Vec3 { x: 100.0, y: 100.0, z: 100.0 },
        )
        .unwrap();
        assert!((yaw - 45.0).abs() < 0.01);
        assert!(pitch < -30.0 && pitch > -40.0);
    }

    #[test]
    fn hlae_xml_has_normal_nonlinear_keyframes() {
        let plan = CinematicPlan {
            format: "tf2-frag-cinematic-camera-plan",
            format_version: 1,
            candidate_id: "test".into(),
            capture_type: "pov".into(),
            primary_kill_event_tick: 100,
            primary_kill_demo_tick: 90,
            victim_user_id: 2,
            attacker_user_id: 1,
            activation_demo_tick: 33,
            interval_per_tick: FALLBACK_TICK_INTERVAL,
            pre_kill_seconds: 1.0,
            collision_map: "disabled".into(),
            collision_coverage: "disabled for victim-tracking test build",
            track_source: "states.ndjson".into(),
            kill_shots: vec![KillShotPlan {
                demo_tick: 90,
                event_ticks: vec![100],
                victim_user_ids: vec![2],
                start_demo_tick: 33,
                end_demo_tick: 100,
                pre_kill_seconds: 0.85,
            }],
            keyframes: (0..4)
                .map(|index| CameraKeyframe {
                    time_seconds: index as f64 * 0.25,
                    demo_tick: index,
                    position: Vec3 { x: index as f32, y: 0.0, z: 64.0 },
                    pitch: 0.0,
                    yaw: 90.0,
                    roll: 0.0,
                    fov: 75.0 - index as f32,
                    target: Vec3::default(),
                })
                .collect(),
        };
        let xml = plan.xml();
        assert_eq!(xml.matches("<p ").count(), 4);
        assert!(xml.contains("<campath hold=\"\">"));
        let commands = plan.setup_commands(Path::new("camera.xml"));
        let setup = commands.find("sv_cheats 1; thirdperson; mirv_input camera").unwrap();
        let end = commands.find("mirv_input end").unwrap();
        let enabled = commands.find("mirv_campath enabled 1").unwrap();
        assert!(setup < end && end < enabled);
    }

    #[test]
    fn selected_camera_windows_are_coalesced_but_remain_bounded() {
        let mut windows = BTreeMap::from([(
            31,
            vec![
                TrackWindow { start_tick: 200, end_tick: 260 },
                TrackWindow { start_tick: 100, end_tick: 180 },
                TrackWindow { start_tick: 175, end_tick: 220 },
                TrackWindow { start_tick: 400, end_tick: 420 },
            ],
        )]);
        coalesce_track_windows(&mut windows);
        let merged = &windows[&31];
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].start_tick, 100);
        assert_eq!(merged[0].end_tick, 260);
        assert_eq!(merged[1].start_tick, 400);
        assert_eq!(merged[1].end_tick, 420);
    }

    #[test]
    fn every_distinct_kill_tick_becomes_a_shot_and_same_tick_victims_are_grouped() {
        let mut candidate = Candidate::default();
        candidate.kills = vec![
            serde_json::json!({"event_tick": 1000, "demo_tick": 900, "victim_user_id": 21}),
            serde_json::json!({"event_tick": 1001, "demo_tick": 900, "victim_user_id": 22}),
            serde_json::json!({"event_tick": 1080, "demo_tick": 980, "victim_user_id": 23}),
            serde_json::json!({"event_tick": 1140, "demo_tick": 1040, "victim_user_id": 24}),
        ];
        let groups = kill_groups(&candidate);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups.iter().map(|group| group.demo_tick).collect::<Vec<_>>(), vec![900, 980, 1040]);
        assert_eq!(groups[0].victim_user_ids, vec![21, 22]);
        assert_eq!(groups[1].victim_user_ids, vec![23]);
        assert_eq!(groups[2].victim_user_ids, vec![24]);
    }

    #[test]
    fn cinematic_uses_shorter_continuous_track_after_a_gap() {
        let point = |tick, valid| TrackPoint {
            tick,
            entity_id: 17,
            generation: 4,
            position: Vec3 { x: tick as f32, y: 0.0, z: 0.0 },
            eye_height: 72.0,
            valid,
        };
        let points = vec![
            point(30, true),
            point(40, false),
            point(60, true),
            point(70, true),
            point(80, true),
            point(90, true),
            point(100, true),
        ];
        let start = continuous_track_start(&points, 0, 100, 0.01, "victim").unwrap();
        assert_eq!(start, 60);
        assert!(100 - start >= MIN_CINEMATIC_PRE_KILL_TICKS);
    }

    #[test]
    fn nineteen_tick_track_from_real_export_is_accepted() {
        let point = |tick, valid| TrackPoint {
            tick,
            entity_id: 23,
            generation: 534,
            position: Vec3 { x: tick as f32, y: 10.0, z: 20.0 },
            eye_height: 72.0,
            valid,
        };
        let points = vec![
            point(60, false),
            point(81, true),
            point(90, true),
            point(100, true),
        ];
        let interval = 0.014_999_999_664_723_871;
        let start = continuous_track_start(&points, 34, 100, interval, "victim").unwrap();
        let pre_ticks = 100 - start;
        let pre_seconds = pre_ticks as f64 * interval;
        assert_eq!(pre_ticks, 19);
        assert!((pre_seconds - 0.285).abs() < 0.000_001);
        assert!(pre_ticks >= MIN_CINEMATIC_PRE_KILL_TICKS);

        let offsets = [-pre_seconds, -pre_seconds * 0.68, -pre_seconds * 0.34, 0.0];
        let mut ticks = offsets
            .iter()
            .map(|seconds| 100 + (*seconds / interval).round() as i64)
            .collect::<Vec<_>>();
        ticks.sort_unstable();
        ticks.dedup();
        assert_eq!(ticks.len(), 4);
    }
}
