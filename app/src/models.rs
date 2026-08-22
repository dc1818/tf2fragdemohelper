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
    pub analysis_scope: String,
    #[serde(default)]
    pub pov_player_user_id: Option<i64>,
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

    pub fn searchable_text(&self, recorded: bool) -> String {
        format!(
            "{} {} {} {} {} {} {} {} {} {}",
            self.source_demo,
            self.map_name,
            self.attacker_class,
            self.attacker_team,
            self.attacker_user_id,
            self.tags.join(" "),
            self.demo_context.mode,
            self.demo_context.mode_label,
            self.demo_context.capture_type,
            if recorded { "recorded" } else { "unrecorded" },
        )
        .to_lowercase()
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WorkerSample {
    pub kind: String,
    pub machine_id: String,
    pub workers: usize,
    pub input_bytes: u64,
    pub wall_seconds: f64,
    pub throughput_mib_s: f64,
    pub peak_memory_bytes: u64,
    pub succeeded: bool,
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
    pub parse_workers_override: Option<usize>,
    pub analysis_workers_override: Option<usize>,
    pub lead_seconds: u32,
    pub outro_seconds: u32,
    pub capture_fps: u32,
    pub jpg_quality: u8,
    pub recording_format: String,
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
            hlae_executable: PathBuf::new(), ffmpeg_executable: PathBuf::new(), recording_output_directory: PathBuf::new(),
            parse_workers_override: None, analysis_workers_override: None, lead_seconds: 8, outro_seconds: 3,
            capture_fps: 120, jpg_quality: 90, recording_format: "MP4 - Standard".into(), resolution: "2560x1440".into(),
            dx_level: "98 (highest)".into(), skybox: "Default".into(), hud: "Kill notices only".into(),
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
        std::fs::read(Self::path())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(temporary, path)?;
        Ok(())
    }
}
