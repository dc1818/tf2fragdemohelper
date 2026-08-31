#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use anyhow::{bail, Context, Result};
use slint::winit_030::{winit, WinitWindowAccessor};
use slint::{ComponentHandle, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    rc::Rc,
    thread,
    time::{Duration, Instant},
};
use tf2_mirv_director::{
    DirectorControl, DirectorSession, DIRECTOR_ACTION_ACK_PREFIX,
    DIRECTOR_ACTION_FILE_PREFIX, DIRECTOR_ACTION_SLOTS, DIRECTOR_KEYFRAME_BEGIN_PREFIX,
    DIRECTOR_KEYFRAME_END_PREFIX, DIRECTOR_TICK_OFFSET_PREFIX,
};

const ONE_SECOND_TICKS: i64 = 67;
const DEMO_READY_MARKER: &str = "TF2FRAG_MANUAL_PAUSED_AT_START";
const DEMO_RELOAD_MARKER: &str = "TF2FRAG_MANUAL_SAFE_RESTART_FROM_ZERO";
const DEMO_READY_TIMEOUT: Duration = Duration::from_secs(20 * 60);

slint::include_modules!();

fn main() -> Result<()> {
    let path = env::args_os()
        .nth(1)
        .context("usage: tf2-mirv-director <director_session.json>")?;
    let session_path = PathBuf::from(path);
    let session = Rc::new(load_session(&session_path)?);
    let telemetry_diagnostic = session_path.with_file_name("director_telemetry.log");
    let _ = fs::write(
        &telemetry_diagnostic,
        format!(
            "TF2 MIRV Director telemetry diagnostic\nsession={}\nconsole_log={}\nmarker={}\nstatus=WAITING_FOR_TF2_LOG\n",
            session_path.display(),
            session.telemetry_log.display(),
            session.telemetry_marker_prefix,
        ),
    );
    wait_for_demo_ready(
        &session.telemetry_log,
        &session.telemetry_marker_prefix,
        &telemetry_diagnostic,
    )?;
    append_telemetry_diagnostic(&telemetry_diagnostic, "status=DEMO_FULLY_LOADED_AND_PAUSED");

    let strip = DirectorStripWindow::new()?;
    let card = DirectorCardWindow::new()?;
    let demo_ready = Rc::new(Cell::new(true));
    let highlighted_cue = Rc::new(Cell::new(None::<usize>));
    let selected_keyframe = Rc::new(Cell::new(None::<(i32, i64)>));
    let last_tick = Rc::new(Cell::new(session.start_tick));
    let action_queue = Rc::new(RefCell::new(DirectorActionQueue::new(
        session.command_cfg_directory.clone(),
    )));

    let shortcut_key = |id: &str, fallback: &str| {
        session
            .shortcuts
            .iter()
            .find(|shortcut| shortcut.id == id)
            .map(|shortcut| shortcut.key.as_str())
            .unwrap_or(fallback)
            .to_owned()
    };
    let panel_toggle_key = shortcut_key("overlay_panel_toggle", "C");
    strip.set_start_tick(to_ui_tick(session.start_tick));
    strip.set_end_tick(to_ui_tick(session.end_tick));
    strip.set_panel_toggle_key(panel_toggle_key.clone().into());
    card.set_panel_toggle_key(panel_toggle_key.clone().into());
    card.set_record_order(
        format!(
            "{} FOLLOW CAMPATH  →  {} RESUME  →  {} RECORD  →  {} STOP",
            shortcut_key("play_campath", "8"),
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
            tick: to_ui_tick(cue.tick),
            position: session.cue_position(cue.tick),
            label: SharedString::from(if cue.label.is_empty() {
                format!("FRAG {}", index + 1)
            } else {
                cue.label.clone()
            }),
            tags: cue.tags.join(", ").into(),
            victims: cue.victims.join(", ").into(),
        })
        .collect::<Vec<_>>();
    strip.set_cues(ModelRc::new(VecModel::from(cue_rows)));
    let empty_keyframes = ModelRc::new(VecModel::from(Vec::<KeyframeRow>::new()));
    strip.set_keyframes(empty_keyframes.clone());
    card.set_keyframes(empty_keyframes);

    let shortcut_rows = session
        .shortcuts
        .iter()
        .map(|shortcut| ShortcutRow {
            key: shortcut.key.clone().into(),
            label: shortcut.label.clone().into(),
        })
        .collect::<Vec<_>>();
    card.set_shortcuts(ModelRc::new(VecModel::from(shortcut_rows)));

    strip.set_selected_cue_index(-1);
    strip.set_has_selected_keyframe(false);
    strip.set_actions_enabled(true);
    card.set_has_selected_keyframe(false);
    card.set_actions_enabled(true);
    update_playback_state(
        &strip,
        &card,
        &session,
        session.start_tick,
        false,
        highlighted_cue.get(),
    );
    strip.set_telemetry_status("WAIT".into());

    let panel_visible = Rc::new(Cell::new(true));
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let panel_visible = panel_visible.clone();
        let demo_ready = demo_ready.clone();
        card.on_hide_requested(move || {
            if !demo_ready.get() {
                return;
            }
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            set_panel_visibility(&strip, &card, &panel_visible, false);
        });
    }
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let selected_keyframe = selected_keyframe.clone();
        let panel_visible = panel_visible.clone();
        let demo_ready = demo_ready.clone();
        strip.on_keyframe_activated(move |id, tick| {
            if !demo_ready.get() {
                return;
            }
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            set_selected_keyframe(&strip, &card, &selected_keyframe, id, tick as i64);
            set_panel_visibility(&strip, &card, &panel_visible, true);
        });
    }
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let selected_keyframe = selected_keyframe.clone();
        let demo_ready = demo_ready.clone();
        card.on_keyframe_activated(move |id, tick| {
            if !demo_ready.get() {
                return;
            }
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            set_selected_keyframe(&strip, &card, &selected_keyframe, id, tick as i64);
        });
    }
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let session = session.clone();
        let highlighted_cue = highlighted_cue.clone();
        let last_tick = last_tick.clone();
        let panel_visible = panel_visible.clone();
        let demo_ready = demo_ready.clone();
        strip.on_cue_activated(move |index| {
            if !demo_ready.get() {
                return;
            }
            let Ok(index) = usize::try_from(index) else {
                return;
            };
            if index >= session.cues.len() {
                return;
            }
            highlighted_cue.set(Some(index));
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            strip.set_selected_cue_index(index.min(i32::MAX as usize) as i32);
            update_playback_state(
                &strip,
                &card,
                &session,
                last_tick.get(),
                true,
                Some(index),
            );
            set_panel_visibility(&strip, &card, &panel_visible, true);
        });
    }
    {
        let weak_card = card.as_weak();
        let session = session.clone();
        let highlighted_cue = highlighted_cue.clone();
        let last_tick = last_tick.clone();
        let action_queue = action_queue.clone();
        let demo_ready = demo_ready.clone();
        card.on_frag_goto_requested(move || {
            if !demo_ready.get() {
                return;
            }
            let Some(card) = weak_card.upgrade() else {
                return;
            };
            let Some(index) = displayed_cue_index(
                &session,
                last_tick.get(),
                highlighted_cue.get(),
            ) else {
                card.set_command_status("NO FRAG IS AVAILABLE TO SEEK".into());
                return;
            };
            let cue = &session.cues[index];
            let target = cue
                .tick
                .saturating_sub(ONE_SECOND_TICKS)
                .max(session.start_tick)
                .max(0);
            enqueue_director_action(
                &action_queue,
                &card,
                format!(
                    "demo_gototick {target}; echo {} {target}",
                    session.telemetry_marker_prefix
                ),
                format!("GO TO FRAG {} AT TICK {target}", index + 1),
            );
        });
    }
    {
        let weak_card = card.as_weak();
        let session = session.clone();
        let action_queue = action_queue.clone();
        let demo_ready = demo_ready.clone();
        card.on_keyframe_action_requested(move |id, action, argument| {
            let Some(card) = weak_card.upgrade() else {
                return;
            };
            if !demo_ready.get() {
                card.set_command_status("WAITING FOR DEMO TO FINISH LOADING".into());
                return;
            }
            let tick = card.get_selected_keyframe_tick() as i64;
            match build_keyframe_action(
                id,
                tick,
                action.as_str(),
                argument.as_str(),
                &session.telemetry_marker_prefix,
                session.start_tick,
            ) {
                Ok((command, label)) => {
                    enqueue_director_action(&action_queue, &card, command, label)
                }
                Err(error) => card.set_command_status(
                    format!("ACTION NOT SENT: {error}").to_ascii_uppercase().into(),
                ),
            }
        });
    }

    card.show()?;
    let docked_monitor = configure_overlay_windows(&strip, &card);

    // TF2 is a borderless top-level window. Reasserting HWND_TOPMOST keeps both
    // Director windows above it without activating either window.
    let topmost_timer = Timer::default();
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let panel_visible = panel_visible.clone();
        let docked_monitor = docked_monitor.clone();
        topmost_timer.start(TimerMode::Repeated, Duration::from_millis(250), move || {
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            refresh_overlay_dock(&strip, &card, &docked_monitor, false);
            let _ = strip.window().with_winit_window(force_topmost);
            if panel_visible.get() {
                let _ = card.window().with_winit_window(force_topmost);
            }
        });
    }

    // The configured key is polled globally on Windows, so it works while TF2
    // owns keyboard focus and still works after the card has been hidden.
    let hotkey_timer = Timer::default();
    #[cfg(target_os = "windows")]
    if let Some(virtual_key) = virtual_key_code(&panel_toggle_key) {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let panel_visible = panel_visible.clone();
        let demo_ready = demo_ready.clone();
        let was_down = Rc::new(Cell::new(false));
        hotkey_timer.start(TimerMode::Repeated, Duration::from_millis(30), move || {
            let down = global_key_is_down(virtual_key);
            if down && !was_down.replace(down) {
                if !demo_ready.get() {
                    return;
                }
                let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                    return;
                };
                let visible = !panel_visible.get();
                set_panel_visibility(&strip, &card, &panel_visible, visible);
            } else if !down {
                was_down.set(false);
            }
        });
    }

    let telemetry_timer = Timer::default();
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let session = session.clone();
        let highlighted_cue = highlighted_cue.clone();
        let selected_keyframe = selected_keyframe.clone();
        let last_tick = last_tick.clone();
        let action_queue = action_queue.clone();
        let demo_ready = demo_ready.clone();
        let mut tail = TickLogTail::new(session.telemetry_log.clone());
        let telemetry_diagnostic = telemetry_diagnostic.clone();
        let mut polls = 0_u32;
        let mut reported_log_open = false;
        let mut reported_missing_log = false;
        let mut reported_no_tick = false;
        let mut reported_first_tick = false;
        let mut reported_error = false;
        telemetry_timer.start(TimerMode::Repeated, Duration::from_millis(15), move || {
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            polls = polls.saturating_add(1);
            match tail.poll(&session.telemetry_marker_prefix) {
                Ok(poll) => {
                    if tail.is_open() && !reported_log_open {
                        reported_log_open = true;
                        reported_missing_log = false;
                        append_telemetry_diagnostic(
                            &telemetry_diagnostic,
                            "status=TF2_CONSOLE_LOG_OPENED",
                        );
                    }
                    if poll.demo_loading {
                        demo_ready.set(false);
                        set_demo_actions_enabled(&strip, &card, false);
                        strip.set_telemetry_status("LOADING".into());
                        card.set_command_status("WAITING FOR DEMO TO FINISH LOADING".into());
                        append_telemetry_diagnostic(
                            &telemetry_diagnostic,
                            "status=SAFE_RESTART_LOADING_ACTIONS_DISABLED",
                        );
                    }
                    if poll.demo_ready {
                        demo_ready.set(true);
                        set_demo_actions_enabled(&strip, &card, true);
                        strip.set_telemetry_status("LIVE".into());
                        card.set_command_status("DEMO LOADED — DIRECTOR ACTIONS READY".into());
                        append_telemetry_diagnostic(
                            &telemetry_diagnostic,
                            "status=DEMO_READY_ACTIONS_ENABLED",
                        );
                    }
                    if !demo_ready.get() {
                        return;
                    }
                    if poll.keyframes_invalidated {
                        clear_keyframe_snapshot(&strip, &card, &selected_keyframe);
                        card.set_command_status(
                            "KEYFRAME IDS CHANGED — WAITING FOR HLAE PRINT".into(),
                        );
                    }
                    if let Some(keys) = poll.keyframe_snapshot {
                        let rows = keys
                            .iter()
                            .filter(|key| {
                                key.tick >= session.start_tick && key.tick <= session.end_tick
                            })
                            .map(|key| KeyframeRow {
                                // HLAE prints and consumes this exact signed-int index.
                                // Do not replace it with an overlay-generated identifier.
                                id: key.id,
                                tick: to_ui_tick(key.tick),
                                position: session.cue_position(key.tick),
                            })
                            .collect::<Vec<_>>();
                        append_telemetry_diagnostic(
                            &telemetry_diagnostic,
                            &format!("status=KEYFRAMES_SYNCED visible_count={}", rows.len()),
                        );
                        let model = ModelRc::new(VecModel::from(rows));
                        strip.set_keyframes(model.clone());
                        card.set_keyframes(model);
                        if let Some((selected_id, _)) = selected_keyframe.get() {
                            if let Some(key) = keys.iter().find(|key| key.id == selected_id) {
                                set_selected_keyframe(
                                    &strip,
                                    &card,
                                    &selected_keyframe,
                                    key.id,
                                    key.tick,
                                );
                            } else {
                                clear_selected_keyframe(&strip, &card, &selected_keyframe);
                            }
                        }
                        card.set_command_status("AUTHORITATIVE HLAE IDS REFRESHED".into());
                    }
                    for sequence in poll.action_acks {
                        let status = action_queue.borrow_mut().acknowledge(sequence);
                        card.set_command_status(status.into());
                    }
                    if !poll.tick_updates.is_empty() {
                        for update in poll.tick_updates {
                            let updated_tick = match update {
                                TickUpdate::Absolute(tick) => tick,
                                TickUpdate::Relative(delta) => {
                                    let tick = last_tick.get().saturating_add(delta).max(0);
                                    append_telemetry_diagnostic(
                                        &telemetry_diagnostic,
                                        &format!(
                                            "status=BACKWARD_SEEK_RESYNC delta={delta} tick={tick}"
                                        ),
                                    );
                                    tick
                                }
                            };
                            last_tick.set(updated_tick);
                        }
                        if !reported_first_tick {
                            reported_first_tick = true;
                            append_telemetry_diagnostic(
                                &telemetry_diagnostic,
                                &format!("status=FIRST_TICK_RECEIVED tick={}", last_tick.get()),
                            );
                        }
                        strip.set_telemetry_status("LIVE".into());
                        update_playback_state(
                            &strip,
                            &card,
                            &session,
                            last_tick.get(),
                            true,
                            highlighted_cue.get(),
                        );
                    } else if polls >= 200 && !reported_first_tick {
                        if tail.is_open() {
                            strip.set_telemetry_status("NO TICK".into());
                            if !reported_no_tick {
                                reported_no_tick = true;
                                append_telemetry_diagnostic(
                                    &telemetry_diagnostic,
                                    "status=LOG_OPEN_BUT_NO_TICK_MARKER",
                                );
                            }
                        } else {
                            strip.set_telemetry_status("NO LOG".into());
                            if !reported_missing_log {
                                reported_missing_log = true;
                                append_telemetry_diagnostic(
                                    &telemetry_diagnostic,
                                    "status=TF2_CONSOLE_LOG_NOT_FOUND",
                                );
                            }
                        }
                    }
                }
                Err(error) => {
                    strip.set_telemetry_status("ERROR".into());
                    if !reported_error {
                        reported_error = true;
                        append_telemetry_diagnostic(
                            &telemetry_diagnostic,
                            &format!("status=READ_ERROR error={error}"),
                        );
                    }
                }
            }
        });
    }

    // Keep the timers alive for the complete Slint event loop.
    let _timer_lifetime = (&topmost_timer, &hotkey_timer, &telemetry_timer);
    strip.run()?;
    Ok(())
}

fn append_telemetry_diagnostic(path: &Path, message: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{message}");
    }
}

fn wait_for_demo_ready(log_path: &Path, prefix: &str, diagnostic_path: &Path) -> Result<()> {
    let started = Instant::now();
    let mut tail = TickLogTail::new(log_path.to_owned());
    loop {
        let poll = tail
            .poll(prefix)
            .with_context(|| format!("could not read TF2 readiness log {}", log_path.display()))?;
        if poll.demo_ready {
            return Ok(());
        }
        if started.elapsed() >= DEMO_READY_TIMEOUT {
            append_telemetry_diagnostic(diagnostic_path, "status=DEMO_READY_TIMEOUT");
            bail!("TF2 did not finish loading and pause within 20 minutes");
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn set_demo_actions_enabled(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    enabled: bool,
) {
    strip.set_actions_enabled(enabled);
    card.set_actions_enabled(enabled);
}

fn update_playback_state(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    session: &DirectorSession,
    tick: i64,
    telemetry_active: bool,
    highlighted_cue: Option<usize>,
) {
    strip.set_current_tick(to_ui_tick(tick));
    strip.set_current_position(session.cue_position(tick));
    strip.set_telemetry_active(
        telemetry_active || matches!(&session.control, DirectorControl::LocalBridge { .. }),
    );
    card.set_current_tick(to_ui_tick(tick));

    let next = session
        .cues
        .iter()
        .enumerate()
        .find(|(_, cue)| cue.tick >= tick);
    if let Some((_, cue)) = next {
        strip.set_next_frag_tick(to_ui_tick(cue.tick));
    } else {
        strip.set_next_frag_tick(0);
    }

    let displayed = highlighted_cue
        .and_then(|index| session.cues.get(index).map(|cue| (index, cue)))
        .or(next);
    if let Some((index, cue)) = displayed {
        card.set_frag_heading(
            if highlighted_cue.is_some() {
                "HIGHLIGHTED FRAG"
            } else {
                "NEXT FRAG"
            }
            .into(),
        );
        card.set_highlighted_frag(highlighted_cue.is_some());
        card.set_next_frag_number((index + 1).min(i32::MAX as usize) as i32);
        card.set_next_frag_tick(to_ui_tick(cue.tick));
        card.set_next_frag_victims(cue.victims.join(", ").into());
        card.set_next_frag_tags(cue.tags.join(", ").into());
    } else {
        card.set_frag_heading("NEXT FRAG".into());
        card.set_highlighted_frag(false);
        card.set_next_frag_number(0);
        card.set_next_frag_tick(0);
        card.set_next_frag_victims(SharedString::default());
        card.set_next_frag_tags("Clip outro".into());
    }
}

fn set_panel_visibility(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    state: &Cell<bool>,
    visible: bool,
) {
    state.set(visible);
    strip.set_panel_visible(visible);
    if visible {
        let _ = card.show();
        let _ = card.window().with_winit_window(force_topmost);
    } else {
        let _ = card.hide();
    }
}

fn set_selected_keyframe(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    state: &Cell<Option<(i32, i64)>>,
    id: i32,
    tick: i64,
) {
    state.set(Some((id, tick)));
    strip.set_has_selected_keyframe(true);
    strip.set_selected_keyframe_id(id);
    card.set_has_selected_keyframe(true);
    card.set_selected_keyframe_id(id);
    card.set_selected_keyframe_tick(to_ui_tick(tick));
}

fn clear_selected_keyframe(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    state: &Cell<Option<(i32, i64)>>,
) {
    state.set(None);
    strip.set_has_selected_keyframe(false);
    card.set_has_selected_keyframe(false);
}

fn clear_keyframe_snapshot(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    selected: &Cell<Option<(i32, i64)>>,
) {
    let empty = ModelRc::new(VecModel::from(Vec::<KeyframeRow>::new()));
    strip.set_keyframes(empty.clone());
    card.set_keyframes(empty);
    clear_selected_keyframe(strip, card, selected);
}

fn displayed_cue_index(
    session: &DirectorSession,
    tick: i64,
    highlighted: Option<usize>,
) -> Option<usize> {
    highlighted.filter(|index| *index < session.cues.len()).or_else(|| {
        session
            .cues
            .iter()
            .position(|cue| cue.tick >= tick)
    })
}

fn enqueue_director_action(
    queue: &RefCell<DirectorActionQueue>,
    card: &DirectorCardWindow,
    command: String,
    label: String,
) {
    match queue.borrow_mut().enqueue(command, label) {
        Ok(status) => card.set_command_status(status.into()),
        Err(error) => card.set_command_status(
            format!("DIRECTOR COMMAND FAILED: {error}")
                .to_ascii_uppercase()
                .into(),
        ),
    }
}

struct QueuedDirectorAction {
    command: String,
    label: String,
}

struct InFlightDirectorAction {
    sequence: u64,
    slot: u16,
    label: String,
}

struct DirectorActionQueue {
    directory: PathBuf,
    pending: VecDeque<QueuedDirectorAction>,
    in_flight: Option<InFlightDirectorAction>,
    next_slot: u16,
    next_sequence: u64,
}

impl DirectorActionQueue {
    fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            pending: VecDeque::new(),
            in_flight: None,
            next_slot: 0,
            next_sequence: 1,
        }
    }

    fn enqueue(&mut self, command: String, label: String) -> Result<String> {
        if command.chars().any(|character| matches!(character, '\r' | '\n')) {
            bail!("an internal command contained an unsafe line break");
        }
        self.pending
            .push_back(QueuedDirectorAction { command, label });
        if self.in_flight.is_none() {
            self.start_next()
        } else {
            Ok(format!(
                "QUEUED • {} ACTION(S) WAITING",
                self.pending.len()
            ))
        }
    }

    fn acknowledge(&mut self, sequence: u64) -> String {
        let Some(in_flight) = self.in_flight.take() else {
            return format!("IGNORED UNEXPECTED TF2 ACK {sequence}");
        };
        if in_flight.sequence != sequence {
            self.in_flight = Some(in_flight);
            return format!("WAITING FOR TF2 ACK {}", self.in_flight.as_ref().unwrap().sequence);
        }

        let consumed = self.action_path(in_flight.slot);
        let _ = replace_action_file(
            &consumed,
            b"// Director action consumed; waiting for this slot to be reused.\n",
        );
        self.next_slot = (in_flight.slot + 1) % DIRECTOR_ACTION_SLOTS;
        let completed = format!("COMPLETE • {}", in_flight.label);
        if self.pending.is_empty() {
            completed
        } else {
            match self.start_next() {
                Ok(status) => format!("{completed} • {status}"),
                Err(error) => format!("{completed} • NEXT ACTION FAILED: {error}"),
            }
        }
    }

    fn start_next(&mut self) -> Result<String> {
        let Some(action) = self.pending.pop_front() else {
            return Ok("DIRECTOR COMMAND QUEUE READY".into());
        };
        fs::create_dir_all(&self.directory).with_context(|| {
            format!(
                "could not access Director command directory {}",
                self.directory.display()
            )
        })?;
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        let slot = self.next_slot;
        let next_slot = (slot + 1) % DIRECTOR_ACTION_SLOTS;
        let contents = format!(
            "{}\ntf2frag_manual_sync_keyframes\necho {DIRECTOR_ACTION_ACK_PREFIX} {sequence}\nalias tf2frag_director_poll tf2frag_director_poll_{next_slot:02}\n",
            action.command
        );
        let path = self.action_path(slot);
        if let Err(error) = replace_action_file(&path, contents.as_bytes()) {
            self.pending.push_front(action);
            return Err(error).with_context(|| {
                format!("could not write Director action slot {}", path.display())
            });
        }
        let label = action.label;
        self.in_flight = Some(InFlightDirectorAction {
            sequence,
            slot,
            label: label.clone(),
        });
        Ok(format!("SENT TO TF2 • {label}"))
    }

    fn action_path(&self, slot: u16) -> PathBuf {
        self.directory
            .join(format!("{DIRECTOR_ACTION_FILE_PREFIX}_{slot:02}.cfg"))
    }
}

fn replace_action_file(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let temp = path.with_extension("cfg.tmp");
    fs::write(&temp, contents)?;
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    if let Err(error) = fs::rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    Ok(())
}

fn build_keyframe_action(
    id: i32,
    tick: i64,
    action: &str,
    argument: &str,
    tick_marker_prefix: &str,
    clip_start_tick: i64,
) -> Result<(String, String)> {
    if id < 0 {
        bail!("HLAE keyframe ID must be zero or greater");
    }
    let select = format!("mirv_campath select #{id} #{id}");
    let command = match action {
        "Go to keyframe" => format!(
            "demo_gototick {tick}; echo {tick_marker_prefix} {tick}"
        ),
        "Go to 1 sec before" => {
            let target = tick
                .saturating_sub(ONE_SECOND_TICKS)
                .max(clip_start_tick)
                .max(0);
            format!("demo_gototick {target}; echo {tick_marker_prefix} {target}")
        }
        "Select only" => select,
        "Add to selection" => format!("mirv_campath select add #{id} #{id}"),
        "Select range through ID" => {
            let end = normalized_nonnegative_id(argument)?;
            format!("mirv_campath select #{id} #{end}")
        }
        "Add range through ID" => {
            let end = normalized_nonnegative_id(argument)?;
            format!("mirv_campath select add #{id} #{end}")
        }
        "Delete keyframe" => format!("mirv_campath remove {id}"),
        "Move time to current" => format!("{select}; mirv_campath edit start"),
        "Shift time by seconds" => {
            let value = normalized_number(argument, "time shift")?;
            format!("{select}; mirv_campath edit start delta{value}")
        }
        "Set absolute time" => {
            let value = normalized_number(argument, "absolute time")?;
            format!("{select}; mirv_campath edit start abs {value}")
        }
        "Position from current camera" => {
            format!("{select}; mirv_campath edit position current")
        }
        "Set position X Y Z" => {
            let values = normalized_triplet(argument, true, "position")?;
            format!("{select}; mirv_campath edit position {values}")
        }
        "Angles from current camera" => {
            format!("{select}; mirv_campath edit angles current")
        }
        "Set angles P Y R" => {
            let values = normalized_triplet(argument, true, "angles")?;
            format!("{select}; mirv_campath edit angles {values}")
        }
        "FOV from current camera" => format!("{select}; mirv_campath edit fov current"),
        "Set FOV" => {
            let value = parse_finite_number(argument, "FOV")?;
            if !(1.0..=179.0).contains(&value) {
                bail!("FOV must be between 1 and 179");
            }
            format!("{select}; mirv_campath edit fov {}", format_number(value))
        }
        "Rotate P Y R" => {
            let values = normalized_triplet(argument, false, "rotation")?;
            format!("{select}; mirv_campath edit rotate {values}")
        }
        "Anchor to current camera" => format!(
            "mirv_campath select all; mirv_campath edit anchor #{id} current; mirv_campath select none"
        ),
        "Align path to keyframe" => format!("mirv_campath offset current#{id}"),
        "Align path with offset" => {
            let value = normalized_number(argument, "path offset")?;
            format!("mirv_campath offset current#{id}{value}")
        }
        "Set selected duration" => {
            let value = parse_finite_number(argument, "duration")?;
            if value <= 0.0 {
                bail!("duration must be greater than zero");
            }
            format!("mirv_campath edit duration {}", format_number(value))
        }
        "Interpolation position" => format!(
            "mirv_campath edit interp position {}",
            normalized_interpolation(argument, false)?
        ),
        "Interpolation rotation" => format!(
            "mirv_campath edit interp rotation {}",
            normalized_interpolation(argument, true)?
        ),
        "Interpolation FOV" => format!(
            "mirv_campath edit interp fov {}",
            normalized_interpolation(argument, false)?
        ),
        "Select all" => "mirv_campath select all".into(),
        "Deselect all" => "mirv_campath select none".into(),
        "Invert selection" => "mirv_campath select invert".into(),
        _ => bail!("unknown keyframe action '{action}'"),
    };
    Ok((command, format!("KEYFRAME {id} • {action}")))
}

fn normalized_nonnegative_id(value: &str) -> Result<i32> {
    let value = value.trim().trim_start_matches('#');
    let parsed = value
        .parse::<i32>()
        .with_context(|| "range end must be a non-negative HLAE ID")?;
    if parsed < 0 {
        bail!("range end must be a non-negative HLAE ID");
    }
    Ok(parsed)
}

fn parse_finite_number(value: &str, label: &str) -> Result<f64> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .with_context(|| format!("{label} requires a numeric value"))?;
    if !parsed.is_finite() {
        bail!("{label} must be finite");
    }
    Ok(parsed)
}

fn normalized_number(value: &str, label: &str) -> Result<String> {
    let parsed = parse_finite_number(value, label)?;
    let formatted = format_number(parsed.abs());
    Ok(if parsed < 0.0 {
        format!("-{formatted}")
    } else {
        format!("+{formatted}")
    })
}

fn normalized_triplet(value: &str, allow_star: bool, label: &str) -> Result<String> {
    let parts = value.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 {
        bail!("{label} requires exactly three space-separated values");
    }
    parts
        .into_iter()
        .map(|part| {
            if allow_star && part == "*" {
                Ok("*".to_owned())
            } else {
                parse_finite_number(part, label).map(format_number)
            }
        })
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.join(" "))
}

fn normalized_interpolation(value: &str, rotation: bool) -> Result<&'static str> {
    let value = value.trim().to_ascii_lowercase();
    match (rotation, value.as_str()) {
        (_, "default") => Ok("default"),
        (false, "linear") => Ok("linear"),
        (false, "cubic") => Ok("cubic"),
        (true, "slinear") => Ok("sLinear"),
        (true, "scubic") => Ok("sCubic"),
        (true, _) => bail!("rotation interpolation must be default, sLinear, or sCubic"),
        (false, _) => bail!("interpolation must be default, linear, or cubic"),
    }
}

fn format_number(value: f64) -> String {
    let mut formatted = format!("{value:.6}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    if formatted == "-0" {
        "0".into()
    } else {
        formatted
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MonitorGeometry {
    position: winit::dpi::PhysicalPosition<i32>,
    size: winit::dpi::PhysicalSize<u32>,
    scale: f64,
}

fn configure_overlay_windows(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
) -> Rc<Cell<Option<MonitorGeometry>>> {
    let weak_strip = strip.as_weak();
    let weak_card = card.as_weak();
    let docked_monitor = Rc::new(Cell::new(None));
    let initial_docked_monitor = docked_monitor.clone();
    Timer::single_shot(Duration::ZERO, move || {
        let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
            return;
        };

        let _ = strip.window().with_winit_window(|native| {
            native.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
            native.set_decorations(false);
            native.set_resizable(false);
            // The narrow strip accepts clicks only over its own top-of-screen
            // area so timeline markers can select their matching cue-card row.
            let _ = native.set_cursor_hittest(true);
            force_topmost(native);
        });

        let _ = card.window().with_winit_window(|native| {
            native.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
            native.set_decorations(false);
            native.set_resizable(false);
            let _ = native.set_cursor_hittest(true);
            force_topmost(native);
        });
        refresh_overlay_dock(&strip, &card, &initial_docked_monitor, true);
    });
    docked_monitor
}

fn refresh_overlay_dock(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    docked_monitor: &Cell<Option<MonitorGeometry>>,
    force: bool,
) {
    let detected = Cell::new(None);
    let _ = strip.window().with_winit_window(|native| {
        let monitor = native
            .current_monitor()
            .or_else(|| native.available_monitors().next());
        detected.set(monitor.map(|monitor| MonitorGeometry {
            position: monitor.position(),
            size: monitor.size(),
            scale: monitor.scale_factor(),
        }));
    });
    let Some(geometry) = detected.get() else {
        return;
    };
    if !force && docked_monitor.get() == Some(geometry) {
        return;
    }
    docked_monitor.set(Some(geometry));
    dock_overlay_windows(strip, card, geometry);
}

fn dock_overlay_windows(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    geometry: MonitorGeometry,
) {
    const STRIP_HEIGHT: f64 = 108.0;
    const CARD_WIDTH: f64 = 360.0;

    let logical_width = geometry.size.width as f64 / geometry.scale;
    let logical_height = geometry.size.height as f64 / geometry.scale;
    let card_width = CARD_WIDTH.min(logical_width);
    let card_height = (logical_height - STRIP_HEIGHT).max(1.0);
    strip.set_compact_layout(logical_width < 1000.0);
    strip.set_narrow_layout(logical_width < 800.0);

    let _ = strip.window().with_winit_window(|native| {
        let _ = native.request_inner_size(winit::dpi::LogicalSize::new(
            logical_width,
            STRIP_HEIGHT,
        ));
        native.set_outer_position(geometry.position);
        force_topmost(native);
    });
    let _ = card.window().with_winit_window(|native| {
        let _ = native.request_inner_size(winit::dpi::LogicalSize::new(card_width, card_height));
        let physical_card_width = (card_width * geometry.scale).round() as u32;
        let x = geometry.position.x
            + geometry.size.width.saturating_sub(physical_card_width) as i32;
        let y = geometry.position.y + (STRIP_HEIGHT * geometry.scale).round() as i32;
        native.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
        force_topmost(native);
    });
}

fn force_topmost(native: &winit::window::Window) {
    native.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
    #[cfg(target_os = "windows")]
    force_topmost_windows(native);
}

#[cfg(target_os = "windows")]
fn force_topmost_windows(native: &winit::window::Window) {
    use std::ffi::c_void;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_SHOWWINDOW: u32 = 0x0040;
    const HWND_TOPMOST: *mut c_void = -1_isize as *mut c_void;

    let Ok(handle) = native.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    unsafe {
        SetWindowPos(
            handle.hwnd.get() as *mut c_void,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
}

struct TickLogTail {
    path: PathBuf,
    file: Option<File>,
    offset: u64,
    carry: String,
    keyframe_capture: Option<Vec<ParsedCampathKey>>,
}

impl TickLogTail {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            file: None,
            offset: 0,
            carry: String::new(),
            keyframe_capture: None,
        }
    }

    fn is_open(&self) -> bool {
        self.file.is_some()
    }

    fn poll(&mut self, prefix: &str) -> std::io::Result<TelemetryPoll> {
        if self.file.is_none() {
            match File::open(&self.path) {
                Ok(file) => self.file = Some(file),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(TelemetryPoll::default())
                }
                Err(error) => return Err(error),
            }
        }
        let file = self.file.as_mut().expect("telemetry file opened");
        let length = file.metadata()?.len();
        if length < self.offset {
            self.offset = 0;
            self.carry.clear();
            self.keyframe_capture = None;
        }
        file.seek(SeekFrom::Start(self.offset))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        self.offset += bytes.len() as u64;
        if bytes.is_empty() {
            return Ok(TelemetryPoll::default());
        }

        let mut text = std::mem::take(&mut self.carry);
        text.push_str(&String::from_utf8_lossy(&bytes));
        let complete_length = text.rfind('\n').map(|index| index + 1).unwrap_or(0);
        let (complete, remainder) = text.split_at(complete_length);
        self.carry = remainder.to_owned();
        let mut poll = TelemetryPoll::default();
        for line in complete.lines() {
            if line_starts_demo_load(line) {
                poll.demo_loading = true;
            }
            if line_marks_demo_ready(line) {
                poll.demo_ready = true;
            }
            if let Some(update) = parse_tick_update(line, prefix) {
                poll.tick_updates.push(update);
            }
            if let Some(sequence) = parse_action_ack(line) {
                poll.action_acks.push(sequence);
            }
            if line.contains(DIRECTOR_KEYFRAME_BEGIN_PREFIX)
                || line.contains("passed? selected? id : tick[offset]")
            {
                self.keyframe_capture = Some(Vec::new());
                continue;
            }
            if line.trim_end().ends_with("----")
                || line.contains(DIRECTOR_KEYFRAME_END_PREFIX)
            {
                if let Some(snapshot) = self.keyframe_capture.take() {
                    poll.keyframe_snapshot = Some(snapshot);
                }
                continue;
            }
            if let (Some(capture), Some(key)) =
                (self.keyframe_capture.as_mut(), parse_campath_key(line))
            {
                capture.push(key);
                continue;
            }
            if is_campath_identity_mutation(line) {
                self.keyframe_capture = None;
                poll.keyframes_invalidated = true;
            }
        }
        Ok(poll)
    }
}

fn line_starts_demo_load(line: &str) -> bool {
    if line.contains(DEMO_RELOAD_MARKER) {
        return true;
    }
    let lower = line.to_ascii_lowercase();
    lower.contains("] playdemo ")
        || lower.contains("playing demo from ")
        || lower.contains("demo playback finished")
}

fn line_marks_demo_ready(line: &str) -> bool {
    line.contains(DEMO_READY_MARKER)
}

#[derive(Default)]
struct TelemetryPoll {
    demo_loading: bool,
    demo_ready: bool,
    tick_updates: Vec<TickUpdate>,
    keyframe_snapshot: Option<Vec<ParsedCampathKey>>,
    keyframes_invalidated: bool,
    action_acks: Vec<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ParsedCampathKey {
    id: i32,
    tick: i64,
    selected: bool,
}

fn is_campath_identity_mutation(line: &str) -> bool {
    let Some((_, command)) = line.split_once("] ") else {
        return false;
    };
    let command = command.trim();
    let lower = command.to_ascii_lowercase();
    lower == "mirv_campath add"
        || lower == "mirv_campath clear"
        || lower.starts_with("mirv_campath remove ")
        || lower.starts_with("mirv_campath load ")
        || lower.starts_with("mirv_campath edit start")
        || lower.starts_with("mirv_campath edit duration ")
        || lower.starts_with("mirv_campath offset ")
}

fn parse_action_ack(line: &str) -> Option<u64> {
    let marker = line.rfind(DIRECTOR_ACTION_ACK_PREFIX)?;
    line[marker + DIRECTOR_ACTION_ACK_PREFIX.len()..]
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

fn parse_campath_key(line: &str) -> Option<ParsedCampathKey> {
    let (left, right) = line.split_once(" : ")?;
    let mut fields = left.split_whitespace().rev();
    let id = fields.next()?.parse::<i32>().ok()?;
    let selected = fields.next()?.eq_ignore_ascii_case("Y");
    let tick_text = right.split(',').next()?.trim();
    let tick = parse_campath_tick(tick_text)?;
    Some(ParsedCampathKey { id, tick, selected })
}

fn parse_campath_tick(value: &str) -> Option<i64> {
    let split = value
        .char_indices()
        .skip(1)
        .find_map(|(index, character)| matches!(character, '+' | '-').then_some(index));
    match split {
        Some(index) => {
            let base = value[..index].trim().parse::<i64>().ok()?;
            let offset = value[index..].trim().parse::<i64>().ok()?;
            Some(base.saturating_add(offset))
        }
        None => value.trim().parse::<i64>().ok(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TickUpdate {
    Absolute(i64),
    Relative(i64),
}

fn parse_tick_update(line: &str, prefix: &str) -> Option<TickUpdate> {
    if let Some(tick) = parse_tick_marker(line, prefix) {
        return Some(TickUpdate::Absolute(tick));
    }
    if let Some(marker) = line.rfind("Current tick: ") {
        let tick = line[marker + "Current tick: ".len()..]
            .split_whitespace()
            .next()?
            .trim_end_matches(',')
            .parse::<i64>()
            .ok()?;
        return Some(TickUpdate::Absolute(tick));
    }
    let marker = line.rfind(DIRECTOR_TICK_OFFSET_PREFIX)?;
    let delta = line[marker + DIRECTOR_TICK_OFFSET_PREFIX.len()..]
        .split_whitespace()
        .next()?
        .parse::<i64>()
        .ok()?;
    Some(TickUpdate::Relative(delta))
}

fn parse_tick_marker(line: &str, prefix: &str) -> Option<i64> {
    let marker = line.rfind(prefix)?;
    let tick = line[marker + prefix.len()..]
        .split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()?;
    tick.is_finite().then(|| tick.round() as i64)
}

fn to_ui_tick(tick: i64) -> i32 {
    tick.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

#[cfg(target_os = "windows")]
fn virtual_key_code(key: &str) -> Option<i32> {
    let key = key.trim().to_ascii_uppercase();
    if key.len() == 1 {
        let byte = key.as_bytes()[0];
        if byte.is_ascii_alphanumeric() {
            return Some(byte as i32);
        }
        return Some(match byte {
            b' ' => 0x20,
            b'[' => 0xDB,
            b']' => 0xDD,
            b'-' | b'_' => 0xBD,
            b'=' => 0xBB,
            b',' => 0xBC,
            b'.' => 0xBE,
            b'/' => 0xBF,
            b'\'' => 0xDE,
            _ => return None,
        });
    }
    match key.as_str() {
        "SPACE" => Some(0x20),
        "TAB" => Some(0x09),
        "ENTER" => Some(0x0D),
        "BACKSPACE" => Some(0x08),
        "DEL" => Some(0x2E),
        "INS" => Some(0x2D),
        "HOME" => Some(0x24),
        "END" => Some(0x23),
        "PGUP" => Some(0x21),
        "PGDN" => Some(0x22),
        "SCROLLLOCK" => Some(0x91),
        "PAUSE" => Some(0x13),
        value if value.starts_with('F') => value[1..]
            .parse::<i32>()
            .ok()
            .filter(|number| (1..=12).contains(number))
            .map(|number| 0x6F + number),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn global_key_is_down(virtual_key: i32) -> bool {
    unsafe { GetAsyncKeyState(virtual_key) < 0 }
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(virtual_key: i32) -> i16;
    fn SetWindowPos(
        window: *mut std::ffi::c_void,
        insert_after: *mut std::ffi::c_void,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> i32;
}

fn load_session(path: &Path) -> Result<DirectorSession> {
    let bytes = fs::read(path)
        .with_context(|| format!("could not read Director session {}", path.display()))?;
    let session: DirectorSession = serde_json::from_slice(&bytes)
        .with_context(|| format!("could not parse Director session {}", path.display()))?;
    session.validate()?;
    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_timestamped_tick_markers() {
        assert_eq!(
            parse_tick_marker(
                "08/30 12:30:22 TF2FRAG_DIRECTOR_TICK 88476",
                "TF2FRAG_DIRECTOR_TICK"
            ),
            Some(88_476)
        );
        assert_eq!(
            parse_tick_marker(
                "08/30 12:30:22 TF2FRAG_DIRECTOR_TICK 88476.000000",
                "TF2FRAG_DIRECTOR_TICK"
            ),
            Some(88_476)
        );
        assert_eq!(parse_tick_marker("unrelated", "TF2FRAG_DIRECTOR_TICK"), None);
        assert_eq!(
            parse_tick_update(
                "TF2FRAG_DIRECTOR_TICK_OFFSET -67",
                "TF2FRAG_DIRECTOR_TICK"
            ),
            Some(TickUpdate::Relative(-67))
        );
        assert_eq!(
            parse_tick_update(
                "TF2FRAG_DIRECTOR_TICK 12000",
                "TF2FRAG_DIRECTOR_TICK"
            ),
            Some(TickUpdate::Absolute(12_000))
        );
        assert_eq!(
            parse_tick_update(
                "08/30 12:30:22 Current tick: 11933, Current demoTime: 179.0",
                "TF2FRAG_DIRECTOR_TICK"
            ),
            Some(TickUpdate::Absolute(11_933))
        );
    }

    #[test]
    fn detects_demo_loading_and_authoritative_ready_markers() {
        assert!(line_starts_demo_load(
            "08/30 12:30:22 TF2FRAG_MANUAL_SAFE_RESTART_FROM_ZERO TARGET 12000"
        ));
        assert!(line_starts_demo_load(
            "08/30 12:30:22 ] playdemo demos/test.dem"
        ));
        assert!(line_starts_demo_load(
            "08/30 12:30:22 Playing demo from demos/test.dem."
        ));
        assert!(!line_marks_demo_ready("Current tick: 12000"));
        assert!(line_marks_demo_ready(
            "08/30 12:30:22 TF2FRAG_MANUAL_PAUSED_AT_START"
        ));
    }

    #[test]
    fn parses_hlae_campath_key_ids_and_ticks() {
        assert_eq!(
            parse_campath_key("Y n 2 : 12345 , 185.175 , 185.175 -> ( 1 2 3 )"),
            Some(ParsedCampathKey {
                id: 2,
                tick: 12_345,
                selected: false,
            })
        );
        assert_eq!(
            parse_campath_key("08/30 12:30:22 Y Y 0 : 12345-10 , 185.175"),
            Some(ParsedCampathKey {
                id: 0,
                tick: 12_335,
                selected: true,
            })
        );
        assert_eq!(parse_campath_tick("12345+10"), Some(12_355));
    }

    #[test]
    fn invalidates_inferred_ids_after_direct_console_mutations() {
        assert!(is_campath_identity_mutation(
            "08/30 12:30:22 ] mirv_campath remove 2"
        ));
        assert!(is_campath_identity_mutation(
            "08/30 12:30:22 ] mirv_campath clear"
        ));
        assert!(is_campath_identity_mutation(
            "08/30 12:30:22 ] mirv_campath edit start delta-1"
        ));
        assert!(!is_campath_identity_mutation(
            "08/30 12:30:22 ] mirv_campath print"
        ));
    }

    #[test]
    fn parses_director_action_acknowledgements() {
        assert_eq!(
            parse_action_ack("08/30 12:30:22 TF2FRAG_DIRECTOR_ACTION_ACK 42"),
            Some(42)
        );
    }

    #[test]
    fn builds_exact_hlae_id_actions_without_persistent_ids() {
        let (delete, _) = build_keyframe_action(
            4,
            12_345,
            "Delete keyframe",
            "",
            "TF2FRAG_DIRECTOR_TICK",
            10_000,
        )
        .unwrap();
        assert_eq!(delete, "mirv_campath remove 4");

        let (position, _) = build_keyframe_action(
            4,
            12_345,
            "Position from current camera",
            "",
            "TF2FRAG_DIRECTOR_TICK",
            10_000,
        )
        .unwrap();
        assert_eq!(
            position,
            "mirv_campath select #4 #4; mirv_campath edit position current"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn maps_default_overlay_hotkey() {
        assert_eq!(virtual_key_code("C"), Some(0x43));
    }
}
