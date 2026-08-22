use crate::models::{AppSettings, Candidate};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecordingEntry {
    pub recording_key: String,
    pub candidate_id: String,
    pub demo_signature: String,
    pub clip_start_tick: i64,
    pub clip_end_tick: i64,
    pub output_path: PathBuf,
    pub output_fingerprint: String,
    pub completed_utc: String,
}

pub struct RecordingIndex {
    path: PathBuf,
    entries: HashMap<String, RecordingEntry>,
}

impl RecordingIndex {
    pub fn load() -> Self {
        let path = index_path();
        let entries = File::open(&path)
            .ok()
            .map(BufReader::new)
            .into_iter()
            .flat_map(|reader| reader.lines().map_while(Result::ok))
            .filter_map(|line| serde_json::from_str::<RecordingEntry>(&line).ok())
            .map(|entry| (entry.recording_key.clone(), entry))
            .collect();
        Self { path, entries }
    }

    pub fn is_recorded(&mut self, candidate: &Candidate, recording_root: Option<&Path>) -> bool {
        let Ok(key) = recording_key(candidate) else { return false };
        if let Some(mut entry) = self.entries.get(&key).cloned() {
            if entry.output_path.exists() {
                if entry.output_fingerprint.is_empty() {
                    entry.output_fingerprint = file_fingerprint(&entry.output_path).unwrap_or_default();
                    self.entries.insert(key, entry.clone());
                    let _ = self.append(entry);
                }
                return true;
            }
            if !entry.output_fingerprint.is_empty() {
                if let Some(found) = find_fingerprint_near(&entry.output_path, &entry.output_fingerprint) {
                    entry.output_path = found;
                    self.entries.insert(key, entry.clone());
                    let _ = self.append(entry);
                    return true;
                }
            }
        }
        if let Some(root) = recording_root.and_then(|root| (!root.as_os_str().is_empty()).then_some(root)) {
            if let Some(output) = find_unindexed_output(root, candidate) {
                return self.register(candidate, output).is_ok();
            }
        }
        false
    }

    pub fn register(&mut self, candidate: &Candidate, output: PathBuf) -> Result<()> {
        let key = recording_key(candidate)?;
        let entry = RecordingEntry {
            recording_key: key.clone(),
            candidate_id: candidate.candidate_id.clone(),
            demo_signature: demo_signature(Path::new(&candidate.source_demo))?,
            clip_start_tick: candidate.clip_start_tick,
            clip_end_tick: candidate.clip_end_tick,
            output_fingerprint: file_fingerprint(&output).unwrap_or_default(),
            output_path: output,
            completed_utc: Utc::now().to_rfc3339(),
        };
        self.entries.insert(key, entry.clone());
        self.append(entry)
    }

    fn append(&self, entry: RecordingEntry) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = OpenOptions::new().create(true).append(true).open(&self.path)?;
        serde_json::to_writer(&mut output, &entry)?;
        output.write_all(b"\n")?;
        Ok(())
    }
}

pub fn preview_candidate(candidate: &Candidate, settings: &AppSettings) -> Result<()> {
    if !cfg!(target_os = "windows") {
        bail!("TF2 demo preview is currently available only where the native TF2 client is installed");
    }
    let tf2 = settings.tf2_executable.as_path();
    if !tf2.is_file() {
        bail!("select tf_win64.exe in Settings");
    }
    let game = tf2.parent().and_then(Path::parent).context("could not find TF2 game directory")?;
    let cfg = game.join("tf").join("cfg").join("tf2fragdemohelper_preview.cfg");
    fs::write(&cfg, format!(
        "sv_lan 1\nsv_master_legacy_mode 1\ncl_predict 0\ndemo_gototick {}\n",
        candidate.clip_start_tick
    ))?;
    Command::new(tf2)
        .args(["-insecure", "-novid", "-console", "+sv_lan", "1", "+exec", "tf2fragdemohelper_preview.cfg", "+playdemo"])
        .arg(&candidate.source_demo)
        .spawn()?;
    Ok(())
}

pub fn prepare_hlae_batch(candidates: &[Candidate], settings: &AppSettings) -> Result<PathBuf> {
    if !cfg!(target_os = "windows") {
        bail!("HLAE recording is Windows-only; parsing, analysis, filtering, and export viewing remain cross-platform");
    }
    if candidates.is_empty() {
        bail!("select one or more candidates");
    }
    if !settings.hlae_executable.is_file() {
        bail!("select hlae.exe in Settings");
    }
    if !settings.tf2_executable.is_file() {
        bail!("select tf_win64.exe in Settings");
    }
    if settings.recording_format.contains("MP4") && !settings.ffmpeg_executable.is_file() {
        bail!("select ffmpeg.exe before MP4 recording");
    }
    fs::create_dir_all(&settings.recording_output_directory)?;
    let session = settings.recording_output_directory.join(format!("Recording Metadata/session_{}", Utc::now().format("%Y%m%d_%H%M%S")));
    fs::create_dir_all(&session)?;
    let queue = candidates.iter().enumerate().map(|(index, candidate)| {
        let (start_tick, end_tick) = clip_window(candidate, settings);
        json!({
        "order":index+1,
        "candidate_id":candidate.candidate_id,
        "source_demo":candidate.source_demo,
        "start_tick":start_tick,
        "end_tick":end_tick,
        "fps":settings.capture_fps,
        "format":settings.recording_format,
        "status":"Pending",
    })}).collect::<Vec<_>>();
    fs::write(session.join("recording_queue.json"), serde_json::to_vec_pretty(&queue)?)?;
    fs::write(session.join("offline_safety.cfg"), b"sv_lan 1\nsv_master_legacy_mode 1\nnet_start 0\n")?;
    fs::write(session.join("tf2fragdemohelper_recording_profile.cfg"), recording_profile_cfg(settings))?;
    // The VDM/config queue is intentionally generated before launch. Every
    // clip receives a terminal stop command and the final clip quits TF2,
    // preventing the old last-frag replay loop.
    for (index, candidate) in candidates.iter().enumerate() {
        fs::write(session.join(format!("clip_{:03}_start.cfg", index + 1)), format!(
            "echo TF2FRAG_RECORD_START {}\nhost_framerate {}\nmirv_streams record fps {}\nmirv_streams record start\n",
            candidate.candidate_id, settings.capture_fps, settings.capture_fps
        ))?;
        fs::write(session.join(format!("clip_{:03}_stop.cfg", index + 1)), format!(
            "mirv_streams record end\necho TF2FRAG_RECORD_END {}\n",
            candidate.candidate_id
        ))?;
    }
    Ok(session)
}

pub fn launch_hlae_batch(candidates: &[Candidate], settings: &AppSettings) -> Result<PathBuf> {
    let session = prepare_hlae_batch(candidates, settings)?;
    let tf_root = settings.tf2_executable.parent().context("could not find the TF2 directory")?;
    let game = tf_root.join("tf");
    if !game.is_dir() {
        bail!("could not find TF2's tf folder next to the selected executable");
    }
    let hlae_root = settings.hlae_executable.parent().context("could not find the HLAE directory")?;
    let x64 = settings.tf2_executable.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.eq_ignore_ascii_case("tf_win64.exe"));
    let hook = if x64 { hlae_root.join("x64/AfxHookSource.dll") } else { hlae_root.join("AfxHookSource.dll") };
    if !hook.is_file() {
        bail!("required HLAE hook is missing: {}", hook.display());
    }

    let session_name = session.file_name().and_then(|name| name.to_str()).unwrap_or("rust_session");
    let staged_root = game.join("demos/tf2fragdemohelper_batch").join(session_name);
    let cfg_root = game.join("cfg/tf2fragdemohelper_batch").join(session_name);
    fs::create_dir_all(&staged_root)?;
    fs::create_dir_all(&cfg_root)?;
    fs::create_dir_all(settings.recording_output_directory.join("Videos"))?;
    fs::create_dir_all(settings.recording_output_directory.join("Image Sequences"))?;
    fs::create_dir_all(game.join("cfg"))?;
    fs::write(game.join("cfg/tf2fragdemohelper_offline.cfg"), offline_cfg())?;
    fs::write(game.join("cfg/tf2fragdemohelper_recording_profile.cfg"), recording_profile_cfg(settings))?;

    let mut by_demo: BTreeMap<PathBuf, Vec<(usize, Candidate)>> = BTreeMap::new();
    for (order, candidate) in candidates.iter().cloned().enumerate() {
        let source = PathBuf::from(&candidate.source_demo);
        if source.is_file() {
            by_demo.entry(source).or_default().push((order + 1, candidate));
        }
    }
    if by_demo.is_empty() {
        bail!("none of the selected candidates reference an existing demo");
    }

    let groups = by_demo.into_iter().collect::<Vec<_>>();
    let mut staged_relatives = Vec::new();
    for (demo_index, (source, clips)) in groups.iter().enumerate() {
        let stem = sanitize(source.file_stem().and_then(|value| value.to_str()).unwrap_or("demo"));
        let staged_name = format!("{:03}_{stem}.dem", demo_index + 1);
        let staged = staged_root.join(&staged_name);
        fs::copy(source, &staged).with_context(|| format!("could not stage {}", source.display()))?;
        let relative = format!("tf2fragdemohelper_batch/{session_name}/{staged_name}");
        staged_relatives.push(relative);

        for (order, candidate) in clips {
            let base = format!("{:03}_{}_t{}-{}", order, sanitize(&candidate.candidate_id), candidate.clip_start_tick, candidate.clip_end_tick);
            let start_cfg = recording_start_cfg(&settings.recording_output_directory, &base, settings);
            fs::write(cfg_root.join(format!("{base}_start.cfg")), start_cfg)?;
            fs::write(cfg_root.join(format!("{base}_stop.cfg")), recording_stop_cfg(settings))?;
        }
    }

    for (demo_index, (_, clips)) in groups.iter().enumerate() {
        let staged = staged_root.join(Path::new(&staged_relatives[demo_index]).file_name().unwrap());
        let next = staged_relatives.get(demo_index + 1).cloned();
        fs::write(staged.with_extension("vdm"), vdm_text(clips, session_name, next.as_deref(), settings))?;
    }

    let (width, height) = parse_resolution(&settings.resolution);
    let dx = settings.dx_level.split_whitespace().next().unwrap_or("98");
    let game_arguments = format!(
        "-steam -insecure +sv_lan 1 -novid -window -noborder -console -no_texture_stream -afxGame tf -w {width} -h {height} -dxlevel {dx} +tf_delete_temp_files 0 +exec tf2fragdemohelper_offline.cfg +exec tf2fragdemohelper_recording_profile.cfg +playdemo {}",
        staged_relatives[0]
    );
    Command::new(&settings.hlae_executable)
        .current_dir(hlae_root)
        .args(["-customLoader", "-autoStart", "-noGui", "-programPath"])
        .arg(&settings.tf2_executable)
        .arg("-cmdLine")
        .arg(game_arguments)
        .arg("-hookDllPath")
        .arg(hook)
        .spawn()?;
    Ok(session)
}

fn clip_window(candidate: &Candidate, settings: &AppSettings) -> (i64, i64) {
    let first = candidate.point_of_kill_ticks.first().copied().unwrap_or(candidate.clip_start_tick);
    let last = candidate.point_of_kill_ticks.last().copied().unwrap_or(candidate.clip_end_tick);
    let start = (first - (settings.lead_seconds as f64 * 66.666_666_7) as i64).max(0);
    let end = last + (settings.outro_seconds as f64 * 66.666_666_7) as i64;
    (start, end.max(start + 1))
}

fn vdm_text(clips: &[(usize, Candidate)], session_name: &str, next_demo: Option<&str>, settings: &AppSettings) -> String {
    let mut lines = vec!["demoactions".to_owned(), "{".to_owned()];
    let mut action = 1;
    let mut previous_end = -1;
    for (order, candidate) in clips {
        let (start, end) = clip_window(candidate, settings);
        let base = format!("{:03}_{}_t{}-{}", order, sanitize(&candidate.candidate_id), candidate.clip_start_tick, candidate.clip_end_tick);
        let seek_at = if previous_end < 0 { 2 } else { previous_end + 2 };
        add_vdm_action(&mut lines, &mut action, "SkipAhead", "Batch seek", seek_at, Some(start), "");
        let focus = if candidate.demo_context.capture_type.eq_ignore_ascii_case("stv") {
            format!("spec_autodirector 0; spec_player #{}; spec_mode 4; ", candidate.attacker_user_id)
        } else { String::new() };
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Start clip", start + 1, None, &format!("{focus}exec tf2fragdemohelper_batch/{session_name}/{base}_start"));
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Stop clip", end, None, &format!("exec tf2fragdemohelper_batch/{session_name}/{base}_stop"));
        previous_end = end + 2;
    }
    let finish = next_demo.map(|demo| format!("playdemo {demo}")).unwrap_or_else(|| "quit".into());
    add_vdm_action(&mut lines, &mut action, "PlayCommands", "Continue batch", previous_end + 2, None, &finish);
    lines.push("}".into());
    lines.join("\n")
}

fn add_vdm_action(lines: &mut Vec<String>, action: &mut i32, factory: &str, name: &str, tick: i64, skip_to: Option<i64>, commands: &str) {
    lines.extend([format!("    \"{}\"", *action), "    {".into(), format!("        factory \"{factory}\""), format!("        name \"{name}\""), format!("        starttick \"{tick}\"")]);
    if let Some(target) = skip_to { lines.push(format!("        skiptotick \"{target}\"")); }
    if !commands.is_empty() { lines.push(format!("        commands \"{}\"", commands.replace('"', "\\\""))); }
    lines.push("    }".into());
    *action += 1;
}

fn recording_start_cfg(output_root: &Path, base: &str, settings: &AppSettings) -> String {
    let fps = settings.capture_fps;
    let output = output_root.join(if settings.recording_format.contains("Image") { "Image Sequences" } else { "Videos" }).join(base);
    let output = output.display().to_string().replace('\\', "/");
    if settings.recording_format.starts_with("TGA") {
        format!("echo TF2FRAG_RECORD_START {base}; host_framerate {fps}; startmovie \"{output}\" raw; hideconsole\n")
    } else if settings.recording_format.starts_with("JPG") {
        format!("echo TF2FRAG_RECORD_START {base}; jpeg_quality {}; host_framerate {fps}; startmovie \"{output}\" jpeg; hideconsole\n", settings.jpg_quality)
    } else {
        let preset = if settings.recording_format.contains("Compatible") { "afxFfmpegYuv420p" } else if settings.recording_format.contains("Lossless") { "afxFfmpegLosslessBest" } else if settings.recording_format.contains("AVI") { "afxFfmpegRaw" } else { "afxFfmpeg" };
        format!("echo TF2FRAG_RECORD_START {base}; host_framerate {fps}; mirv_streams record fps {fps}; mirv_streams record screen enabled 1; mirv_streams record screen settings {preset}; mirv_streams record name \"{output}\"; mirv_streams record start; hideconsole\n")
    }
}

fn recording_stop_cfg(settings: &AppSettings) -> String {
    let stop = if settings.recording_format.contains("Image") { "endmovie" } else { "mirv_streams record end" };
    format!("echo TF2FRAG_RECORD_END; {stop}; host_framerate 0\n")
}

fn offline_cfg() -> &'static str {
    "// Generated by TF2 Frag Demo Helper. Offline demo playback only.\nsv_lan 1\nsv_master_legacy_mode 1\ncl_allowdownload 0\ncl_downloadfilter none\nalias connect \"echo BLOCKED: recording mode cannot connect to servers\"\nalias retry \"echo BLOCKED: recording mode cannot reconnect\"\nengine_no_focus_sleep 0\nsnd_mute_losefocus 0\n"
}

fn parse_resolution(value: &str) -> (u32, u32) {
    value.split_once('x').and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?))).unwrap_or((2560, 1440))
}

fn sanitize(value: &str) -> String {
    value.chars().map(|character| if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') { character } else { '_' }).collect()
}

fn recording_profile_cfg(settings: &AppSettings) -> String {
    let mut lines = vec![
        "// Generated by TF2 Frag Demo Helper (Rust)".to_owned(),
        "sv_lan 1".to_owned(),
        "sv_master_legacy_mode 1".to_owned(),
        format!("mat_motion_blur_enabled {}", bool_num(settings.motion_blur)),
        format!("mat_motion_blur_forward_enabled {}", bool_num(settings.motion_blur)),
        format!("r_drawviewmodel {}", bool_num(settings.viewmodels.eq_ignore_ascii_case("On"))),
        format!("viewmodel_fov_demo {}", settings.viewmodel_fov),
        format!("hud_combattext {}", bool_num(!settings.disable_combat_text)),
        format!("hud_combattext_healing {}", bool_num(!settings.disable_combat_text)),
        format!("tf_dingalingaling {}", bool_num(!settings.disable_hit_sounds)),
        format!("tf_dingalingaling_lasthit {}", bool_num(!settings.disable_hit_sounds)),
        format!("voice_enable {}", bool_num(!settings.disable_voice_chat)),
        format!("cl_hud_minmode {}", bool_num(settings.minimal_hud)),
        format!("cl_hud_playerclass_use_playermodel {}", bool_num(settings.hud_player_model)),
        format!("crosshair {}", bool_num(!settings.disable_crosshair)),
    ];
    if settings.disable_crosshair_switching {
        lines.push("alias crosshair_switcher_disabled \"\"".into());
    }
    if settings.maximum_graphics {
        lines.extend([
            "mat_picmip -1", "mat_antialias 8", "mat_forceaniso 16", "mat_hdr_level 2", "r_lod 0", "r_rootlod 0",
            "r_shadowrendertotexture 1", "r_waterforceexpensive 1", "r_waterforcereflectentities 1", "mat_specular 1",
        ].into_iter().map(str::to_owned));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn bool_num(value: bool) -> u8 { if value { 1 } else { 0 } }

fn recording_key(candidate: &Candidate) -> Result<String> {
    let demo = demo_signature(Path::new(&candidate.source_demo))?;
    let mut hash = Sha256::new();
    hash.update(demo.as_bytes());
    hash.update(candidate.clip_start_tick.to_le_bytes());
    hash.update(candidate.clip_end_tick.to_le_bytes());
    hash.update(candidate.attacker_user_id.to_le_bytes());
    Ok(hex::encode(hash.finalize()))
}

fn demo_signature(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path).with_context(|| format!("demo missing: {}", path.display()))?;
    let mut file = File::open(path)?;
    let mut head = vec![0u8; 1024 * 1024];
    let read = file.read(&mut head)?;
    let mut hash = Sha256::new();
    hash.update(metadata.len().to_le_bytes());
    hash.update(&head[..read]);
    Ok(hex::encode(hash.finalize()))
}

fn file_fingerprint(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 { break }
        hash.update(&buffer[..read]);
    }
    Ok(hex::encode(hash.finalize()))
}

fn find_fingerprint_near(original: &Path, fingerprint: &str) -> Option<PathBuf> {
    let root = original.parent()?.parent().unwrap_or_else(|| original.parent().unwrap());
    WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .find(|path| file_fingerprint(path).ok().as_deref() == Some(fingerprint))
}

fn find_unindexed_output(root: &Path, candidate: &Candidate) -> Option<PathBuf> {
    if !root.is_dir() {
        return None;
    }
    let id = sanitize(&candidate.candidate_id).to_lowercase();
    let ticks = format!("t{}-{}", candidate.clip_start_tick, candidate.clip_end_tick);
    WalkDir::new(root)
        .max_depth(4)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && entry.metadata().is_ok_and(|metadata| metadata.len() > 0))
        .map(|entry| entry.into_path())
        .find(|path| {
            let name = path.file_stem().and_then(|value| value.to_str()).unwrap_or_default().to_lowercase();
            name.contains(&id) && name.contains(&ticks)
        })
}

fn index_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TF2FragDemoHelper")
        .join("recorded_clip_index.ndjson")
}
