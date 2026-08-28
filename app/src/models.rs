use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

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
    pub performance_profile: String,
    pub lead_seconds: u32,
    pub outro_seconds: u32,
    pub capture_fps: u32,
    pub jpg_quality: u8,
    pub recording_format: String,
    pub camera_mode: String,
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
    pub custom_resources: Vec<PathBuf>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            output_directory: PathBuf::new(), item_schema: PathBuf::new(), tf2_executable: PathBuf::new(),
            hlae_executable: PathBuf::new(), ffmpeg_executable: PathBuf::new(), recording_output_directory: PathBuf::new(), performance_profile: "High".into(),
            lead_seconds: 8, outro_seconds: 3,
            capture_fps: 120, jpg_quality: 90, recording_format: "MP4 - Standard".into(), camera_mode: "Original Camera".into(), resolution: "2560x1440".into(),
            mp4_compatibility: "DaVinci Resolve / Universal".into(), mp4_video_codec: "H.264 / libx264".into(),
            mp4_pixel_format: "yuv420p".into(), mp4_h264_profile: "High".into(), mp4_crf: 18,
            mp4_encoder_preset: "medium".into(), mp4_audio_codec: "AAC".into(), mp4_audio_bitrate_kbps: 192,
            avi_video_codec: "Original HLAE Raw".into(), avi_pixel_format: "HLAE Native".into(), dnxhr_profile: "HQ".into(),
            dx_level: "98".into(), skybox: "Default".into(), hud: "Kill notices only".into(),
            viewmodels: "On".into(), viewmodel_fov: 70, maximum_graphics: true, motion_blur: true,
            disable_hit_sounds: true, disable_voice_chat: true, minimal_hud: true, disable_combat_text: true,
            disable_crosshair: true, disable_crosshair_switching: true, hud_player_model: false,
            isolate_custom_resources: true, disable_announcer_voices: true, disable_applause_sounds: true,
            disable_domination_sounds: true, custom_resources: Vec::new(),
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
        self.camera_mode = if self.camera_mode.eq_ignore_ascii_case("Cinematic Kill Shot") {
            "Cinematic Kill Shot"
        } else {
            "Original Camera"
        }
        .into();
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
}
