use crate::models::Candidate;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
};

const FALLBACK_TICK_INTERVAL: f64 = 1.0 / 66.666_666_7;
const PRE_KILL_SECONDS: f64 = 1.0;
const POST_KILL_SECONDS: f64 = 0.70;
const VICTIM_HOLD_SECONDS: f64 = 0.32;
const MAX_TRACK_GAP_SECONDS: f64 = 0.22;
const CAMERA_RADIUS: f32 = 12.0;

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
    pub collision_map: String,
    pub collision_coverage: &'static str,
    pub track_source: String,
    pub keyframes: Vec<CameraKeyframe>,
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

pub fn plan_candidates(candidates: &[Candidate], game_directory: &Path) -> Result<Vec<CinematicPlan>> {
    let mut groups = BTreeMap::<PathBuf, Vec<usize>>::new();
    for (index, candidate) in candidates.iter().enumerate() {
        groups.entry(camera_state_path(candidate)?).or_default().push(index);
    }
    let mut output = vec![None; candidates.len()];
    let mut geometry_cache = HashMap::<String, (String, BspGeometry)>::new();
    for (state_path, indices) in groups {
        let mut windows = BTreeMap::<i64, Vec<TrackWindow>>::new();
        for &index in &indices {
            let candidate = &candidates[index];
            let primary = candidate
                .kills
                .iter()
                .filter_map(kill_spec)
                .last()
                .context("cinematic angle unavailable: a selected candidate has no usable primary kill")?;
            let interval = camera_interval(candidate);
            let padding = ((PRE_KILL_SECONDS + MAX_TRACK_GAP_SECONDS) / interval).ceil() as i64;
            let window = TrackWindow {
                start_tick: (primary.demo_tick - padding).max(0),
                end_tick: primary.demo_tick + (MAX_TRACK_GAP_SECONDS / interval).ceil() as i64,
            };
            windows.entry(candidate.attacker_user_id).or_default().push(window);
            windows.entry(primary.victim_user_id).or_default().push(window);
        }
        coalesce_track_windows(&mut windows);
        let tracks = read_tracks(&state_path, &windows)?;
        for index in indices {
            let candidate = &candidates[index];
            let map_key = candidate.map_name.trim().to_ascii_lowercase();
            if !geometry_cache.contains_key(&map_key) {
                geometry_cache.insert(map_key.clone(), load_map_geometry(game_directory, &candidate.map_name)?);
            }
            let (geometry_label, geometry) = geometry_cache.get(&map_key).unwrap();
            output[index] = Some(plan_candidate_from_tracks(
                candidate,
                &state_path,
                &tracks,
                geometry_label,
                geometry,
            )?);
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
    geometry_label: &str,
    geometry: &BspGeometry,
) -> Result<CinematicPlan> {
    let kills = candidate
        .kills
        .iter()
        .filter_map(kill_spec)
        .collect::<Vec<_>>();
    let primary = kills
        .last()
        .context("cinematic angle unavailable: the candidate has no kill with a victim ID and demo tick")?;
    let interval = camera_interval(candidate);
    let pre_ticks = (PRE_KILL_SECONDS / interval).round() as i64;
    let post_ticks = (POST_KILL_SECONDS / interval).round() as i64;
    let start_tick = (primary.demo_tick - pre_ticks).max(0);
    let impact_tick = primary.demo_tick;
    let end_tick = impact_tick + post_ticks;
    let attacker = tracks.get(&candidate.attacker_user_id).with_context(|| {
        format!("cinematic angle unavailable: attacker {} has no camera track", candidate.attacker_user_id)
    })?;
    let victim = tracks.get(&primary.victim_user_id).with_context(|| {
        format!("cinematic angle unavailable: victim {} has no camera track", primary.victim_user_id)
    })?;
    require_continuous_track(attacker, start_tick, impact_tick, interval, "attacker")?;
    require_continuous_track(victim, start_tick, impact_tick, interval, "victim")?;

    let offsets = [-PRE_KILL_SECONDS, -0.62, -0.30, 0.0, VICTIM_HOLD_SECONDS, POST_KILL_SECONDS];
    let sample_ticks = offsets
        .iter()
        .map(|seconds| impact_tick + (*seconds / interval).round() as i64)
        .collect::<Vec<_>>();
    let impact_victim = interpolate_track(victim, impact_tick, interval, "victim")?;
    let mut subjects = Vec::new();
    for &tick in &sample_ticks {
        let attacker_point = interpolate_track(attacker, tick.min(impact_tick), interval, "attacker")?;
        let victim_point = if tick <= impact_tick {
            interpolate_track(victim, tick, interval, "victim")?
        } else {
            impact_victim.clone()
        };
        subjects.push((attacker_point.eye_position(), victim_point.eye_position()));
    }

    let mut best: Option<(f32, Vec<CameraKeyframe>)> = None;
    for side in [-1.0_f32, 1.0] {
        for distance in [260.0_f32, 330.0, 410.0] {
            for height in [90.0_f32, 140.0, 190.0] {
                for along in [-90.0_f32, 0.0, 90.0] {
                    if let Some((score, keys)) = build_path_candidate(
                        geometry,
                        &sample_ticks,
                        &offsets,
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
    let (_, keyframes) = best.context(
        "cinematic angle unavailable: every generated camera path was blocked by map geometry or lost sight of the victim",
    )?;
    let activation_demo_tick = (primary.demo_tick - pre_ticks).max(candidate.clip_start_tick + 2);
    Ok(CinematicPlan {
        format: "tf2-frag-cinematic-camera-plan",
        format_version: 1,
        candidate_id: candidate.candidate_id.clone(),
        capture_type: candidate.demo_context.capture_type.clone(),
        primary_kill_event_tick: primary.event_tick,
        primary_kill_demo_tick: primary.demo_tick,
        victim_user_id: primary.victim_user_id,
        attacker_user_id: candidate.attacker_user_id,
        activation_demo_tick,
        interval_per_tick: interval,
        collision_map: geometry_label.to_owned(),
        collision_coverage: "solid BSP brushes with swept camera-radius and subject visibility tests",
        track_source: state_path.display().to_string(),
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

fn require_continuous_track(
    points: &[TrackPoint],
    start: i64,
    end: i64,
    interval: f64,
    label: &str,
) -> Result<()> {
    let max_gap = (MAX_TRACK_GAP_SECONDS / interval).ceil().max(2.0) as i64;
    let relevant = points
        .iter()
        .filter(|point| point.tick >= start - max_gap && point.tick <= end)
        .collect::<Vec<_>>();
    let first = relevant.first().with_context(|| format!(
        "cinematic angle unavailable: the {label} was not networked in the required pre-kill window"
    ))?;
    let last = relevant.last().unwrap();
    if !first.valid || first.tick > start + max_gap || !last.valid || last.tick < end - max_gap {
        bail!("cinematic angle unavailable: the {label} track does not cover the complete pre-kill window");
    }
    let generation = first.generation;
    let entity_id = first.entity_id;
    for pair in relevant.windows(2) {
        let left = pair[0];
        let right = pair[1];
        if !left.valid
            || !right.valid
            || left.generation != generation
            || right.generation != generation
            || left.entity_id != entity_id
            || right.entity_id != entity_id
            || right.tick - left.tick > max_gap
        {
            let gap_start = left.tick.max(start);
            let gap_end = right.tick.min(end);
            bail!(
                "cinematic angle unavailable: the {label} camera track has a PVS, identity, or networking gap from demo tick {gap_start} to {gap_end}"
            );
        }
    }
    Ok(())
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
    geometry: &BspGeometry,
    ticks: &[i64],
    offsets: &[f64],
    subjects: &[(Vec3, Vec3)],
    side: f32,
    distance: f32,
    height: f32,
    along: f32,
) -> Option<(f32, Vec<CameraKeyframe>)> {
    let mut keys = Vec::with_capacity(ticks.len());
    let mut score = 0.0_f32;
    for (index, ((&tick, &seconds), &(attacker, victim))) in ticks
        .iter()
        .zip(offsets)
        .zip(subjects)
        .enumerate()
    {
        let horizontal = Vec3 { x: victim.x - attacker.x, y: victim.y - attacker.y, z: 0.0 }
            .normalized()?;
        let perpendicular = Vec3 { x: -horizontal.y * side, y: horizontal.x * side, z: 0.0 };
        let midpoint = attacker.lerp(victim, 0.55);
        let phase = match index {
            0 => 1.12,
            1 => 1.02,
            2 => 0.92,
            3 | 4 => 0.86,
            _ => 0.98,
        };
        let camera = midpoint
            .add(perpendicular.scale(distance * phase))
            .add(horizontal.scale(along * (1.0 - index as f32 * 0.04)))
            .add(Vec3 { x: 0.0, y: 0.0, z: height * (0.92 + index as f32 * 0.025) });
        let target = attacker.lerp(victim, 0.68);
        if geometry.point_in_solid(camera, CAMERA_RADIUS)
            || geometry.segment_hits(camera, victim, 0.0)
            || (index >= 2 && index <= 4 && geometry.segment_hits(camera, attacker, 0.0))
        {
            return None;
        }
        let attacker_ray = attacker.sub(camera).normalized()?;
        let victim_ray = victim.sub(camera).normalized()?;
        let framing_angle = attacker_ray.dot(victim_ray).clamp(-1.0, 1.0).acos().to_degrees();
        if framing_angle > 62.0 {
            return None;
        }
        let (pitch, yaw) = look_angles(camera, target)?;
        let fov = match index {
            0 => 75.0,
            1 => 69.0,
            2 => 61.0,
            3 => 56.0,
            4 => 58.0,
            _ => 70.0,
        };
        score += framing_angle * 0.25 + (camera.sub(target).length() - 330.0).abs() * 0.01;
        keys.push(CameraKeyframe {
            time_seconds: seconds + PRE_KILL_SECONDS,
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
        if geometry.segment_hits(pair[0].position, pair[1].position, CAMERA_RADIUS) {
            return None;
        }
        let duration = (pair[1].time_seconds - pair[0].time_seconds).max(0.001) as f32;
        let speed = pair[1].position.sub(pair[0].position).length() / duration;
        if speed > 1_200.0 {
            return None;
        }
        score += speed * 0.004;
    }
    Some((score, keys))
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

fn safe_map_relative(map_name: &str) -> Result<PathBuf> {
    let trimmed = map_name.trim().trim_end_matches(".bsp");
    if trimmed.is_empty() {
        bail!("cinematic angle unavailable: candidate map name is missing");
    }
    let relative = Path::new(trimmed);
    if relative.is_absolute()
        || relative.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("cinematic angle unavailable: unsafe map name {map_name}");
    }
    Ok(relative.with_extension("bsp"))
}

fn load_map_geometry(game_directory: &Path, map_name: &str) -> Result<(String, BspGeometry)> {
    let relative = safe_map_relative(map_name)?;
    let loose = game_directory.join("maps").join(&relative);
    if loose.is_file() {
        return Ok((
            loose.display().to_string(),
            BspGeometry::load(&loose).with_context(|| {
                format!("cinematic angle unavailable: could not read exact map geometry from {}", loose.display())
            })?,
        ));
    }

    let target = format!("maps/{}", relative.to_string_lossy().replace('\\', "/"));
    let mut vpk_directories = fs::read_dir(game_directory)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.to_ascii_lowercase().ends_with("_dir.vpk"))
        })
        .collect::<Vec<_>>();
    vpk_directories.sort_by_key(|path| {
        let name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default().to_ascii_lowercase();
        if name.contains("maps") { 0 } else if name.contains("misc") { 1 } else { 2 }
    });
    for directory in vpk_directories {
        if let Some(bytes) = read_vpk_file(&directory, &target)? {
            let label = format!("{}::{target}", directory.display());
            return Ok((label, BspGeometry::from_bytes(&bytes)?));
        }
    }
    bail!(
        "cinematic angle unavailable: exact BSP geometry for {map_name} was not found loose under {} or in a TF2 VPK",
        game_directory.join("maps").display()
    )
}

#[derive(Clone, Copy, Debug)]
struct Plane {
    normal: Vec3,
    distance: f32,
}

#[derive(Default)]
struct BspGeometry {
    solid_brushes: Vec<Vec<Plane>>,
}

impl BspGeometry {
    fn load(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 1036 || &bytes[0..4] != b"VBSP" {
            bail!("not a Source BSP file");
        }
        let version = read_i32(&bytes, 4)?;
        if !(19..=21).contains(&version) {
            bail!("unsupported Source BSP version {version}");
        }
        let plane_lump = lump(&bytes, 1)?;
        let brush_lump = lump(&bytes, 18)?;
        let side_lump = lump(&bytes, 19)?;
        if plane_lump.compressed || brush_lump.compressed || side_lump.compressed {
            bail!("compressed BSP collision lumps are not supported safely");
        }
        if plane_lump.length % 20 != 0 || brush_lump.length % 12 != 0 || side_lump.length % 8 != 0 {
            bail!("malformed BSP collision lump sizes");
        }
        let mut planes = Vec::new();
        for offset in (plane_lump.offset..plane_lump.offset + plane_lump.length).step_by(20) {
            planes.push(Plane {
                normal: Vec3 {
                    x: read_f32(&bytes, offset)?,
                    y: read_f32(&bytes, offset + 4)?,
                    z: read_f32(&bytes, offset + 8)?,
                },
                distance: read_f32(&bytes, offset + 12)?,
            });
        }
        let mut sides = Vec::<usize>::new();
        for offset in (side_lump.offset..side_lump.offset + side_lump.length).step_by(8) {
            sides.push(read_u16(&bytes, offset)? as usize);
        }
        let mut solid_brushes = Vec::new();
        for offset in (brush_lump.offset..brush_lump.offset + brush_lump.length).step_by(12) {
            let first_side = read_i32(&bytes, offset)?;
            let side_count = read_i32(&bytes, offset + 4)?;
            let contents = read_i32(&bytes, offset + 8)?;
            if contents & 1 == 0 || first_side < 0 || side_count < 4 {
                continue;
            }
            let range = first_side as usize..first_side as usize + side_count as usize;
            if range.end > sides.len() {
                bail!("BSP brush references invalid sides");
            }
            let brush_planes = sides[range]
                .iter()
                .map(|&index| planes.get(index).copied().context("BSP brush references invalid plane"))
                .collect::<Result<Vec<_>>>()?;
            solid_brushes.push(brush_planes);
        }
        if solid_brushes.is_empty() {
            bail!("BSP contains no usable solid brushes");
        }
        Ok(Self { solid_brushes })
    }

    fn point_in_solid(&self, point: Vec3, radius: f32) -> bool {
        self.solid_brushes.iter().any(|brush| {
            brush
                .iter()
                .all(|plane| point.dot(plane.normal) <= plane.distance + radius)
        })
    }

    fn segment_hits(&self, start: Vec3, end: Vec3, radius: f32) -> bool {
        self.solid_brushes
            .iter()
            .any(|brush| segment_intersects_brush(start, end, radius, brush))
    }
}

fn read_vpk_file(directory_path: &Path, target: &str) -> Result<Option<Vec<u8>>> {
    const VPK_SIGNATURE: u32 = 0x55aa_1234;
    const DIRECTORY_ARCHIVE: u16 = 0x7fff;
    const MAX_BSP_BYTES: u32 = 512 * 1024 * 1024;
    let mut directory = File::open(directory_path)?;
    let mut fixed = [0u8; 12];
    directory.read_exact(&mut fixed)?;
    let signature = u32::from_le_bytes(fixed[0..4].try_into().unwrap());
    let version = u32::from_le_bytes(fixed[4..8].try_into().unwrap());
    let tree_length = u32::from_le_bytes(fixed[8..12].try_into().unwrap()) as usize;
    if signature != VPK_SIGNATURE || !matches!(version, 1 | 2) {
        return Ok(None);
    }
    let header_size = if version == 2 {
        let mut extended = [0u8; 16];
        directory.read_exact(&mut extended)?;
        28u64
    } else {
        12u64
    };
    if tree_length == 0 || tree_length > 128 * 1024 * 1024 {
        bail!("invalid VPK directory tree length in {}", directory_path.display());
    }
    let mut tree = vec![0u8; tree_length];
    directory.read_exact(&mut tree)?;
    let target = target.replace('\\', "/").to_ascii_lowercase();
    let mut cursor = 0usize;
    loop {
        let extension = read_tree_string(&tree, &mut cursor)?;
        if extension.is_empty() {
            break;
        }
        loop {
            let directory_name = read_tree_string(&tree, &mut cursor)?;
            if directory_name.is_empty() {
                break;
            }
            loop {
                let filename = read_tree_string(&tree, &mut cursor)?;
                if filename.is_empty() {
                    break;
                }
                let entry_end = cursor.checked_add(18).context("invalid VPK entry offset")?;
                let entry = tree.get(cursor..entry_end).context("truncated VPK entry")?;
                let preload_bytes = u16::from_le_bytes(entry[4..6].try_into().unwrap()) as usize;
                let archive_index = u16::from_le_bytes(entry[6..8].try_into().unwrap());
                let entry_offset = u32::from_le_bytes(entry[8..12].try_into().unwrap());
                let entry_length = u32::from_le_bytes(entry[12..16].try_into().unwrap());
                let terminator = u16::from_le_bytes(entry[16..18].try_into().unwrap());
                if terminator != 0xffff {
                    bail!("invalid VPK entry terminator in {}", directory_path.display());
                }
                cursor = entry_end;
                let preload_end = cursor
                    .checked_add(preload_bytes)
                    .context("invalid VPK preload size")?;
                let preload = tree.get(cursor..preload_end).context("truncated VPK preload data")?;
                cursor = preload_end;
                let path = if directory_name == " " {
                    format!("{filename}.{extension}")
                } else {
                    format!("{directory_name}/{filename}.{extension}")
                }
                .replace('\\', "/")
                .to_ascii_lowercase();
                if path != target {
                    continue;
                }
                if entry_length > MAX_BSP_BYTES {
                    bail!("map BSP in {} is too large to validate safely", directory_path.display());
                }
                let mut output = Vec::with_capacity(preload_bytes + entry_length as usize);
                output.extend_from_slice(preload);
                if entry_length == 0 {
                    return Ok(Some(output));
                }
                let mut data = vec![0u8; entry_length as usize];
                if archive_index == DIRECTORY_ARCHIVE {
                    directory.seek(SeekFrom::Start(
                        header_size + tree_length as u64 + u64::from(entry_offset),
                    ))?;
                    directory.read_exact(&mut data)?;
                } else {
                    let name = directory_path
                        .file_name()
                        .and_then(|value| value.to_str())
                        .context("VPK directory filename is not UTF-8")?;
                    let prefix = name.strip_suffix("_dir.vpk").context("VPK directory name is invalid")?;
                    let archive_path = directory_path.with_file_name(format!("{prefix}_{archive_index:03}.vpk"));
                    let mut archive = File::open(&archive_path).with_context(|| {
                        format!("missing VPK archive {}", archive_path.display())
                    })?;
                    archive.seek(SeekFrom::Start(u64::from(entry_offset)))?;
                    archive.read_exact(&mut data)?;
                }
                output.extend_from_slice(&data);
                return Ok(Some(output));
            }
        }
    }
    Ok(None)
}

fn read_tree_string(tree: &[u8], cursor: &mut usize) -> Result<String> {
    let start = *cursor;
    let remaining = tree.get(start..).context("VPK directory cursor is out of range")?;
    let end = remaining
        .iter()
        .position(|byte| *byte == 0)
        .map(|offset| start + offset)
        .context("unterminated VPK directory string")?;
    *cursor = end + 1;
    Ok(String::from_utf8_lossy(&tree[start..end]).into_owned())
}

fn segment_intersects_brush(start: Vec3, end: Vec3, radius: f32, brush: &[Plane]) -> bool {
    let mut enter = 0.0_f32;
    let mut exit = 1.0_f32;
    for plane in brush {
        let expanded_distance = plane.distance + radius;
        let start_distance = start.dot(plane.normal) - expanded_distance;
        let end_distance = end.dot(plane.normal) - expanded_distance;
        if start_distance > 0.0 && end_distance > 0.0 {
            return false;
        }
        if start_distance <= 0.0 && end_distance <= 0.0 {
            continue;
        }
        let fraction = start_distance / (start_distance - end_distance);
        if start_distance > end_distance {
            enter = enter.max(fraction);
        } else {
            exit = exit.min(fraction);
        }
        if enter > exit {
            return false;
        }
    }
    enter <= exit && exit >= 0.0 && enter <= 1.0
}

#[derive(Clone, Copy)]
struct Lump {
    offset: usize,
    length: usize,
    compressed: bool,
}

fn lump(bytes: &[u8], index: usize) -> Result<Lump> {
    let base = 8 + index * 16;
    let offset = read_i32(bytes, base)?;
    let length = read_i32(bytes, base + 4)?;
    let four_cc = read_i32(bytes, base + 12)?;
    if offset < 0 || length < 0 || offset as usize + length as usize > bytes.len() {
        bail!("BSP lump {index} is out of range");
    }
    Ok(Lump {
        offset: offset as usize,
        length: length as usize,
        compressed: four_cc != 0,
    })
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32> {
    let value = bytes.get(offset..offset + 4).context("unexpected end of BSP")?;
    Ok(i32::from_le_bytes(value.try_into().unwrap()))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let value = bytes.get(offset..offset + 2).context("unexpected end of BSP")?;
    Ok(u16::from_le_bytes(value.try_into().unwrap()))
}

fn read_f32(bytes: &[u8], offset: usize) -> Result<f32> {
    let value = bytes.get(offset..offset + 4).context("unexpected end of BSP")?;
    Ok(f32::from_le_bytes(value.try_into().unwrap()))
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
            collision_map: "map.bsp".into(),
            collision_coverage: "test",
            track_source: "states.ndjson".into(),
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
    fn brush_trace_rejects_wall_crossing() {
        let brush = vec![
            Plane { normal: Vec3 { x: 1.0, y: 0.0, z: 0.0 }, distance: 10.0 },
            Plane { normal: Vec3 { x: -1.0, y: 0.0, z: 0.0 }, distance: 10.0 },
            Plane { normal: Vec3 { x: 0.0, y: 1.0, z: 0.0 }, distance: 10.0 },
            Plane { normal: Vec3 { x: 0.0, y: -1.0, z: 0.0 }, distance: 10.0 },
            Plane { normal: Vec3 { x: 0.0, y: 0.0, z: 1.0 }, distance: 10.0 },
            Plane { normal: Vec3 { x: 0.0, y: 0.0, z: -1.0 }, distance: 10.0 },
        ];
        assert!(segment_intersects_brush(
            Vec3 { x: -20.0, y: 0.0, z: 0.0 },
            Vec3 { x: 20.0, y: 0.0, z: 0.0 },
            0.0,
            &brush,
        ));
        assert!(!segment_intersects_brush(
            Vec3 { x: -20.0, y: 20.0, z: 0.0 },
            Vec3 { x: 20.0, y: 20.0, z: 0.0 },
            0.0,
            &brush,
        ));
    }

    #[test]
    fn vpk_reader_finds_preloaded_map_case_insensitively() {
        let payload = b"VBSP-test-payload";
        let mut tree = Vec::new();
        tree.extend_from_slice(b"bsp\0maps\0cp_test\0");
        tree.extend_from_slice(&0u32.to_le_bytes());
        tree.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        tree.extend_from_slice(&0x7fffu16.to_le_bytes());
        tree.extend_from_slice(&0u32.to_le_bytes());
        tree.extend_from_slice(&0u32.to_le_bytes());
        tree.extend_from_slice(&0xffffu16.to_le_bytes());
        tree.extend_from_slice(payload);
        tree.extend_from_slice(b"\0\0\0");

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0x55aa_1234u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&(tree.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&tree);

        let path = std::env::temp_dir().join(format!(
            "tf2fragdemohelper_camera_{}_dir.vpk",
            std::process::id()
        ));
        fs::write(&path, bytes).unwrap();
        let loaded = read_vpk_file(&path, "MAPS/CP_TEST.BSP").unwrap().unwrap();
        let _ = fs::remove_file(path);
        assert_eq!(loaded, payload);
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
}
