#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use anyhow::{Context, Result};
use slint::{ModelRc, SharedString, VecModel};
use std::{env, fs, path::Path};
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
    window.run()?;
    Ok(())
}

fn load_session(path: &Path) -> Result<DirectorSession> {
    let bytes = fs::read(path)
        .with_context(|| format!("could not read Director session {}", path.display()))?;
    let session: DirectorSession = serde_json::from_slice(&bytes)
        .with_context(|| format!("could not parse Director session {}", path.display()))?;
    session.validate()?;
    Ok(session)
}
