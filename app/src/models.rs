use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DemoContext {
    #[serde(default)]
    pub capture_type: String,
    #[serde(default)]
    pub capture_confidence: String,
    #[serde(default)]
    pub capture_evidence: Vec<String>,
    #[serde(default)]
    pub header_nick: Option<String>,
    #[serde(default)]
    pub analysis_scope: String,
    #[serde(default)]
    pub pov_player_user_id: Option<i64>,
    #[serde(default)]
    pub roster_match_available: bool,
    #[serde(default)]
    pub scope_reason: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub mode_label: String,
    #[serde(default)]
    pub mode_confidence: String,
    #[serde(default)]
    pub mode_evidence: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TickTagGroup {
    #[serde(default)]
    pub demo_tick: i64,
    #[serde(default)]
    pub server_ticks: Vec<i64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Candidate {
    #[serde(default)]
    pub candidate_id: String,
    #[serde(default)]
    pub source_demo: String,
    #[serde(default)]
    pub map_name: String,
    #[serde(default)]
    pub round_index: i64,
    #[serde(default)]
    pub overall_score: f64,
    #[serde(default)]
    pub attacker_user_id: i64,
    #[serde(default)]
    pub attacker_class: String,
    #[serde(default)]
    pub attacker_team: String,
    #[serde(default)]
    pub clip_start_tick: i64,
    #[serde(default)]
    pub clip_end_tick: i64,
    #[serde(default)]
    pub point_of_kill_ticks: Vec<i64>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Tags assigned to the kill event(s) at each distinct demo tick.
    /// Same-tick kills share one group so the UI does not pretend the demo
    /// timeline can distinguish events that occurred in the same frame.
    #[serde(default)]
    pub tick_tags: Vec<TickTagGroup>,
    /// Candidate-wide tags such as multi-kill, rapid sequence, objective
    /// conversion, or round clinch. `tags` remains the flattened union for
    /// backwards-compatible filtering and imported-export support.
    #[serde(default)]
    pub sequence_tags: Vec<String>,
    /// The strongest positive tag according to the score evidence. Recording
    /// output uses this stable value as its category folder.
    #[serde(default)]
    pub primary_tag: String,
    #[serde(default)]
    pub metrics: Value,
    #[serde(default)]
    pub kills: Vec<Value>,
    #[serde(default)]
    pub score_breakdown: Vec<Value>,
    #[serde(default)]
    pub demo_context: DemoContext,
    #[serde(default)]
    pub bookmark_comment: String,
    #[serde(default)]
    pub bookmark_tick: Option<i64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl Candidate {
    pub fn kill_count(&self) -> usize {
        self.metrics
            .get("kills")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(self.kills.len())
    }

    pub fn inferred_primary_tag(&self) -> String {
        // A candidate's output category follows its story chronologically: the
        // first tag on the earliest kill tick wins. This makes Details and the
        // recording folders agree, even if a later tick earns more points.
        if let Some(tag) = self
            .tick_tags
            .iter()
            .filter(|group| !group.tags.is_empty())
            .min_by_key(|group| group.demo_tick)
            .and_then(|group| group.tags.iter().find(|tag| !tag.trim().is_empty()))
        {
            return tag.clone();
        }
        if !self.primary_tag.trim().is_empty()
            && self.tags.iter().any(|tag| tag == &self.primary_tag)
        {
            return self.primary_tag.clone();
        }
        infer_primary_tag(&self.tags, &self.tick_tags, &self.sequence_tags, &self.score_breakdown)
            .unwrap_or_else(|| "other".into())
    }
}

fn infer_primary_tag(
    tags: &[String],
    tick_tags: &[TickTagGroup],
    sequence_tags: &[String],
    score_breakdown: &[Value],
) -> Option<String> {
    let unique = tags.iter().filter(|tag| !tag.trim().is_empty()).collect::<BTreeSet<_>>();
    unique
        .into_iter()
        .map(|tag| {
            let occurrence_count = tick_tags
                .iter()
                .filter(|group| group.tags.iter().any(|candidate| candidate == tag))
                .count();
            let sequence_wide = sequence_tags.iter().any(|candidate| candidate == tag);
            let evidence = score_breakdown
                .iter()
                .filter_map(|item| {
                    let points = item.get("points").and_then(Value::as_f64)?;
                    if points <= 0.0 {
                        return None;
                    }
                    let reason = item.get("reason").and_then(Value::as_str).unwrap_or_default();
                    let strength = tag_reason_match_strength(tag, reason);
                    (strength > 0.0).then_some(points * strength)
                })
                .fold(0.0_f64, f64::max);
            let specificity = normalized_words(tag).len();
            (tag, evidence, occurrence_count, sequence_wide, specificity)
        })
        .max_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then(left.2.cmp(&right.2))
                .then(left.3.cmp(&right.3))
                .then(left.4.cmp(&right.4))
                // Prefer the alphabetically earlier identifier for a stable tie.
                .then_with(|| right.0.cmp(left.0))
        })
        .map(|entry| entry.0.clone())
}

fn tag_reason_match_strength(tag: &str, reason: &str) -> f64 {
    let tag_words = normalized_words(tag);
    let reason_words = normalized_words(reason);
    if tag_words.is_empty() || reason_words.is_empty() {
        return 0.0;
    }
    if tag_words.iter().all(|word| reason_words.contains(word)) {
        return 1.0;
    }

    // Current sequence tags whose score-reason wording is intentionally more
    // descriptive. New tags normally need no entry here because identifier
    // words are matched generically above and below.
    let explicit = matches!(
        (tag, reason),
        ("multi_kill", "additional_kills")
            | ("double_airshot_sequence", "multiple_confirmed_airshots")
            | ("team_wipe", "sequence_finished_enemy_team")
            | ("medic_force", "enemy_medic_forced_uber_after_sequence")
            | ("player_count_swing", "sequence_created_player_count_window")
            | ("round_clinch", "team_won_immediately_after_sequence")
            | ("building_to_kill_sequence", "building_destruction_led_to_kills")
            | ("capture_denial_followup", "kill_sequence_blocked_capture")
            | ("payload_progress_followup", "kill_sequence_led_to_payload_progress")
    );
    if explicit {
        return 1.0;
    }

    let overlap = tag_words
        .iter()
        .filter(|word| reason_words.contains(*word))
        .count();
    if overlap == 0 {
        0.0
    } else {
        (overlap as f64 / tag_words.len() as f64) * 0.65
    }
}

fn normalized_words(value: &str) -> BTreeSet<String> {
    value
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| {
            if word.len() > 4 {
                word.strip_suffix('s').unwrap_or(word).to_owned()
            } else {
                word.to_owned()
            }
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct DemoJob {
    pub order: usize,
    pub demo_path: PathBuf,
    pub export_directory: PathBuf,
    pub source_bytes: u64,
    pub parsed_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct AppSettings {
    pub output_directory: PathBuf,
    pub item_schema: PathBuf,
    pub tf2_executable: PathBuf,
    pub hlae_executable: PathBuf,
    pub ffmpeg_executable: PathBuf,
    pub recording_output_directory: PathBuf,
    /// Additional safe command-line switches used only for Helper-launched
    /// manual HLAE sessions. Session ownership, offline safety, demo loading,
    /// and the automatic TF2 CFG polling queue remain managed by the Helper.
    pub manual_hlae_launch_options: String,
    pub performance_profile: String,
    pub lead_seconds: u32,
    pub outro_seconds: u32,
    pub capture_fps: u32,
    pub jpg_quality: u8,
    pub recording_format: String,
    pub mp4_compatibility: String,
    pub mp4_video_codec: String,
    pub mp4_pixel_format: String,
    pub mp4_h264_profile: String,
    pub mp4_crf: u8,
    pub mp4_encoder_preset: String,
    pub mp4_audio_codec: String,
    pub mp4_audio_bitrate_kbps: u32,
    pub avi_video_codec: String,
    pub avi_pixel_format: String,
    pub dnxhr_profile: String,
    pub resolution: String,
    pub dx_level: String,
    pub skybox: String,
    pub hud: String,
    pub viewmodels: String,
    pub viewmodel_fov: u32,
    pub maximum_graphics: bool,
    pub motion_blur: bool,
    pub disable_hit_sounds: bool,
    pub disable_voice_chat: bool,
    pub minimal_hud: bool,
    pub disable_combat_text: bool,
    pub disable_crosshair: bool,
    pub disable_crosshair_switching: bool,
    pub hud_player_model: bool,
    pub isolate_custom_resources: bool,
    pub disable_announcer_voices: bool,
    pub disable_applause_sounds: bool,
    pub disable_domination_sounds: bool,
    pub mirv_shortcuts: MirvShortcuts,
    pub custom_resources: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MirvShortcuts {
    pub advance_time: String,
    pub toggle_hud: String,
    pub show_help: String,
    pub back_one_second: String,
    pub safe_restart: String,
    pub next_kill_tick: String,
    pub pause_resume: String,
    pub enter_camera: String,
    pub add_keyframe: String,
    pub play_campath: String,
    pub draw_campath: String,
    pub start_recording: String,
    pub stop_recording: String,
    pub print_keyframes: String,
    pub save_campath: String,
    pub load_campath: String,
    pub execute_director_action: String,
    pub overlay_panel_toggle: String,
    pub overlay_interaction_toggle: String,
    #[serde(default)]
    pub overlay_panel_toggle_default_version: u8,
}

impl Default for MirvShortcuts {
    fn default() -> Self {
        Self {
            advance_time: "[".into(),
            toggle_hud: "]".into(),
            show_help: "1".into(),
            back_one_second: "2".into(),
            safe_restart: "3".into(),
            next_kill_tick: "4".into(),
            pause_resume: "5".into(),
            enter_camera: "6".into(),
            add_keyframe: "7".into(),
            play_campath: "8".into(),
            draw_campath: "/".into(),
            start_recording: "9".into(),
            stop_recording: "0".into(),
            print_keyframes: "-".into(),
            save_campath: "=".into(),
            load_campath: "F8".into(),
            execute_director_action: "'".into(),
            overlay_panel_toggle: "C".into(),
            overlay_interaction_toggle: "F11".into(),
            overlay_panel_toggle_default_version: 3,
        }
    }
}

impl MirvShortcuts {
    pub fn is_reserved_arrow_key(value: &str) -> bool {
        matches!(
            value.trim().to_ascii_uppercase().as_str(),
            "UPARROW" | "DOWNARROW" | "LEFTARROW" | "RIGHTARROW"
        )
    }

    fn normalized_key(value: &str, fallback: &str) -> String {
        let value = value.trim();
        let upper = value.to_ascii_uppercase();
        let safe_named_key = matches!(
            upper.as_str(),
            "SPACE" | "TAB" | "ENTER" | "BACKSPACE" | "DEL" | "INS" | "HOME" | "END"
                | "PGUP" | "PGDN" | "SCROLLLOCK" | "PAUSE"
                | "F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8"
                | "F9" | "F10" | "F11" | "F12"
        );
        let safe_printable_key = value.chars().count() == 1
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric()
                    || matches!(character, '_' | '[' | ']' | '-' | '=' | ',' | '.' | '/' | '\'')
            });
        if !Self::is_reserved_arrow_key(value) && safe_named_key {
            upper
        } else if !Self::is_reserved_arrow_key(value) && safe_printable_key {
            value.into()
        } else {
            fallback.into()
        }
    }

    pub fn normalize(&mut self) {
        // HOME, Y, and S were short-lived overlay defaults. Migrate only the
        // matching default at each version so a deliberately customized key is
        // preserved, and never re-migrate after the C default is recorded.
        if self.overlay_panel_toggle_default_version == 0 {
            if self.overlay_panel_toggle.eq_ignore_ascii_case("HOME") {
                self.overlay_panel_toggle = "C".into();
            }
            self.overlay_panel_toggle_default_version = 3;
        } else if self.overlay_panel_toggle_default_version == 1 {
            if self.overlay_panel_toggle.eq_ignore_ascii_case("Y") {
                self.overlay_panel_toggle = "C".into();
            }
            self.overlay_panel_toggle_default_version = 3;
        } else if self.overlay_panel_toggle_default_version == 2 {
            if self.overlay_panel_toggle.eq_ignore_ascii_case("S") {
                self.overlay_panel_toggle = "C".into();
            }
            self.overlay_panel_toggle_default_version = 3;
        }
        let defaults = Self::default();
        self.advance_time = Self::normalized_key(&self.advance_time, &defaults.advance_time);
        self.toggle_hud = Self::normalized_key(&self.toggle_hud, &defaults.toggle_hud);
        self.show_help = Self::normalized_key(&self.show_help, &defaults.show_help);
        self.back_one_second = Self::normalized_key(&self.back_one_second, &defaults.back_one_second);
        self.safe_restart = Self::normalized_key(&self.safe_restart, &defaults.safe_restart);
        self.next_kill_tick = Self::normalized_key(&self.next_kill_tick, &defaults.next_kill_tick);
        self.pause_resume = Self::normalized_key(&self.pause_resume, &defaults.pause_resume);
        self.enter_camera = Self::normalized_key(&self.enter_camera, &defaults.enter_camera);
        self.add_keyframe = Self::normalized_key(&self.add_keyframe, &defaults.add_keyframe);
        self.play_campath = Self::normalized_key(&self.play_campath, &defaults.play_campath);
        self.draw_campath = Self::normalized_key(&self.draw_campath, &defaults.draw_campath);
        self.start_recording = Self::normalized_key(&self.start_recording, &defaults.start_recording);
        self.stop_recording = Self::normalized_key(&self.stop_recording, &defaults.stop_recording);
        self.print_keyframes = Self::normalized_key(&self.print_keyframes, &defaults.print_keyframes);
        self.save_campath = Self::normalized_key(&self.save_campath, &defaults.save_campath);
        self.load_campath = Self::normalized_key(&self.load_campath, &defaults.load_campath);
        self.execute_director_action = Self::normalized_key(
            &self.execute_director_action,
            &defaults.execute_director_action,
        );
        self.overlay_panel_toggle = Self::normalized_key(
            &self.overlay_panel_toggle,
            &defaults.overlay_panel_toggle,
        );
        self.overlay_interaction_toggle = Self::normalized_key(
            &self.overlay_interaction_toggle,
            &defaults.overlay_interaction_toggle,
        );

        let mut seen = BTreeSet::new();
        let keys = [
            &self.advance_time, &self.toggle_hud, &self.show_help, &self.back_one_second,
            &self.safe_restart, &self.next_kill_tick, &self.pause_resume, &self.enter_camera,
            &self.add_keyframe, &self.play_campath, &self.draw_campath,
            &self.start_recording, &self.stop_recording,
            &self.print_keyframes, &self.save_campath,
            &self.load_campath,
            &self.execute_director_action,
            &self.overlay_panel_toggle,
            &self.overlay_interaction_toggle,
        ];
        if keys.iter().any(|key| !seen.insert(key.to_ascii_lowercase())) {
            *self = defaults;
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            output_directory: PathBuf::new(), item_schema: PathBuf::new(), tf2_executable: PathBuf::new(),
            hlae_executable: PathBuf::new(), ffmpeg_executable: PathBuf::new(), recording_output_directory: PathBuf::new(), manual_hlae_launch_options: String::new(), performance_profile: "High".into(),
            lead_seconds: 8, outro_seconds: 3,
            capture_fps: 120, jpg_quality: 90, recording_format: "MP4 - Standard".into(), resolution: "2560x1440".into(),
            mp4_compatibility: "DaVinci Resolve / Universal".into(), mp4_video_codec: "H.264 / libx264".into(),
            mp4_pixel_format: "yuv420p".into(), mp4_h264_profile: "High".into(), mp4_crf: 18,
            mp4_encoder_preset: "medium".into(), mp4_audio_codec: "AAC".into(), mp4_audio_bitrate_kbps: 192,
            avi_video_codec: "Original HLAE Raw".into(), avi_pixel_format: "HLAE Native".into(), dnxhr_profile: "HQ".into(),
            dx_level: "98".into(), skybox: "Default".into(), hud: "Kill notices only".into(),
            viewmodels: "On".into(), viewmodel_fov: 70, maximum_graphics: true, motion_blur: true,
            disable_hit_sounds: true, disable_voice_chat: true, minimal_hud: true, disable_combat_text: true,
            disable_crosshair: true, disable_crosshair_switching: true, hud_player_model: false,
            isolate_custom_resources: true, disable_announcer_voices: true, disable_applause_sounds: true,
            disable_domination_sounds: true, mirv_shortcuts: MirvShortcuts::default(), custom_resources: Vec::new(),
        }
    }
}

impl AppSettings {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("TF2FragDemoHelper")
            .join("settings.json")
    }

    pub fn load() -> Self {
        let mut settings: Self = std::fs::read(Self::path())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        settings.normalize_encoding_options();
        settings.normalize_recording_options();
        settings
    }

    pub fn normalize_encoding_options(&mut self) {
        if !matches!(self.mp4_compatibility.as_str(), "DaVinci Resolve / Universal" | "Maximum Color Quality" | "Custom") {
            self.mp4_compatibility = "DaVinci Resolve / Universal".into();
        }
        self.mp4_video_codec = "H.264 / libx264".into();
        if !matches!(self.mp4_pixel_format.as_str(), "yuv420p" | "yuv422p" | "yuv444p") {
            self.mp4_pixel_format = "yuv420p".into();
        }
        self.mp4_h264_profile = match self.mp4_pixel_format.as_str() {
            "yuv422p" => "High 4:2:2",
            "yuv444p" => "High 4:4:4 Predictive",
            _ => "High",
        }.into();
        self.mp4_crf = self.mp4_crf.min(35);
        if !matches!(self.mp4_encoder_preset.as_str(), "veryfast" | "faster" | "fast" | "medium" | "slow" | "slower") {
            self.mp4_encoder_preset = "medium".into();
        }
        self.mp4_audio_codec = "AAC".into();
        if !matches!(self.mp4_audio_bitrate_kbps, 192 | 256 | 320) {
            self.mp4_audio_bitrate_kbps = 192;
        }
        if !matches!(self.avi_video_codec.as_str(), "Original HLAE Raw" | "FFV1 Lossless" | "HuffYUV Lossless") {
            self.avi_video_codec = "Original HLAE Raw".into();
        }
        self.avi_pixel_format = match self.avi_video_codec.as_str() {
            "FFV1 Lossless" if matches!(self.avi_pixel_format.as_str(), "bgr0" | "yuv422p" | "yuv444p") => self.avi_pixel_format.clone(),
            "FFV1 Lossless" => "bgr0".into(),
            "HuffYUV Lossless" if matches!(self.avi_pixel_format.as_str(), "rgb24" | "yuv422p") => self.avi_pixel_format.clone(),
            "HuffYUV Lossless" => "rgb24".into(),
            _ => "HLAE Native".into(),
        };
        if !matches!(self.dnxhr_profile.as_str(), "LB" | "SQ" | "HQ" | "HQX" | "444") {
            self.dnxhr_profile = "HQ".into();
        }
    }

    pub fn normalize_recording_options(&mut self) {
        self.dx_level = match self.dx_level.split_whitespace().next().unwrap_or_default().to_ascii_lowercase().as_str() {
            "98" => "98",
            "95" => "95",
            "90" => "90",
            "81" => "81",
            "80" => "80",
            _ => "Default",
        }.into();

        const SKYBOXES: [&str; 12] = [
            "Default", "realsky1", "realsky3", "sky27", "sky41", "sky56",
            "sky_sky1_01", "sky_sky2_01", "sky_sky3_01", "sky_sky4_01",
            "sky_sky5_01", "sky_sky6_01",
        ];
        self.skybox = SKYBOXES.iter()
            .find(|option| option.eq_ignore_ascii_case(&self.skybox))
            .copied()
            .unwrap_or("Default")
            .into();

        const HUDS: [&str; 4] = [
            "Kill notices only", "Keep current", "Default TF2 HUD", "Medic recording HUD",
        ];
        self.hud = HUDS.iter()
            .find(|option| option.eq_ignore_ascii_case(&self.hud))
            .copied()
            .unwrap_or("Kill notices only")
            .into();

        self.viewmodels = if self.viewmodels.eq_ignore_ascii_case("Off") { "Off" } else { "On" }.into();
        self.mirv_shortcuts.normalize();
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, serde_json::to_vec_pretty(self)?)?;
        if let Err(first_error) = std::fs::rename(&temporary, &path) {
            // Windows does not replace an existing destination with rename.
            // Keep the fully written temporary file until the old settings file
            // has been removed, then complete the replacement.
            if path.is_file() {
                std::fs::remove_file(&path)?;
                std::fs::rename(&temporary, &path)?;
            } else {
                return Err(first_error.into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod settings_tests {
    use super::*;

    #[test]
    fn older_settings_without_encoding_fields_receive_safe_defaults() {
        let mut settings: AppSettings = serde_json::from_str(r#"{"recording_format":"MP4 - Standard","capture_fps":240}"#)
            .expect("older settings deserialize");
        settings.normalize_encoding_options();
        assert_eq!(settings.recording_format, "MP4 - Standard");
        assert_eq!(settings.capture_fps, 240);
        assert_eq!(settings.mp4_compatibility, "DaVinci Resolve / Universal");
        assert_eq!(settings.mp4_pixel_format, "yuv420p");
        assert_eq!(settings.mp4_h264_profile, "High");
        assert_eq!(settings.mp4_crf, 18);
        assert_eq!(settings.mp4_encoder_preset, "medium");
        assert_eq!(settings.mp4_audio_bitrate_kbps, 192);
        assert_eq!(settings.avi_video_codec, "Original HLAE Raw");
        assert_eq!(settings.avi_pixel_format, "HLAE Native");
        assert_eq!(settings.dnxhr_profile, "HQ");
    }

    #[test]
    fn legacy_recording_choices_migrate_to_selectable_values() {
        let mut settings = AppSettings {
            dx_level: "98 (highest)".into(),
            skybox: "default".into(),
            hud: "medic recording hud".into(),
            viewmodels: "Default".into(),
            ..AppSettings::default()
        };
        settings.normalize_recording_options();
        assert_eq!(settings.dx_level, "98");
        assert_eq!(settings.skybox, "Default");
        assert_eq!(settings.hud, "Medic recording HUD");
        assert_eq!(settings.viewmodels, "On");
    }

    #[test]
    fn invalid_recording_choices_fall_back_safely() {
        let mut settings = AppSettings {
            dx_level: "unsupported".into(),
            skybox: "missing_sky".into(),
            hud: "missing hud".into(),
            viewmodels: "missing".into(),
            ..AppSettings::default()
        };
        settings.normalize_recording_options();
        assert_eq!(settings.dx_level, "Default");
        assert_eq!(settings.skybox, "Default");
        assert_eq!(settings.hud, "Kill notices only");
        assert_eq!(settings.viewmodels, "On");
    }

    #[test]
    fn mirv_shortcuts_keep_the_pdf_defaults() {
        let shortcuts = MirvShortcuts::default();
        assert_eq!(shortcuts.advance_time, "[");
        assert_eq!(shortcuts.toggle_hud, "]");
        assert_eq!(shortcuts.show_help, "1");
        assert_eq!(shortcuts.save_campath, "=");
        assert_eq!(shortcuts.load_campath, "F8");
        assert_eq!(shortcuts.draw_campath, "/");
        assert_eq!(shortcuts.execute_director_action, "'");
        assert_eq!(shortcuts.overlay_panel_toggle, "C");
        assert_eq!(shortcuts.overlay_interaction_toggle, "F11");
        assert_eq!(shortcuts.overlay_panel_toggle_default_version, 3);
    }

    #[test]
    fn legacy_overlay_defaults_migrate_once_to_c() {
        let mut shortcuts: MirvShortcuts = serde_json::from_str(
            r#"{"overlay_panel_toggle":"HOME"}"#,
        )
        .unwrap();
        shortcuts.normalize();
        assert_eq!(shortcuts.overlay_panel_toggle, "C");
        assert_eq!(shortcuts.overlay_panel_toggle_default_version, 3);

        shortcuts.overlay_panel_toggle = "HOME".into();
        shortcuts.normalize();
        assert_eq!(shortcuts.overlay_panel_toggle, "HOME");

        let mut y_default: MirvShortcuts = serde_json::from_str(
            r#"{"overlay_panel_toggle":"Y","overlay_panel_toggle_default_version":1}"#,
        )
        .unwrap();
        y_default.normalize();
        assert_eq!(y_default.overlay_panel_toggle, "C");
        assert_eq!(y_default.overlay_panel_toggle_default_version, 3);

        let mut s_default: MirvShortcuts = serde_json::from_str(
            r#"{"overlay_panel_toggle":"S","overlay_panel_toggle_default_version":2}"#,
        )
        .unwrap();
        s_default.normalize();
        assert_eq!(s_default.overlay_panel_toggle, "C");
        assert_eq!(s_default.overlay_panel_toggle_default_version, 3);

        let mut custom: MirvShortcuts = serde_json::from_str(
            r#"{"overlay_panel_toggle":"Q","overlay_panel_toggle_default_version":1}"#,
        )
        .unwrap();
        custom.normalize();
        assert_eq!(custom.overlay_panel_toggle, "Q");
        assert_eq!(custom.overlay_panel_toggle_default_version, 3);

        let mut current_custom: MirvShortcuts = serde_json::from_str(
            r#"{"overlay_panel_toggle":"S","overlay_panel_toggle_default_version":3}"#,
        )
        .unwrap();
        current_custom.normalize();
        assert_eq!(current_custom.overlay_panel_toggle, "S");
    }

    #[test]
    fn mirv_shortcuts_reject_every_arrow_key_name() {
        for arrow in ["UPARROW", "downarrow", "LeftArrow", "RIGHTARROW"] {
            let mut shortcuts = MirvShortcuts::default();
            shortcuts.advance_time = arrow.into();
            shortcuts.normalize();
            assert_eq!(shortcuts.advance_time, "[");
            assert!(!shortcuts.advance_time.eq_ignore_ascii_case(arrow));
        }
    }

    #[test]
    fn mirv_shortcuts_accept_captured_printable_and_named_keys() {
        let mut shortcuts = MirvShortcuts::default();
        shortcuts.advance_time = "q".into();
        shortcuts.toggle_hud = "f12".into();
        shortcuts.normalize();
        assert_eq!(shortcuts.advance_time, "q");
        assert_eq!(shortcuts.toggle_hud, "F12");
    }

}

#[cfg(test)]
mod candidate_tag_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn primary_tag_uses_the_first_tag_on_the_earliest_tick() {
        let candidate = Candidate {
            tags: vec!["projectile_kill".into(), "confirmed_airshot".into(), "late_round".into()],
            tick_tags: vec![TickTagGroup {
                demo_tick: 1_000,
                server_ticks: vec![9_000],
                tags: vec!["projectile_kill".into(), "confirmed_airshot".into()],
            }],
            sequence_tags: vec!["late_round".into()],
            score_breakdown: vec![
                json!({"reason":"state_confirmed_airshot","points":20.0}),
                json!({"reason":"projectile_sequence","points":8.0}),
                json!({"reason":"late_round","points":8.0}),
            ],
            ..Candidate::default()
        };
        assert_eq!(candidate.inferred_primary_tag(), "projectile_kill");
    }

    #[test]
    fn future_matching_tag_identifiers_are_automatically_eligible() {
        let candidate = Candidate {
            tags: vec!["future_combo".into(), "projectile_kill".into()],
            sequence_tags: vec!["future_combo".into()],
            score_breakdown: vec![
                json!({"reason":"future_combo","points":40.0}),
                json!({"reason":"projectile_sequence","points":8.0}),
            ],
            ..Candidate::default()
        };
        assert_eq!(candidate.inferred_primary_tag(), "future_combo");
    }
}
