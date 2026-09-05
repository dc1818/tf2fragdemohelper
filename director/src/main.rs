#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod rcon;

use anyhow::{bail, Context, Result};
use rcon::CommandDelivery as RconDelivery;
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
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};
use tf2_mirv_director::{
    DirectorControl, DirectorSession, DIRECTOR_ACTION_ACK_PREFIX, DIRECTOR_ACTION_FILE_PREFIX,
    DIRECTOR_ACTION_SLOTS, DIRECTOR_KEYFRAME_BEGIN_PREFIX, DIRECTOR_KEYFRAME_DIRTY_MARKER,
    DIRECTOR_KEYFRAME_END_PREFIX, DIRECTOR_LOAD_CAMPATH_REQUEST_MARKER,
    DIRECTOR_POLL_READY_MARKER, DIRECTOR_POLL_UNAVAILABLE_MARKER, DIRECTOR_TICK_OFFSET_PREFIX,
};

const ONE_SECOND_TICKS: i64 = 67;
const DEMO_READY_MARKER: &str = "TF2FRAG_MANUAL_PAUSED_AT_START";
const DEMO_RELOAD_MARKER: &str = "TF2FRAG_MANUAL_SAFE_RESTART_FROM_ZERO";
const DEMO_READY_TIMEOUT: Duration = Duration::from_secs(20 * 60);
const DIRECT_CONTROL_READY_TIMEOUT: Duration = Duration::from_secs(15);
const DIRECTOR_ACTION_ACK_TIMEOUT: Duration = Duration::from_secs(8);
const RCON_RETRY_DELAY: Duration = Duration::from_millis(250);
const MAX_CAMPATH_XML_BYTES: u64 = 16 * 1024 * 1024;
const MAX_CAMPATH_POINTS: usize = 100_000;

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
            "TF2 MIRV Director telemetry diagnostic\nsession={}\nconsole_log={}\nmarker={}\ncontrol={:?}\nstatus=WAITING_FOR_TF2_LOG\n",
            session_path.display(),
            session.telemetry_log.display(),
            session.telemetry_marker_prefix,
            session.control,
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

    let shortcut_key = |id: &str, fallback: &str| {
        session
            .shortcuts
            .iter()
            .find(|shortcut| shortcut.id == id)
            .map(|shortcut| shortcut.key.as_str())
            .unwrap_or(fallback)
            .to_owned()
    };
    let execute_action_key = shortcut_key("execute_director_action", "'");
    let automatic_cfg_mailbox = matches!(&session.control, DirectorControl::CfgMailbox);
    let action_queue = Rc::new(RefCell::new(DirectorActionQueue::new(
        session.command_cfg_directory.clone(),
        execute_action_key.clone(),
        automatic_cfg_mailbox,
    )));
    let (direct_action_sender, direct_action_results) = match &session.control {
        DirectorControl::LocalRcon { endpoint, password } => {
            let (sender, results) = spawn_direct_action_worker(endpoint.clone(), password.clone());
            (Some(sender), Some(results))
        }
        _ => (None, None),
    };
    let panel_toggle_key = shortcut_key("overlay_panel_toggle", "C");
    let interaction_toggle_key = shortcut_key("overlay_interaction_toggle", "F11");
    let load_campath_key = shortcut_key("load_campath", "F8");
    strip.set_start_tick(to_ui_tick(session.start_tick));
    strip.set_end_tick(to_ui_tick(session.end_tick));
    strip.set_panel_toggle_key(panel_toggle_key.clone().into());
    card.set_panel_toggle_key(panel_toggle_key.clone().into());
    strip.set_interaction_toggle_key(interaction_toggle_key.clone().into());
    card.set_interaction_toggle_key(interaction_toggle_key.clone().into());
    card.set_load_campath_key(load_campath_key.clone().into());
    strip.set_interaction_mode(false);
    card.set_interaction_mode(false);
    let initial_command_status = if automatic_cfg_mailbox {
        SharedString::from("CHECKING TF2 COMMAND QUEUE • CLICK AN ACTION")
    } else if direct_action_sender.is_some() {
        SharedString::from("DIRECT CONTROL LISTENING • CLICK AN ACTION")
    } else {
        SharedString::from(format!(
            "READY • CLICK AN ACTION, THEN PRESS {execute_action_key} IN TF2"
        ))
    };
    card.set_command_status(initial_command_status);
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
    let interaction_mode = Rc::new(Cell::new(false));
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let panel_visible = panel_visible.clone();
        let interaction_mode = interaction_mode.clone();
        let selected_keyframe = selected_keyframe.clone();
        let demo_ready = demo_ready.clone();
        card.on_hide_requested(move || {
            if !demo_ready.get() {
                return;
            }
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            if interaction_mode.get() {
                toggle_overlay_interaction(&strip, &card, &interaction_mode);
            }
            clear_selected_keyframe(&strip, &card, &selected_keyframe);
            set_panel_visibility(&strip, &card, &panel_visible, false);
        });
    }
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let interaction_mode = interaction_mode.clone();
        let selected_keyframe = selected_keyframe.clone();
        card.on_interaction_toggle_requested(move || {
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            clear_selected_keyframe(&strip, &card, &selected_keyframe);
            toggle_overlay_interaction(&strip, &card, &interaction_mode);
        });
    }
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let selected_keyframe = selected_keyframe.clone();
        strip.on_dismiss_selection_requested(move || {
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            clear_selected_keyframe(&strip, &card, &selected_keyframe);
        });
    }
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let selected_keyframe = selected_keyframe.clone();
        card.on_dismiss_selection_requested(move || {
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            clear_selected_keyframe(&strip, &card, &selected_keyframe);
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
        let selected_keyframe = selected_keyframe.clone();
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
            clear_selected_keyframe(&strip, &card, &selected_keyframe);
            strip.set_selected_cue_index(index.min(i32::MAX as usize) as i32);
            update_playback_state(&strip, &card, &session, last_tick.get(), true, Some(index));
            set_panel_visibility(&strip, &card, &panel_visible, true);
        });
    }
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let session = session.clone();
        let highlighted_cue = highlighted_cue.clone();
        let last_tick = last_tick.clone();
        let action_queue = action_queue.clone();
        let direct_action_sender = direct_action_sender.clone();
        let selected_keyframe = selected_keyframe.clone();
        let demo_ready = demo_ready.clone();
        card.on_frag_goto_requested(move || {
            if !demo_ready.get() {
                return;
            }
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            clear_selected_keyframe(&strip, &card, &selected_keyframe);
            let Some(index) = displayed_cue_index(&session, last_tick.get(), highlighted_cue.get())
            else {
                card.set_command_status("NO FRAG IS AVAILABLE TO SEEK".into());
                return;
            };
            let cue = &session.cues[index];
            let target = cue
                .tick
                .saturating_sub(ONE_SECOND_TICKS)
                .max(session.start_tick)
                .max(0);
            dispatch_director_action(
                direct_action_sender.as_ref(),
                &action_queue,
                &card,
                format!(
                    "demo_gototick {target} 0 1; echo {} {target}",
                    session.telemetry_marker_prefix
                ),
                format!("GO TO FRAG {} AT TICK {target}", index + 1),
            );
        });
    }
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let session = session.clone();
        let action_queue = action_queue.clone();
        let direct_action_sender = direct_action_sender.clone();
        let selected_keyframe = selected_keyframe.clone();
        let demo_ready = demo_ready.clone();
        card.on_keyframe_action_requested(move |id, action, setting, value| {
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
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
                setting.as_str(),
                value.as_str(),
                &session.telemetry_marker_prefix,
                session.start_tick,
            ) {
                Ok((mut command, label)) => {
                    if action.as_str() == "Delete keyframe"
                        || action.as_str() == "Edit keyframe"
                    {
                        command.push_str(&format!("; echo {DIRECTOR_KEYFRAME_DIRTY_MARKER}"));
                    }
                    dispatch_director_action(
                        direct_action_sender.as_ref(),
                        &action_queue,
                        &card,
                        command,
                        label,
                    );
                    clear_selected_keyframe(&strip, &card, &selected_keyframe);
                }
                Err(error) => card.set_command_status(
                    format!("ACTION NOT SENT: {error}")
                        .to_ascii_uppercase()
                        .into(),
                ),
            }
        });
    }
    {
        let weak_card = card.as_weak();
        let action_queue = action_queue.clone();
        let direct_action_sender = direct_action_sender.clone();
        card.on_load_campath_requested(move || {
            let Some(card) = weak_card.upgrade() else {
                return;
            };
            choose_and_load_campath(direct_action_sender.as_ref(), &action_queue, &card);
        });
    }
    {
        let weak_card = card.as_weak();
        let action_queue = action_queue.clone();
        let direct_action_sender = direct_action_sender.clone();
        card.on_quit_tf2_requested(move || {
            let Some(card) = weak_card.upgrade() else {
                return;
            };
            dispatch_director_action(
                direct_action_sender.as_ref(),
                &action_queue,
                &card,
                "quit".into(),
                "CLOSE TF2".into(),
            );
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

    // A separate global shortcut temporarily gives the overlay normal focus so
    // TF2 releases its captured mouse. Pressing it again returns focus to TF2.
    let interaction_hotkey_timer = Timer::default();
    #[cfg(target_os = "windows")]
    if let Some(virtual_key) = virtual_key_code(&interaction_toggle_key) {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let panel_visible = panel_visible.clone();
        let interaction_mode = interaction_mode.clone();
        let was_down = Rc::new(Cell::new(false));
        interaction_hotkey_timer.start(
            TimerMode::Repeated,
            Duration::from_millis(30),
            move || {
                let down = global_key_is_down(virtual_key);
                if down && !was_down.replace(down) {
                    let (Some(strip), Some(card)) =
                        (weak_strip.upgrade(), weak_card.upgrade())
                    else {
                        return;
                    };
                    if !panel_visible.get() {
                        set_panel_visibility(&strip, &card, &panel_visible, true);
                    }
                    toggle_overlay_interaction(&strip, &card, &interaction_mode);
                } else if !down {
                    was_down.set(false);
                }
            },
        );
    }

    // Dismiss the editor when the user clicks back into TF2. WindowFromPoint
    // distinguishes a real game click from a click on either non-activating
    // overlay window, even while TF2 remains the foreground process.
    let selection_dismiss_timer = Timer::default();
    #[cfg(target_os = "windows")]
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let selected_keyframe = selected_keyframe.clone();
        let was_down = Rc::new(Cell::new(false));
        selection_dismiss_timer.start(
            TimerMode::Repeated,
            Duration::from_millis(30),
            move || {
                let down = [0x01, 0x02, 0x04]
                    .into_iter()
                    .any(global_key_is_down);
                if down && !was_down.replace(down) && cursor_is_over_tf2() {
                    let (Some(strip), Some(card)) =
                        (weak_strip.upgrade(), weak_card.upgrade())
                    else {
                        return;
                    };
                    clear_selected_keyframe(&strip, &card, &selected_keyframe);
                } else if !down {
                    was_down.set(false);
                }
            },
        );
    }

    let direct_action_timer = Timer::default();
    if let Some(results) = direct_action_results {
        let weak_card = card.as_weak();
        direct_action_timer.start(TimerMode::Repeated, Duration::from_millis(20), move || {
            let Some(card) = weak_card.upgrade() else {
                return;
            };
            while let Ok(result) = results.try_recv() {
                match result {
                    DirectActionResult::Confirmed { label } => {
                        card.set_command_status(format!("COMPLETE • {label}").into());
                    }
                    DirectActionResult::SentUnconfirmed { label, reason } => {
                        card.set_command_status(
                            format!("SENT • {label} • CONFIRMATION LOST: {reason}")
                                .to_ascii_uppercase()
                                .into(),
                        );
                    }
                    DirectActionResult::Unavailable { label, reason } => {
                        card.set_command_status(
                            format!(
                                "NOT SENT • {label} • TF2 DIRECT CONTROL UNAVAILABLE: {reason}"
                            )
                            .to_ascii_uppercase()
                            .into(),
                        );
                    }
                }
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
        let direct_action_sender = direct_action_sender.clone();
        let demo_ready = demo_ready.clone();
        let mut tail = TickLogTail::new(session.telemetry_log.clone());
        let telemetry_diagnostic = telemetry_diagnostic.clone();
        let mut polls = 0_u32;
        let mut reported_log_open = false;
        let mut reported_missing_log = false;
        let mut reported_no_tick = false;
        let mut reported_first_tick = false;
        let mut reported_error = false;
        let mut keyframe_refresh_due = None::<Instant>;
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
                        clear_selected_keyframe(&strip, &card, &selected_keyframe);
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
                    if poll.director_poll_ready {
                        let status = action_queue.borrow_mut().set_poll_available(true);
                        card.set_command_status(status.into());
                        append_telemetry_diagnostic(
                            &telemetry_diagnostic,
                            "status=TF2_CFG_POLLER_READY",
                        );
                    }
                    if poll.director_poll_unavailable {
                        let status = action_queue.borrow_mut().set_poll_available(false);
                        card.set_command_status(status.into());
                        append_telemetry_diagnostic(
                            &telemetry_diagnostic,
                            "status=TF2_CFG_POLLER_UNAVAILABLE_INPUT_FALLBACK_ENABLED",
                        );
                    }
                    if !demo_ready.get() {
                        return;
                    }
                    if poll.load_campath_requested {
                        append_telemetry_diagnostic(
                            &telemetry_diagnostic,
                            "status=LOAD_CAMPATH_FILE_PICKER_REQUESTED",
                        );
                        choose_and_load_campath(
                            direct_action_sender.as_ref(),
                            &action_queue,
                            &card,
                        );
                    }
                    if poll.keyframes_refreshing {
                        // Keep the last complete HLAE table visible while the next
                        // print is still arriving. Clearing it here caused every
                        // mutation/seek to flash an empty timeline.
                        card.set_command_status(
                            "KEYFRAMES REFRESHING — KEEPING LAST VERIFIED MARKERS".into(),
                        );
                        // HLAE can commit an add/remove on the following frame,
                        // so the immediate print in the same TF2 command buffer
                        // can still contain the previous table. Re-query once
                        // the mutation has settled, including while paused.
                        keyframe_refresh_due = Some(Instant::now() + Duration::from_millis(150));
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
                    if let Some(status) = action_queue.borrow_mut().take_timeout_status() {
                        card.set_command_status(status.into());
                    }
                    if keyframe_refresh_due.is_some_and(|due| Instant::now() >= due) {
                        keyframe_refresh_due = None;
                        dispatch_director_action(
                            direct_action_sender.as_ref(),
                            &action_queue,
                            &card,
                            "tf2frag_manual_sync_keyframes".into(),
                            "REFRESH KEYFRAME MARKERS".into(),
                        );
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
    let _timer_lifetime = (
        &topmost_timer,
        &hotkey_timer,
        &interaction_hotkey_timer,
        &selection_dismiss_timer,
        &direct_action_timer,
        &telemetry_timer,
    );
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

fn set_demo_actions_enabled(strip: &DirectorStripWindow, card: &DirectorCardWindow, enabled: bool) {
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
        telemetry_active
            || matches!(
                &session.control,
                DirectorControl::LocalRcon { .. } | DirectorControl::CfgMailbox
            ),
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

fn toggle_overlay_interaction(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    state: &Cell<bool>,
) {
    let enabled = !state.get();
    state.set(enabled);
    strip.set_interaction_mode(enabled);
    card.set_interaction_mode(enabled);
    set_overlay_interaction_windows(strip, card, enabled);
    card.set_command_status(
        if enabled {
            "DIRECTOR CLICK MODE — PRESS THE FOCUS SHORTCUT AGAIN TO RETURN TO TF2"
        } else {
            "TF2 FOCUS RESTORED — DIRECTOR ACTIONS READY"
        }
        .into(),
    );
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

fn displayed_cue_index(
    session: &DirectorSession,
    tick: i64,
    highlighted: Option<usize>,
) -> Option<usize> {
    highlighted
        .filter(|index| *index < session.cues.len())
        .or_else(|| session.cues.iter().position(|cue| cue.tick >= tick))
}

fn dispatch_director_action(
    direct: Option<&Sender<DirectActionRequest>>,
    queue: &RefCell<DirectorActionQueue>,
    card: &DirectorCardWindow,
    command: String,
    label: String,
) {
    if let Some(direct) = direct {
        let status_label = label.clone();
        match direct.send(DirectActionRequest { command, label }) {
            Ok(()) => {
                card.set_command_status(format!("SENDING • {status_label}").into());
                return;
            }
            Err(error) => {
                let request = error.0;
                card.set_command_status(
                    format!(
                        "NOT SENT • {} • DIRECT CONTROL WORKER STOPPED",
                        request.label
                    )
                    .to_ascii_uppercase()
                    .into(),
                );
                return;
            }
        }
    }
    match queue.borrow_mut().enqueue(command, label) {
        Ok(status) => card.set_command_status(status.into()),
        Err(error) => card.set_command_status(
            format!("DIRECTOR COMMAND FAILED: {error}")
                .to_ascii_uppercase()
                .into(),
        ),
    }
}

fn choose_and_load_campath(
    direct: Option<&Sender<DirectActionRequest>>,
    queue: &RefCell<DirectorActionQueue>,
    card: &DirectorCardWindow,
) {
    let Some(path) = rfd::FileDialog::new()
        .set_title("Load an HLAE campath XML")
        .add_filter("HLAE campath XML", &["xml"])
        .pick_file()
    else {
        card.set_command_status("CAMPATH LOAD CANCELED".into());
        return;
    };

    match validated_campath_load_command(&path) {
        Ok((command, point_count)) => dispatch_director_action(
            direct,
            queue,
            card,
            command,
            format!("LOAD CAMPATH • {point_count} KEYFRAME(S)"),
        ),
        Err(error) => {
            let message = format!("The selected file is not a valid HLAE campath XML.\n\n{error:#}");
            card.set_command_status("CAMPATH NOT LOADED • INVALID XML".into());
            rfd::MessageDialog::new()
                .set_title("Invalid HLAE campath XML")
                .set_description(&message)
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        }
    }
}

fn validated_campath_load_command(path: &Path) -> Result<(String, usize)> {
    let point_count = validate_campath_xml(path)?;
    if !path.is_absolute() {
        bail!("the selected campath path is not absolute");
    }
    let path_text = path
        .to_str()
        .context("the campath path is not valid Unicode")?;
    if path_text
        .chars()
        .any(|character| character.is_control() || matches!(character, '"' | ';'))
    {
        bail!("the campath path contains characters that are unsafe in a TF2 command");
    }
    let command_path = path_text.replace('\\', "/");
    Ok((
        format!(
            "mirv_input end; mirv_campath load \"{command_path}\"; thirdperson; r_drawviewmodel 0; mirv_campath enabled 1; echo {DIRECTOR_KEYFRAME_DIRTY_MARKER}"
        ),
        point_count,
    ))
}

fn validate_campath_xml(path: &Path) -> Result<usize> {
    if !path.is_file() {
        bail!("the selected path is not a file");
    }
    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
    {
        bail!("the file must use the .xml extension");
    }
    let metadata = fs::metadata(path)
        .with_context(|| format!("could not inspect {}", path.display()))?;
    if metadata.len() == 0 {
        bail!("the XML file is empty");
    }
    if metadata.len() > MAX_CAMPATH_XML_BYTES {
        bail!("the XML file is larger than 16 MiB");
    }
    let contents = fs::read_to_string(path)
        .with_context(|| format!("could not read {} as UTF-8 XML", path.display()))?;
    let document = roxmltree::Document::parse(&contents).context("the XML is not well formed")?;
    let root = document.root_element();
    if root.tag_name().name() != "campath" {
        bail!("the root element must be <campath>");
    }
    let points = root
        .children()
        .find(|node| node.is_element() && node.tag_name().name() == "points")
        .context("the <campath> element does not contain <points>")?;

    let mut point_count = 0_usize;
    for point in points
        .children()
        .filter(|node| node.is_element() && node.tag_name().name() == "p")
    {
        point_count += 1;
        if point_count > MAX_CAMPATH_POINTS {
            bail!("the campath contains more than {MAX_CAMPATH_POINTS} points");
        }
        for attribute in ["t", "x", "y", "z", "fov"] {
            let value = point.attribute(attribute).with_context(|| {
                format!("campath point {point_count} is missing '{attribute}'")
            })?;
            let number = value.parse::<f64>().with_context(|| {
                format!("campath point {point_count} has an invalid '{attribute}' value")
            })?;
            if !number.is_finite() {
                bail!("campath point {point_count} has a non-finite '{attribute}' value");
            }
        }

        let euler = ["rx", "ry", "rz"];
        let quaternion = ["qw", "qx", "qy", "qz"];
        let euler_count = euler
            .iter()
            .filter(|attribute| point.attribute(**attribute).is_some())
            .count();
        let quaternion_count = quaternion
            .iter()
            .filter(|attribute| point.attribute(**attribute).is_some())
            .count();
        if !matches!(euler_count, 0 | 3) || !matches!(quaternion_count, 0 | 4) {
            bail!("campath point {point_count} has an incomplete rotation");
        }
        if euler_count == 0 && quaternion_count == 0 {
            bail!("campath point {point_count} has no Euler or quaternion rotation");
        }
        for attribute in euler.into_iter().chain(quaternion) {
            let Some(value) = point.attribute(attribute) else {
                continue;
            };
            let number = value.parse::<f64>().with_context(|| {
                format!("campath point {point_count} has an invalid '{attribute}' value")
            })?;
            if !number.is_finite() {
                bail!("campath point {point_count} has a non-finite '{attribute}' value");
            }
        }
    }
    if point_count == 0 {
        bail!("the campath does not contain any <p> keyframes");
    }
    Ok(point_count)
}

struct DirectActionRequest {
    command: String,
    label: String,
}

enum DirectActionResult {
    Confirmed { label: String },
    SentUnconfirmed { label: String, reason: String },
    Unavailable { label: String, reason: String },
}

fn spawn_direct_action_worker(
    endpoint: String,
    password: String,
) -> (Sender<DirectActionRequest>, Receiver<DirectActionResult>) {
    let (request_sender, request_receiver) = mpsc::channel::<DirectActionRequest>();
    let (result_sender, result_receiver) = mpsc::channel::<DirectActionResult>();
    thread::spawn(move || {
        while let Ok(request) = request_receiver.recv() {
            let rcon_command = format!("{}; tf2frag_manual_sync_keyframes", request.command);
            let deadline = Instant::now() + DIRECT_CONTROL_READY_TIMEOUT;
            let result = loop {
                match rcon::execute_once(&endpoint, &password, &rcon_command) {
                    Ok(RconDelivery::Confirmed(_)) => {
                        break DirectActionResult::Confirmed {
                            label: request.label,
                        };
                    }
                    Ok(RconDelivery::SentUnconfirmed(reason)) => {
                        // The command may already have reached TF2. Retrying it
                        // could duplicate a destructive keyframe edit.
                        break DirectActionResult::SentUnconfirmed {
                            label: request.label,
                            reason,
                        };
                    }
                    Err(_) if Instant::now() < deadline => {
                        thread::sleep(RCON_RETRY_DELAY);
                    }
                    Err(error) => {
                        break DirectActionResult::Unavailable {
                            label: request.label,
                            reason: format!("{error:#}"),
                        };
                    }
                }
            };
            if result_sender.send(result).is_err() {
                break;
            }
        }
    });
    (request_sender, result_receiver)
}

struct QueuedDirectorAction {
    command: String,
    label: String,
}

struct InFlightDirectorAction {
    sequence: u64,
    slot: u16,
    label: String,
    started_at: Instant,
    timeout_reported: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DirectorQueueTransport {
    ManualKey,
    PollPending,
    Polling,
    InputFallback,
}

struct DirectorActionQueue {
    directory: PathBuf,
    execute_key: String,
    transport: DirectorQueueTransport,
    pending: VecDeque<QueuedDirectorAction>,
    in_flight: Option<InFlightDirectorAction>,
    next_slot: u16,
    next_sequence: u64,
}

impl DirectorActionQueue {
    fn new(directory: PathBuf, execute_key: String, automatic: bool) -> Self {
        Self {
            directory,
            execute_key,
            transport: if automatic {
                DirectorQueueTransport::PollPending
            } else {
                DirectorQueueTransport::ManualKey
            },
            pending: VecDeque::new(),
            in_flight: None,
            next_slot: 0,
            next_sequence: 1,
        }
    }

    fn enqueue(&mut self, command: String, label: String) -> Result<String> {
        if command
            .chars()
            .any(|character| matches!(character, '\r' | '\n'))
        {
            bail!("an internal command contained an unsafe line break");
        }
        self.pending
            .push_back(QueuedDirectorAction { command, label });
        if self.in_flight.is_none() {
            self.start_next()
        } else {
            if self.transport != DirectorQueueTransport::ManualKey {
                Ok(format!("QUEUED • {} ACTION(S) WAITING", self.pending.len()))
            } else {
                Ok(format!(
                    "QUEUED • {} ACTION(S) WAITING • PRESS {} AFTER THE CURRENT ACTION",
                    self.pending.len(),
                    self.execute_key,
                ))
            }
        }
    }

    fn set_poll_available(&mut self, available: bool) -> String {
        if self.transport == DirectorQueueTransport::ManualKey {
            return "DIRECTOR MANUAL ACTION KEY READY".into();
        }
        if available {
            self.transport = DirectorQueueTransport::Polling;
            if let Some(in_flight) = &self.in_flight {
                format!("WAITING FOR TF2 • {}", in_flight.label)
            } else {
                "TF2 COMMAND QUEUE READY • CLICK AN ACTION".into()
            }
        } else {
            self.transport = DirectorQueueTransport::InputFallback;
            if self.in_flight.is_some() {
                self.trigger_input_fallback()
            } else {
                format!(
                    "TF2 WAIT IS UNAVAILABLE • INPUT FALLBACK READY • EMERGENCY KEY {}",
                    self.execute_key
                )
            }
        }
    }

    fn take_timeout_status(&mut self) -> Option<String> {
        let in_flight = self.in_flight.as_mut()?;
        if in_flight.timeout_reported || in_flight.started_at.elapsed() < DIRECTOR_ACTION_ACK_TIMEOUT
        {
            return None;
        }
        in_flight.timeout_reported = true;
        Some(format!(
            "TF2 DID NOT ACKNOWLEDGE • {} • EMERGENCY KEY {}",
            in_flight.label, self.execute_key
        ))
    }

    fn acknowledge(&mut self, sequence: u64) -> String {
        let Some(in_flight) = self.in_flight.take() else {
            return format!("IGNORED UNEXPECTED TF2 ACK {sequence}");
        };
        if in_flight.sequence != sequence {
            self.in_flight = Some(in_flight);
            return format!(
                "WAITING FOR TF2 ACK {}",
                self.in_flight.as_ref().unwrap().sequence
            );
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
            "alias tf2frag_director_poll_action tf2frag_director_execute_{next_slot:02}\n{}\ntf2frag_manual_sync_keyframes\necho {DIRECTOR_ACTION_ACK_PREFIX} {sequence}\n",
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
            started_at: Instant::now(),
            timeout_reported: false,
        });
        Ok(match self.transport {
            DirectorQueueTransport::ManualKey => format!(
                "QUEUED • {label} • RETURN TO TF2 + PRESS {}",
                self.execute_key
            ),
            DirectorQueueTransport::PollPending | DirectorQueueTransport::Polling => {
                format!("WAITING FOR TF2 • {label}")
            }
            DirectorQueueTransport::InputFallback => self.trigger_input_fallback(),
        })
    }

    fn trigger_input_fallback(&self) -> String {
        let label = self
            .in_flight
            .as_ref()
            .map(|action| action.label.as_str())
            .unwrap_or("DIRECTOR ACTION");
        match send_tf2_bound_key(&self.execute_key) {
            Ok(()) => format!("WAITING FOR TF2 • {label} • INPUT FALLBACK SENT"),
            Err(error) => format!(
                "QUEUED • {label} • INPUT FALLBACK FAILED: {error} • PRESS {} IN TF2",
                self.execute_key
            )
            .to_ascii_uppercase(),
        }
    }

    fn action_path(&self, slot: u16) -> PathBuf {
        self.directory
            .join(format!("{DIRECTOR_ACTION_FILE_PREFIX}_{slot:02}.cfg"))
    }
}

fn replace_action_file(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let temp = path.with_extension("cfg.tmp");
    fs::write(&temp, contents)?;
    if let Err(error) = replace_temp_file(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn replace_temp_file(temp: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(temp, destination)
}

#[cfg(target_os = "windows")]
fn replace_temp_file(temp: &Path, destination: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;
    let from = temp
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let to = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut last_error = None;
    for _ in 0..20 {
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } != 0
        {
            return Ok(());
        }
        last_error = Some(std::io::Error::last_os_error());
        thread::sleep(Duration::from_millis(5));
    }
    Err(last_error.unwrap_or_else(std::io::Error::last_os_error))
}

fn build_keyframe_action(
    id: i32,
    tick: i64,
    action: &str,
    setting: &str,
    value: &str,
    tick_marker_prefix: &str,
    clip_start_tick: i64,
) -> Result<(String, String)> {
    if id < 0 {
        bail!("HLAE keyframe ID must be zero or greater");
    }
    let select = format!("mirv_campath select #{id} #{id}");
    let command = match action {
        "Edit keyframe" => build_keyframe_edit(id, &select, setting, value)?,
        "Go to keyframe" => format!("demo_gototick {tick} 0 1; echo {tick_marker_prefix} {tick}"),
        "Go to 1 sec before" => {
            let target = tick
                .saturating_sub(ONE_SECOND_TICKS)
                .max(clip_start_tick)
                .max(0);
            format!("demo_gototick {target} 0 1; echo {tick_marker_prefix} {target}")
        }
        "Select only" => select,
        "Add to selection" => format!("mirv_campath select add #{id} #{id}"),
        "Select range through ID" => {
            let end = normalized_nonnegative_id(value)?;
            format!("mirv_campath select #{id} #{end}")
        }
        "Add range through ID" => {
            let end = normalized_nonnegative_id(value)?;
            format!("mirv_campath select add #{id} #{end}")
        }
        "Delete keyframe" => format!("mirv_campath remove {id}"),
        "Select all" => "mirv_campath select all".into(),
        "Deselect all" => "mirv_campath select none".into(),
        "Invert selection" => "mirv_campath select invert".into(),
        _ => bail!("unknown keyframe action '{action}'"),
    };
    let label = if action == "Edit keyframe" {
        format!("KEYFRAME {id} • {setting}")
    } else {
        format!("KEYFRAME {id} • {action}")
    };
    Ok((command, label))
}

fn build_keyframe_edit(id: i32, select: &str, setting: &str, value: &str) -> Result<String> {
    Ok(match setting {
        "Time — Move to current" => format!("{select}; mirv_campath edit start"),
        "Time — Shift by seconds" => {
            let value = normalized_number(value, "time shift")?;
            format!("{select}; mirv_campath edit start delta{value}")
        }
        "Time — Set absolute seconds" => {
            let value = parse_finite_number(value, "absolute time")?;
            format!("{select}; mirv_campath edit start abs {}", format_number(value))
        }
        "Position — Current camera" => {
            format!("{select}; mirv_campath edit position current")
        }
        "Position — Set X Y Z" => {
            let values = normalized_triplet(value, true, "position")?;
            format!("{select}; mirv_campath edit position {values}")
        }
        "Angles — Current camera" => {
            format!("{select}; mirv_campath edit angles current")
        }
        "Angles — Set Pitch Yaw Roll" => {
            let values = normalized_triplet(value, true, "angles")?;
            format!("{select}; mirv_campath edit angles {values}")
        }
        "FOV — Current camera" => format!("{select}; mirv_campath edit fov current"),
        "FOV — Set value" => {
            let value = parse_finite_number(value, "FOV")?;
            if !(1.0..=179.0).contains(&value) {
                bail!("FOV must be between 1 and 179");
            }
            format!("{select}; mirv_campath edit fov {}", format_number(value))
        }
        "Rotate — Pitch Yaw Roll" => {
            let values = normalized_triplet(value, false, "rotation")?;
            format!("{select}; mirv_campath edit rotate {values}")
        }
        "Anchor — Current camera" => format!(
            "mirv_campath select all; mirv_campath edit anchor #{id} current; mirv_campath select none"
        ),
        "Path offset — Align keyframe to current" => {
            format!("mirv_campath offset current#{id}")
        }
        "Path offset — Add seconds" => {
            let value = normalized_number(value, "path offset")?;
            format!("mirv_campath offset current#{id}{value}")
        }
        "Duration — Set seconds" => {
            let value = parse_finite_number(value, "duration")?;
            if value <= 0.0 {
                bail!("duration must be greater than zero");
            }
            format!(
                "mirv_campath select all; mirv_campath edit duration {}; mirv_campath select none",
                format_number(value)
            )
        }
        "Interpolation — Position" => format!(
            "mirv_campath edit interp position {}",
            normalized_interpolation(value, false)?
        ),
        "Interpolation — Rotation" => format!(
            "mirv_campath edit interp rotation {}",
            normalized_interpolation(value, true)?
        ),
        "Interpolation — FOV" => format!(
            "mirv_campath edit interp fov {}",
            normalized_interpolation(value, false)?
        ),
        _ => bail!("unknown keyframe edit setting '{setting}'"),
    })
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
            #[cfg(target_os = "windows")]
            set_window_nonactivating(native, true);
            force_topmost(native);
        });

        let _ = card.window().with_winit_window(|native| {
            native.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
            native.set_decorations(false);
            native.set_resizable(false);
            let _ = native.set_cursor_hittest(true);
            #[cfg(target_os = "windows")]
            set_window_nonactivating(native, true);
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
        let _ =
            native.request_inner_size(winit::dpi::LogicalSize::new(logical_width, STRIP_HEIGHT));
        native.set_outer_position(geometry.position);
        force_topmost(native);
    });
    let _ = card.window().with_winit_window(|native| {
        let _ = native.request_inner_size(winit::dpi::LogicalSize::new(card_width, card_height));
        let physical_card_width = (card_width * geometry.scale).round() as u32;
        let x =
            geometry.position.x + geometry.size.width.saturating_sub(physical_card_width) as i32;
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

#[cfg(target_os = "windows")]
fn set_window_nonactivating(native: &winit::window::Window, nonactivating: bool) {
    use std::ffi::c_void;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_NOACTIVATE: isize = 0x0800_0000;
    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOZORDER: u32 = 0x0004;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_FRAMECHANGED: u32 = 0x0020;

    let Ok(handle) = native.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let window = handle.hwnd.get() as *mut c_void;
    unsafe {
        let styles = GetWindowLongPtrW(window, GWL_EXSTYLE);
        let updated = if nonactivating {
            styles | WS_EX_NOACTIVATE
        } else {
            styles & !WS_EX_NOACTIVATE
        };
        SetWindowLongPtrW(window, GWL_EXSTYLE, updated);
        SetWindowPos(
            window,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

fn set_overlay_interaction_windows(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    enabled: bool,
) {
    #[cfg(target_os = "windows")]
    {
        let _ = strip
            .window()
            .with_winit_window(|native| set_window_nonactivating(native, !enabled));
        let _ = card
            .window()
            .with_winit_window(|native| set_window_nonactivating(native, !enabled));
        if enabled {
            let _ = card.window().with_winit_window(focus_overlay_window);
        } else {
            focus_tf2_window();
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (strip, card, enabled);
    }
}

#[cfg(target_os = "windows")]
fn focus_overlay_window(native: &winit::window::Window) {
    use std::ffi::c_void;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let Ok(handle) = native.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    focus_window_with_foreground_queue(handle.hwnd.get() as *mut c_void);
}

#[cfg(target_os = "windows")]
fn focus_tf2_window() {
    if let Some(window) = find_tf2_window_handle() {
        focus_window_with_foreground_queue(window);
    }
}

#[cfg(target_os = "windows")]
fn focus_window_with_foreground_queue(window: *mut std::ffi::c_void) {
    if window.is_null() {
        return;
    }
    unsafe {
        let foreground = GetForegroundWindow();
        let foreground_thread = if foreground.is_null() {
            0
        } else {
            GetWindowThreadProcessId(foreground, std::ptr::null_mut())
        };
        let current_thread = GetCurrentThreadId();
        let attached = foreground_thread != 0
            && foreground_thread != current_thread
            && AttachThreadInput(current_thread, foreground_thread, 1) != 0;
        SetForegroundWindow(window);
        SetFocus(window);
        if attached {
            AttachThreadInput(current_thread, foreground_thread, 0);
        }
    }
}

struct TickLogTail {
    path: PathBuf,
    file: Option<File>,
    offset: u64,
    carry: String,
    keyframe_capture: Option<Vec<ParsedCampathKey>>,
    keyframe_capture_invalid: bool,
    keyframe_capture_saw_header: bool,
}

impl TickLogTail {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            file: None,
            offset: 0,
            carry: String::new(),
            keyframe_capture: None,
            keyframe_capture_invalid: false,
            keyframe_capture_saw_header: false,
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
            self.keyframe_capture_invalid = false;
            self.keyframe_capture_saw_header = false;
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
            self.consume_line(line, prefix, &mut poll);
        }
        Ok(poll)
    }

    fn consume_line(&mut self, line: &str, prefix: &str, poll: &mut TelemetryPoll) {
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
        if line.contains(DIRECTOR_POLL_READY_MARKER) {
            poll.director_poll_ready = true;
        }
        if line.contains(DIRECTOR_POLL_UNAVAILABLE_MARKER) {
            poll.director_poll_unavailable = true;
        }
        if line.contains(DIRECTOR_LOAD_CAMPATH_REQUEST_MARKER) {
            poll.load_campath_requested = true;
        }
        if line.contains(DIRECTOR_KEYFRAME_BEGIN_PREFIX) {
            self.keyframe_capture = Some(Vec::new());
            self.keyframe_capture_invalid = false;
            self.keyframe_capture_saw_header = false;
            return;
        }
        if line.contains(DIRECTOR_KEYFRAME_END_PREFIX) {
            if let Some(snapshot) = self.keyframe_capture.take() {
                // A future/unknown HLAE row format must never erase the last
                // verified marker set. A truly empty path has no data rows and
                // remains a valid empty snapshot.
                if !self.keyframe_capture_invalid && self.keyframe_capture_saw_header {
                    poll.keyframe_snapshot = Some(snapshot);
                }
            }
            self.keyframe_capture_invalid = false;
            self.keyframe_capture_saw_header = false;
            return;
        }
        // HLAE prints a header and dashed separators before its rows. They are
        // part of the active capture, not an empty snapshot boundary.
        if self.keyframe_capture.is_some()
            && (line.contains("passed? selected? id : tick[offset]")
                || line.trim_end().ends_with("----"))
        {
            if line.contains("passed? selected? id : tick[offset]") {
                self.keyframe_capture_saw_header = true;
            }
            return;
        }
        if let (Some(capture), Some(key)) =
            (self.keyframe_capture.as_mut(), parse_campath_key(line))
        {
            capture.push(key);
            return;
        }
        if self.keyframe_capture.is_some() && line.contains(" : ") {
            self.keyframe_capture_invalid = true;
            return;
        }
        if is_campath_identity_mutation(line) {
            self.keyframe_capture = None;
            self.keyframe_capture_invalid = false;
            self.keyframe_capture_saw_header = false;
            poll.keyframes_refreshing = true;
        }
        if line.contains(DIRECTOR_KEYFRAME_DIRTY_MARKER) {
            self.keyframe_capture = None;
            self.keyframe_capture_invalid = false;
            self.keyframe_capture_saw_header = false;
            poll.keyframes_refreshing = true;
        }
    }
}

fn line_starts_demo_load(line: &str) -> bool {
    if line.contains(DEMO_RELOAD_MARKER) {
        return true;
    }
    let lower = line.to_ascii_lowercase();
    // "Playing demo from ..." is also printed by demo_gototick seeks. Treating
    // it as a reload disabled the Director forever because an ordinary seek has
    // no matching manual-ready marker.
    lower.contains("] playdemo ")
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
    keyframes_refreshing: bool,
    action_acks: Vec<u64>,
    director_poll_ready: bool,
    director_poll_unavailable: bool,
    load_campath_requested: bool,
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
    // AdvancedFX removed commas from mirv_campath print in newer releases.
    // The tick remains the first field in both the old comma-separated and
    // current whitespace-separated formats.
    let tick_text = right.split_whitespace().next()?.trim_end_matches(',');
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
fn send_tf2_bound_key(key: &str) -> Result<()> {
    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const KEYEVENTF_SCANCODE: u32 = 0x0008;
    const MAPVK_VK_TO_VSC_EX: u32 = 4;

    // Unit tests exercise queue sequencing without a running TF2 process.
    if cfg!(test) {
        return Ok(());
    }

    let virtual_key = virtual_key_code(key)
        .with_context(|| format!("the configured fallback key '{key}' has no Windows key code"))?;
    let tf2_window = find_tf2_window_handle().context("TF2 window not found")?;
    if unsafe { GetForegroundWindow() } != tf2_window {
        bail!("TF2 is not the foreground window");
    }
    let mapped = unsafe { MapVirtualKeyW(virtual_key as u32, MAPVK_VK_TO_VSC_EX) };
    let scan_code = (mapped & 0xff) as u16;
    if scan_code == 0 {
        bail!("Windows could not map the fallback key to a scan code");
    }
    let extended = if mapped & 0xff00 != 0 {
        KEYEVENTF_EXTENDEDKEY
    } else {
        0
    };
    let inputs = [
        WindowsInput {
            kind: INPUT_KEYBOARD,
            data: WindowsInputData {
                keyboard: WindowsKeyboardInput {
                    virtual_key: 0,
                    scan_code,
                    flags: KEYEVENTF_SCANCODE | extended,
                    time: 0,
                    extra_info: 0,
                },
            },
        },
        WindowsInput {
            kind: INPUT_KEYBOARD,
            data: WindowsInputData {
                keyboard: WindowsKeyboardInput {
                    virtual_key: 0,
                    scan_code,
                    flags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP | extended,
                    time: 0,
                    extra_info: 0,
                },
            },
        },
    ];
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<WindowsInput>() as i32,
        )
    };
    if sent != inputs.len() as u32 {
        bail!("Windows did not deliver the fallback key to TF2");
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn find_tf2_window_handle() -> Option<*mut std::ffi::c_void> {
    let mut search = Tf2WindowSearch {
        window: std::ptr::null_mut(),
    };
    unsafe {
        EnumWindows(
            Some(find_tf2_window),
            &mut search as *mut Tf2WindowSearch as isize,
        );
    }
    (!search.window.is_null()).then_some(search.window)
}

#[cfg(target_os = "windows")]
fn cursor_is_over_tf2() -> bool {
    const GA_ROOT: u32 = 2;
    let Some(tf2) = find_tf2_window_handle() else {
        return false;
    };
    let mut point = WindowsPoint { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut point) == 0 {
            return false;
        }
        let pointed = WindowFromPoint(point);
        !pointed.is_null() && GetAncestor(pointed, GA_ROOT) == tf2
    }
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
#[repr(C)]
struct WindowsPoint {
    x: i32,
    y: i32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct WindowsInput {
    kind: u32,
    data: WindowsInputData,
}

#[cfg(target_os = "windows")]
#[repr(C)]
union WindowsInputData {
    mouse: WindowsMouseInput,
    keyboard: WindowsKeyboardInput,
    hardware: WindowsHardwareInput,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
#[repr(C)]
struct WindowsMouseInput {
    x: i32,
    y: i32,
    mouse_data: u32,
    flags: u32,
    time: u32,
    extra_info: usize,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
#[repr(C)]
struct WindowsKeyboardInput {
    virtual_key: u16,
    scan_code: u16,
    flags: u32,
    time: u32,
    extra_info: usize,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
#[repr(C)]
struct WindowsHardwareInput {
    message: u32,
    low: u16,
    high: u16,
}

#[cfg(target_os = "windows")]
struct Tf2WindowSearch {
    window: *mut std::ffi::c_void,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn find_tf2_window(window: *mut std::ffi::c_void, state: isize) -> i32 {
    const BUFFER_LENGTH: usize = 128;
    let mut class_name = [0_u16; BUFFER_LENGTH];
    let class_length = GetClassNameW(window, class_name.as_mut_ptr(), BUFFER_LENGTH as i32);
    if class_length <= 0
        || String::from_utf16_lossy(&class_name[..class_length as usize]) != "Valve001"
    {
        return 1;
    }
    let mut title = [0_u16; BUFFER_LENGTH];
    let title_length = GetWindowTextW(window, title.as_mut_ptr(), BUFFER_LENGTH as i32);
    if title_length <= 0
        || !String::from_utf16_lossy(&title[..title_length as usize])
            .to_ascii_lowercase()
            .contains("team fortress 2")
    {
        return 1;
    }
    (*(state as *mut Tf2WindowSearch)).window = window;
    0
}

#[cfg(not(target_os = "windows"))]
fn send_tf2_bound_key(_key: &str) -> Result<()> {
    if cfg!(test) {
        Ok(())
    } else {
        bail!("automatic TF2 input is Windows-only")
    }
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
    fn EnumWindows(
        callback: Option<unsafe extern "system" fn(*mut std::ffi::c_void, isize) -> i32>,
        state: isize,
    ) -> i32;
    fn GetClassNameW(window: *mut std::ffi::c_void, class_name: *mut u16, maximum: i32) -> i32;
    fn GetAsyncKeyState(virtual_key: i32) -> i16;
    fn GetCursorPos(point: *mut WindowsPoint) -> i32;
    fn GetAncestor(window: *mut std::ffi::c_void, flags: u32) -> *mut std::ffi::c_void;
    fn GetForegroundWindow() -> *mut std::ffi::c_void;
    fn GetWindowThreadProcessId(window: *mut std::ffi::c_void, process_id: *mut u32) -> u32;
    fn GetWindowLongPtrW(window: *mut std::ffi::c_void, index: i32) -> isize;
    fn GetWindowTextW(window: *mut std::ffi::c_void, title: *mut u16, maximum: i32) -> i32;
    fn MapVirtualKeyW(code: u32, map_type: u32) -> u32;
    fn SendInput(count: u32, inputs: *const WindowsInput, size: i32) -> u32;
    fn SetWindowPos(
        window: *mut std::ffi::c_void,
        insert_after: *mut std::ffi::c_void,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> i32;
    fn SetWindowLongPtrW(window: *mut std::ffi::c_void, index: i32, value: isize) -> isize;
    fn SetForegroundWindow(window: *mut std::ffi::c_void) -> i32;
    fn SetFocus(window: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn AttachThreadInput(thread: u32, attach_to: u32, attach: i32) -> i32;
    fn WindowFromPoint(point: WindowsPoint) -> *mut std::ffi::c_void;
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
    fn MoveFileExW(existing: *const u16, replacement: *const u16, flags: u32) -> i32;
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

    fn write_test_xml(name: &str, contents: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "tf2fragdemohelper-{name}-{}-{nonce}.xml",
            std::process::id()
        ));
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn validates_hlae_campath_xml_and_builds_the_documented_load_sequence() {
        let path = write_test_xml(
            "valid-campath",
            r#"<?xml version="1.0"?>
<campath><points>
<p t="1.25" x="10" y="20" z="30" rx="0" ry="90" rz="0" fov="75" />
<p t="2.50" x="40" y="50" z="60" qw="1" qx="0" qy="0" qz="0" fov="90" />
</points></campath>"#,
        );
        let (command, point_count) = validated_campath_load_command(&path).unwrap();
        let expected_path = path.to_string_lossy().replace('\\', "/");
        assert_eq!(point_count, 2);
        assert!(command.contains(&format!("mirv_campath load \"{expected_path}\"")));
        assert!(command.contains("mirv_campath enabled 1"));
        assert!(command.ends_with(DIRECTOR_KEYFRAME_DIRTY_MARKER));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_generic_xml_and_incomplete_hlae_points() {
        let generic = write_test_xml("generic", "<document><points /></document>");
        assert!(validate_campath_xml(&generic).is_err());
        fs::remove_file(generic).unwrap();

        let incomplete = write_test_xml(
            "incomplete",
            r#"<campath><points><p t="1" x="2" y="3" z="4" rx="0" ry="0" fov="90" /></points></campath>"#,
        );
        assert!(validate_campath_xml(&incomplete).is_err());
        fs::remove_file(incomplete).unwrap();
    }

    #[test]
    fn detects_cross_platform_load_campath_shortcut_marker() {
        let mut tail = TickLogTail::new(PathBuf::new());
        let mut poll = TelemetryPoll::default();
        tail.consume_line(
            &format!("08/30 12:30:22 {DIRECTOR_LOAD_CAMPATH_REQUEST_MARKER}"),
            "TF2FRAG_DIRECTOR_TICK",
            &mut poll,
        );
        assert!(poll.load_campath_requested);
    }

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
        assert_eq!(
            parse_tick_marker("unrelated", "TF2FRAG_DIRECTOR_TICK"),
            None
        );
        assert_eq!(
            parse_tick_update("TF2FRAG_DIRECTOR_TICK_OFFSET -67", "TF2FRAG_DIRECTOR_TICK"),
            Some(TickUpdate::Relative(-67))
        );
        assert_eq!(
            parse_tick_update("TF2FRAG_DIRECTOR_TICK 12000", "TF2FRAG_DIRECTOR_TICK"),
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
        assert!(!line_starts_demo_load(
            "08/30 12:30:22 Playing demo from demos/test.dem."
        ));
        assert!(!line_starts_demo_load(
            "08/30 12:30:22 ] demo_gototick 12345 0 1"
        ));
        assert!(!line_starts_demo_load(
            "08/30 12:30:22 Demo playback finished."
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
        assert_eq!(
            parse_campath_key("Y n 3 : 12400 185.970 185.970 -> ( 1 2 3 )"),
            Some(ParsedCampathKey {
                id: 3,
                tick: 12_400,
                selected: false,
            })
        );
    }

    #[test]
    fn incomplete_hlae_print_never_publishes_an_empty_keyframe_snapshot() {
        let mut tail = TickLogTail::new(PathBuf::new());

        let mut mutation = TelemetryPoll::default();
        tail.consume_line(
            "08/30 12:30:22 ] mirv_campath add",
            "TF2FRAG_DIRECTOR_TICK",
            &mut mutation,
        );
        assert!(mutation.keyframes_refreshing);
        assert!(mutation.keyframe_snapshot.is_none());

        let mut partial = TelemetryPoll::default();
        for line in [
            DIRECTOR_KEYFRAME_BEGIN_PREFIX,
            "passed? selected? id : tick[offset] , time [s]",
            "----------------------------------------------------",
            "Y n 0 : 12000 , 179.1",
        ] {
            tail.consume_line(line, "TF2FRAG_DIRECTOR_TICK", &mut partial);
        }
        assert!(partial.keyframe_snapshot.is_none());

        let mut completed = TelemetryPoll::default();
        tail.consume_line(
            "Y n 1 : 12100 180.6 180.6 -> ( 1 2 3 )",
            "TF2FRAG_DIRECTOR_TICK",
            &mut completed,
        );
        assert!(completed.keyframe_snapshot.is_none());
        tail.consume_line(
            DIRECTOR_KEYFRAME_END_PREFIX,
            "TF2FRAG_DIRECTOR_TICK",
            &mut completed,
        );
        assert_eq!(
            completed.keyframe_snapshot,
            Some(vec![
                ParsedCampathKey {
                    id: 0,
                    tick: 12_000,
                    selected: false,
                },
                ParsedCampathKey {
                    id: 1,
                    tick: 12_100,
                    selected: false,
                },
            ])
        );
    }

    #[test]
    fn only_a_real_hlae_table_can_publish_an_empty_keyframe_snapshot() {
        let mut tail = TickLogTail::new(PathBuf::new());
        let mut missing_table = TelemetryPoll::default();
        for line in [
            DIRECTOR_KEYFRAME_BEGIN_PREFIX,
            DIRECTOR_KEYFRAME_END_PREFIX,
        ] {
            tail.consume_line(line, "TF2FRAG_DIRECTOR_TICK", &mut missing_table);
        }
        assert!(missing_table.keyframe_snapshot.is_none());

        let mut empty_table = TelemetryPoll::default();
        for line in [
            DIRECTOR_KEYFRAME_BEGIN_PREFIX,
            "passed? selected? id : tick[offset] , time [s]",
            "----------------------------------------------------",
            DIRECTOR_KEYFRAME_END_PREFIX,
        ] {
            tail.consume_line(line, "TF2FRAG_DIRECTOR_TICK", &mut empty_table);
        }
        assert_eq!(empty_table.keyframe_snapshot, Some(Vec::new()));
    }

    #[test]
    fn explicit_keyframe_dirty_marker_requests_a_settled_refresh() {
        let mut tail = TickLogTail::new(PathBuf::new());
        let mut poll = TelemetryPoll::default();
        tail.consume_line(
            DIRECTOR_KEYFRAME_DIRTY_MARKER,
            "TF2FRAG_DIRECTOR_TICK",
            &mut poll,
        );
        assert!(poll.keyframes_refreshing);
        assert!(poll.keyframe_snapshot.is_none());
    }

    #[test]
    fn unknown_nonempty_hlae_rows_cannot_erase_verified_keyframes() {
        let mut tail = TickLogTail::new(PathBuf::new());
        let mut poll = TelemetryPoll::default();
        for line in [
            DIRECTOR_KEYFRAME_BEGIN_PREFIX,
            "Y n 0 : future-format-without-a-readable-tick",
            DIRECTOR_KEYFRAME_END_PREFIX,
        ] {
            tail.consume_line(line, "TF2FRAG_DIRECTOR_TICK", &mut poll);
        }
        assert!(poll.keyframe_snapshot.is_none());
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
    fn queued_action_is_written_for_tf2_polling_delivery() {
        let directory = std::env::temp_dir().join(format!(
            "tf2frag-director-action-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        let mut queue = DirectorActionQueue::new(directory.clone(), "'".into(), true);
        let status = queue
            .enqueue("mirv_campath print".into(), "PRINT KEYFRAMES".into())
            .unwrap();
        assert!(status.contains("WAITING FOR TF2"));
        assert!(!status.contains("PRESS"));

        let action =
            fs::read_to_string(directory.join(format!("{DIRECTOR_ACTION_FILE_PREFIX}_00.cfg")))
                .unwrap();
        assert!(action.contains("mirv_campath print"));
        assert!(action.starts_with(
            "alias tf2frag_director_poll_action tf2frag_director_execute_01"
        ));
        assert!(!action.contains("wait"));

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn acknowledged_mailbox_actions_advance_exactly_one_slot() {
        let directory = std::env::temp_dir().join(format!(
            "tf2frag-director-mailbox-sequence-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        let mut queue = DirectorActionQueue::new(directory.clone(), "'".into(), true);
        queue.enqueue("echo FIRST".into(), "FIRST".into()).unwrap();
        let waiting = queue
            .enqueue("echo SECOND".into(), "SECOND".into())
            .unwrap();
        assert!(waiting.contains("1 ACTION(S) WAITING"));
        assert!(!directory
            .join(format!("{DIRECTOR_ACTION_FILE_PREFIX}_01.cfg"))
            .exists());

        let completed = queue.acknowledge(1);
        assert!(completed.contains("COMPLETE • FIRST"));
        assert!(completed.contains("WAITING FOR TF2 • SECOND"));
        let second =
            fs::read_to_string(directory.join(format!("{DIRECTOR_ACTION_FILE_PREFIX}_01.cfg")))
                .unwrap();
        assert!(second.contains("echo SECOND"));
        assert!(second.contains(&format!("{DIRECTOR_ACTION_ACK_PREFIX} 2")));
        assert!(second.starts_with(
            "alias tf2frag_director_poll_action tf2frag_director_execute_02"
        ));

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn unavailable_wait_switches_only_to_the_verified_input_fallback() {
        let directory = std::env::temp_dir().join(format!(
            "tf2frag-director-fallback-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        let mut queue = DirectorActionQueue::new(directory.clone(), "'".into(), true);
        queue.enqueue("echo FALLBACK".into(), "FALLBACK".into()).unwrap();
        let status = queue.set_poll_available(false);
        assert_eq!(queue.transport, DirectorQueueTransport::InputFallback);
        assert!(status.contains("INPUT FALLBACK SENT"));
        assert!(queue.in_flight.is_some());

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn unacknowledged_actions_report_a_real_tf2_timeout() {
        let directory = std::env::temp_dir().join(format!(
            "tf2frag-director-timeout-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        let mut queue = DirectorActionQueue::new(directory.clone(), "'".into(), true);
        queue.enqueue("echo TIMEOUT".into(), "TIMEOUT".into()).unwrap();
        queue.in_flight.as_mut().unwrap().started_at =
            Instant::now() - DIRECTOR_ACTION_ACK_TIMEOUT - Duration::from_millis(1);
        let status = queue.take_timeout_status().unwrap();
        assert!(status.contains("TF2 DID NOT ACKNOWLEDGE"));
        assert!(queue.take_timeout_status().is_none());

        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn builds_exact_hlae_id_actions_without_persistent_ids() {
        let (goto, _) = build_keyframe_action(
            4,
            12_345,
            "Go to keyframe",
            "",
            "",
            "TF2FRAG_DIRECTOR_TICK",
            10_000,
        )
        .unwrap();
        assert_eq!(
            goto,
            "demo_gototick 12345 0 1; echo TF2FRAG_DIRECTOR_TICK 12345"
        );

        let (move_to_current, _) = build_keyframe_action(
            4,
            12_345,
            "Edit keyframe",
            "Time — Move to current",
            "",
            "TF2FRAG_DIRECTOR_TICK",
            10_000,
        )
        .unwrap();
        assert_eq!(
            move_to_current,
            "mirv_campath select #4 #4; mirv_campath edit start"
        );

        let (delete, _) = build_keyframe_action(
            4,
            12_345,
            "Delete keyframe",
            "",
            "",
            "TF2FRAG_DIRECTOR_TICK",
            10_000,
        )
        .unwrap();
        assert_eq!(delete, "mirv_campath remove 4");

        let (position, _) = build_keyframe_action(
            4,
            12_345,
            "Edit keyframe",
            "Position — Current camera",
            "",
            "TF2FRAG_DIRECTOR_TICK",
            10_000,
        )
        .unwrap();
        assert_eq!(
            position,
            "mirv_campath select #4 #4; mirv_campath edit position current"
        );

        let (fov, _) = build_keyframe_action(
            4,
            12_345,
            "Edit keyframe",
            "FOV — Set value",
            "90",
            "TF2FRAG_DIRECTOR_TICK",
            10_000,
        )
        .unwrap();
        assert_eq!(fov, "mirv_campath select #4 #4; mirv_campath edit fov 90");

        let (rotation_interp, _) = build_keyframe_action(
            4,
            12_345,
            "Edit keyframe",
            "Interpolation — Rotation",
            "sCubic",
            "TF2FRAG_DIRECTOR_TICK",
            10_000,
        )
        .unwrap();
        assert_eq!(rotation_interp, "mirv_campath edit interp rotation sCubic");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn maps_default_overlay_hotkey() {
        assert_eq!(virtual_key_code("C"), Some(0x43));
    }
}
