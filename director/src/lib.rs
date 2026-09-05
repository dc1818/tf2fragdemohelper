use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DIRECTOR_SESSION_SCHEMA: u32 = 7;
pub const DIRECTOR_TICK_MARKER_PREFIX: &str = "TF2FRAG_DIRECTOR_TICK";
pub const DIRECTOR_TICK_OFFSET_PREFIX: &str = "TF2FRAG_DIRECTOR_TICK_OFFSET";
pub const DIRECTOR_KEYFRAME_BEGIN_PREFIX: &str = "TF2FRAG_DIRECTOR_KEYFRAMES_BEGIN";
pub const DIRECTOR_KEYFRAME_END_PREFIX: &str = "TF2FRAG_DIRECTOR_KEYFRAMES_END";
pub const DIRECTOR_KEYFRAME_DIRTY_MARKER: &str = "TF2FRAG_DIRECTOR_KEYFRAMES_DIRTY";
pub const DIRECTOR_ACTION_ACK_PREFIX: &str = "TF2FRAG_DIRECTOR_ACTION_ACK";
pub const DIRECTOR_POLL_READY_MARKER: &str = "TF2FRAG_DIRECTOR_POLL_READY";
pub const DIRECTOR_POLL_UNAVAILABLE_MARKER: &str = "TF2FRAG_DIRECTOR_POLL_UNAVAILABLE";
pub const DIRECTOR_ACTION_FILE_PREFIX: &str = "tf2fragdemohelper_director_action";
pub const DIRECTOR_ACTION_SLOTS: u16 = 64;

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
    /// TF2's temporary console log. The Director tails only its own tick markers.
    pub telemetry_log: PathBuf,
    pub telemetry_marker_prefix: String,
    /// Temporary TF2 cfg directory used by the injection-free Director command queue.
    pub command_cfg_directory: PathBuf,
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
        if self.telemetry_marker_prefix.trim().is_empty() {
            bail!("Director telemetry marker prefix cannot be empty");
        }
        if self.command_cfg_directory.as_os_str().is_empty() {
            bail!("Director command cfg directory cannot be empty");
        }
        if let DirectorControl::LocalRcon { endpoint, password } = &self.control {
            let address = endpoint
                .parse::<std::net::SocketAddr>()
                .map_err(|_| anyhow::anyhow!("Director RCON endpoint is invalid"))?;
            if !address.ip().is_loopback() {
                bail!("Director RCON endpoint must use the loopback interface");
            }
            if password.len() < 16
                || !password
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
            {
                bail!("Director RCON password must be a session-generated hex secret");
            }
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
    LocalRcon { endpoint: String, password: String },
    /// Director writes a private CFG action slot that TF2 polls from its own
    /// command buffer. The slot advances only after TF2 echoes the action
    /// sequence acknowledgement. A dedicated bind remains available only when
    /// TF2 reports that its `wait` command is unavailable.
    CfgMailbox,
}

impl Default for DirectorControl {
    fn default() -> Self {
        Self::HotkeysOnly
    }
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
            telemetry_log: PathBuf::from("tf/tf2fragdemohelper_recording.log"),
            telemetry_marker_prefix: DIRECTOR_TICK_MARKER_PREFIX.into(),
            command_cfg_directory: PathBuf::from("tf/cfg"),
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
    fn shortcuts_round_trip_with_the_session() {
        let session = session();
        let json = serde_json::to_string(&session).unwrap();
        let restored: DirectorSession = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.shortcuts[0].key, "7");
        assert_eq!(restored.shortcuts[0].label, "Add keyframe");
        assert_eq!(
            restored.telemetry_marker_prefix,
            DIRECTOR_TICK_MARKER_PREFIX
        );
    }

    #[test]
    fn local_rcon_is_restricted_to_a_loopback_endpoint() {
        let mut session = session();
        session.control = DirectorControl::LocalRcon {
            endpoint: "127.0.0.1:32145".into(),
            password: "0123456789abcdef0123456789abcdef".into(),
        };
        session.validate().unwrap();

        session.control = DirectorControl::LocalRcon {
            endpoint: "192.168.1.10:32145".into(),
            password: "0123456789abcdef0123456789abcdef".into(),
        };
        assert!(session.validate().is_err());
    }

    #[test]
    fn cfg_mailbox_round_trips_with_the_session() {
        let mut session = session();
        session.control = DirectorControl::CfgMailbox;
        session.validate().unwrap();
        let json = serde_json::to_string(&session).unwrap();
        let restored: DirectorSession = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.control, DirectorControl::CfgMailbox);
    }
}
