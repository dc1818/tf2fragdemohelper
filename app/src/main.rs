#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod analyzer;
mod batch;
mod filter;
mod models;
mod recording;
mod scheduler;

use crate::{
    batch::{BatchController, ProgressEvent},
    filter::CandidateFilter,
    models::{AppSettings, Candidate},
    recording::{launch_hlae_batch, preview_candidate, recover_interrupted_profile, RecordingIndex},
    scheduler::PerformanceProfile,
};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use slint::{Model, ModelRc, SharedString, VecModel, Weak};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    thread,
};

slint::include_modules!();

struct State {
    demos: Vec<PathBuf>,
    export_root: Option<PathBuf>,
    candidates: Vec<Candidate>,
    visible: Vec<usize>,
    selected: Vec<bool>,
    settings: AppSettings,
    controller: Option<BatchController>,
    recording_index: RecordingIndex,
}

fn main() -> Result<()> {
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
    ui.set_export_directory(settings.output_directory.display().to_string().into());
    ui.set_item_schema(settings.item_schema.display().to_string().into());
    ui.set_tf2_path(settings.tf2_executable.display().to_string().into());
    ui.set_hlae_path(settings.hlae_executable.display().to_string().into());
    ui.set_ffmpeg_path(settings.ffmpeg_executable.display().to_string().into());
    ui.set_recording_directory(settings.recording_output_directory.display().to_string().into());
    ui.set_lead_seconds(settings.lead_seconds as i32);
    ui.set_outro_seconds(settings.outro_seconds as i32);
    ui.set_capture_fps(settings.capture_fps.to_string().into());
    ui.set_jpg_quality(settings.jpg_quality as i32);
    ui.set_recording_format(settings.recording_format.clone().into());
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
    if recovered_recording_profile {
        ui.set_status_text("Recovered TF2 files from an interrupted recording session".into());
    }

    let state = Arc::new(Mutex::new(State {
        demos: Vec::new(), export_root: None, candidates: Vec::new(), visible: Vec::new(), selected: Vec::new(),
        settings, controller: None, recording_index: RecordingIndex::load(),
    }));
    bind_file_callbacks(&ui, &state);
    bind_batch_callbacks(&ui, &state);
    bind_candidate_callbacks(&ui, &state);
    bind_settings_callbacks(&ui, &state);
    ui.run()?;
    Ok(())
}

fn bind_file_callbacks(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let weak = ui.as_weak();
    let state = state.clone();
    ui.on_choose_demos(move || {
        let files = rfd::FileDialog::new().add_filter("TF2 demos", &["dem"]).pick_files().unwrap_or_default();
        if files.is_empty() { return }
        state.lock().demos = files.clone();
        if let Some(ui) = weak.upgrade() {
            ui.set_demo_paths(files.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join("; ").into());
        }
    });
    let weak = ui.as_weak();
    ui.on_choose_export_directory(move || {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            if let Some(ui) = weak.upgrade() { ui.set_export_directory(path.display().to_string().into()); }
        }
    });
    let weak = ui.as_weak();
    ui.on_choose_item_schema(move || {
        if let Some(path) = rfd::FileDialog::new().add_filter("TF2 item schema", &["txt"]).pick_file() {
            if let Some(ui) = weak.upgrade() { ui.set_item_schema(path.display().to_string().into()); }
        }
    });
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
        let schema = (!schema_text.trim().is_empty()).then(|| PathBuf::from(schema_text));
        let performance_profile = PerformanceProfile::from_setting(&ui.get_performance_profile().to_string());
        let controller = BatchController::new();
        state_for_start.lock().controller = Some(controller.clone());
        ui.set_busy(true);
        ui.set_has_export(false);
        ui.set_selected_count(0);
        ui.set_selected_page(0);
        ui.set_progress_value(0.0);
        ui.set_log_text("".into());
        ui.set_log_scroll_offset(0.0);
        ui.set_status_text("Preparing Rust resource plan...".into());
        let weak_for_thread = weak.clone();
        let state_for_thread = state_for_start.clone();
        thread::spawn(move || {
            let progress_weak = weak_for_thread.clone();
            let sink: batch::ProgressSink = Arc::new(move |event| {
                let weak = progress_weak.clone();
                let _ = slint::invoke_from_event_loop(move || update_progress(&weak, event));
            });
            let result = batch::run_batch(demos, output, schema, performance_profile, controller, sink);
            if let Ok(root) = result {
                match load_candidates(&root.join("frag_candidates.ndjson")) {
                    Ok(candidates) => {
                        let mut state = state_for_thread.lock();
                        state.export_root = Some(root);
                        state.selected = vec![false; candidates.len()];
                        state.candidates = candidates;
                        drop(state);
                        let state = state_for_thread.clone();
                        let weak = weak_for_thread.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak.upgrade() {
                                refresh_candidates(&ui, &state, "", 0);
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
        if let Some(controller) = &state_for_cancel.lock().controller { controller.cancel(); }
    });

    let weak = ui.as_weak();
    let state_for_load = state.clone();
    ui.on_load_export(move || {
        let Some(root) = rfd::FileDialog::new().pick_folder() else { return };
        let path = if root.join("frag_candidates.ndjson").is_file() { root.join("frag_candidates.ndjson") } else { return };
        match load_candidates(&path) {
            Ok(candidates) => {
                let mut state = state_for_load.lock();
                state.export_root = Some(root);
                state.selected = vec![false; candidates.len()];
                state.candidates = candidates;
                drop(state);
                if let Some(ui) = weak.upgrade() {
                    refresh_candidates(&ui, &state_for_load, "", 0);
                    ui.set_has_export(true);
                    ui.set_selected_page(1);
                    ui.set_status_text("Loaded parsed export".into());
                }
            }
            Err(error) => if let Some(ui) = weak.upgrade() { ui.set_status_text(error.to_string().into()); },
        }
    });

    let weak = ui.as_weak();
    let state_for_open = state.clone();
    ui.on_open_export(move || {
        if let Some(path) = state_for_open.lock().export_root.clone() {
            if let Err(error) = open_path(&path) {
                if let Some(ui) = weak.upgrade() { ui.set_status_text(error.to_string().into()); }
            }
        }
    });
}

fn bind_candidate_callbacks(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let weak = ui.as_weak();
    let state_for_filter = state.clone();
    ui.on_refresh_filter(move |filter, score| {
        if let Some(ui) = weak.upgrade() { refresh_candidates(&ui, &state_for_filter, &filter, score); }
    });
    let weak = ui.as_weak();
    let state_for_toggle = state.clone();
    ui.on_toggle_candidate(move |visible_index| {
        let mut state = state_for_toggle.lock();
        if let Some(candidate_index) = state.visible.get(visible_index as usize).copied() {
            state.selected[candidate_index] = !state.selected[candidate_index];
            let filter = weak.upgrade().map(|ui| ui.get_filter_text().to_string()).unwrap_or_default();
            let score = weak.upgrade().map(|ui| ui.get_minimum_score()).unwrap_or_default();
            drop(state);
            if let Some(ui) = weak.upgrade() {
                refresh_candidates(&ui, &state_for_toggle, &filter, score);
            }
        }
    });
    let weak = ui.as_weak();
    let state_for_drag = state.clone();
    ui.on_drag_select_candidates(move |start_index, delta_pixels, selecting| {
        let Some(ui) = weak.upgrade() else { return };
        let mut state = state_for_drag.lock();
        if state.visible.is_empty() { return }
        let last = state.visible.len().saturating_sub(1) as i32;
        let start = start_index.clamp(0, last);
        let target = (start + (delta_pixels / 30.0).round() as i32).clamp(0, last);
        let first_row = start.min(target) as usize;
        let last_row = start.max(target) as usize;
        let model = ui.get_candidate_rows();
        for visible_row in first_row..=last_row {
            if let Some(candidate_index) = state.visible.get(visible_row).copied() {
                state.selected[candidate_index] = selecting;
                if let Some(mut row) = model.row_data(visible_row) {
                    row.selected = selecting;
                    model.set_row_data(visible_row, row);
                }
            }
        }
        ui.set_selected_count(state.selected.iter().filter(|selected| **selected).count().min(i32::MAX as usize) as i32);
    });
    let weak = ui.as_weak();
    let state_for_all = state.clone();
    ui.on_select_all_visible(move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut state = state_for_all.lock();
        let visible = state.visible.clone();
        let deselect = !visible.is_empty() && visible.iter().all(|index| state.selected.get(*index).copied().unwrap_or(false));
        let model = ui.get_candidate_rows();
        for (visible_row, candidate_index) in visible.into_iter().enumerate() {
            state.selected[candidate_index] = !deselect;
            if let Some(mut row) = model.row_data(visible_row) {
                row.selected = !deselect;
                model.set_row_data(visible_row, row);
            }
        }
        ui.set_selected_count(state.selected.iter().filter(|selected| **selected).count().min(i32::MAX as usize) as i32);
    });
    let weak = ui.as_weak();
    let state_for_preview = state.clone();
    ui.on_preview_selected(move || {
        let Some(ui) = weak.upgrade() else { return };
        let (candidate, mut settings) = {
            let state = state_for_preview.lock();
            let selected = selected_candidates(&state);
            if selected.len() != 1 {
                ui.set_status_text("Select exactly one candidate to preview".into());
                return;
            }
            (selected[0].clone(), state.settings.clone())
        };

        let entered_path = PathBuf::from(ui.get_tf2_path().to_string());
        if entered_path.is_file() {
            settings.tf2_executable = entered_path;
        }
        if !settings.tf2_executable.is_file() {
            let mut dialog = rfd::FileDialog::new().set_title("Select the Team Fortress 2 Executable");
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

        let result = preview_candidate(&candidate, &settings);
        ui.set_status_text(result.map(|_| "TF2 preview launched".into()).unwrap_or_else(|error| error.to_string()).into());
    });
    let weak = ui.as_weak();
    let state_for_record = state.clone();
    ui.on_record_selected(move || {
        let Some(ui) = weak.upgrade() else { return };
        let (selected, mut settings) = {
            let state = state_for_record.lock();
            (selected_candidates(&state).into_iter().cloned().collect::<Vec<_>>(), state.settings.clone())
        };
        settings.tf2_executable = PathBuf::from(ui.get_tf2_path().to_string());
        settings.hlae_executable = PathBuf::from(ui.get_hlae_path().to_string());
        settings.ffmpeg_executable = PathBuf::from(ui.get_ffmpeg_path().to_string());
        settings.recording_output_directory = PathBuf::from(ui.get_recording_directory().to_string());
        settings.lead_seconds = ui.get_lead_seconds().max(0) as u32;
        settings.outro_seconds = ui.get_outro_seconds().max(0) as u32;
        settings.capture_fps = ui.get_capture_fps().parse().unwrap_or(120);
        settings.jpg_quality = ui.get_jpg_quality().clamp(1, 100) as u8;
        settings.recording_format = ui.get_recording_format().to_string();
        settings.performance_profile = ui.get_performance_profile().to_string();
        let result = launch_hlae_batch(&selected, &settings);
        ui.set_status_text(result.map(|path| format!("Offline HLAE recording launched: {}", path.display())).unwrap_or_else(|error| error.to_string()).into());
    });
}

fn bind_settings_callbacks(ui: &AppWindow, state: &Arc<Mutex<State>>) {
    let weak = ui.as_weak();
    ui.on_choose_setting_path(move |kind| {
        let kind = kind.to_string();
        let path = if kind == "recording-output" { rfd::FileDialog::new().pick_folder() } else { rfd::FileDialog::new().pick_file() };
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
        }
    });
    let weak = ui.as_weak();
    let state_for_save = state.clone();
    ui.on_save_settings(move || {
        let Some(ui) = weak.upgrade() else { return };
        let mut state = state_for_save.lock();
        state.settings.output_directory = PathBuf::from(ui.get_export_directory().to_string());
        state.settings.item_schema = PathBuf::from(ui.get_item_schema().to_string());
        state.settings.tf2_executable = PathBuf::from(ui.get_tf2_path().to_string());
        state.settings.hlae_executable = PathBuf::from(ui.get_hlae_path().to_string());
        state.settings.ffmpeg_executable = PathBuf::from(ui.get_ffmpeg_path().to_string());
        state.settings.recording_output_directory = PathBuf::from(ui.get_recording_directory().to_string());
        state.settings.lead_seconds = ui.get_lead_seconds() as u32;
        state.settings.outro_seconds = ui.get_outro_seconds() as u32;
        state.settings.capture_fps = ui.get_capture_fps().parse().unwrap_or(120);
        state.settings.jpg_quality = ui.get_jpg_quality() as u8;
        state.settings.recording_format = ui.get_recording_format().to_string();
        state.settings.performance_profile = ui.get_performance_profile().to_string();
        state.settings.resolution = ui.get_resolution().to_string();
        state.settings.dx_level = ui.get_dx_level().to_string();
        state.settings.skybox = ui.get_skybox().to_string();
        state.settings.hud = ui.get_hud().to_string();
        state.settings.viewmodels = ui.get_viewmodels().to_string();
        state.settings.viewmodel_fov = ui.get_viewmodel_fov() as u32;
        state.settings.maximum_graphics = ui.get_maximum_graphics();
        state.settings.motion_blur = ui.get_motion_blur();
        state.settings.disable_hit_sounds = ui.get_disable_hit_sounds();
        state.settings.disable_voice_chat = ui.get_disable_voice_chat();
        state.settings.minimal_hud = ui.get_minimal_hud();
        state.settings.disable_combat_text = ui.get_disable_combat_text();
        state.settings.disable_crosshair = ui.get_disable_crosshair();
        state.settings.disable_crosshair_switching = ui.get_disable_crosshair_switching();
        state.settings.hud_player_model = ui.get_hud_player_model();
        state.settings.isolate_custom_resources = ui.get_isolate_custom_resources();
        state.settings.disable_announcer_voices = ui.get_disable_announcer();
        state.settings.disable_applause_sounds = ui.get_disable_applause();
        state.settings.disable_domination_sounds = ui.get_disable_domination();
        let status = state.settings.save().map(|_| "Settings saved".to_owned()).unwrap_or_else(|error| error.to_string());
        ui.set_status_text(status.into());
    });
}

fn update_progress(weak: &Weak<AppWindow>, event: ProgressEvent) {
    let Some(ui) = weak.upgrade() else { return };
    match event {
        ProgressEvent::Plan(plan) => {
            ui.set_resource_plan_text(format!("{} performance | {} logical CPUs | adaptive parser ceiling {} | adaptive analyzer ceiling {} | available RAM {:.1} GB", plan.performance_profile.label(), plan.logical_processors, plan.parse_worker_ceiling, plan.analysis_worker_ceiling, plan.available_memory_bytes as f64 / 1_073_741_824.0).into());
        }
        ProgressEvent::Log(line) => {
            let mut text = ui.get_log_text().to_string();
            if text.len() > 200_000 { text.drain(..100_000); }
            text.push_str(&line); text.push('\n');
            ui.set_log_text(text.into());
            ui.set_log_scroll_offset(-1_000_000.0);
        }
        ProgressEvent::Phase { phase, completed, total, fraction, eta_seconds, active_workers, worker_limit } => {
            ui.set_progress_value(fraction);
            let eta = eta_seconds.map(format_duration).unwrap_or_else(|| "estimating…".into());
            ui.set_status_text(format!("Phase {phase} of 2: {completed}/{total} | active workers {active_workers}/{worker_limit} | ETA {eta}").into());
        }
        ProgressEvent::Complete { export_root, candidates } => {
            ui.set_busy(false); ui.set_progress_value(1.0); ui.set_status_text(format!("Complete: {candidates} candidates — {}", export_root.display()).into());
        }
        ProgressEvent::Failed(error) => { ui.set_busy(false); ui.set_status_text(format!("Failed: {error}").into()); }
        ProgressEvent::Cancelled => { ui.set_busy(false); ui.set_status_text("Cancelled; completed exports were retained".into()); }
    }
}

fn refresh_candidates(ui: &AppWindow, state: &Arc<Mutex<State>>, filter: &str, minimum_score: i32) {
    let expression = CandidateFilter::parse(filter);
    let mut state = state.lock();
    state.visible.clear();
    let mut rows = Vec::new();
    for index in 0..state.candidates.len() {
        let recorded = {
            let candidate = state.candidates[index].clone();
            let root = state.settings.recording_output_directory.clone();
            state.recording_index.is_recorded(&candidate, Some(&root))
        };
        let candidate = state.candidates[index].clone();
        if candidate.overall_score < minimum_score as f64 || !expression.matches(&candidate, recorded) { continue }
        state.visible.push(index);
        rows.push(CandidateRow {
            rank: (index + 1) as i32,
            score: format!("{:.1}", candidate.overall_score).into(),
            kills: candidate.kill_count().to_string().into(),
            attacker: format!("#{}", candidate.attacker_user_id).into(),
            class_name: candidate.attacker_class.clone().into(),
            team: candidate.attacker_team.clone().into(),
            demo: Path::new(&candidate.source_demo).file_name().and_then(|name| name.to_str()).unwrap_or(&candidate.source_demo).into(),
            map_name: candidate.map_name.clone().into(),
            mode: if candidate.demo_context.mode_label.is_empty() { "Unknown / Mixed" } else { &candidate.demo_context.mode_label }.into(),
            demo_type: candidate.demo_context.capture_type.to_uppercase().into(),
            recorded: if recorded { "Recorded" } else { "" }.into(),
            ticks: candidate.point_of_kill_ticks.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ").into(),
            tags: candidate.tags.join(", ").into(),
            selected: state.selected.get(index).copied().unwrap_or(false),
        });
    }
    ui.set_candidate_summary(format!("{} of {} ranked candidates", rows.len(), state.candidates.len()).into());
    ui.set_candidate_rows(ModelRc::new(VecModel::from(rows)));
    ui.set_selected_count(state.selected.iter().filter(|selected| **selected).count().min(i32::MAX as usize) as i32);
}

fn selected_candidates(state: &State) -> Vec<&Candidate> {
    state.candidates.iter().zip(&state.selected).filter_map(|(candidate, selected)| selected.then_some(candidate)).collect()
}

fn load_candidates(path: &Path) -> Result<Vec<Candidate>> {
    BufReader::new(File::open(path).with_context(|| format!("missing {}", path.display()))?)
        .lines()
        .filter_map(|line| match line {
            Ok(line) if !line.trim().is_empty() => Some(serde_json::from_str(&line).map_err(anyhow::Error::from)),
            Ok(_) => None,
            Err(error) => Some(Err(error.into())),
        })
        .collect()
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours > 0 { format!("{hours}h {minutes:02}m") } else if minutes > 0 { format!("{minutes}m {seconds:02}s") } else { format!("{seconds}s") }
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
