use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DIRECTOR_SESSION_SCHEMA: u32 = 2;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DirectorSession {
    pub schema_version: u32,
    pub candidate_id: String,
    pub demo_file: String,
    pub map_name: String,
    pub start_tick: i64,
    pub end_tick: i64,
    pub cues: Vec<DirectorCue>,
    pub whole_candidate_tags: Vec<String>,
    pub shortcuts: Vec<DirectorShortcut>,
    pub campath_file: PathBuf,
    pub output_directory: PathBuf,
    pub control: DirectorControl,
}

impl DirectorSession {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != DIRECTOR_SESSION_SCHEMA {
            bail!(
                "unsupported Director session schema {} (expected {})",
                self.schema_version,
                DIRECTOR_SESSION_SCHEMA
            );
        }
        if self.end_tick <= self.start_tick {
            bail!("Director session end tick must be after its start tick");
        }
        if self
            .cues
            .iter()
            .any(|cue| cue.tick < self.start_tick || cue.tick > self.end_tick)
        {
            bail!("a Director cue falls outside the selected clip window");
        }
        Ok(())
    }

    pub fn cue_position(&self, tick: i64) -> f32 {
        let span = (self.end_tick - self.start_tick).max(1) as f32;
        ((tick - self.start_tick) as f32 / span).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DirectorCue {
    pub tick: i64,
    pub label: String,
    pub tags: Vec<String>,
    pub victims: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DirectorShortcut {
    /// Stable identifier used for workflow summaries such as record order.
    pub id: String,
    /// The normalized TF2 key name written to the temporary manual-session CFG.
    pub key: String,
    /// Short, user-facing action label rendered in the overlay.
    pub label: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum DirectorControl {
    HotkeysOnly,
    LocalBridge { endpoint: String, token: String },
}

impl Default for DirectorControl {
    fn default() -> Self {
        Self::HotkeysOnly
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeRequest {
    GetState,
    SetPaused { paused: bool },
    SkipSeconds { seconds: f64 },
    SetCamera { pose: CameraPose },
    AddKeyframe,
    ReplaceKeyframe { id: u64, pose: CameraPose },
    RemoveKeyframe { id: u64 },
    EnableCampath { enabled: bool },
    DrawCampath { enabled: bool },
    SaveCampath { path: PathBuf },
    LoadCampath { path: PathBuf },
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CameraPose {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pitch: f64,
    pub yaw: f64,
    pub roll: f64,
    pub fov: f64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BridgeState {
    pub demo_tick: i64,
    pub demo_time: f64,
    pub paused: bool,
    pub camera: CameraPose,
    pub campath_enabled: bool,
    pub campath_keys: Vec<CampathKey>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CampathKey {
    pub id: u64,
    pub demo_tick: i64,
    pub demo_time: f64,
    pub camera: CameraPose,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> DirectorSession {
        DirectorSession {
            schema_version: DIRECTOR_SESSION_SCHEMA,
            candidate_id: "r2-p7-t12000".into(),
            start_tick: 10_000,
            end_tick: 14_000,
            cues: vec![DirectorCue {
                tick: 12_000,
                label: "Kill 1".into(),
                tags: vec!["confirmed_airshot".into()],
                victims: vec!["Medic".into()],
            }],
            shortcuts: vec![DirectorShortcut {
                id: "add_keyframe".into(),
                key: "7".into(),
                label: "Add keyframe".into(),
            }],
            ..DirectorSession::default()
        }
    }

    #[test]
    fn validates_and_positions_cues() {
        let session = session();
        session.validate().unwrap();
        assert_eq!(session.cue_position(12_000), 0.5);
    }

    #[test]
    fn rejects_cues_outside_the_clip() {
        let mut session = session();
        session.cues[0].tick = 15_000;
        assert!(session.validate().is_err());
    }

    #[test]
    fn bridge_requests_round_trip_as_tagged_json() {
        let request = BridgeRequest::SkipSeconds { seconds: 0.25 };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("skip_seconds"));
        assert_eq!(serde_json::from_str::<BridgeRequest>(&json).unwrap(), request);
    }

    #[test]
    fn shortcuts_round_trip_with_the_session() {
        let session = session();
        let json = serde_json::to_string(&session).unwrap();
        let restored: DirectorSession = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.shortcuts[0].key, "7");
        assert_eq!(restored.shortcuts[0].label, "Add keyframe");
    }
}
