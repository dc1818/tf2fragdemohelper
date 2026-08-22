use crate::models::{AppSettings, Candidate};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
    thread,
    time::{Duration, SystemTime},
};
use walkdir::WalkDir;
use zip::ZipArchive;

const PROFILE_FOLDER: &str = "tf2fragdemohelper_recording";
const PROFILE_CFG: &str = "tf2fragdemohelper_recording_profile.cfg";
const RESOURCE_CACHE_VERSION: &str = "bundled_resources_v2";

#[derive(Clone)]
struct DemoSignatureCacheEntry {
    length: u64,
    modified: Option<SystemTime>,
    signature: String,
}

static DEMO_SIGNATURE_CACHE: OnceLock<Mutex<HashMap<PathBuf, DemoSignatureCacheEntry>>> = OnceLock::new();

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RecordingProfileSession {
    session_id: String,
    game_directory: PathBuf,
    backup_directory: PathBuf,
    original_custom_existed: bool,
    original_profile_existed: bool,
    original_profile_cfg_existed: bool,
    isolated_custom: bool,
    tf_process_name: String,
    #[serde(default)]
    original_config_existed: bool,
    #[serde(default)]
    original_video_existed: bool,
    #[serde(default)]
    original_offline_cfg_existed: bool,
    #[serde(default)]
    hitsound_files: Vec<PathBuf>,
}

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

#[derive(Clone)]
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
            .flat_map(|reader| reader.lines().map_while(|line| line.ok()))
            .filter_map(|line| serde_json::from_str::<RecordingEntry>(&line).ok())
            .map(|entry| (entry.recording_key.clone(), entry))
            .collect();
        Self { path, entries }
    }

    /// Fast UI-facing status lookup.  It deliberately does not walk the output directory:
    /// walking it once for every candidate made large exported batches appear to freeze.
    /// A single background reconciliation pass handles outputs that have not yet been indexed.
    pub fn is_recorded_indexed(&self, candidate: &Candidate) -> bool {
        let Ok(key) = recording_key(candidate) else { return false };
        self.entries.get(&key).is_some_and(|entry| entry.output_path.is_file())
    }

    /// Scan a recording directory once and add the outputs whose generated clip base names
    /// match candidates.  This preserves detection of pre-existing outputs without doing an
    /// expensive directory walk for every row in the candidate table.
    pub fn reconcile_output_root(&mut self, candidates: &[Candidate], root: &Path) -> usize {
        if !root.is_dir() {
            return 0;
        }

        let mut targets: HashMap<(String, i64), Vec<&Candidate>> = HashMap::new();
        for candidate in candidates {
            targets
                .entry((sanitize(&candidate.candidate_id).to_lowercase(), candidate.clip_start_tick))
                .or_default()
                .push(candidate);
        }
        let mut missing_fingerprints: HashMap<String, Vec<String>> = HashMap::new();
        for (key, entry) in &self.entries {
            if !entry.output_fingerprint.is_empty() && !entry.output_path.is_file() {
                missing_fingerprints
                    .entry(entry.output_fingerprint.clone())
                    .or_default()
                    .push(key.clone());
            }
        }

        let mut found = Vec::new();
        let mut relocated = Vec::new();
        for entry in WalkDir::new(root).max_depth(4).into_iter().filter_map(|entry| entry.ok()) {
            if !entry.file_type().is_file() || !entry.metadata().is_ok_and(|metadata| metadata.len() > 0) {
                continue;
            }
            let Some(stem) = entry.path().file_stem().and_then(|value| value.to_str()) else { continue };
            if !missing_fingerprints.is_empty() {
                if let Ok(fingerprint) = file_fingerprint(entry.path()) {
                    if let Some(keys) = missing_fingerprints.remove(&fingerprint) {
                        relocated.extend(keys.into_iter().map(|key| (key, entry.path().to_path_buf())));
                    }
                }
            }
            let stem_lower = stem.to_lowercase();
            let Some((prefix, ticks)) = stem_lower.rsplit_once("_t") else { continue };
            let Some((order, candidate_id)) = prefix.split_once('_') else { continue };
            if !order.chars().all(|character| character.is_ascii_digit()) {
                continue;
            }
            let Some((start_text, end_text)) = ticks.split_once('-') else { continue };
            let Ok(start_tick) = start_text.parse::<i64>() else { continue };
            let Some(matches) = targets.get(&(candidate_id.to_owned(), start_tick)) else { continue };
            for candidate in matches {
                // Movie frame image files add digits after the end tick, so accept a prefix.
                if end_text.starts_with(&candidate.clip_end_tick.to_string()) {
                    found.push(((*candidate).clone(), entry.path().to_path_buf()));
                    break;
                }
            }
        }

        let mut added = 0;
        for (candidate, path) in found {
            if !self.is_recorded_indexed(&candidate) && self.register(&candidate, path).is_ok() {
                added += 1;
            }
        }
        for (key, path) in relocated {
            if let Some(mut entry) = self.entries.get(&key).cloned() {
                entry.output_path = path;
                self.entries.insert(key, entry.clone());
                let _ = self.append(entry);
            }
        }
        added
    }

    pub fn merge_missing_entries(&mut self, other: &RecordingIndex) {
        for (key, entry) in &other.entries {
            self.entries.entry(key.clone()).or_insert_with(|| entry.clone());
        }
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

pub fn preview_candidate(candidate: &Candidate, settings: &AppSettings) -> Result<i64> {
    if !cfg!(target_os = "windows") {
        bail!("TF2 demo preview is currently available only where the native TF2 client is installed");
    }
    let tf2 = settings.tf2_executable.as_path();
    if !tf2.is_file() {
        bail!("select tf_win64.exe in Settings");
    }
    let source = Path::new(&candidate.source_demo);
    if !source.is_file() {
        bail!("the selected candidate's original demo is missing: {}", source.display());
    }
    let tf_process_name = tf2.file_name().and_then(|name| name.to_str()).unwrap_or("tf_win64.exe").to_owned();
    if windows_process_is_running(&tf_process_name)
        || windows_process_is_running("tf.exe")
        || windows_process_is_running("tf_win64.exe")
    {
        bail!("close TF2 before previewing a candidate so the new demo and VDM seek are not ignored");
    }

    let game = tf2_game_directory(tf2)?;
    let demo_directory = game.join("demos/tf2fragdemohelper");
    fs::create_dir_all(&demo_directory)?;
    for entry in fs::read_dir(&demo_directory)? {
        let path = entry?.path();
        let is_stale_vdm = path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("vdm"))
            && path.file_stem().and_then(|value| value.to_str()).is_some_and(|value| value.starts_with("tf2fragdemohelper_temp_"));
        if is_stale_vdm {
            let _ = fs::remove_file(path);
        }
    }

    let source_stem = sanitize(source.file_stem().and_then(|value| value.to_str()).unwrap_or("candidate_demo"));
    let signature = demo_signature(source)?;
    let signature_prefix = signature.get(..12).unwrap_or(&signature);
    let staged_name = format!("tf2fragdemohelper_temp_{source_stem}_{signature_prefix}.dem");
    let staged_path = demo_directory.join(&staged_name);
    let source_length = fs::metadata(source)?.len();
    if !staged_path.is_file() || fs::metadata(&staged_path).map(|metadata| metadata.len()).unwrap_or_default() != source_length {
        fs::copy(source, &staged_path).with_context(|| format!("could not stage {} for TF2 preview", source.display()))?;
    }

    let (target_tick, _) = clip_window(candidate, settings);
    let playback_vdm = staged_path.with_extension("vdm");
    fs::write(&playback_vdm, preview_vdm_text(candidate, target_tick))?;

    let cfg = game.join("cfg").join("tf2fragdemohelper_preview.cfg");
    fs::write(&cfg, format!("{}cl_predict 0\n", offline_cfg()))?;
    let staged_relative = format!("demos/tf2fragdemohelper/{staged_name}");
    let mut child = Command::new(tf2)
        .current_dir(tf2.parent().unwrap_or(game.as_path()))
        .args(["-insecure", "-novid", "-console", "+sv_lan", "1", "+exec", "tf2fragdemohelper_preview.cfg", "+playdemo"])
        .arg(staged_relative)
        .spawn()?;
    thread::spawn(move || {
        let _ = child.wait();
        thread::sleep(Duration::from_millis(2500));
        while windows_process_is_running(&tf_process_name) {
            thread::sleep(Duration::from_secs(1));
        }
        let _ = fs::remove_file(playback_vdm);
    });
    Ok(target_tick)
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
    recover_interrupted_profile()?;
    let session = prepare_hlae_batch(candidates, settings)?;
    let diagnostic_log = session.join("hlae_recording_diagnostics.log");
    log_recording_diagnostic(&diagnostic_log, "Recording session created");
    let game = tf2_game_directory(&settings.tf2_executable)?;
    let hlae_root = settings.hlae_executable.parent().context("could not find the HLAE directory")?;
    let x64 = settings.tf2_executable.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.eq_ignore_ascii_case("tf_win64.exe"));
    let hook = if x64 { hlae_root.join("x64/AfxHookSource.dll") } else { hlae_root.join("AfxHookSource.dll") };
    if !hook.is_file() {
        log_recording_diagnostic(&diagnostic_log, format!("ERROR: required HLAE hook is missing: {}", hook.display()));
        bail!("required HLAE hook is missing: {}", hook.display());
    }

    let tf_process_name = settings.tf2_executable.file_name().and_then(|name| name.to_str()).unwrap_or("tf_win64.exe");
    if windows_process_is_running(tf_process_name) {
        log_recording_diagnostic(&diagnostic_log, "ERROR: TF2 was already running; recording was not started");
        bail!("close TF2 before starting an isolated recording session");
    }

    let session_name = session.file_name().and_then(|name| name.to_str()).unwrap_or("rust_session");
    let staged_root = game.join("demos/tf2fragdemohelper_batch").join(session_name);
    let cfg_root = game.join("cfg/tf2fragdemohelper_batch").join(session_name);
    fs::create_dir_all(&staged_root)?;
    fs::create_dir_all(&cfg_root)?;
    fs::create_dir_all(settings.recording_output_directory.join("Videos"))?;
    fs::create_dir_all(settings.recording_output_directory.join("Image Sequences"))?;
    fs::create_dir_all(game.join("cfg"))?;

    let mut by_demo: BTreeMap<PathBuf, Vec<(usize, Candidate)>> = BTreeMap::new();
    for (order, candidate) in candidates.iter().cloned().enumerate() {
        let source = PathBuf::from(&candidate.source_demo);
        if source.is_file() {
            by_demo.entry(source).or_default().push((order + 1, candidate));
        }
    }
    if by_demo.is_empty() {
        log_recording_diagnostic(&diagnostic_log, "ERROR: no selected candidates referenced an existing demo");
        bail!("none of the selected candidates reference an existing demo");
    }

    for clips in by_demo.values_mut() {
        clips.sort_by_key(|(_, candidate)| clip_window(candidate, settings).0);
    }
    let groups = by_demo.into_iter().collect::<Vec<_>>();
    let mut staged_relatives = Vec::new();
    for (demo_index, (source, clips)) in groups.iter().enumerate() {
        let stem = sanitize(source.file_stem().and_then(|value| value.to_str()).unwrap_or("demo"));
        let staged_name = format!("{:03}_{stem}.dem", demo_index + 1);
        let staged = staged_root.join(&staged_name);
        fs::copy(source, &staged).with_context(|| format!("could not stage {}", source.display()))?;
        let relative = format!("demos/tf2fragdemohelper_batch/{session_name}/{staged_name}");
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
    log_recording_diagnostic(&diagnostic_log, format!(
        "Prepared offline launch\nTF2 executable: {}\nHLAE executable: {}\nGame directory: {}\nHook DLL: {}\nHLAE options: -customLoader -autoStart -noGui -programPath <TF2> -cmdLine <shown below> -hookDllPath <hook DLL>\nCandidates: {}\nDemos in queue: {}\nFormat: {} at {} FPS\nInitial staged demo: {}\nTF2 command line: {}",
        settings.tf2_executable.display(), settings.hlae_executable.display(), game.display(), hook.display(), candidates.len(), groups.len(), settings.recording_format, settings.capture_fps, staged_relatives[0], game_arguments
    ));
    let profile = stage_recording_profile(game.as_path(), session_name, tf_process_name, settings)?;
    log_recording_diagnostic(&diagnostic_log, format!("Recording profile staged; original TF2 files are backed up in {}", profile.backup_directory.display()));
    let launch_log = session.join("hlae_launch.log");
    let launch_log_file = File::create(&launch_log)?;
    let mut launch = Command::new(&settings.hlae_executable);
    launch
        .current_dir(hlae_root)
        .args(["-customLoader", "-autoStart", "-noGui", "-programPath"])
        .arg(&settings.tf2_executable)
        .arg("-cmdLine")
        .arg(game_arguments)
        .arg("-hookDllPath")
        .arg(hook)
        .stdout(Stdio::from(launch_log_file.try_clone()?))
        .stderr(Stdio::from(launch_log_file));
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        launch.creation_flags(CREATE_NO_WINDOW);
    }
    let launch_result = launch.spawn();
    let mut child = match launch_result {
        Ok(child) => {
            log_recording_diagnostic(&diagnostic_log, format!("HLAE process started with PID {}. Console output is in {}", child.id(), launch_log.display()));
            child
        }
        Err(error) => {
            log_recording_diagnostic(&diagnostic_log, format!("ERROR: HLAE process could not start: {error}"));
            let _ = restore_recording_profile(&profile);
            return Err(error.into());
        }
    };
    thread::spawn(move || {
        match child.wait() {
            Ok(status) => log_recording_diagnostic(&diagnostic_log, format!("HLAE process exited with status {status}")),
            Err(error) => log_recording_diagnostic(&diagnostic_log, format!("ERROR: could not wait for HLAE: {error}")),
        }
        if wait_for_tf2_exit(&profile.tf_process_name) {
            log_recording_diagnostic(&diagnostic_log, "TF2 was detected and has exited; restoring original TF2 files");
        } else {
            log_recording_diagnostic(&diagnostic_log, "ERROR: TF2 was never detected within 30 seconds of HLAE exiting; restoring original TF2 files");
        }
        if let Err(error) = restore_recording_profile(&profile) {
            log_recording_diagnostic(&diagnostic_log, format!("ERROR: restore failed: {error}"));
            let _ = fs::write(profile.backup_directory.join("RESTORE_REQUIRED.txt"), error.to_string());
        } else {
            log_recording_diagnostic(&diagnostic_log, "Restore verification passed; original TF2 files were restored");
        }
    });
    Ok(session)
}

pub fn recover_interrupted_profile() -> Result<bool> {
    let pointer = active_profile_path();
    if !pointer.is_file() {
        return Ok(false);
    }
    let profile: RecordingProfileSession = serde_json::from_slice(&fs::read(&pointer)?)
        .context("the interrupted recording recovery file is invalid")?;
    if windows_process_is_running(&profile.tf_process_name) {
        bail!("a TF2 recording session is still active; close TF2 before reopening the helper");
    }
    if let Err(error) = restore_recording_profile(&profile) {
        let _ = fs::write(profile.backup_directory.join("RESTORE_REQUIRED.txt"), error.to_string());
        return Err(error);
    }
    Ok(true)
}

/// Match the previous application's FormClosing behavior: if the helper is
/// closed during recording, stop the TF2 instance launched for that session
/// and restore the original custom files and configuration before exiting.
pub fn shutdown_active_recording() -> Result<bool> {
    let pointer = active_profile_path();
    if !pointer.is_file() {
        return Ok(false);
    }
    let profile: RecordingProfileSession = serde_json::from_slice(&fs::read(&pointer)?)
        .context("the active recording recovery file is invalid")?;
    if windows_process_is_running(&profile.tf_process_name) {
        stop_windows_process(&profile.tf_process_name)?;
        for _ in 0..20 {
            if !windows_process_is_running(&profile.tf_process_name) { break; }
            thread::sleep(Duration::from_millis(250));
        }
        if windows_process_is_running(&profile.tf_process_name) {
            bail!("TF2 did not close; original recording files remain safely backed up in {}", profile.backup_directory.display());
        }
    }
    if let Err(error) = restore_recording_profile(&profile) {
        let _ = fs::write(profile.backup_directory.join("RESTORE_REQUIRED.txt"), error.to_string());
        return Err(error);
    }
    Ok(true)
}

fn stage_recording_profile(game: &Path, session_id: &str, tf_process_name: &str, settings: &AppSettings) -> Result<RecordingProfileSession> {
    let custom = game.join("custom");
    let cfg = game.join("cfg").join(PROFILE_CFG);
    let backup = game.join("tf2fragdemohelper_backups").join(session_id);
    let custom_backup = backup.join("custom_original");
    let profile = custom.join(PROFILE_FOLDER);
    let profile_backup = backup.join("custom_profile_original");
    let cfg_backup = backup.join(PROFILE_CFG);
    let config = game.join("cfg").join("config.cfg");
    let config_backup = backup.join("config.cfg");
    let video = game.join("cfg").join("video.txt");
    let video_backup = backup.join("video.txt");
    let offline_config_path = game.join("cfg").join("tf2fragdemohelper_offline.cfg");
    let offline_cfg_backup = backup.join("tf2fragdemohelper_offline.cfg");
    fs::create_dir_all(&backup)?;
    let hitsound_files = backup_hitsound_files(&custom, &backup.join("hitsounds_original"))?;
    let session = RecordingProfileSession {
        session_id: session_id.to_owned(),
        game_directory: game.to_path_buf(),
        backup_directory: backup,
        original_custom_existed: custom.is_dir(),
        original_profile_existed: profile.is_dir(),
        original_profile_cfg_existed: cfg.is_file(),
        isolated_custom: settings.isolate_custom_resources,
        tf_process_name: tf_process_name.to_owned(),
        original_config_existed: config.is_file(),
        original_video_existed: video.is_file(),
        original_offline_cfg_existed: offline_config_path.is_file(),
        hitsound_files,
    };
    write_active_profile(&session)?;

    let result = (|| -> Result<()> {
        if session.original_profile_cfg_existed {
            fs::rename(&cfg, &cfg_backup)?;
        }
        if session.original_config_existed { fs::copy(&config, &config_backup)?; }
        if session.original_video_existed { fs::copy(&video, &video_backup)?; }
        if session.original_offline_cfg_existed { fs::copy(&offline_config_path, &offline_cfg_backup)?; }
        if session.isolated_custom {
            if session.original_custom_existed {
                fs::rename(&custom, &custom_backup).context("could not back up TF2's custom folder")?;
            }
            fs::create_dir_all(&custom)?;
            for selected in &settings.custom_resources {
                let source = if selected.exists() { selected.clone() } else { custom_backup.join(selected.file_name().unwrap_or_default()) };
                let destination = custom.join(source.file_name().unwrap_or_default());
                copy_path(&source, &destination)?;
            }
        } else {
            fs::create_dir_all(&custom)?;
            if session.original_profile_existed {
                fs::rename(&profile, &profile_backup)?;
            }
        }
        install_recording_resources(&custom, settings)?;
        fs::create_dir_all(game.join("cfg"))?;
        fs::write(&cfg, recording_profile_cfg(settings))?;
        fs::write(&offline_config_path, offline_cfg())?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = restore_recording_profile(&session);
        return Err(error);
    }
    Ok(session)
}

fn restore_recording_profile(session: &RecordingProfileSession) -> Result<()> {
    let _restore_guard = restore_lock().lock();
    if !active_profile_path().is_file() {
        return Ok(());
    }
    let custom = session.game_directory.join("custom");
    let cfg = session.game_directory.join("cfg").join(PROFILE_CFG);
    let custom_backup = session.backup_directory.join("custom_original");
    let profile = custom.join(PROFILE_FOLDER);
    let profile_backup = session.backup_directory.join("custom_profile_original");
    let cfg_backup = session.backup_directory.join(PROFILE_CFG);
    let config = session.game_directory.join("cfg").join("config.cfg");
    let config_backup = session.backup_directory.join("config.cfg");
    let video = session.game_directory.join("cfg").join("video.txt");
    let video_backup = session.backup_directory.join("video.txt");
    let offline_cfg = session.game_directory.join("cfg").join("tf2fragdemohelper_offline.cfg");
    let offline_cfg_backup = session.backup_directory.join("tf2fragdemohelper_offline.cfg");
    let config_existed = session.original_config_existed || config_backup.is_file();
    let video_existed = session.original_video_existed || video_backup.is_file();
    let offline_cfg_existed = session.original_offline_cfg_existed || offline_cfg_backup.is_file();

    if session.isolated_custom {
        if custom.exists() {
            fs::remove_dir_all(&custom)?;
        }
        if session.original_custom_existed {
            fs::rename(&custom_backup, &custom).context("could not restore TF2's custom folder")?;
        }
    } else {
        if profile.exists() {
            fs::remove_dir_all(&profile)?;
        }
        if session.original_profile_existed {
            fs::rename(&profile_backup, &profile)?;
        }
        if !session.original_custom_existed && custom.is_dir() && fs::read_dir(&custom)?.next().is_none() {
            fs::remove_dir(&custom)?;
        }
    }
    restore_hitsound_files(session)?;
    if cfg.exists() {
        fs::remove_file(&cfg)?;
    }
    if session.original_profile_cfg_existed {
        fs::rename(&cfg_backup, &cfg)?;
    }
    if config_existed && config_backup.is_file() { fs::copy(&config_backup, &config)?; }
    else if !config_existed && config.exists() { fs::remove_file(&config)?; }
    if video_existed && video_backup.is_file() { fs::copy(&video_backup, &video)?; }
    else if !video_existed && video.exists() { fs::remove_file(&video)?; }
    if offline_cfg.exists() { fs::remove_file(&offline_cfg)?; }
    if offline_cfg_existed && offline_cfg_backup.is_file() { fs::copy(&offline_cfg_backup, &offline_cfg)?; }
    verify_restored_profile(session)?;
    if session.backup_directory.is_dir() {
        fs::remove_dir_all(&session.backup_directory)?;
    }
    let pointer = active_profile_path();
    if pointer.exists() {
        fs::remove_file(pointer)?;
    }
    Ok(())
}

fn restore_lock() -> &'static Mutex<()> {
    static RESTORE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    RESTORE_LOCK.get_or_init(|| Mutex::new(()))
}

fn backup_hitsound_files(custom: &Path, backup: &Path) -> Result<Vec<PathBuf>> {
    if !custom.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(custom).into_iter().filter_map(|entry| entry.ok()).filter(|entry| entry.file_type().is_file()) {
        let relative = entry.path().strip_prefix(custom).context("hitsound path is outside tf/custom")?;
        if !is_hitsound_file(relative) {
            continue;
        }
        let destination = backup.join(relative);
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent)?; }
        fs::copy(entry.path(), &destination)?;
        files.push(relative.to_path_buf());
    }
    Ok(files)
}

fn is_hitsound_file(relative: &Path) -> bool {
    let normalized = relative.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
    let file_name = relative.file_name().and_then(|name| name.to_str()).unwrap_or_default().to_ascii_lowercase();
    file_name.contains("hitsound")
        || file_name.contains("killsound")
        || normalized.contains("/sound/ui/")
        || normalized.starts_with("sound/ui/")
}

fn restore_hitsound_files(session: &RecordingProfileSession) -> Result<()> {
    let source_root = session.backup_directory.join("hitsounds_original");
    let custom = session.game_directory.join("custom");
    for relative in &session.hitsound_files {
        let source = source_root.join(relative);
        if !source.is_file() {
            bail!("hitsound backup is missing: {}", source.display());
        }
        let destination = custom.join(relative);
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent)?; }
        fs::copy(source, destination)?;
    }
    Ok(())
}

fn verify_restored_profile(session: &RecordingProfileSession) -> Result<()> {
    let game = &session.game_directory;
    let backup = &session.backup_directory;
    let custom = game.join("custom");
    let profile = custom.join(PROFILE_FOLDER);
    let profile_cfg = game.join("cfg").join(PROFILE_CFG);
    let config = game.join("cfg/config.cfg");
    let video = game.join("cfg/video.txt");
    let offline = game.join("cfg/tf2fragdemohelper_offline.cfg");
    let config_backup = backup.join("config.cfg");
    let video_backup = backup.join("video.txt");
    let offline_backup = backup.join("tf2fragdemohelper_offline.cfg");
    let config_existed = session.original_config_existed || config_backup.is_file();
    let video_existed = session.original_video_existed || video_backup.is_file();
    let offline_existed = session.original_offline_cfg_existed || offline_backup.is_file();
    let mut problems = Vec::new();

    if session.original_custom_existed != custom.is_dir() {
        problems.push("tf/custom was not returned to its original presence state".to_owned());
    }
    if session.isolated_custom && backup.join("custom_original").exists() {
        problems.push("the original tf/custom folder is still staged in the backup".to_owned());
    }
    if !session.isolated_custom && session.original_profile_existed != profile.is_dir() {
        problems.push("the previous helper recording-resource folder was not restored".to_owned());
    }
    if session.original_profile_cfg_existed != profile_cfg.is_file() {
        problems.push("the previous recording profile CFG was not restored".to_owned());
    }
    if config_existed {
        if !files_match(&config, &config_backup)? { problems.push("config.cfg does not match its backup; hitsound settings may not be restored".to_owned()); }
    } else if config.exists() {
        problems.push("temporary config.cfg still exists".to_owned());
    }
    if video_existed {
        if !files_match(&video, &video_backup)? { problems.push("video.txt does not match its backup".to_owned()); }
    } else if video.exists() {
        problems.push("temporary video.txt still exists".to_owned());
    }
    if offline_existed {
        if !files_match(&offline, &offline_backup)? { problems.push("the previous offline CFG was not restored".to_owned()); }
    } else if offline.exists() {
        problems.push("temporary offline CFG still exists".to_owned());
    }
    for relative in &session.hitsound_files {
        if !files_match(&custom.join(relative), &backup.join("hitsounds_original").join(relative))? {
            problems.push(format!("custom hitsound was not restored: {}", relative.display()));
        }
    }
    if !problems.is_empty() {
        bail!("restore verification failed: {}. Original backups remain in {}", problems.join("; "), backup.display());
    }
    Ok(())
}

fn files_match(left: &Path, right: &Path) -> Result<bool> {
    if !left.is_file() || !right.is_file() { return Ok(false); }
    if fs::metadata(left)?.len() != fs::metadata(right)?.len() { return Ok(false); }
    let mut left_file = File::open(left)?;
    let mut right_file = File::open(right)?;
    let mut left_buffer = [0_u8; 64 * 1024];
    let mut right_buffer = [0_u8; 64 * 1024];
    loop {
        let left_read = left_file.read(&mut left_buffer)?;
        let right_read = right_file.read(&mut right_buffer)?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] { return Ok(false); }
        if left_read == 0 { return Ok(true); }
    }
}

fn write_active_profile(session: &RecordingProfileSession) -> Result<()> {
    let pointer = active_profile_path();
    if let Some(parent) = pointer.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = pointer.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(session)?)?;
    if pointer.exists() {
        fs::remove_file(&pointer)?;
    }
    fs::rename(temporary, pointer)?;
    Ok(())
}

fn active_profile_path() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("TF2FragDemoHelper").join("active_recording_profile.json")
}

fn install_recording_resources(custom: &Path, settings: &AppSettings) -> Result<()> {
    let needs_assets = settings.disable_announcer_voices
        || settings.disable_applause_sounds
        || settings.disable_domination_sounds
        || !settings.skybox.eq_ignore_ascii_case("Default")
        || (!settings.hud.eq_ignore_ascii_case("Keep current") && !settings.hud.eq_ignore_ascii_case("Default TF2 HUD"));
    if !needs_assets {
        return Ok(());
    }
    let resources = find_recording_resources()?;
    let profile = custom.join(PROFILE_FOLDER);
    fs::create_dir_all(&profile)?;
    for (enabled, name) in [
        (settings.disable_announcer_voices, "no_announcer_voices.vpk"),
        (settings.disable_applause_sounds, "no_applause_sounds.vpk"),
        (settings.disable_domination_sounds, "no_domination_sounds.vpk"),
    ] {
        let source = resources.join("custom").join(name);
        if enabled && source.is_file() {
            fs::copy(source, profile.join(name))?;
        }
    }
    if settings.hud.eq_ignore_ascii_case("Kill notices only") {
        copy_directory(&resources.join("hud/hud_killnotices"), &profile)?;
    } else if settings.hud.eq_ignore_ascii_case("Medic recording HUD") {
        copy_directory(&resources.join("hud/hud_medic"), &profile)?;
    }
    if !settings.skybox.eq_ignore_ascii_case("Default") {
        install_skybox(&resources.join("skybox"), &profile.join("materials/skybox"), &settings.skybox)?;
    }
    Ok(())
}

fn find_recording_resources() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let executable_directory = executable.parent().context("application directory is unavailable")?;
    for candidate in [
        executable_directory.join("recording_resources"),
        executable_directory.join("../recording_resources"),
        PathBuf::from("recording_resources"),
    ] {
        if candidate.join("custom").is_dir() {
            return Ok(candidate);
        }
    }
    let cache = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")).join("TF2FragDemoHelper").join(RESOURCE_CACHE_VERSION);
    if cache.join("complete.marker").is_file() && cache.join("custom").is_dir() {
        return Ok(cache);
    }
    let archive_directory = [
        executable_directory.join("recording_resources_archive"),
        executable_directory.join("../recording_resources_archive"),
        executable_directory.join("../../recording_resources_archive"),
        PathBuf::from("recording_resources_archive"),
    ].into_iter().find(|path| path.join("resources.part000").is_file())
        .context("bundled recording resources are missing; keep recording_resources_archive beside the application")?;
    extract_resource_archive(&archive_directory, &cache)?;
    Ok(cache)
}

fn extract_resource_archive(parts_directory: &Path, cache: &Path) -> Result<()> {
    let parent = cache.parent().context("resource cache has no parent")?;
    fs::create_dir_all(parent)?;
    let staging = parent.join(format!("{RESOURCE_CACHE_VERSION}_staging"));
    let joined = parent.join(format!("{RESOURCE_CACHE_VERSION}.zip"));
    if staging.exists() { fs::remove_dir_all(&staging)?; }
    if joined.exists() { fs::remove_file(&joined)?; }
    fs::create_dir_all(&staging)?;

    let mut parts = fs::read_dir(parts_directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.starts_with("resources.part")))
        .collect::<Vec<_>>();
    parts.sort();
    if parts.is_empty() { bail!("recording resource archive has no parts"); }
    let mut output = File::create(&joined)?;
    for part in parts {
        io::copy(&mut File::open(part)?, &mut output)?;
    }
    drop(output);

    let mut archive = ZipArchive::new(File::open(&joined)?)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let relative = entry.enclosed_name().context("recording resource archive contains an unsafe path")?;
        let destination = staging.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(destination)?;
        } else {
            if let Some(parent) = destination.parent() { fs::create_dir_all(parent)?; }
            io::copy(&mut entry, &mut File::create(destination)?)?;
        }
    }
    fs::write(staging.join("complete.marker"), b"TF2 Frag Demo Helper recording resources v2\n")?;
    if cache.exists() { fs::remove_dir_all(cache)?; }
    fs::rename(&staging, cache)?;
    fs::remove_file(joined)?;
    Ok(())
}

fn install_skybox(source: &Path, destination: &Path, selected: &str) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("vmt")) {
            fs::copy(&path, destination.join(path.file_name().unwrap_or_default()))?;
        }
    }
    for side in ["bk", "dn", "ft", "lf", "rt", "up"] {
        let texture = source.join(format!("{selected}{side}.vtf"));
        if !texture.is_file() { bail!("selected recording skybox is incomplete: {}", texture.display()); }
        for entry in fs::read_dir(destination)? {
            let path = entry?.path();
            if path.file_stem().and_then(|value| value.to_str()).is_some_and(|value| value.ends_with(side))
                && path.extension().and_then(|value| value.to_str()) == Some("vmt") {
                fs::copy(&texture, path.with_extension("vtf"))?;
            }
        }
    }
    Ok(())
}

fn copy_path(source: &Path, destination: &Path) -> Result<()> {
    if source.is_dir() { copy_directory(source, destination) }
    else if source.is_file() { fs::copy(source, destination).map(|_| ()).map_err(Into::into) }
    else { bail!("selected custom resource is missing: {}", source.display()) }
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_dir() { bail!("required recording resource is missing: {}", source.display()); }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        let target = destination.join(path.file_name().unwrap_or_default());
        if path.is_dir() { copy_directory(&path, &target)?; }
        else { fs::copy(path, target)?; }
    }
    Ok(())
}

fn windows_process_is_running(image_name: &str) -> bool {
    if !cfg!(target_os = "windows") { return false; }
    let mut tasklist = Command::new("tasklist");
    tasklist.args(["/FI", &format!("IMAGENAME eq {image_name}"), "/NH"]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        tasklist.creation_flags(CREATE_NO_WINDOW);
    }
    tasklist
        .output().ok().is_some_and(|output| String::from_utf8_lossy(&output.stdout).to_ascii_lowercase().contains(&image_name.to_ascii_lowercase()))
}

fn stop_windows_process(image_name: &str) -> Result<()> {
    if !cfg!(target_os = "windows") { return Ok(()); }
    let mut taskkill = Command::new("taskkill");
    taskkill.args(["/IM", image_name, "/T", "/F"]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        taskkill.creation_flags(CREATE_NO_WINDOW);
    }
    let output = taskkill.output()?;
    if !output.status.success() && windows_process_is_running(image_name) {
        bail!("could not close TF2: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

/// Resolve TF2's actual game directory from both current x64 installs
/// (`tf/win64/tf_win64.exe`) and older layouts (`tf/tf.exe`).  Treating the
/// executable's parent as the install root created the invalid `tf/win64/tf`
/// path responsible for Windows error 3 during preview and HLAE launch.
fn tf2_game_directory(executable: &Path) -> Result<PathBuf> {
    let binary_directory = executable.parent().context("could not find the TF2 executable directory")?;
    let direct_game = binary_directory.join("cfg");
    if direct_game.is_dir() {
        return Ok(binary_directory.to_path_buf());
    }
    let sibling_game = binary_directory.join("tf");
    if sibling_game.join("cfg").is_dir() {
        return Ok(sibling_game);
    }
    if let Some(game) = binary_directory.parent() {
        if game.join("cfg").is_dir() {
            return Ok(game.to_path_buf());
        }
    }
    if let Some(root) = binary_directory.parent() {
        let game = root.join("tf");
        if game.join("cfg").is_dir() {
            return Ok(game);
        }
    }
    bail!("could not locate TF2's tf/cfg directory from {}", executable.display())
}

fn wait_for_tf2_exit(image_name: &str) -> bool {
    let mut was_seen = false;
    for _ in 0..30 {
        if windows_process_is_running(image_name) {
            was_seen = true;
            break;
        }
        thread::sleep(Duration::from_secs(1));
    }
    while windows_process_is_running(image_name) {
        was_seen = true;
        thread::sleep(Duration::from_secs(1));
    }
    was_seen
}

fn log_recording_diagnostic(path: &Path, message: impl AsRef<str>) {
    let result = (|| -> io::Result<()> {
        let mut log = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(log, "[{}] {}", Utc::now().to_rfc3339(), message.as_ref())
    })();
    let _ = result;
}

fn clip_window(candidate: &Candidate, settings: &AppSettings) -> (i64, i64) {
    let first = candidate.point_of_kill_ticks.first().copied().unwrap_or(candidate.clip_start_tick);
    let last = candidate.point_of_kill_ticks.last().copied().unwrap_or(candidate.clip_end_tick);
    let lead_ticks = (settings.lead_seconds as f64 * 66.666_666_7).round() as i64;
    let outro_ticks = (settings.outro_seconds as f64 * 66.666_666_7).round() as i64;
    let start = (first - lead_ticks).max(0);
    let end = last + outro_ticks;
    (start, end.max(start + 1))
}

fn candidate_needs_spectator_focus(candidate: &Candidate) -> bool {
    if candidate.attacker_user_id <= 0 {
        return false;
    }
    if candidate.demo_context.capture_type.eq_ignore_ascii_case("stv") {
        return true;
    }
    // Older exports could mislabel an STV as POV when dem_usercmd packets were
    // present.  A supposed POV with no identified POV player and all-player
    // analysis cannot safely preserve a single recorded POV, so explicitly
    // focus the selected candidate just as an STV requires.
    candidate.demo_context.analysis_scope.eq_ignore_ascii_case("all_players")
        && candidate.demo_context.pov_player_user_id.is_none()
}

fn preview_vdm_text(candidate: &Candidate, target_tick: i64) -> String {
    let mut lines = vec!["demoactions".to_owned(), "{".to_owned()];
    let mut action = 1;
    add_vdm_action(
        &mut lines,
        &mut action,
        "SkipAhead",
        "TF2 Frag Demo Helper seek",
        1,
        Some(target_tick),
        "",
    );
    if candidate_needs_spectator_focus(candidate) {
        add_vdm_action(
            &mut lines,
            &mut action,
            "PlayCommands",
            "Focus selected STV candidate",
            target_tick + 1,
            None,
            &format!("spec_autodirector 0; spec_player #{}; spec_mode 4", candidate.attacker_user_id),
        );
    }
    lines.push("}".into());
    lines.join("\n")
}

fn vdm_text(clips: &[(usize, Candidate)], session_name: &str, next_demo: Option<&str>, settings: &AppSettings) -> String {
    let mut lines = vec!["demoactions".to_owned(), "{".to_owned()];
    let mut action = 1;
    add_vdm_action(&mut lines, &mut action, "PlayCommands", "Apply movie profile", 1, None, "exec tf2fragdemohelper_recording_profile");
    let recorder_flush_ticks = (66.666_666_7_f64 * 2.0).round() as i64;
    let mut previous_finalize_tick = -1;
    for (order, candidate) in clips {
        let (mut start, mut end) = clip_window(candidate, settings);
        if start <= previous_finalize_tick {
            start = previous_finalize_tick + 2;
        }
        if end <= start {
            end = start + 1;
        }
        let base = format!("{:03}_{}_t{}-{}", order, sanitize(&candidate.candidate_id), candidate.clip_start_tick, candidate.clip_end_tick);
        let seek_at = if previous_finalize_tick < 0 { 2 } else { previous_finalize_tick + 2 };
        add_vdm_action(&mut lines, &mut action, "SkipAhead", "Batch seek", seek_at, Some(start), "");
        let focus = if candidate_needs_spectator_focus(candidate) {
            format!("spec_autodirector 0; spec_player #{}; spec_mode 4; ", candidate.attacker_user_id)
        } else { String::new() };
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Start clip", start + 1, None, &format!("{focus}exec tf2fragdemohelper_batch/{session_name}/{base}_start"));
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Stop clip", end, None, &format!("exec tf2fragdemohelper_batch/{session_name}/{base}_stop"));
        previous_finalize_tick = end + recorder_flush_ticks;
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Finalize clip", previous_finalize_tick, None, &format!("echo TF2FRAG_RECORD_FINALIZED {base}"));
    }
    let finish = next_demo.map(|demo| format!("playdemo {demo}")).unwrap_or_else(|| "quit".into());
    add_vdm_action(&mut lines, &mut action, "PlayCommands", "Continue batch", previous_finalize_tick + 2, None, &finish);
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
    "// Generated by TF2 Frag Demo Helper. Offline demo playback only.\nsv_lan 1\nsv_master_legacy_mode 1\nnet_start 0\ncl_allowdownload 0\ncl_downloadfilter none\nalias connect \"echo BLOCKED: recording mode cannot connect to servers\"\nalias retry \"echo BLOCKED: recording mode cannot reconnect\"\nalias openserverbrowser \"echo BLOCKED: recording mode is offline only\"\nengine_no_focus_sleep 0\nsnd_mute_losefocus 0\n"
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
    let cache_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let modified = metadata.modified().ok();
    let cache = DEMO_SIGNATURE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(entry) = cache.lock().get(&cache_key) {
        if entry.length == metadata.len() && entry.modified == modified {
            return Ok(entry.signature.clone());
        }
    }
    let mut file = File::open(path)?;
    let mut head = vec![0u8; 1024 * 1024];
    let read = file.read(&mut head)?;
    let mut hash = Sha256::new();
    hash.update(metadata.len().to_le_bytes());
    hash.update(&head[..read]);
    let signature = hex::encode(hash.finalize());
    cache.lock().insert(cache_key, DemoSignatureCacheEntry {
        length: metadata.len(),
        modified,
        signature: signature.clone(),
    });
    Ok(signature)
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

fn index_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TF2FragDemoHelper")
        .join("recorded_clip_index.ndjson")
}
