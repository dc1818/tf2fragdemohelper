#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use anyhow::{Context, Result};
use slint::winit_030::{winit, WinitWindowAccessor};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::{env, fs, path::Path, time::Duration};
use tf2_mirv_director::{DirectorControl, DirectorSession};

slint::include_modules!();

fn main() -> Result<()> {
    let path = env::args_os()
        .nth(1)
        .context("usage: tf2-mirv-director <director_session.json>")?;
    let session = load_session(Path::new(&path))?;
    let window = DirectorWindow::new()?;

    window.set_candidate_id(session.candidate_id.clone().into());
    window.set_demo_file(session.demo_file.clone().into());
    window.set_map_name(session.map_name.clone().into());
    window.set_start_tick(session.start_tick as i32);
    window.set_end_tick(session.end_tick as i32);
    window.set_output_directory(session.output_directory.display().to_string().into());
    window.set_campath_file(session.campath_file.display().to_string().into());
    window.set_whole_tags(session.whole_candidate_tags.join(", ").into());
    window.set_bridge_ready(matches!(session.control, DirectorControl::LocalBridge { .. }));

    let shortcut_key = |id: &str, fallback: &str| {
        session
            .shortcuts
            .iter()
            .find(|shortcut| shortcut.id == id)
            .map(|shortcut| shortcut.key.as_str())
            .unwrap_or(fallback)
            .to_owned()
    };
    window.set_next_cue_key(shortcut_key("next_kill_tick", "4").into());
    window.set_record_order(
        format!(
            "{} RESUME  →  {} START  →  {} STOP",
            shortcut_key("pause_resume", "5"),
            shortcut_key("start_recording", "9"),
            shortcut_key("stop_recording", "0")
        )
        .into(),
    );

    let cue_rows = session
        .cues
        .iter()
        .enumerate()
        .map(|(index, cue)| CueRow {
            tick: cue.tick as i32,
            position: session.cue_position(cue.tick),
            label: SharedString::from(if cue.label.is_empty() {
                format!("CUE {}", index + 1)
            } else {
                cue.label.clone()
            }),
            tags: cue.tags.join(", ").into(),
            victims: cue.victims.join(", ").into(),
        })
        .collect::<Vec<_>>();
    window.set_cues(ModelRc::new(VecModel::from(cue_rows)));

    let mut shortcut_rows = session
        .shortcuts
        .iter()
        .map(|shortcut| ShortcutRow {
            key: shortcut.key.clone().into(),
            label: shortcut.label.clone().into(),
        })
        .collect::<Vec<_>>();
    let right = shortcut_rows.split_off((shortcut_rows.len() + 1) / 2);
    window.set_shortcuts_left(ModelRc::new(VecModel::from(shortcut_rows)));
    window.set_shortcuts_right(ModelRc::new(VecModel::from(right)));

    configure_overlay_window(&window);
    window.run()?;
    Ok(())
}

/// Keep the companion visible beside the smaller TF2 window without taking
/// mouse clicks or keyboard focus away from MIRV camera controls.
fn configure_overlay_window(window: &DirectorWindow) {
    let weak = window.as_weak();
    slint::Timer::single_shot(Duration::ZERO, move || {
        let Some(window) = weak.upgrade() else { return };
        let _ = window.window().with_winit_window(|native| {
            native.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
            native.set_decorations(false);
            native.set_resizable(false);
            let _ = native.set_cursor_hittest(false);

            let Some(monitor) = native
                .current_monitor()
                .or_else(|| native.available_monitors().next())
            else {
                return;
            };
            let monitor_position = monitor.position();
            let monitor_size = monitor.size();
            let overlay_size = native.outer_size();
            let x = monitor_position.x
                + monitor_size.width.saturating_sub(overlay_size.width + 14) as i32;
            let y = monitor_position.y + 14;
            native.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
        });
    });
}

fn load_session(path: &Path) -> Result<DirectorSession> {
    let bytes = fs::read(path)
        .with_context(|| format!("could not read Director session {}", path.display()))?;
    let session: DirectorSession = serde_json::from_slice(&bytes)
        .with_context(|| format!("could not parse Director session {}", path.display()))?;
    session.validate()?;
    Ok(session)
}
