#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use anyhow::{Context, Result};
use slint::winit_030::{winit, WinitWindowAccessor};
use slint::{ComponentHandle, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::{
    cell::Cell,
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};
use tf2_mirv_director::{
    DirectorControl, DirectorSession, DIRECTOR_TICK_OFFSET_PREFIX,
};

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
    let strip = DirectorStripWindow::new()?;
    let card = DirectorCardWindow::new()?;

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
    let draw_campath_key = shortcut_key("draw_campath", "/");
    strip.set_start_tick(to_ui_tick(session.start_tick));
    strip.set_end_tick(to_ui_tick(session.end_tick));
    strip.set_panel_toggle_key(panel_toggle_key.clone().into());
    card.set_panel_toggle_key(panel_toggle_key.clone().into());
    card.set_draw_campath_key(draw_campath_key.into());
    card.set_record_order(
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

    let shortcut_rows = session
        .shortcuts
        .iter()
        .map(|shortcut| ShortcutRow {
            key: shortcut.key.clone().into(),
            label: shortcut.label.clone().into(),
        })
        .collect::<Vec<_>>();
    card.set_shortcuts(ModelRc::new(VecModel::from(shortcut_rows)));

    update_playback_state(&strip, &card, &session, session.start_tick, false);
    strip.set_telemetry_status("WAIT".into());

    let panel_visible = Rc::new(Cell::new(true));
    {
        let weak_strip = strip.as_weak();
        let weak_card = card.as_weak();
        let panel_visible = panel_visible.clone();
        card.on_hide_requested(move || {
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            set_panel_visibility(&strip, &card, &panel_visible, false);
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
        let was_down = Rc::new(Cell::new(false));
        hotkey_timer.start(TimerMode::Repeated, Duration::from_millis(30), move || {
            let down = global_key_is_down(virtual_key);
            if down && !was_down.replace(down) {
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
        let mut tail = TickLogTail::new(session.telemetry_log.clone());
        let telemetry_diagnostic = telemetry_diagnostic.clone();
        let mut polls = 0_u32;
        let mut reported_log_open = false;
        let mut reported_missing_log = false;
        let mut reported_no_tick = false;
        let mut reported_first_tick = false;
        let mut reported_error = false;
        let mut last_tick = session.start_tick;
        telemetry_timer.start(TimerMode::Repeated, Duration::from_millis(15), move || {
            let (Some(strip), Some(card)) = (weak_strip.upgrade(), weak_card.upgrade()) else {
                return;
            };
            polls = polls.saturating_add(1);
            match tail.poll(&session.telemetry_marker_prefix) {
                Ok(updates) => {
                    if tail.is_open() && !reported_log_open {
                        reported_log_open = true;
                        reported_missing_log = false;
                        append_telemetry_diagnostic(
                            &telemetry_diagnostic,
                            "status=TF2_CONSOLE_LOG_OPENED",
                        );
                    }
                    if !updates.is_empty() {
                        for update in updates {
                            last_tick = match update {
                                TickUpdate::Absolute(tick) => tick,
                                TickUpdate::Relative(delta) => {
                                    let tick = last_tick.saturating_add(delta).max(0);
                                    append_telemetry_diagnostic(
                                        &telemetry_diagnostic,
                                        &format!(
                                            "status=BACKWARD_SEEK_RESYNC delta={delta} tick={tick}"
                                        ),
                                    );
                                    tick
                                }
                            };
                        }
                        if !reported_first_tick {
                            reported_first_tick = true;
                            append_telemetry_diagnostic(
                                &telemetry_diagnostic,
                                &format!("status=FIRST_TICK_RECEIVED tick={last_tick}"),
                            );
                        }
                        strip.set_telemetry_status("LIVE".into());
                        update_playback_state(&strip, &card, &session, last_tick, true);
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

fn update_playback_state(
    strip: &DirectorStripWindow,
    card: &DirectorCardWindow,
    session: &DirectorSession,
    tick: i64,
    telemetry_active: bool,
) {
    strip.set_current_tick(to_ui_tick(tick));
    strip.set_current_position(session.cue_position(tick));
    strip.set_telemetry_active(
        telemetry_active || matches!(&session.control, DirectorControl::LocalBridge { .. }),
    );
    card.set_current_tick(to_ui_tick(tick));

    if let Some((index, cue)) = session
        .cues
        .iter()
        .enumerate()
        .find(|(_, cue)| cue.tick >= tick)
    {
        strip.set_next_frag_tick(to_ui_tick(cue.tick));
        card.set_next_frag_number((index + 1).min(i32::MAX as usize) as i32);
        card.set_next_frag_tick(to_ui_tick(cue.tick));
        card.set_next_frag_victims(cue.victims.join(", ").into());
        card.set_next_frag_tags(cue.tags.join(", ").into());
    } else {
        strip.set_next_frag_tick(0);
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
            let _ = native.set_cursor_hittest(false);
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
    const CARD_HEIGHT: f64 = 650.0;

    let logical_width = geometry.size.width as f64 / geometry.scale;
    let logical_height = geometry.size.height as f64 / geometry.scale;
    let card_width = CARD_WIDTH.min(logical_width);
    let card_height = CARD_HEIGHT.min((logical_height - STRIP_HEIGHT).max(1.0));

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
}

impl TickLogTail {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            file: None,
            offset: 0,
            carry: String::new(),
        }
    }

    fn is_open(&self) -> bool {
        self.file.is_some()
    }

    fn poll(&mut self, prefix: &str) -> std::io::Result<Vec<TickUpdate>> {
        if self.file.is_none() {
            match File::open(&self.path) {
                Ok(file) => self.file = Some(file),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
                Err(error) => return Err(error),
            }
        }
        let file = self.file.as_mut().expect("telemetry file opened");
        let length = file.metadata()?.len();
        if length < self.offset {
            self.offset = 0;
            self.carry.clear();
        }
        file.seek(SeekFrom::Start(self.offset))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        self.offset += bytes.len() as u64;
        if bytes.is_empty() {
            return Ok(Vec::new());
        }

        let mut text = std::mem::take(&mut self.carry);
        text.push_str(&String::from_utf8_lossy(&bytes));
        let complete_length = text.rfind('\n').map(|index| index + 1).unwrap_or(0);
        let (complete, remainder) = text.split_at(complete_length);
        self.carry = remainder.to_owned();
        Ok(complete
            .lines()
            .filter_map(|line| parse_tick_update(line, prefix))
            .collect())
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
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn maps_default_overlay_hotkey() {
        assert_eq!(virtual_key_code("C"), Some(0x43));
    }
}
