use crate::{
    models::{AppSettings, Candidate},
    preflight::{disk_space_for, format_bytes, require_disk_space},
};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, OnceLock},
    thread,
    time::{Duration, Instant, SystemTime},
};
use tf2_mirv_director::{
    DirectorControl, DirectorCue, DirectorSession, DirectorShortcut, DIRECTOR_SESSION_SCHEMA,
    DIRECTOR_TICK_MARKER_PREFIX,
};
use walkdir::WalkDir;
use zip::ZipArchive;

const PROFILE_FOLDER: &str = "tf2fragdemohelper_recording";
const PROFILE_CFG: &str = "tf2fragdemohelper_recording_profile.cfg";
const RESOURCE_CACHE_VERSION: &str = "bundled_resources_v2";
const RECORDING_FLUSH_TICKS: i64 = 133;
const VDM_ACTION_GAP_TICKS: i64 = 2;
const MANUAL_SEEK_STEP_TICKS: i64 = 15_000;
const TF2_ABSENT_CONFIRMATIONS: u8 = 8;
const MAX_RECOVERY_DIRECTORY_ENTRIES: usize = 512;
const MAX_RECOVERY_SESSIONS_PER_STARTUP: usize = 32;
const MAX_RECOVERY_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
const MAX_RECOVERY_CLIPS_PER_SESSION: usize = 1_000;
const MAX_AUTOMATIC_RECOVERY_ATTEMPTS: u32 = 3;
const MAX_RECORDING_INDEX_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RECORDING_INDEX_ENTRIES: usize = 500_000;
const RECOVERY_ATTEMPTS_FILE: &str = "automatic_recovery_attempts.txt";
const RECOVERY_DISABLED_FILE: &str = "AUTOMATIC_RECOVERY_DISABLED.txt";

#[derive(Clone)]
struct DemoSignatureCacheEntry {
    length: u64,
    modified: Option<SystemTime>,
    signature: String,
}

static DEMO_SIGNATURE_CACHE: OnceLock<Mutex<HashMap<PathBuf, DemoSignatureCacheEntry>>> =
    OnceLock::new();
static PORTABLE_DEMO_SIGNATURE_CACHE: OnceLock<Mutex<HashMap<PathBuf, DemoSignatureCacheEntry>>> =
    OnceLock::new();

#[derive(Clone, Debug)]
pub enum RecordingProgress {
    Status(String),
    ClipStarted {
        candidate_id: String,
        current: usize,
        total: usize,
    },
    ClipCompleted {
        candidate_id: String,
        output_path: PathBuf,
    },
    Finished {
        completed: usize,
        failed: usize,
        session: Option<PathBuf>,
    },
    ManualFinished {
        output_path: PathBuf,
        session: Option<PathBuf>,
    },
}

pub type RecordingProgressSink = Arc<dyn Fn(RecordingProgress) + Send + Sync>;

#[derive(Clone, Debug)]
pub struct ManualHlaeLaunch {
    pub target_tick: i64,
    pub output_path: PathBuf,
    pub session: PathBuf,
    pub director_session: PathBuf,
}

#[derive(Clone, Debug, Default)]
pub struct RecordingRecoveryReport {
    pub scanned_sessions: usize,
    pub recovered_clips: usize,
    pub indexed_clips: usize,
    pub removed_sessions: usize,
    pub retained_sessions: usize,
    pub deferred_sessions: usize,
    pub disabled_sessions: usize,
    pub errors: Vec<String>,
}

impl RecordingRecoveryReport {
    pub fn summary(&self) -> String {
        if self.scanned_sessions == 0
            && self.deferred_sessions == 0
            && self.disabled_sessions == 0
            && self.errors.is_empty()
        {
            return "No interrupted recording outputs needed recovery".into();
        }
        let mut summary = format!(
            "Recording recovery finished: {} session(s) checked, {} clip(s) finalized, {} existing output(s) indexed, {} completed session(s) cleaned up, {} session(s) retained",
            self.scanned_sessions,
            self.recovered_clips,
            self.indexed_clips,
            self.removed_sessions,
            self.retained_sessions,
        );
        if self.disabled_sessions > 0 {
            summary.push_str(&format!(
                "; {} damaged/repeatedly failing session(s) disabled from automatic recovery",
                self.disabled_sessions
            ));
        }
        if self.deferred_sessions > 0 {
            summary.push_str(&format!(
                "; {} additional session(s) deferred to a later launch",
                self.deferred_sessions
            ));
        }
        if !self.errors.is_empty() {
            summary.push_str(&format!(
                "; {} issue(s) remain in the retained session logs",
                self.errors.len()
            ));
        }
        summary
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct StoredRecordingManifest {
    output_format: String,
    ffmpeg_executable: PathBuf,
    game_directory: PathBuf,
    encoding: Option<StoredMp4Encoding>,
    avi_encoding: Option<StoredAviEncoding>,
    dnxhr_encoding: Option<StoredDnxhrEncoding>,
    clips: Vec<StoredRecordingClip>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct StoredMp4Encoding {
    compatibility: String,
    video_codec: String,
    pixel_format: String,
    h264_profile: String,
    crf: u8,
    encoder_preset: String,
    audio_codec: String,
    audio_bitrate_kbps: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct StoredAviEncoding {
    video_codec: String,
    pixel_format: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct StoredDnxhrEncoding {
    profile: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EffectiveMp4Encoding {
    compatibility: String,
    video_codec: String,
    pixel_format: String,
    h264_profile: String,
    ffmpeg_profile: String,
    crf: Option<u8>,
    encoder_preset: Option<String>,
    audio_codec: String,
    audio_bitrate_kbps: u32,
    custom_hlae_preset: bool,
}

impl EffectiveMp4Encoding {
    fn summary(&self, resolution: &str, fps: u32) -> String {
        let quality = match (self.crf, self.encoder_preset.as_deref()) {
            (Some(crf), Some(preset)) => format!("Quality: CRF {crf}\nEncoder Preset: {preset}"),
            _ => "Quality: Existing HLAE afxFfmpeg preset".into(),
        };
        format!(
            "Video Encoding\n{} / {}\n{} @ {} FPS\nPixel Format: {}\n{}\nAudio: {} {} kbps\nCompatibility: {}",
            self.video_codec,
            self.h264_profile,
            resolution,
            fps,
            self.pixel_format,
            quality,
            self.audio_codec,
            self.audio_bitrate_kbps,
            self.compatibility,
        )
    }

    fn diagnostic(&self) -> String {
        let quality = match (self.crf, self.encoder_preset.as_deref()) {
            (Some(crf), Some(preset)) => {
                format!("[Recording] CRF: {crf}\n[Recording] Preset: {preset}")
            }
            _ => "[Recording] Quality: existing HLAE afxFfmpeg preset".into(),
        };
        format!(
            "[Recording] Encoder: libx264\n[Recording] H.264 Profile: {}\n[Recording] Pixel Format: {}\n{}\n[Recording] Audio: AAC {}k\n[Recording] Compatibility: {}",
            self.ffmpeg_profile,
            self.pixel_format,
            quality,
            self.audio_bitrate_kbps,
            self.compatibility,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EffectiveAviEncoding {
    display_codec: String,
    ffmpeg_codec: Option<String>,
    pixel_format: String,
    custom_hlae_preset: bool,
}

impl EffectiveAviEncoding {
    fn summary(&self, resolution: &str, fps: u32) -> String {
        format!(
            "Video Encoding\nAVI / {}\n{} @ {} FPS\nPixel Format: {}\nAudio: PCM 16-bit",
            self.display_codec, resolution, fps, self.pixel_format,
        )
    }

    fn diagnostic(&self) -> String {
        format!(
            "[Recording] Container: AVI\n[Recording] Encoder: {}\n[Recording] Pixel Format: {}\n[Recording] Audio: PCM s16le",
            self.ffmpeg_codec.as_deref().unwrap_or("existing HLAE afxFfmpegRaw preset"),
            self.pixel_format,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EffectiveDnxhrEncoding {
    profile: String,
    ffmpeg_profile: String,
    pixel_format: String,
    bit_depth: u8,
}

impl EffectiveDnxhrEncoding {
    fn summary(&self, resolution: &str, fps: u32) -> String {
        format!(
            "Video Encoding\nAvid DNxHR {} / MOV\n{} @ {} FPS\nPixel Format: {} ({}-bit)\nAudio: PCM 16-bit\nEditing intermediate: very large files",
            self.profile, resolution, fps, self.pixel_format, self.bit_depth,
        )
    }

    fn diagnostic(&self) -> String {
        format!(
            "[Recording] Container: MOV\n[Recording] Encoder: dnxhd\n[Recording] DNxHR Profile: {}\n[Recording] Pixel Format: {} ({}-bit)\n[Recording] Audio: PCM s16le",
            self.ffmpeg_profile, self.pixel_format, self.bit_depth,
        )
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct StoredRecordingClip {
    order: usize,
    source_demo: String,
    candidate_id: String,
    recording_key: String,
    demo_content_signature: String,
    start_tick: i64,
    end_tick: i64,
    candidate_clip_start_tick: Option<i64>,
    candidate_clip_end_tick: Option<i64>,
    attacker_user_id: i64,
    recording_identifier: String,
    expected_output_path: PathBuf,
    actual_output_path: Option<PathBuf>,
    output_fingerprint: Option<String>,
    working_path: Option<PathBuf>,
    frames_path: Option<PathBuf>,
    audio_path: Option<PathBuf>,
    native_capture_base: String,
    replace_existing: bool,
}

#[derive(Clone, Debug)]
pub struct RecordingSpaceEstimate {
    pub clip_count: usize,
    pub duration_seconds: f64,
    pub frame_count: u64,
    pub final_output_bytes: u64,
    pub peak_working_bytes: u64,
    pub safety_headroom_bytes: u64,
    pub required_free_bytes: u64,
    pub available_free_bytes: u64,
    pub output_volume: PathBuf,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub jpg_quality: u8,
    pub encoding_summary: Option<String>,
}

impl RecordingSpaceEstimate {
    pub fn has_enough_space(&self) -> bool {
        self.available_free_bytes >= self.required_free_bytes
    }

    pub fn ensure_space(&self, output: &Path) -> Result<()> {
        require_disk_space(output, self.required_free_bytes, "HLAE recording")?;
        Ok(())
    }

    pub fn summary(&self) -> String {
        let status = if self.has_enough_space() {
            "PASS"
        } else {
            "BLOCKED — insufficient free space"
        };
        let quality = if self.format == "JPG Image Sequence" {
            format!(" at quality {}", self.jpg_quality)
        } else {
            String::new()
        };
        let mut summary = format!(
            "Recording pre-flight ({status})\n{} clip(s), {:.1} seconds, {} frames\n{}x{} at {} FPS, {}{}\nEstimated finalized output: {}\nEstimated peak working use (including the largest in-progress encoded capture): {}\nSafety headroom (20%, minimum 1 GB): {}\nRequired free space: {}\nAvailable on {}: {}\nCompressed-video estimates vary with scene complexity.",
            self.clip_count,
            self.duration_seconds,
            format_number(self.frame_count),
            self.width,
            self.height,
            self.fps,
            self.format,
            quality,
            format_bytes(self.final_output_bytes as f64),
            format_bytes(self.peak_working_bytes as f64),
            format_bytes(self.safety_headroom_bytes as f64),
            format_bytes(self.required_free_bytes as f64),
            self.output_volume.display(),
            format_bytes(self.available_free_bytes as f64),
        );
        if let Some(encoding) = &self.encoding_summary {
            summary.push_str("\n\n");
            summary.push_str(encoding);
        }
        summary
    }
}

fn effective_mp4_encoding(settings: &AppSettings) -> Result<Option<EffectiveMp4Encoding>> {
    if !settings.recording_format.starts_with("MP4")
        || settings.recording_format.contains("Lossless")
    {
        return Ok(None);
    }
    let audio_bitrate_kbps = match settings.mp4_audio_bitrate_kbps {
        192 | 256 | 320 => settings.mp4_audio_bitrate_kbps,
        value => bail!("unsupported AAC bitrate: {value} kbps"),
    };
    let compatibility = settings.mp4_compatibility.as_str();
    if compatibility == "Maximum Color Quality" {
        return Ok(Some(EffectiveMp4Encoding {
            compatibility: compatibility.into(),
            video_codec: "H.264 / libx264".into(),
            pixel_format: "yuv444p".into(),
            h264_profile: "High 4:4:4 Predictive".into(),
            ffmpeg_profile: "high444".into(),
            crf: None,
            encoder_preset: None,
            audio_codec: "AAC".into(),
            audio_bitrate_kbps,
            custom_hlae_preset: false,
        }));
    }
    let (pixel_format, h264_profile, ffmpeg_profile, crf, encoder_preset) =
        if compatibility == "Custom" {
            if settings.mp4_video_codec != "H.264 / libx264" {
                bail!("unsupported MP4 video codec: {}", settings.mp4_video_codec);
            }
            if settings.mp4_audio_codec != "AAC" {
                bail!("unsupported MP4 audio codec: {}", settings.mp4_audio_codec);
            }
            let (profile_name, ffmpeg_name) = match settings.mp4_pixel_format.as_str() {
                "yuv420p" => ("High", "high"),
                "yuv422p" => ("High 4:2:2", "high422"),
                "yuv444p" => ("High 4:4:4 Predictive", "high444"),
                value => bail!("unsupported MP4 pixel format: {value}"),
            };
            if settings.mp4_h264_profile != profile_name {
                bail!(
                    "{} requires the {} H.264 profile",
                    settings.mp4_pixel_format,
                    profile_name
                );
            }
            if settings.mp4_crf > 35 {
                bail!("MP4 CRF must be between 0 and 35");
            }
            if !matches!(
                settings.mp4_encoder_preset.as_str(),
                "veryfast" | "faster" | "fast" | "medium" | "slow" | "slower"
            ) {
                bail!(
                    "unsupported x264 encoder preset: {}",
                    settings.mp4_encoder_preset
                );
            }
            (
                settings.mp4_pixel_format.as_str(),
                profile_name,
                ffmpeg_name,
                settings.mp4_crf,
                settings.mp4_encoder_preset.clone(),
            )
        } else if compatibility == "DaVinci Resolve / Universal" {
            ("yuv420p", "High", "high", 18, "medium".into())
        } else {
            bail!("unsupported MP4 compatibility preset: {compatibility}");
        };
    Ok(Some(EffectiveMp4Encoding {
        compatibility: compatibility.into(),
        video_codec: "H.264 / libx264".into(),
        pixel_format: pixel_format.into(),
        h264_profile: h264_profile.into(),
        ffmpeg_profile: ffmpeg_profile.into(),
        crf: Some(crf),
        encoder_preset: Some(encoder_preset),
        audio_codec: "AAC".into(),
        audio_bitrate_kbps,
        custom_hlae_preset: true,
    }))
}

fn effective_avi_encoding(settings: &AppSettings) -> Result<Option<EffectiveAviEncoding>> {
    if settings.recording_format != "AVI - Raw" {
        return Ok(None);
    }
    let encoding = match settings.avi_video_codec.as_str() {
        "Original HLAE Raw" => EffectiveAviEncoding {
            display_codec: "Uncompressed Raw (original preset)".into(),
            ffmpeg_codec: None,
            pixel_format: "HLAE Native".into(),
            custom_hlae_preset: false,
        },
        "FFV1 Lossless" => EffectiveAviEncoding {
            display_codec: "FFV1 Lossless".into(),
            ffmpeg_codec: Some("ffv1".into()),
            pixel_format: match settings.avi_pixel_format.as_str() {
                "bgr0" | "yuv422p" | "yuv444p" => settings.avi_pixel_format.clone(),
                value => bail!("unsupported FFV1 pixel format: {value}"),
            },
            custom_hlae_preset: true,
        },
        "HuffYUV Lossless" => EffectiveAviEncoding {
            display_codec: "HuffYUV Lossless".into(),
            ffmpeg_codec: Some("huffyuv".into()),
            pixel_format: match settings.avi_pixel_format.as_str() {
                "rgb24" | "yuv422p" => settings.avi_pixel_format.clone(),
                value => bail!("unsupported HuffYUV pixel format: {value}"),
            },
            custom_hlae_preset: true,
        },
        value => bail!("unsupported AVI video codec: {value}"),
    };
    if settings.avi_video_codec == "Original HLAE Raw"
        && settings.avi_pixel_format != encoding.pixel_format
    {
        bail!(
            "{} requires the {} pixel format",
            settings.avi_video_codec,
            encoding.pixel_format
        );
    }
    Ok(Some(encoding))
}

fn effective_dnxhr_encoding(settings: &AppSettings) -> Result<Option<EffectiveDnxhrEncoding>> {
    if settings.recording_format != "MOV - DNxHR" {
        return Ok(None);
    }
    let (ffmpeg_profile, pixel_format, bit_depth) = match settings.dnxhr_profile.as_str() {
        "LB" => ("dnxhr_lb", "yuv422p", 8),
        "SQ" => ("dnxhr_sq", "yuv422p", 8),
        "HQ" => ("dnxhr_hq", "yuv422p", 8),
        "HQX" => ("dnxhr_hqx", "yuv422p10le", 10),
        "444" => ("dnxhr_444", "gbrp10le", 10),
        value => bail!("unsupported DNxHR profile: {value}"),
    };
    Ok(Some(EffectiveDnxhrEncoding {
        profile: settings.dnxhr_profile.clone(),
        ffmpeg_profile: ffmpeg_profile.into(),
        pixel_format: pixel_format.into(),
        bit_depth,
    }))
}

fn effective_encoding_summary(
    settings: &AppSettings,
    resolution: &str,
    fps: u32,
) -> Result<Option<String>> {
    if let Some(encoding) = effective_mp4_encoding(settings)? {
        return Ok(Some(encoding.summary(resolution, fps)));
    }
    if let Some(encoding) = effective_avi_encoding(settings)? {
        return Ok(Some(encoding.summary(resolution, fps)));
    }
    if let Some(encoding) = effective_dnxhr_encoding(settings)? {
        return Ok(Some(encoding.summary(resolution, fps)));
    }
    Ok(None)
}

fn encoded_extension(settings: &AppSettings) -> &'static str {
    if settings.recording_format.contains("AVI") {
        "avi"
    } else if settings.recording_format == "MOV - DNxHR" {
        "mov"
    } else {
        "mp4"
    }
}

fn encoded_media_name(settings: &AppSettings) -> &'static str {
    match encoded_extension(settings) {
        "avi" => "video.avi",
        "mov" => "video.mov",
        _ => "video.mp4",
    }
}

fn encoded_muxing_name(settings: &AppSettings) -> &'static str {
    match encoded_extension(settings) {
        "avi" => "video_muxing.avi",
        "mov" => "video_muxing.mov",
        _ => "video_muxing.mp4",
    }
}

fn mp4_encoding_manifest(settings: &AppSettings) -> Option<StoredMp4Encoding> {
    settings
        .recording_format
        .starts_with("MP4")
        .then(|| StoredMp4Encoding {
            compatibility: settings.mp4_compatibility.clone(),
            video_codec: settings.mp4_video_codec.clone(),
            pixel_format: settings.mp4_pixel_format.clone(),
            h264_profile: settings.mp4_h264_profile.clone(),
            crf: settings.mp4_crf,
            encoder_preset: settings.mp4_encoder_preset.clone(),
            audio_codec: settings.mp4_audio_codec.clone(),
            audio_bitrate_kbps: settings.mp4_audio_bitrate_kbps,
        })
}

fn avi_encoding_manifest(settings: &AppSettings) -> Option<StoredAviEncoding> {
    (settings.recording_format == "AVI - Raw").then(|| StoredAviEncoding {
        video_codec: settings.avi_video_codec.clone(),
        pixel_format: settings.avi_pixel_format.clone(),
    })
}

fn dnxhr_encoding_manifest(settings: &AppSettings) -> Option<StoredDnxhrEncoding> {
    (settings.recording_format == "MOV - DNxHR").then(|| StoredDnxhrEncoding {
        profile: settings.dnxhr_profile.clone(),
    })
}

pub fn estimate_recording_space(
    candidates: &[Candidate],
    settings: &AppSettings,
) -> Result<RecordingSpaceEstimate> {
    if candidates.is_empty() {
        bail!("select one or more candidates");
    }
    if settings.recording_output_directory.as_os_str().is_empty() {
        bail!("choose a recording output location");
    }
    let (width, height) = parse_resolution(&settings.resolution);
    let fps = settings.capture_fps.max(1);
    let encoding_summary = effective_encoding_summary(settings, &settings.resolution, fps)?;
    let pixels = width as f64 * height as f64;
    let quality = settings.jpg_quality.clamp(1, 100) as f64;
    let bytes_per_frame = match settings.recording_format.as_str() {
        "JPG Image Sequence" => pixels * (0.025 + 0.0032 * quality),
        "TGA Image Sequence" => pixels * 3.0 + 18.0,
        "MP4 - Standard" => pixels * 0.01125,
        "MP4 - Compatible" => pixels * 0.00875,
        "MP4 - Lossless" => pixels * 1.25,
        "AVI - Raw" => pixels * 3.0,
        "MOV - DNxHR" => {
            let reference_bytes = match settings.dnxhr_profile.as_str() {
                "LB" => 188_416.0,
                "SQ" => 602_112.0,
                "HQ" | "HQX" => 909_312.0,
                "444" => 1_822_720.0,
                _ => 909_312.0,
            };
            reference_bytes * pixels / (1920.0 * 1080.0)
        }
        _ => pixels * 0.01125,
    };
    let image_sequence = settings.recording_format.contains("Image");
    let raw_audio_bytes_per_second = 48_000.0 * 2.0 * 2.0;
    let final_audio_bytes_per_second = if image_sequence
        || settings.recording_format.contains("AVI")
        || settings.recording_format == "MOV - DNxHR"
    {
        raw_audio_bytes_per_second
    } else {
        settings.mp4_audio_bitrate_kbps as f64 * 1_000.0 / 8.0
    };
    let mut duration_seconds = 0.0;
    let mut frame_count = 0u64;
    let mut final_output_bytes = 0u64;
    let mut largest_encoded_working_clip = 0u64;
    for candidate in candidates {
        let (start, end) = clip_window(candidate, settings);
        let seconds = (end - start).max(1) as f64 / 66.666_666_7;
        let frames = (seconds * fps as f64).ceil().max(1.0) as u64;
        let video_bytes = (frames as f64 * bytes_per_frame).ceil() as u64;
        let raw_audio_bytes = (seconds * raw_audio_bytes_per_second).ceil() as u64;
        let final_audio_bytes = (seconds * final_audio_bytes_per_second).ceil() as u64;
        duration_seconds += seconds;
        frame_count = frame_count.saturating_add(frames);
        final_output_bytes = final_output_bytes
            .saturating_add(video_bytes)
            .saturating_add(final_audio_bytes);
        if !image_sequence {
            largest_encoded_working_clip =
                largest_encoded_working_clip.max(video_bytes.saturating_add(raw_audio_bytes));
        }
    }
    let metadata_allowance = 32 * 1024 * 1024 + candidates.len() as u64 * 64 * 1024;
    let peak_working_bytes = final_output_bytes
        .saturating_add(largest_encoded_working_clip)
        .saturating_add(metadata_allowance);
    let safety_headroom_bytes = (peak_working_bytes / 5).max(1024 * 1024 * 1024);
    let required_free_bytes = peak_working_bytes.saturating_add(safety_headroom_bytes);
    let disk = disk_space_for(&settings.recording_output_directory)?;
    Ok(RecordingSpaceEstimate {
        clip_count: candidates.len(),
        duration_seconds,
        frame_count,
        final_output_bytes,
        peak_working_bytes,
        safety_headroom_bytes,
        required_free_bytes,
        available_free_bytes: disk.available_bytes,
        output_volume: disk.mount_point,
        format: settings.recording_format.clone(),
        width,
        height,
        fps,
        jpg_quality: settings.jpg_quality,
        encoding_summary,
    })
}

#[derive(Clone, Debug)]
struct PreparedClip {
    order: usize,
    candidate: Candidate,
    start_tick: i64,
    end_tick: i64,
    config_base: String,
    capture_base: String,
    recording_key: String,
    demo_signature: String,
    recording_identifier: String,
    working_path: Option<PathBuf>,
    final_output_path: PathBuf,
    frames_path: Option<PathBuf>,
    audio_path: Option<PathBuf>,
    replace_existing: bool,
}

#[derive(Default)]
struct RecordingMarkerState {
    batch_finished: bool,
    finalized_markers: usize,
    started_clips: HashSet<String>,
}

struct TemporaryPathGuard {
    paths: Vec<PathBuf>,
    armed: bool,
}

impl TemporaryPathGuard {
    fn new(paths: Vec<PathBuf>) -> Self {
        Self { paths, armed: true }
    }
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for TemporaryPathGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        for path in &self.paths {
            if path.is_dir() {
                let _ = fs::remove_dir_all(path);
            } else if path.is_file() {
                let _ = fs::remove_file(path);
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RecordingProfileSession {
    session_id: String,
    game_directory: PathBuf,
    backup_directory: PathBuf,
    original_custom_existed: bool,
    original_profile_existed: bool,
    original_profile_cfg_existed: bool,
    isolated_custom: bool,
    tf_process_name: String,
    #[serde(default)]
    original_config_existed: bool,
    #[serde(default)]
    original_video_existed: bool,
    #[serde(default)]
    original_offline_cfg_existed: bool,
    #[serde(default)]
    original_cfg_folder_existed: bool,
    #[serde(default)]
    original_cfg_overrides_existed: bool,
    #[serde(default)]
    hitsound_files: Vec<PathBuf>,
    #[serde(default)]
    dx_level_was_applied: bool,
    #[serde(default)]
    original_dx_level_existed: bool,
    #[serde(default)]
    original_dx_level_type: String,
    #[serde(default)]
    original_dx_level_data: String,
    #[serde(default)]
    temporary_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RecordingEntry {
    pub recording_key: String,
    pub candidate_id: String,
    pub demo_signature: String,
    pub clip_start_tick: i64,
    pub clip_end_tick: i64,
    pub output_path: PathBuf,
    pub output_fingerprint: String,
    #[serde(alias = "recorded_utc")]
    pub completed_utc: String,
}

#[derive(Clone)]
pub struct RecordingIndex {
    path: PathBuf,
    entries: HashMap<String, RecordingEntry>,
}

impl RecordingIndex {
    pub fn empty() -> Self {
        Self {
            path: index_path(),
            entries: HashMap::new(),
        }
    }

    pub fn load() -> Self {
        let path = index_path();
        Self::load_from_path(
            &path,
            MAX_RECORDING_INDEX_BYTES,
            MAX_RECORDING_INDEX_ENTRIES,
        )
    }

    fn load_from_path(path: &Path, max_bytes: u64, max_entries: usize) -> Self {
        // A retained or corrupted index must never allocate without a bound or
        // delay startup indefinitely. A truncated final line is simply ignored.
        let entries = File::open(path)
            .ok()
            .map(|file| BufReader::new(file.take(max_bytes.saturating_add(1))))
            .into_iter()
            .flat_map(|reader| reader.lines().map_while(|line| line.ok()))
            .take(max_entries)
            .filter_map(|line| serde_json::from_str::<RecordingEntry>(&line).ok())
            .map(|entry| (entry.recording_key.clone(), entry))
            .collect();
        Self {
            path: path.to_path_buf(),
            entries,
        }
    }

    /// Fast UI-facing status lookup.  It deliberately does not walk the output directory:
    /// walking it once for every candidate made large exported batches appear to freeze.
    /// A single background reconciliation pass handles outputs that have not yet been indexed.
    pub fn is_recorded_indexed(&self, candidate: &Candidate) -> bool {
        recording_keys(candidate).into_iter().flatten().any(|key| {
            self.entries
                .get(&key)
                .is_some_and(|entry| output_still_exists(&entry.output_path))
        })
    }

    /// Existing helper-owned outputs for a candidate. Used only for an
    /// explicitly confirmed re-record, and only deleted after the replacement
    /// has finalized and been indexed successfully.
    fn existing_outputs(&self, candidate: &Candidate) -> Vec<PathBuf> {
        let mut outputs = HashSet::new();
        for key in recording_keys(candidate).into_iter().flatten() {
            if let Some(entry) = self.entries.get(&key) {
                if output_still_exists(&entry.output_path) {
                    outputs.insert(entry.output_path.clone());
                }
            }
        }
        outputs.into_iter().collect()
    }

    /// Scan a recording directory once and add the outputs whose generated clip base names
    /// match candidates.  This preserves detection of pre-existing outputs without doing an
    /// expensive directory walk for every row in the candidate table.
    pub fn reconcile_output_root(&mut self, candidates: &[Candidate], root: &Path) -> usize {
        if !root.is_dir() {
            return 0;
        }
        let outputs = final_recording_outputs(root);
        let mut added = 0;
        for candidate in candidates {
            if self.is_recorded_indexed(candidate) {
                continue;
            }
            let key = match recording_key(candidate) {
                Ok(key) => key,
                Err(_) => continue,
            };
            let token = recording_key_token(&key);
            let candidate_id = sanitize(&candidate.candidate_id).to_lowercase();
            let old_ticks = format!("t{}-{}", candidate.clip_start_tick, candidate.clip_end_tick);
            if let Some(output) = outputs.iter().find(|path| {
                let name = recording_output_name(path);
                name.contains(&format!("__k{token}"))
                    || (name.contains(&candidate_id) && name.contains(&old_ticks))
            }) {
                if self.register(candidate, output.clone()).is_ok() {
                    added += 1;
                }
            }
        }

        let mut missing_fingerprints: HashMap<String, Vec<String>> = HashMap::new();
        for (key, entry) in &self.entries {
            if !entry.output_fingerprint.is_empty() && !output_still_exists(&entry.output_path) {
                missing_fingerprints
                    .entry(entry.output_fingerprint.clone())
                    .or_default()
                    .push(key.clone());
            }
        }
        let mut relocated = Vec::new();
        if !missing_fingerprints.is_empty() {
            for output in &outputs {
                if let Ok(fingerprint) = output_fingerprint(output) {
                    if let Some(keys) = missing_fingerprints.remove(&fingerprint) {
                        relocated.extend(keys.into_iter().map(|key| (key, output.clone())));
                    }
                }
            }
        }
        for (key, path) in relocated {
            if let Some(mut entry) = self.entries.get(&key).cloned() {
                entry.output_path = path;
                self.entries.insert(key, entry.clone());
                let _ = self.append(entry);
            }
        }
        added
    }

    pub fn merge_missing_entries(&mut self, other: &RecordingIndex) {
        for (key, entry) in &other.entries {
            let current_exists = self
                .entries
                .get(key)
                .is_some_and(|current| output_still_exists(&current.output_path));
            if !current_exists && output_still_exists(&entry.output_path) {
                self.entries.insert(key.clone(), entry.clone());
            }
        }
    }

    pub fn register(&mut self, candidate: &Candidate, output: PathBuf) -> Result<()> {
        self.register_with_fingerprint(candidate, output, None)
    }

    fn register_with_fingerprint(
        &mut self,
        candidate: &Candidate,
        output: PathBuf,
        fingerprint: Option<String>,
    ) -> Result<()> {
        let key = recording_key(candidate)?;
        let entry = RecordingEntry {
            recording_key: key,
            candidate_id: candidate.candidate_id.clone(),
            demo_signature: portable_demo_signature(Path::new(&candidate.source_demo))?,
            clip_start_tick: candidate.clip_start_tick,
            clip_end_tick: candidate.clip_end_tick,
            output_fingerprint: fingerprint
                .unwrap_or_else(|| output_fingerprint(&output).unwrap_or_default()),
            output_path: output,
            completed_utc: Utc::now().to_rfc3339(),
        };
        self.store_entry(entry)
    }

    fn register_recovered(
        &mut self,
        clip: &PreparedClip,
        output: PathBuf,
        fingerprint: String,
    ) -> Result<()> {
        if clip.recording_key.trim().is_empty() {
            bail!("the retained clip has no recording key");
        }
        let entry = RecordingEntry {
            recording_key: clip.recording_key.clone(),
            candidate_id: clip.candidate.candidate_id.clone(),
            demo_signature: clip.demo_signature.clone(),
            clip_start_tick: clip.candidate.clip_start_tick,
            clip_end_tick: clip.candidate.clip_end_tick,
            output_path: output,
            output_fingerprint: fingerprint,
            completed_utc: Utc::now().to_rfc3339(),
        };
        self.store_entry(entry)
    }

    fn store_entry(&mut self, entry: RecordingEntry) -> Result<()> {
        let key = entry.recording_key.clone();
        self.entries.insert(key, entry.clone());
        self.append(entry)
    }

    fn append(&self, entry: RecordingEntry) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        serde_json::to_writer(&mut output, &entry)?;
        output.write_all(b"\n")?;
        Ok(())
    }
}

pub fn preview_candidate(candidate: &Candidate, settings: &AppSettings) -> Result<i64> {
    if !cfg!(target_os = "windows") {
        bail!("TF2 demo preview is currently available only where the native TF2 client is installed");
    }
    let tf2 = settings.tf2_executable.as_path();
    if !tf2.is_file() {
        bail!("select tf_win64.exe in Settings");
    }
    validate_tf2_executable(tf2)?;
    let source = Path::new(&candidate.source_demo);
    if !source.is_file() {
        bail!("the selected candidate's original demo is missing: {}", source.display());
    }
    let tf_process_name = tf2.file_name().and_then(|name| name.to_str()).unwrap_or("tf_win64.exe").to_owned();
    if windows_process_is_running(&tf_process_name)
        || windows_process_is_running("tf.exe")
        || windows_process_is_running("tf_win64.exe")
    {
        bail!("close TF2 before previewing a candidate so the new demo and VDM seek are not ignored");
    }

    let game = tf2_game_directory(tf2)?;
    let demo_directory = game.join("demos/tf2fragdemohelper");
    fs::create_dir_all(&demo_directory)?;
    for entry in fs::read_dir(&demo_directory)? {
        let path = entry?.path();
        let is_stale_vdm = path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("vdm"))
            && path.file_stem().and_then(|value| value.to_str()).is_some_and(|value| value.starts_with("tf2fragdemohelper_temp_"));
        if is_stale_vdm {
            let _ = fs::remove_file(path);
        }
    }

    let source_stem = sanitize(source.file_stem().and_then(|value| value.to_str()).unwrap_or("candidate_demo"));
    let signature = demo_signature(source)?;
    let signature_prefix = signature.get(..12).unwrap_or(&signature);
    let staged_name = format!("tf2fragdemohelper_temp_{source_stem}_{signature_prefix}.dem");
    let staged_path = demo_directory.join(&staged_name);
    let source_length = fs::metadata(source)?.len();
    if !staged_path.is_file() || fs::metadata(&staged_path).map(|metadata| metadata.len()).unwrap_or_default() != source_length {
        fs::copy(source, &staged_path).with_context(|| format!("could not stage {} for TF2 preview", source.display()))?;
    }
    let staged_relative = format!("demos/tf2fragdemohelper/{staged_name}");

    let (target_tick, _) = clip_window(candidate, settings);
    let playback_vdm = staged_path.with_extension("vdm");
    fs::write(&playback_vdm, preview_vdm_text(candidate, target_tick))?;

    let cfg = game.join("cfg").join("tf2fragdemohelper_preview.cfg");
    let previous_preview_cfg = fs::read(&cfg).ok();
    fs::write(&cfg, format!("{}cl_predict 0\n", offline_cfg()))?;
    let working_directory = game.parent().filter(|path| path.is_dir()).unwrap_or(game.as_path());
    let mut launch = Command::new(tf2);
    launch.current_dir(working_directory)
        .args(["-insecure", "-novid", "-console", "-game"])
        .arg(&game)
        .args(["+sv_lan", "1", "+exec", "tf2fragdemohelper_preview.cfg", "+playdemo"])
        .arg(&staged_relative);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        launch.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = match launch.spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = fs::remove_file(&playback_vdm);
            let _ = fs::remove_file(&staged_path);
            match &previous_preview_cfg {
                Some(bytes) => { let _ = fs::write(&cfg, bytes); }
                None => { let _ = fs::remove_file(&cfg); }
            }
            return Err(error).with_context(|| format!(
                "could not launch TF2 from {} with game directory {}",
                tf2.display(), game.display()
            ));
        }
    };
    thread::spawn(move || {
        let _ = child.wait();
        thread::sleep(Duration::from_millis(2500));
        while windows_process_is_running(&tf_process_name) {
            thread::sleep(Duration::from_secs(1));
        }
        let _ = fs::remove_file(playback_vdm);
        let _ = fs::remove_file(staged_path);
        match previous_preview_cfg {
            Some(bytes) => { let _ = fs::write(cfg, bytes); }
            None => { let _ = fs::remove_file(cfg); }
        }
    });
    Ok(target_tick)
}

fn director_display_tag(tag: &str) -> String {
    tag.trim().replace(['_', '-'], " ")
}

fn build_director_session(
    candidate: &Candidate,
    target_tick: i64,
    end_tick: i64,
    output_path: &Path,
    telemetry_log: &Path,
    settings: &AppSettings,
) -> DirectorSession {
    let mut victims_by_tick = BTreeMap::<i64, BTreeSet<String>>::new();
    for kill in &candidate.kills {
        let Some(tick) = kill
            .get("demo_tick")
            .or_else(|| kill.get("tick"))
            .and_then(serde_json::Value::as_i64)
        else {
            continue;
        };
        if let Some(name) = kill
            .get("victim_name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            victims_by_tick.entry(tick).or_default().insert(name.into());
        }
    }

    let mut cue_ticks = candidate.point_of_kill_ticks.clone();
    cue_ticks.extend(candidate.tick_tags.iter().map(|group| group.demo_tick));
    cue_ticks.sort_unstable();
    cue_ticks.dedup();
    let cues = cue_ticks
        .into_iter()
        .enumerate()
        .map(|(index, tick)| DirectorCue {
            tick,
            label: format!("FRAG CUE {}", index + 1),
            tags: candidate
                .tick_tags
                .iter()
                .find(|group| group.demo_tick == tick)
                .map(|group| {
                    group
                        .tags
                        .iter()
                        .map(|tag| director_display_tag(tag))
                        .collect()
                })
                .unwrap_or_default(),
            victims: victims_by_tick
                .remove(&tick)
                .map(|names| names.into_iter().collect())
                .unwrap_or_default(),
        })
        .collect();

    let mut shortcuts = settings.mirv_shortcuts.clone();
    shortcuts.normalize();
    let shortcuts = [
        ("advance_time", &shortcuts.advance_time, "Advance 0.25 sec"),
        ("toggle_hud", &shortcuts.toggle_hud, "Toggle HUD"),
        ("show_help", &shortcuts.show_help, "Show controls"),
        ("back_one_second", &shortcuts.back_one_second, "Back 1 sec"),
        ("safe_restart", &shortcuts.safe_restart, "Safe clip restart"),
        ("next_kill_tick", &shortcuts.next_kill_tick, "Next frag tick"),
        ("pause_resume", &shortcuts.pause_resume, "Pause / resume"),
        ("enter_camera", &shortcuts.enter_camera, "MIRV camera"),
        ("add_keyframe", &shortcuts.add_keyframe, "Add keyframe"),
        ("play_campath", &shortcuts.play_campath, "Play campath"),
        ("start_recording", &shortcuts.start_recording, "Start recording"),
        ("stop_recording", &shortcuts.stop_recording, "Stop recording"),
        ("print_keyframes", &shortcuts.print_keyframes, "Print keyframes"),
        ("save_campath", &shortcuts.save_campath, "Save campath XML"),
        (
            "overlay_panel_toggle",
            &shortcuts.overlay_panel_toggle,
            "Hide / show cue panel",
        ),
    ]
    .into_iter()
    .map(|(id, key, label)| DirectorShortcut {
        id: id.into(),
        key: key.clone(),
        label: label.into(),
    })
    .collect();

    DirectorSession {
        schema_version: DIRECTOR_SESSION_SCHEMA,
        candidate_id: candidate.candidate_id.clone(),
        demo_file: Path::new(&candidate.source_demo)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&candidate.source_demo)
            .to_owned(),
        map_name: candidate.map_name.clone(),
        start_tick: target_tick,
        end_tick,
        cues,
        whole_candidate_tags: candidate
            .sequence_tags
            .iter()
            .map(|tag| director_display_tag(tag))
            .collect(),
        shortcuts,
        campath_file: output_path.join("camera_path.xml"),
        output_directory: output_path.to_owned(),
        telemetry_log: telemetry_log.to_owned(),
        telemetry_marker_prefix: DIRECTOR_TICK_MARKER_PREFIX.into(),
        control: DirectorControl::HotkeysOnly,
    }
}

fn write_director_session(
    candidate: &Candidate,
    target_tick: i64,
    end_tick: i64,
    output_path: &Path,
    telemetry_log: &Path,
    settings: &AppSettings,
) -> Result<PathBuf> {
    let session = build_director_session(
        candidate,
        target_tick,
        end_tick,
        output_path,
        telemetry_log,
        settings,
    );
    session.validate()?;
    let path = output_path.join("director_session.json");
    fs::write(&path, serde_json::to_vec_pretty(&session)?)
        .with_context(|| format!("could not write Director session {}", path.display()))?;
    Ok(path)
}

fn launch_director_companion(session_path: &Path) -> Result<Option<std::process::Child>> {
    let current = std::env::current_exe().context("could not locate the helper executable")?;
    let directory = current
        .parent()
        .context("could not locate the helper executable directory")?;
    let names: &[&str] = if cfg!(target_os = "windows") {
        &["TF2_MIRV_Director.exe", "tf2-mirv-director.exe"]
    } else {
        &["TF2_MIRV_Director", "tf2-mirv-director"]
    };
    let Some(executable) = names
        .iter()
        .map(|name| directory.join(name))
        .find(|path| path.is_file())
    else {
        return Ok(None);
    };
    let child = Command::new(&executable)
        .arg(session_path)
        .spawn()
        .with_context(|| format!("could not launch {}", executable.display()))?;
    Ok(Some(child))
}

pub fn launch_manual_hlae_candidate(
    candidate: &Candidate,
    settings: &AppSettings,
    progress: Option<RecordingProgressSink>,
) -> Result<ManualHlaeLaunch> {
    if !cfg!(target_os = "windows") {
        bail!("manual HLAE camera playback is Windows-only");
    }
    recover_interrupted_profile()?;
    if !settings.tf2_executable.is_file() {
        bail!("select tf_win64.exe in Settings");
    }
    if !settings.hlae_executable.is_file() {
        bail!("select hlae.exe in Settings");
    }
    validate_tf2_executable(&settings.tf2_executable)?;
    validate_named_executable(&settings.hlae_executable, &["hlae.exe"], "HLAE")?;
    if !settings.recording_format.contains("Image") {
        if !settings.ffmpeg_executable.is_file() {
            bail!("select ffmpeg.exe before using the manual recording hotkey with an encoded format");
        }
        validate_named_executable(&settings.ffmpeg_executable, &["ffmpeg.exe"], "FFmpeg")?;
    }
    validate_selected_encoder(settings)?;
    let source = Path::new(&candidate.source_demo);
    if !source.is_file() {
        bail!("the selected candidate's original demo is missing: {}", source.display());
    }

    let game = tf2_game_directory(&settings.tf2_executable)?;
    let hlae_root = settings
        .hlae_executable
        .parent()
        .context("could not find the HLAE directory")?;
    let x64 = settings
        .tf2_executable
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("tf_win64.exe"));
    let hook = if x64 {
        hlae_root.join("x64/AfxHookSource.dll")
    } else {
        hlae_root.join("AfxHookSource.dll")
    };
    if !hook.is_file() {
        bail!("required HLAE hook is missing: {}", hook.display());
    }
    let tf_process_name = settings
        .tf2_executable
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("tf_win64.exe")
        .to_owned();
    if windows_process_is_running(&tf_process_name)
        || windows_process_is_running("tf.exe")
        || windows_process_is_running("tf_win64.exe")
    {
        bail!("close TF2 before launching the isolated manual HLAE session");
    }

    fs::create_dir_all(&settings.recording_output_directory)?;
    let session = recording_sessions_root().join(format!(
        "tf2fragdemohelper_manual_{}",
        Utc::now().format("%Y%m%d_%H%M%S_%3f")
    ));
    fs::create_dir_all(&session)?;
    let session_name = session
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("tf2fragdemohelper_manual")
        .to_owned();
    let diagnostic_log = session.join("manual_hlae_diagnostics.log");
    let staged_root = game
        .join("demos/tf2fragdemohelper_manual")
        .join(&session_name);
    fs::create_dir_all(&staged_root)?;
    let mut staging_guard =
        TemporaryPathGuard::new(vec![staged_root.clone(), session.clone()]);

    let demo_stem = sanitize(
        source
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("candidate_demo"),
    );
    let candidate_name = sanitize(if candidate.candidate_id.trim().is_empty() {
        "candidate"
    } else {
        &candidate.candidate_id
    });
    let staged_name = format!("{demo_stem}__{candidate_name}.dem");
    let staged_path = staged_root.join(&staged_name);
    if fs::hard_link(source, &staged_path).is_err() {
        fs::copy(source, &staged_path)
            .with_context(|| format!("could not stage {}", source.display()))?;
    }
    let (target_tick, end_tick) = clip_window(candidate, settings);
    fs::write(
        staged_path.with_extension("vdm"),
        manual_hlae_vdm_text(candidate, target_tick, end_tick),
    )?;
    let staged_relative = format!(
        "demos/tf2fragdemohelper_manual/{session_name}/{staged_name}"
    );

    let output_path = settings
        .recording_output_directory
        .join("Manual HLAE")
        .join(candidate_output_category(candidate))
        .join(format!(
            "{demo_stem}__{candidate_name}__t{target_tick}-{end_tick}__{}",
            Utc::now().format("%Y%m%d_%H%M%S")
        ));
    fs::create_dir_all(&output_path)?;
    let telemetry_log = game.join("tf2fragdemohelper_recording.log");
    let _ = fs::remove_file(&telemetry_log);
    let director_session = write_director_session(
        candidate,
        target_tick,
        end_tick,
        &output_path,
        &telemetry_log,
        settings,
    )?;
    let launch_log = session.join("hlae_launch.log");
    let launch_log_file = File::create(&launch_log)?;
    let profile = stage_recording_profile(
        &game,
        &session_name,
        &tf_process_name,
        settings,
        vec![staged_root.clone(), session.clone()],
    )
    .context("could not stage the temporary TF2/HLAE profile")?;

    let cfg_root = game.join("cfg");
    let cfg_result = (|| -> Result<()> {
        fs::write(
            cfg_root.join("tf2fragdemohelper_manual.cfg"),
            manual_hotkey_cfg(candidate, target_tick, &staged_relative, settings),
        )?;
        fs::write(
            cfg_root.join("tf2fragdemohelper_manual_start.cfg"),
            manual_recording_start_cfg(settings, &output_path),
        )?;
        fs::write(
            cfg_root.join("tf2fragdemohelper_manual_stop.cfg"),
            manual_recording_stop_cfg(settings),
        )?;
        fs::write(
            cfg_root.join("tf2fragdemohelper_manual_save.cfg"),
            format!(
                "mirv_campath save \"{}\"\necho TF2FRAG_MANUAL_CAMPATH_SAVED\n",
                output_path.join("camera_path.xml").display().to_string().replace('\\', "/")
            ),
        )?;
        Ok(())
    })();
    if let Err(error) = cfg_result {
        let _ = restore_recording_profile(&profile);
        return Err(error).context("could not install the temporary manual HLAE hotkeys");
    }

    let (width, height) = parse_resolution(&settings.resolution);
    let dx_argument = if settings
        .dx_level
        .trim()
        .to_ascii_lowercase()
        .starts_with("default")
    {
        String::new()
    } else {
        format!(
            "-dxlevel {} ",
            settings.dx_level.split_whitespace().next().unwrap_or("98")
        )
    };
    let game_arguments = format!(
        "-steam -insecure +sv_lan 1 -novid -window -noborder -console -no_texture_stream -afxGame tf -w {width} -h {height} {dx_argument}+tf_delete_temp_files 0 +exec tf2fragdemohelper_offline.cfg +exec tf2fragdemohelper_recording_profile.cfg +exec tf2fragdemohelper_manual.cfg +playdemo {staged_relative}"
    );
    log_recording_diagnostic(
        &diagnostic_log,
        format!(
            "Manual HLAE session prepared; target tick {target_tick}; last requested tick {end_tick}; output {}; command line: {game_arguments}",
            output_path.display()
        ),
    );
    let launch_log_stdout = match launch_log_file.try_clone() {
        Ok(file) => file,
        Err(error) => {
            let _ = restore_recording_profile(&profile);
            return Err(error).context("could not prepare the HLAE launch log");
        }
    };
    let mut launch = Command::new(&settings.hlae_executable);
    launch
        .current_dir(hlae_root)
        .args(["-customLoader", "-autoStart", "-noGui", "-programPath"])
        .arg(&settings.tf2_executable)
        .arg("-cmdLine")
        .arg(game_arguments)
        .arg("-hookDllPath")
        .arg(hook)
        .stdout(Stdio::from(launch_log_stdout))
        .stderr(Stdio::from(launch_log_file));
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        launch.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = match launch.spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = restore_recording_profile(&profile);
            return Err(error).context("could not launch TF2 through HLAE");
        }
    };
    let mut director_child = match launch_director_companion(&director_session) {
        Ok(child) => child,
        Err(error) => {
            log_recording_diagnostic(
                &diagnostic_log,
                format!("WARNING: TF2 MIRV Director did not open: {error:#}"),
            );
            None
        }
    };

    let monitor_log = diagnostic_log.clone();
    let monitor_output = output_path.clone();
    let monitor_session = session.clone();
    let monitor_progress = progress.clone();
    staging_guard.disarm();
    thread::spawn(move || {
        if let Some(sink) = &monitor_progress {
            sink(RecordingProgress::Status(format!(
                "Manual HLAE paused at tick {target_tick} — 6 camera, 7 keyframe, 9/0 record"
            )));
        }
        let startup_deadline = Instant::now() + Duration::from_secs(90);
        let mut tf2_seen = false;
        while Instant::now() < startup_deadline {
            if windows_process_state(&profile.tf_process_name) == Some(true) {
                tf2_seen = true;
                break;
            }
            if child.try_wait().ok().flatten().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(250));
        }
        if tf2_seen {
            let mut absent_confirmations = 0u8;
            while absent_confirmations < TF2_ABSENT_CONFIRMATIONS {
                match windows_process_state(&profile.tf_process_name) {
                    Some(true) | None => absent_confirmations = 0,
                    Some(false) => absent_confirmations += 1,
                }
                thread::sleep(Duration::from_millis(500));
            }
        } else {
            log_recording_diagnostic(
                &monitor_log,
                "WARNING: TF2 was not observed after the HLAE launch",
            );
        }
        wait_for_hlae_shutdown(&mut child, &monitor_log);
        if let Some(director) = director_child.as_mut() {
            if director.try_wait().ok().flatten().is_none() {
                let _ = director.kill();
                let _ = director.wait();
            }
        }
        log_recording_diagnostic(&monitor_log, "TF2 closed; restoring the original TF2 profile");
        let session_for_logs = if let Err(error) = restore_recording_profile(&profile) {
            log_recording_diagnostic(&monitor_log, format!("ERROR: restore failed: {error}"));
            let _ = fs::write(
                profile.backup_directory.join("RESTORE_REQUIRED.txt"),
                error.to_string(),
            );
            Some(monitor_session)
        } else {
            None
        };
        if let Some(sink) = &monitor_progress {
            sink(RecordingProgress::ManualFinished {
                output_path: monitor_output,
                session: session_for_logs,
            });
        }
    });

    Ok(ManualHlaeLaunch {
        target_tick,
        output_path,
        session,
        director_session,
    })
}

pub fn prepare_hlae_batch(candidates: &[Candidate], settings: &AppSettings) -> Result<PathBuf> {
    if !cfg!(target_os = "windows") {
        bail!("HLAE recording is Windows-only; parsing, analysis, filtering, and export viewing remain cross-platform");
    }
    if candidates.is_empty() {
        bail!("select one or more candidates");
    }
    if !settings.hlae_executable.is_file() {
        bail!("select hlae.exe in Settings");
    }
    if !settings.tf2_executable.is_file() {
        bail!("select tf_win64.exe in Settings");
    }
    validate_named_executable(&settings.hlae_executable, &["hlae.exe"], "HLAE")?;
    validate_tf2_executable(&settings.tf2_executable)?;
    if !settings.recording_format.contains("Image") && !settings.ffmpeg_executable.is_file() {
        bail!("select ffmpeg.exe before encoded video recording");
    }
    if !settings.recording_format.contains("Image") {
        validate_named_executable(&settings.ffmpeg_executable, &["ffmpeg.exe"], "FFmpeg")?;
    }
    validate_selected_encoder(settings)?;
    let estimate = estimate_recording_space(candidates, settings)?;
    estimate.ensure_space(&settings.recording_output_directory)?;
    fs::create_dir_all(&settings.recording_output_directory)?;
    let sessions_root = recording_sessions_root();
    let output_disk = disk_space_for(&settings.recording_output_directory)?;
    let session_disk = disk_space_for(&sessions_root)?;
    if output_disk.mount_point != session_disk.mount_point {
        let session_working_bytes = estimate
            .peak_working_bytes
            .saturating_sub(estimate.final_output_bytes);
        let session_headroom = (session_working_bytes / 5).max(1024 * 1024 * 1024);
        require_disk_space(
            &sessions_root,
            session_working_bytes.saturating_add(session_headroom),
            "HLAE recording session data",
        )?;
    }
    fs::create_dir_all(&sessions_root)?;
    let session = sessions_root.join(format!(
        "tf2fragdemohelper_batch_{}",
        Utc::now().format("%Y%m%d_%H%M%S_%3f")
    ));
    fs::create_dir_all(&session)?;
    fs::write(session.join("offline_safety.cfg"), offline_cfg().as_bytes())?;
    fs::write(
        session.join("tf2fragdemohelper_recording_profile.cfg"),
        recording_profile_cfg(settings),
    )?;
    Ok(session)
}

pub fn launch_hlae_batch(
    candidates: &[Candidate],
    settings: &AppSettings,
    replace_existing: bool,
    progress: Option<RecordingProgressSink>,
) -> Result<PathBuf> {
    recover_interrupted_profile()?;
    let session = prepare_hlae_batch(candidates, settings)?;
    let diagnostic_log = session.join("hlae_recording_diagnostics.log");
    log_recording_diagnostic(&diagnostic_log, "Recording session created");
    if let Ok(estimate) = estimate_recording_space(candidates, settings) {
        let _ = fs::write(
            session.join("RECORDING_PRE_FLIGHT_ESTIMATE.txt"),
            format!("{}\n", estimate.summary()),
        );
        log_recording_diagnostic(&diagnostic_log, estimate.summary().replace('\n', " | "));
    }
    let game = tf2_game_directory(&settings.tf2_executable)?;
    let hlae_root = settings
        .hlae_executable
        .parent()
        .context("could not find the HLAE directory")?;
    let x64 = settings
        .tf2_executable
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("tf_win64.exe"));
    let hook = if x64 {
        hlae_root.join("x64/AfxHookSource.dll")
    } else {
        hlae_root.join("AfxHookSource.dll")
    };
    if !hook.is_file() {
        log_recording_diagnostic(
            &diagnostic_log,
            format!("ERROR: required HLAE hook is missing: {}", hook.display()),
        );
        bail!("required HLAE hook is missing: {}", hook.display());
    }

    let tf_process_name = settings
        .tf2_executable
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("tf_win64.exe");
    if windows_process_is_running(tf_process_name) {
        log_recording_diagnostic(
            &diagnostic_log,
            "ERROR: TF2 was already running; recording was not started",
        );
        bail!("close TF2 before starting an isolated recording session");
    }

    let session_name = session
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rust_session");
    let staged_root = game
        .join("demos/tf2fragdemohelper_batch")
        .join(session_name);
    let cfg_root = game.join("cfg/tf2fragdemohelper_batch").join(session_name);
    fs::create_dir_all(&staged_root)?;
    fs::create_dir_all(&cfg_root)?;
    let mut staging_guard = TemporaryPathGuard::new(vec![staged_root.clone(), cfg_root.clone()]);
    fs::create_dir_all(game.join("cfg"))?;

    let mut by_demo: BTreeMap<PathBuf, Vec<(usize, Candidate)>> = BTreeMap::new();
    for (order, candidate) in candidates.iter().cloned().enumerate() {
        let source = PathBuf::from(&candidate.source_demo);
        if source.is_file() {
            by_demo
                .entry(source)
                .or_default()
                .push((order + 1, candidate));
        }
    }
    if by_demo.is_empty() {
        log_recording_diagnostic(
            &diagnostic_log,
            "ERROR: no selected candidates referenced an existing demo",
        );
        bail!("none of the selected candidates reference an existing demo");
    }

    let unique_demo_count = by_demo.len();
    let mut groups = Vec::new();
    for (source, mut clips) in by_demo {
        clips.sort_by_key(|(_, candidate)| clip_window(candidate, settings).0);
        for pass in split_recording_passes(&clips, settings) {
            groups.push((source.clone(), pass));
        }
    }
    let prepared_clips = prepare_recording_clips(
        candidates,
        session_name,
        &session,
        settings,
        replace_existing,
    )
    .context("could not prepare the recording working paths")?;
    let prepared_by_order = prepared_clips
        .iter()
        .map(|clip| (clip.order, clip))
        .collect::<HashMap<_, _>>();
    write_recording_manifest(&session, &prepared_clips, settings, &game, "Pending", None)?;
    let mut staged_relatives = Vec::new();
    for (demo_index, (source, clips)) in groups.iter().enumerate() {
        let stem = sanitize(
            source
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("demo"),
        );
        let staged_name = format!("{:03}_{stem}.dem", demo_index + 1);
        let staged = staged_root.join(&staged_name);
        if fs::hard_link(source, &staged).is_err() {
            fs::copy(source, &staged)
                .with_context(|| format!("could not stage {}", source.display()))?;
        }
        let relative = format!("demos/tf2fragdemohelper_batch/{session_name}/{staged_name}");
        staged_relatives.push(relative);
        let first_window = clips
            .first()
            .map(|(_, candidate)| clip_window(candidate, settings));
        let last_window = clips
            .last()
            .map(|(_, candidate)| clip_window(candidate, settings));
        log_recording_diagnostic(
            &diagnostic_log,
            format!(
                "Playback pass {}/{}: {} candidate(s) from {} (first window {:?}, last window {:?})",
                demo_index + 1,
                groups.len(),
                clips.len(),
                source.display(),
                first_window,
                last_window
            ),
        );

        for (order, candidate) in clips {
            let base = format!(
                "{:03}_{}_t{}-{}",
                order,
                sanitize(&candidate.candidate_id),
                candidate.clip_start_tick,
                candidate.clip_end_tick
            );
            let prepared = prepared_by_order
                .get(order)
                .context("recording clip preparation did not match the demo queue")?;
            let start_cfg = recording_start_cfg(prepared, settings);
            fs::write(cfg_root.join(format!("{base}_start.cfg")), start_cfg)?;
            fs::write(
                cfg_root.join(format!("{base}_stop.cfg")),
                recording_stop_cfg(settings, &prepared.config_base),
            )?;
        }
    }

    for (demo_index, (_, clips)) in groups.iter().enumerate() {
        let staged = staged_root.join(
            Path::new(&staged_relatives[demo_index])
                .file_name()
                .unwrap(),
        );
        let next = staged_relatives.get(demo_index + 1).cloned();
        let vdm = vdm_text(clips, session_name, next.as_deref(), settings)
            .context("could not build the recording VDM")?;
        fs::write(staged.with_extension("vdm"), vdm)
            .context("could not write the recording VDM")?;
    }

    let (width, height) = parse_resolution(&settings.resolution);
    let dx_argument = if settings
        .dx_level
        .trim()
        .to_ascii_lowercase()
        .starts_with("default")
    {
        String::new()
    } else {
        format!(
            "-dxlevel {} ",
            settings.dx_level.split_whitespace().next().unwrap_or("98")
        )
    };
    let game_arguments = format!(
        "-steam -insecure +sv_lan 1 -novid -window -noborder -console -no_texture_stream -afxGame tf -w {width} -h {height} {dx_argument}+tf_delete_temp_files 0 +exec tf2fragdemohelper_offline.cfg +exec tf2fragdemohelper_recording_profile.cfg +playdemo {}",
        staged_relatives[0]
    );
    let encoding_log = match effective_mp4_encoding(settings)? {
        Some(encoding) => {
            let hlae_command = hlae_custom_mp4_preset_command(&encoding)
                .unwrap_or_else(|| "mirv_streams record screen settings afxFfmpeg".into());
            format!(
                "{}\n[Recording] HLAE FFmpeg command: {hlae_command}",
                encoding.diagnostic()
            )
        }
        None if settings.recording_format == "MOV - DNxHR" => {
            let encoding =
                effective_dnxhr_encoding(settings)?.context("DNxHR settings are missing")?;
            let hlae_command = hlae_custom_dnxhr_preset_command(&encoding);
            format!(
                "{}\n[Recording] HLAE FFmpeg command: {hlae_command}",
                encoding.diagnostic()
            )
        }
        None if settings.recording_format.contains("AVI") => {
            let encoding = effective_avi_encoding(settings)?.context("AVI settings are missing")?;
            let hlae_command = hlae_custom_avi_preset_command(&encoding)
                .unwrap_or_else(|| "mirv_streams record screen settings afxFfmpegRaw".into());
            format!(
                "{}\n[Recording] HLAE FFmpeg command: {hlae_command}",
                encoding.diagnostic()
            )
        }
        None if settings.recording_format.contains("Lossless") => {
            "[Recording] Encoder: existing lossless HLAE preset afxFfmpegLosslessBest".into()
        }
        None => "[Recording] Encoder: native TF2 image sequence".into(),
    };
    log_recording_diagnostic(&diagnostic_log, format!(
        "Prepared offline launch\nTF2 executable: {}\nHLAE executable: {}\nGame directory: {}\nHook DLL: {}\nHLAE options: -customLoader -autoStart -noGui -programPath <TF2> -cmdLine <shown below> -hookDllPath <hook DLL>\nCandidates: {}\nUnique demos: {}\nPlayback passes: {}\nFormat: {} at {} FPS\n{}\nInitial staged demo: {}\nTF2 command line: {}",
        settings.tf2_executable.display(), settings.hlae_executable.display(), game.display(), hook.display(), candidates.len(), unique_demo_count, groups.len(), settings.recording_format, settings.capture_fps, encoding_log, staged_relatives[0], game_arguments
    ));
    let profile = stage_recording_profile(
        game.as_path(),
        session_name,
        tf_process_name,
        settings,
        vec![staged_root.clone(), cfg_root.clone()],
    )
    .context("could not stage the temporary TF2 recording profile")?;
    log_recording_diagnostic(
        &diagnostic_log,
        format!(
            "Recording profile staged; original TF2 files are backed up in {}",
            profile.backup_directory.display()
        ),
    );
    let recording_log = game.join("tf2fragdemohelper_recording.log");
    let _ = fs::remove_file(&recording_log);
    let launch_log = session.join("hlae_launch.log");
    let launch_log_file = File::create(&launch_log)?;
    let mut launch = Command::new(&settings.hlae_executable);
    launch
        .current_dir(hlae_root)
        .args(["-customLoader", "-autoStart", "-noGui", "-programPath"])
        .arg(&settings.tf2_executable)
        .arg("-cmdLine")
        .arg(game_arguments)
        .arg("-hookDllPath")
        .arg(hook)
        .stdout(Stdio::from(launch_log_file.try_clone()?))
        .stderr(Stdio::from(launch_log_file));
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        launch.creation_flags(CREATE_NO_WINDOW);
    }
    let launch_result = launch.spawn();
    let mut child = match launch_result {
        Ok(child) => {
            log_recording_diagnostic(
                &diagnostic_log,
                format!(
                    "HLAE process started with PID {}. Console output is in {}",
                    child.id(),
                    launch_log.display()
                ),
            );
            if let Some(sink) = &progress {
                sink(RecordingProgress::Status(format!(
                    "HLAE started; recording {} selected clip(s)",
                    prepared_clips.len()
                )));
            }
            child
        }
        Err(error) => {
            log_recording_diagnostic(
                &diagnostic_log,
                format!("ERROR: HLAE process could not start: {error}"),
            );
            let _ = restore_recording_profile(&profile);
            return Err(error.into());
        }
    };
    let finalizer_settings = settings.clone();
    let finalizer_session = session.clone();
    let finalizer_game = game.clone();
    staging_guard.disarm();
    thread::spawn(move || {
        if let Some(sink) = &progress {
            sink(RecordingProgress::Status(
                "Recording active — waiting for TF2 batch completion".into(),
            ));
        }
        let (completed, failed) = finalize_recording_session(
            &mut child,
            &profile.tf_process_name,
            &finalizer_game,
            &finalizer_session,
            &prepared_clips,
            &finalizer_settings,
            progress.as_ref(),
            &diagnostic_log,
        );
        if let Some(sink) = &progress {
            sink(RecordingProgress::Status("Archiving recording logs".into()));
        }
        let tf2_console_log = finalizer_game.join("tf2fragdemohelper_recording.log");
        if tf2_console_log.is_file() {
            match fs::copy(&tf2_console_log, finalizer_session.join("tf2_console.log")) {
                Ok(_) => {
                    let _ = fs::remove_file(&tf2_console_log);
                }
                Err(error) => log_recording_diagnostic(
                    &diagnostic_log,
                    format!("WARNING: could not archive TF2 console log: {error}"),
                ),
            }
        }
        if let Some(sink) = &progress {
            sink(RecordingProgress::Status("Restoring TF2 files".into()));
        }
        let restore_succeeded = if let Err(error) = restore_recording_profile(&profile) {
            log_recording_diagnostic(&diagnostic_log, format!("ERROR: restore failed: {error}"));
            let _ = fs::write(
                profile.backup_directory.join("RESTORE_REQUIRED.txt"),
                error.to_string(),
            );
            false
        } else {
            log_recording_diagnostic(
                &diagnostic_log,
                "Restore verification passed; original TF2 files were restored",
            );
            true
        };
        let session_for_logs = if completed == prepared_clips.len()
            && failed == 0
            && restore_succeeded
        {
            match remove_completed_recording_session(&finalizer_session) {
                Ok(()) => None,
                Err(error) => {
                    log_recording_diagnostic(
                        &diagnostic_log,
                        format!("WARNING: completed session data could not be removed: {error}"),
                    );
                    Some(finalizer_session.clone())
                }
            }
        } else {
            Some(finalizer_session.clone())
        };
        if let Some(sink) = &progress {
            sink(RecordingProgress::Finished {
                completed,
                failed,
                session: session_for_logs,
            });
        }
    });
    Ok(session)
}

pub fn recover_interrupted_profile() -> Result<bool> {
    let pointer = active_profile_path();
    if !pointer.is_file() {
        return Ok(false);
    }
    let profile: RecordingProfileSession = serde_json::from_slice(&fs::read(&pointer)?)
        .context("the interrupted recording recovery file is invalid")?;
    if windows_process_is_running(&profile.tf_process_name) {
        bail!("a TF2 recording session is still active; close TF2 before reopening the helper");
    }
    if let Err(error) = restore_recording_profile(&profile) {
        let _ = fs::write(
            profile.backup_directory.join("RESTORE_REQUIRED.txt"),
            error.to_string(),
        );
        return Err(error);
    }
    Ok(true)
}

/// Match the previous application's FormClosing behavior: if the helper is
/// closed during recording, stop the TF2 instance launched for that session
/// and restore the original custom files and configuration before exiting.
pub fn shutdown_active_recording() -> Result<bool> {
    let pointer = active_profile_path();
    if !pointer.is_file() {
        return Ok(false);
    }
    let profile: RecordingProfileSession = serde_json::from_slice(&fs::read(&pointer)?)
        .context("the active recording recovery file is invalid")?;
    if windows_process_is_running(&profile.tf_process_name) {
        stop_windows_process(&profile.tf_process_name)?;
        for _ in 0..20 {
            if !windows_process_is_running(&profile.tf_process_name) {
                break;
            }
            thread::sleep(Duration::from_millis(250));
        }
        if windows_process_is_running(&profile.tf_process_name) {
            bail!(
                "TF2 did not close; original recording files remain safely backed up in {}",
                profile.backup_directory.display()
            );
        }
    }
    if let Err(error) = restore_recording_profile(&profile) {
        let _ = fs::write(
            profile.backup_directory.join("RESTORE_REQUIRED.txt"),
            error.to_string(),
        );
        return Err(error);
    }
    Ok(true)
}

fn stage_recording_profile(
    game: &Path,
    session_id: &str,
    tf_process_name: &str,
    settings: &AppSettings,
    temporary_paths: Vec<PathBuf>,
) -> Result<RecordingProfileSession> {
    let custom = game.join("custom");
    let cfg_folder = game.join("cfg");
    let cfg = game.join("cfg").join(PROFILE_CFG);
    let backup = game.join("tf2fragdemohelper_backups").join(session_id);
    let cfg_folder_backup = backup.join("cfg_original");
    let custom_backup = backup.join("custom_original");
    let profile = custom.join(PROFILE_FOLDER);
    let profile_backup = backup.join("custom_profile_original");
    let cfg_backup = backup.join(PROFILE_CFG);
    let config = game.join("cfg").join("config.cfg");
    let config_backup = backup.join("config.cfg");
    let video = game.join("cfg").join("video.txt");
    let video_backup = backup.join("video.txt");
    let offline_config_path = game.join("cfg").join("tf2fragdemohelper_offline.cfg");
    let offline_cfg_backup = backup.join("tf2fragdemohelper_offline.cfg");
    fs::create_dir_all(&backup)?;
    let hitsound_files = backup_hitsound_files(&custom, &backup.join("hitsounds_original"))?;
    let (
        dx_level_was_applied,
        original_dx_level_existed,
        original_dx_level_type,
        original_dx_level_data,
    ) = capture_dx_level(&settings.dx_level)?;
    let session = RecordingProfileSession {
        session_id: session_id.to_owned(),
        game_directory: game.to_path_buf(),
        backup_directory: backup,
        original_custom_existed: custom.is_dir(),
        original_profile_existed: profile.is_dir(),
        original_profile_cfg_existed: cfg.is_file(),
        isolated_custom: settings.isolate_custom_resources,
        tf_process_name: tf_process_name.to_owned(),
        original_config_existed: config.is_file(),
        original_video_existed: video.is_file(),
        original_offline_cfg_existed: offline_config_path.is_file(),
        original_cfg_folder_existed: cfg_folder.is_dir(),
        original_cfg_overrides_existed: cfg_folder.join("overrides").is_dir(),
        hitsound_files,
        dx_level_was_applied,
        original_dx_level_existed,
        original_dx_level_type,
        original_dx_level_data,
        temporary_paths,
    };
    write_active_profile(&session)?;

    let result = (|| -> Result<()> {
        // Use a complete working copy of tf/cfg. TF2 can write more than
        // config.cfg while it is running; moving the original folder aside
        // preserves cfg/overrides and every other user script byte-for-byte.
        if session.original_cfg_folder_existed {
            fs::rename(&cfg_folder, &cfg_folder_backup)
                .context("could not back up TF2's cfg folder")?;
            fs::create_dir_all(&cfg_folder)?;
            copy_path(&cfg_folder_backup, &cfg_folder)?;
        } else {
            fs::create_dir_all(&cfg_folder)?;
        }
        // Keep the older individual-file backup route only for a legacy TF2
        // install without a cfg directory. It also lets interrupted sessions
        // created by an older build recover normally.
        if !session.original_cfg_folder_existed {
            if session.original_profile_cfg_existed {
                fs::rename(&cfg, &cfg_backup)?;
            }
            if session.original_config_existed {
                fs::copy(&config, &config_backup)?;
            }
            if session.original_video_existed {
                fs::copy(&video, &video_backup)?;
            }
            if session.original_offline_cfg_existed {
                fs::copy(&offline_config_path, &offline_cfg_backup)?;
            }
        }
        if session.isolated_custom {
            if session.original_custom_existed {
                fs::rename(&custom, &custom_backup)
                    .context("could not back up TF2's custom folder")?;
            }
            fs::create_dir_all(&custom)?;
            for selected in &settings.custom_resources {
                let source = if selected.exists() {
                    selected.clone()
                } else {
                    custom_backup.join(selected.file_name().unwrap_or_default())
                };
                let destination = custom.join(source.file_name().unwrap_or_default());
                copy_path(&source, &destination)?;
            }
        } else {
            fs::create_dir_all(&custom)?;
            if session.original_profile_existed {
                fs::rename(&profile, &profile_backup)?;
            }
        }
        install_recording_resources(&custom, settings)?;
        install_recording_message_suppression(&custom)?;
        fs::write(&cfg, recording_profile_cfg(settings))?;
        fs::write(&offline_config_path, offline_cfg())?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = restore_recording_profile(&session);
        return Err(error);
    }
    Ok(session)
}

fn restore_recording_profile(session: &RecordingProfileSession) -> Result<()> {
    let _restore_guard = restore_lock().lock();
    if !active_profile_path().is_file() {
        return Ok(());
    }
    let custom = session.game_directory.join("custom");
    let cfg_folder = session.game_directory.join("cfg");
    let cfg_folder_backup = session.backup_directory.join("cfg_original");
    let cfg = session.game_directory.join("cfg").join(PROFILE_CFG);
    let custom_backup = session.backup_directory.join("custom_original");
    let profile = custom.join(PROFILE_FOLDER);
    let profile_backup = session.backup_directory.join("custom_profile_original");
    let cfg_backup = session.backup_directory.join(PROFILE_CFG);
    let config = session.game_directory.join("cfg").join("config.cfg");
    let config_backup = session.backup_directory.join("config.cfg");
    let video = session.game_directory.join("cfg").join("video.txt");
    let video_backup = session.backup_directory.join("video.txt");
    let offline_cfg = session
        .game_directory
        .join("cfg")
        .join("tf2fragdemohelper_offline.cfg");
    let offline_cfg_backup = session
        .backup_directory
        .join("tf2fragdemohelper_offline.cfg");
    let cfg_snapshot_exists = cfg_folder_backup.is_dir();
    let config_existed = session.original_config_existed || config_backup.is_file();
    let video_existed = session.original_video_existed || video_backup.is_file();
    let offline_cfg_existed = session.original_offline_cfg_existed || offline_cfg_backup.is_file();

    if session.isolated_custom {
        if custom.exists() {
            fs::remove_dir_all(&custom)?;
        }
        if session.original_custom_existed {
            fs::rename(&custom_backup, &custom).context("could not restore TF2's custom folder")?;
        }
    } else {
        if profile.exists() {
            fs::remove_dir_all(&profile)?;
        }
        if session.original_profile_existed {
            fs::rename(&profile_backup, &profile)?;
        }
        if !session.original_custom_existed
            && custom.is_dir()
            && fs::read_dir(&custom)?.next().is_none()
        {
            fs::remove_dir(&custom)?;
        }
    }
    restore_hitsound_files(session)?;
    if cfg_snapshot_exists {
        if cfg_folder.exists() {
            fs::remove_dir_all(&cfg_folder)?;
        }
        if session.original_cfg_folder_existed {
            fs::rename(&cfg_folder_backup, &cfg_folder)
                .context("could not restore TF2's original cfg folder")?;
        }
    } else {
        // Legacy fallback for a session started by an earlier build, which
        // backed up the individual files rather than the whole cfg folder.
        if cfg.exists() {
            fs::remove_file(&cfg)?;
        }
        if session.original_profile_cfg_existed {
            fs::rename(&cfg_backup, &cfg)?;
        }
        if config_existed && config_backup.is_file() {
            fs::copy(&config_backup, &config)?;
        } else if !config_existed && config.exists() {
            fs::remove_file(&config)?;
        }
        if video_existed && video_backup.is_file() {
            fs::copy(&video_backup, &video)?;
        } else if !video_existed && video.exists() {
            fs::remove_file(&video)?;
        }
        if offline_cfg.exists() {
            fs::remove_file(&offline_cfg)?;
        }
        if offline_cfg_existed && offline_cfg_backup.is_file() {
            fs::copy(&offline_cfg_backup, &offline_cfg)?;
        }
    }
    restore_dx_level(session)?;
    verify_restored_profile(session)?;
    cleanup_temporary_paths(session);
    if session.backup_directory.is_dir() {
        fs::remove_dir_all(&session.backup_directory)?;
    }
    let pointer = active_profile_path();
    if pointer.exists() {
        fs::remove_file(pointer)?;
    }
    Ok(())
}

fn restore_lock() -> &'static Mutex<()> {
    static RESTORE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    RESTORE_LOCK.get_or_init(|| Mutex::new(()))
}

const TF2_SETTINGS_REGISTRY_KEY: &str = r"HKCU\Software\Valve\Source\tf\Settings";

fn capture_dx_level(selected: &str) -> Result<(bool, bool, String, String)> {
    let applied = cfg!(target_os = "windows")
        && !selected.trim().is_empty()
        && !selected.trim().to_ascii_lowercase().starts_with("default");
    if !applied {
        return Ok((false, false, String::new(), String::new()));
    }
    let output = hidden_command(
        "reg.exe",
        &["query", TF2_SETTINGS_REGISTRY_KEY, "/v", "DXLevel_V1"],
    )?;
    if !output.status.success() {
        return Ok((true, false, String::new(), String::new()));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let values = line.split_whitespace().collect::<Vec<_>>();
        if values
            .first()
            .is_some_and(|value| value.eq_ignore_ascii_case("DXLevel_V1"))
            && values.len() >= 3
        {
            return Ok((true, true, values[1].to_owned(), values[2..].join(" ")));
        }
    }
    Ok((true, false, String::new(), String::new()))
}

fn restore_dx_level(session: &RecordingProfileSession) -> Result<()> {
    if !session.dx_level_was_applied || !cfg!(target_os = "windows") {
        return Ok(());
    }
    let output = if session.original_dx_level_existed {
        hidden_command(
            "reg.exe",
            &[
                "add",
                TF2_SETTINGS_REGISTRY_KEY,
                "/v",
                "DXLevel_V1",
                "/t",
                &session.original_dx_level_type,
                "/d",
                &session.original_dx_level_data,
                "/f",
            ],
        )?
    } else {
        hidden_command(
            "reg.exe",
            &[
                "delete",
                TF2_SETTINGS_REGISTRY_KEY,
                "/v",
                "DXLevel_V1",
                "/f",
            ],
        )?
    };
    if !output.status.success() && session.original_dx_level_existed {
        bail!(
            "could not restore TF2 DXLevel_V1: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let (_, existed, value_type, data) = capture_dx_level("98")?;
    if existed != session.original_dx_level_existed
        || (existed
            && (!value_type.eq_ignore_ascii_case(&session.original_dx_level_type)
                || data != session.original_dx_level_data))
    {
        bail!(
            "TF2 DXLevel_V1 restore verification failed; original backups remain in {}",
            session.backup_directory.display()
        );
    }
    Ok(())
}

fn hidden_command(program: &str, arguments: &[&str]) -> Result<std::process::Output> {
    let mut command = Command::new(program);
    command
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    Ok(command.output()?)
}

fn cleanup_temporary_paths(session: &RecordingProfileSession) {
    for path in &session.temporary_paths {
        let allowed_game_path = path
            .strip_prefix(&session.game_directory)
            .ok()
            .map(|relative| {
                relative
                    .to_string_lossy()
                    .replace('\\', "/")
                    .to_ascii_lowercase()
            })
            .is_some_and(|normalized| {
                normalized.starts_with("demos/tf2fragdemohelper_batch/")
                    || normalized.starts_with("cfg/tf2fragdemohelper_batch/")
                    || normalized.starts_with("demos/tf2fragdemohelper_manual/")
            });
        let sessions_root = recording_sessions_root();
        let allowed_manual_session = path.parent().is_some_and(|parent| parent == sessions_root)
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("tf2fragdemohelper_manual_"));
        if !allowed_game_path && !allowed_manual_session {
            continue;
        }
        if path.is_dir() {
            let _ = fs::remove_dir_all(path);
        } else if path.is_file() {
            let _ = fs::remove_file(path);
        }
        if let Some(parent) = path.parent() {
            remove_empty_tree(parent);
        }
    }
}

fn backup_hitsound_files(custom: &Path, backup: &Path) -> Result<Vec<PathBuf>> {
    if !custom.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(custom)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let relative = entry
            .path()
            .strip_prefix(custom)
            .context("hitsound path is outside tf/custom")?;
        if !is_hitsound_file(relative) {
            continue;
        }
        let destination = backup.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(entry.path(), &destination)?;
        files.push(relative.to_path_buf());
    }
    Ok(files)
}

fn is_hitsound_file(relative: &Path) -> bool {
    let normalized = relative
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let file_name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    file_name.contains("hitsound")
        || file_name.contains("killsound")
        || normalized.contains("/sound/ui/")
        || normalized.starts_with("sound/ui/")
}

fn restore_hitsound_files(session: &RecordingProfileSession) -> Result<()> {
    let source_root = session.backup_directory.join("hitsounds_original");
    let custom = session.game_directory.join("custom");
    for relative in &session.hitsound_files {
        let source = source_root.join(relative);
        if !source.is_file() {
            bail!("hitsound backup is missing: {}", source.display());
        }
        let destination = custom.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, destination)?;
    }
    Ok(())
}

fn verify_restored_profile(session: &RecordingProfileSession) -> Result<()> {
    let game = &session.game_directory;
    let backup = &session.backup_directory;
    let custom = game.join("custom");
    let cfg_folder = game.join("cfg");
    let cfg_folder_backup = backup.join("cfg_original");
    let overrides = cfg_folder.join("overrides");
    let profile = custom.join(PROFILE_FOLDER);
    let profile_cfg = game.join("cfg").join(PROFILE_CFG);
    let config = game.join("cfg/config.cfg");
    let video = game.join("cfg/video.txt");
    let offline = game.join("cfg/tf2fragdemohelper_offline.cfg");
    let config_backup = backup.join("config.cfg");
    let video_backup = backup.join("video.txt");
    let offline_backup = backup.join("tf2fragdemohelper_offline.cfg");
    let config_existed = session.original_config_existed || config_backup.is_file();
    let video_existed = session.original_video_existed || video_backup.is_file();
    let offline_existed = session.original_offline_cfg_existed || offline_backup.is_file();
    let mut problems = Vec::new();

    if session.original_custom_existed != custom.is_dir() {
        problems.push("tf/custom was not returned to its original presence state".to_owned());
    }
    if session.isolated_custom && backup.join("custom_original").exists() {
        problems.push("the original tf/custom folder is still staged in the backup".to_owned());
    }
    if !session.isolated_custom && session.original_profile_existed != profile.is_dir() {
        problems.push("the previous helper recording-resource folder was not restored".to_owned());
    }
    if session.original_cfg_folder_existed != cfg_folder.is_dir() {
        problems.push("tf/cfg was not returned to its original presence state".to_owned());
    }
    if session.original_cfg_overrides_existed != overrides.is_dir() {
        problems.push("cfg/overrides was not returned to its original presence state".to_owned());
    }
    if cfg_folder_backup.exists() {
        problems.push("the original tf/cfg folder is still staged in the backup".to_owned());
    }
    if session.original_profile_cfg_existed != profile_cfg.is_file() {
        problems.push("the previous recording profile CFG was not restored".to_owned());
    }
    if !session.original_cfg_folder_existed && config_existed {
        if !files_match(&config, &config_backup)? {
            problems.push(
                "config.cfg does not match its backup; hitsound settings may not be restored"
                    .to_owned(),
            );
        }
    } else if !session.original_cfg_folder_existed && config.exists() {
        problems.push("temporary config.cfg still exists".to_owned());
    }
    if !session.original_cfg_folder_existed && video_existed {
        if !files_match(&video, &video_backup)? {
            problems.push("video.txt does not match its backup".to_owned());
        }
    } else if !session.original_cfg_folder_existed && video.exists() {
        problems.push("temporary video.txt still exists".to_owned());
    }
    if !session.original_cfg_folder_existed && offline_existed {
        if !files_match(&offline, &offline_backup)? {
            problems.push("the previous offline CFG was not restored".to_owned());
        }
    } else if !session.original_cfg_folder_existed && offline.exists() {
        problems.push("temporary offline CFG still exists".to_owned());
    }
    for relative in &session.hitsound_files {
        if !files_match(
            &custom.join(relative),
            &backup.join("hitsounds_original").join(relative),
        )? {
            problems.push(format!(
                "custom hitsound was not restored: {}",
                relative.display()
            ));
        }
    }
    if !problems.is_empty() {
        bail!(
            "restore verification failed: {}. Original backups remain in {}",
            problems.join("; "),
            backup.display()
        );
    }
    Ok(())
}

fn files_match(left: &Path, right: &Path) -> Result<bool> {
    if !left.is_file() || !right.is_file() {
        return Ok(false);
    }
    if fs::metadata(left)?.len() != fs::metadata(right)?.len() {
        return Ok(false);
    }
    let mut left_file = File::open(left)?;
    let mut right_file = File::open(right)?;
    let mut left_buffer = [0_u8; 64 * 1024];
    let mut right_buffer = [0_u8; 64 * 1024];
    loop {
        let left_read = left_file.read(&mut left_buffer)?;
        let right_read = right_file.read(&mut right_buffer)?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
    }
}

fn write_active_profile(session: &RecordingProfileSession) -> Result<()> {
    let pointer = active_profile_path();
    if let Some(parent) = pointer.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = pointer.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(session)?)?;
    if pointer.exists() {
        fs::remove_file(&pointer)?;
    }
    fs::rename(temporary, pointer)?;
    Ok(())
}

fn active_profile_path() -> PathBuf {
    application_data_root().join("active_recording_profile.json")
}

fn install_recording_resources(custom: &Path, settings: &AppSettings) -> Result<()> {
    let needs_assets = settings.disable_announcer_voices
        || settings.disable_applause_sounds
        || settings.disable_domination_sounds
        || !settings.skybox.eq_ignore_ascii_case("Default")
        || (!settings.hud.eq_ignore_ascii_case("Keep current")
            && !settings.hud.eq_ignore_ascii_case("Default TF2 HUD"));
    if !needs_assets {
        return Ok(());
    }
    let resources = find_recording_resources()?;
    let profile = custom.join(PROFILE_FOLDER);
    fs::create_dir_all(&profile)?;
    for (enabled, name) in [
        (settings.disable_announcer_voices, "no_announcer_voices.vpk"),
        (settings.disable_applause_sounds, "no_applause_sounds.vpk"),
        (
            settings.disable_domination_sounds,
            "no_domination_sounds.vpk",
        ),
    ] {
        let source = resources.join("custom").join(name);
        if enabled && source.is_file() {
            fs::copy(source, profile.join(name))?;
        }
    }
    if settings.hud.eq_ignore_ascii_case("Kill notices only") {
        copy_directory(&resources.join("hud/hud_killnotices"), &profile)?;
    } else if settings.hud.eq_ignore_ascii_case("Medic recording HUD") {
        copy_directory(&resources.join("hud/hud_medic"), &profile)?;
    }
    if !settings.skybox.eq_ignore_ascii_case("Default") {
        install_skybox(
            &resources.join("skybox"),
            &profile.join("materials/skybox"),
            &settings.skybox,
        )?;
    }
    Ok(())
}

fn install_recording_message_suppression(custom: &Path) -> Result<()> {
    // VoteStart / VotePass / VoteFailed are demo user-messages. Disabling
    // fresh server voting does not remove messages that are already recorded
    // in the demo, so the temporary recording HUD also makes both vote panels
    // non-rendering. The complete custom folder/profile is restored after TF2
    // closes, so this never changes the player's normal HUD installation.
    let destination = custom
        .join(PROFILE_FOLDER)
        .join("resource")
        .join("ui")
        .join("votehud.res");
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &destination,
        r#""Resource/UI/VoteHud.res"
{
    "VoteActive"
    {
        "ControlName" "EditablePanel"
        "fieldName" "VoteActive"
        "xpos" "-10000"
        "ypos" "-10000"
        "wide" "0"
        "tall" "0"
        "visible" "0"
        "enabled" "0"
    }
    "VoteSetupDialog"
    {
        "ControlName" "CVoteSetupDialog"
        "fieldName" "VoteSetupDialog"
        "xpos" "-10000"
        "ypos" "-10000"
        "wide" "0"
        "tall" "0"
        "visible" "0"
        "enabled" "0"
    }
}
"#,
    )
    .with_context(|| {
        format!(
            "could not install the temporary recording vote-HUD override at {}",
            destination.display()
        )
    })?;
    Ok(())
}

fn find_recording_resources() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let executable_directory = executable
        .parent()
        .context("application directory is unavailable")?;
    for candidate in [
        executable_directory.join("recording_resources"),
        executable_directory.join("../recording_resources"),
        PathBuf::from("recording_resources"),
    ] {
        if candidate.join("custom").is_dir() {
            return Ok(candidate);
        }
    }
    let cache = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TF2FragDemoHelper")
        .join(RESOURCE_CACHE_VERSION);
    if cache.join("complete.marker").is_file() && cache.join("custom").is_dir() {
        return Ok(cache);
    }
    let archive_directory = [
        executable_directory.join("recording_resources_archive"),
        executable_directory.join("../recording_resources_archive"),
        executable_directory.join("../../recording_resources_archive"),
        PathBuf::from("recording_resources_archive"),
    ].into_iter().find(|path| path.join("resources.part000").is_file())
        .context("bundled recording resources are missing; keep recording_resources_archive beside the application")?;
    extract_resource_archive(&archive_directory, &cache)?;
    Ok(cache)
}

fn extract_resource_archive(parts_directory: &Path, cache: &Path) -> Result<()> {
    let parent = cache.parent().context("resource cache has no parent")?;
    fs::create_dir_all(parent)?;
    let staging = parent.join(format!("{RESOURCE_CACHE_VERSION}_staging"));
    let joined = parent.join(format!("{RESOURCE_CACHE_VERSION}.zip"));
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    if joined.exists() {
        fs::remove_file(&joined)?;
    }
    fs::create_dir_all(&staging)?;

    let mut parts = fs::read_dir(parts_directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("resources.part"))
        })
        .collect::<Vec<_>>();
    parts.sort();
    if parts.is_empty() {
        bail!("recording resource archive has no parts");
    }
    let mut output = File::create(&joined)?;
    for part in parts {
        io::copy(&mut File::open(part)?, &mut output)?;
    }
    drop(output);

    let mut archive = ZipArchive::new(File::open(&joined)?)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let relative = entry
            .enclosed_name()
            .context("recording resource archive contains an unsafe path")?;
        let destination = staging.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(destination)?;
        } else {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            io::copy(&mut entry, &mut File::create(destination)?)?;
        }
    }
    fs::write(
        staging.join("complete.marker"),
        b"TF2 Frag Demo Helper recording resources v2\n",
    )?;
    if cache.exists() {
        fs::remove_dir_all(cache)?;
    }
    fs::rename(&staging, cache)?;
    fs::remove_file(joined)?;
    Ok(())
}

fn install_skybox(source: &Path, destination: &Path, selected: &str) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("vmt"))
        {
            fs::copy(
                &path,
                destination.join(path.file_name().unwrap_or_default()),
            )?;
        }
    }
    for side in ["bk", "dn", "ft", "lf", "rt", "up"] {
        let texture = source.join(format!("{selected}{side}.vtf"));
        if !texture.is_file() {
            bail!(
                "selected recording skybox is incomplete: {}",
                texture.display()
            );
        }
        for entry in fs::read_dir(destination)? {
            let path = entry?.path();
            if path
                .file_stem()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.ends_with(side))
                && path.extension().and_then(|value| value.to_str()) == Some("vmt")
            {
                fs::copy(&texture, path.with_extension("vtf"))?;
            }
        }
    }
    Ok(())
}

fn copy_path(source: &Path, destination: &Path) -> Result<()> {
    if source.is_dir() {
        copy_directory(source, destination)
    } else if source.is_file() {
        fs::copy(source, destination)
            .map(|_| ())
            .map_err(Into::into)
    } else {
        bail!("selected custom resource is missing: {}", source.display())
    }
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_dir() {
        bail!(
            "required recording resource is missing: {}",
            source.display()
        );
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        let target = destination.join(path.file_name().unwrap_or_default());
        if path.is_dir() {
            copy_directory(&path, &target)?;
        } else {
            fs::copy(path, target)?;
        }
    }
    Ok(())
}

fn windows_process_is_running(image_name: &str) -> bool {
    // Fail closed: an unavailable Windows process query must never permit a
    // second launch or trigger restoration while TF2 could still be running.
    windows_process_state(image_name).unwrap_or(true)
}

/// `None` means Windows could not answer the query. A failed `tasklist` poll
/// must not be interpreted as TF2 exiting during an active recording.
fn windows_process_state(image_name: &str) -> Option<bool> {
    if !cfg!(target_os = "windows") {
        return Some(false);
    }
    let mut tasklist = Command::new("tasklist");
    tasklist.args(["/FI", &format!("IMAGENAME eq {image_name}"), "/NH"]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        tasklist.creation_flags(CREATE_NO_WINDOW);
    }
    let output = tasklist.output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .to_ascii_lowercase()
            .contains(&image_name.to_ascii_lowercase()),
    )
}

fn stop_windows_process(image_name: &str) -> Result<()> {
    if !cfg!(target_os = "windows") {
        return Ok(());
    }
    let mut taskkill = Command::new("taskkill");
    taskkill.args(["/IM", image_name, "/T", "/F"]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        taskkill.creation_flags(CREATE_NO_WINDOW);
    }
    let output = taskkill.output()?;
    if !output.status.success() && windows_process_is_running(image_name) {
        bail!(
            "could not close TF2: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// Resolve TF2's actual game directory from both current x64 installs
/// (`tf/win64/tf_win64.exe`) and older layouts (`tf/tf.exe`).  Treating the
/// executable's parent as the install root created the invalid `tf/win64/tf`
/// path responsible for Windows error 3 during preview and HLAE launch.
fn tf2_game_directory(executable: &Path) -> Result<PathBuf> {
    let binary_directory = executable
        .parent()
        .context("could not find the TF2 executable directory")?;
    let direct_game = binary_directory.join("cfg");
    if direct_game.is_dir() {
        return Ok(binary_directory.to_path_buf());
    }
    let sibling_game = binary_directory.join("tf");
    if sibling_game.join("cfg").is_dir() {
        return Ok(sibling_game);
    }
    if let Some(game) = binary_directory.parent() {
        if game.join("cfg").is_dir() {
            return Ok(game.to_path_buf());
        }
    }
    if let Some(root) = binary_directory.parent() {
        let game = root.join("tf");
        if game.join("cfg").is_dir() {
            return Ok(game);
        }
    }
    bail!(
        "could not locate TF2's tf/cfg directory from {}",
        executable.display()
    )
}

fn validate_tf2_executable(executable: &Path) -> Result<()> {
    validate_named_executable(executable, &["tf_win64.exe", "tf.exe"], "TF2")
}

fn validate_named_executable(executable: &Path, expected: &[&str], label: &str) -> Result<()> {
    let actual = executable
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if expected
        .iter()
        .any(|name| actual.eq_ignore_ascii_case(name))
    {
        return Ok(());
    }
    bail!(
        "{label} executable must be {}; selected {}",
        expected.join(" or "),
        executable.display()
    )
}

fn log_recording_diagnostic(path: &Path, message: impl AsRef<str>) {
    let result = (|| -> io::Result<()> {
        let mut log = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(log, "[{}] {}", Utc::now().to_rfc3339(), message.as_ref())
    })();
    let _ = result;
}

fn log_recording_finalize(session: &Path, message: impl AsRef<str>) {
    log_recording_diagnostic(&session.join("recording_finalize.log"), message);
}

fn clip_window(candidate: &Candidate, settings: &AppSettings) -> (i64, i64) {
    let first = candidate
        .point_of_kill_ticks
        .first()
        .copied()
        .unwrap_or(candidate.clip_start_tick);
    let last = candidate
        .point_of_kill_ticks
        .last()
        .copied()
        .unwrap_or(candidate.clip_end_tick);
    let lead_ticks = (settings.lead_seconds as f64 * 66.666_666_7).round() as i64;
    let outro_ticks = (settings.outro_seconds as f64 * 66.666_666_7).round() as i64;
    let start = (first - lead_ticks).max(0);
    let end = last + outro_ticks;
    (start, end.max(start + 1))
}

/// Assign exact clip windows to independent demo playbacks whenever the
/// recorder flush period makes a forward-only VDM schedule impossible. This
/// preserves every requested lead-in and never seeks past a candidate's kill.
fn split_recording_passes(
    clips: &[(usize, Candidate)],
    settings: &AppSettings,
) -> Vec<Vec<(usize, Candidate)>> {
    let windows = clips
        .iter()
        .map(|(_, candidate)| clip_window(candidate, settings))
        .collect::<Vec<_>>();
    partition_recording_windows(&windows)
        .into_iter()
        .map(|indices| {
            indices
                .into_iter()
                .map(|index| clips[index].clone())
                .collect()
        })
        .collect()
}

fn partition_recording_windows(windows: &[(i64, i64)]) -> Vec<Vec<usize>> {
    let mut passes: Vec<Vec<usize>> = Vec::new();
    let mut pass_finalize_ticks: Vec<i64> = Vec::new();
    for (index, &(start, end)) in windows.iter().enumerate() {
        let compatible = pass_finalize_ticks
            .iter()
            .position(|finalize_tick| start > *finalize_tick + VDM_ACTION_GAP_TICKS);
        if let Some(pass_index) = compatible {
            passes[pass_index].push(index);
            pass_finalize_ticks[pass_index] = end + RECORDING_FLUSH_TICKS;
        } else {
            passes.push(vec![index]);
            pass_finalize_ticks.push(end + RECORDING_FLUSH_TICKS);
        }
    }
    passes
}

fn candidate_needs_spectator_focus(candidate: &Candidate) -> bool {
    if candidate.attacker_user_id <= 0 {
        return false;
    }
    if candidate.demo_context.capture_type.eq_ignore_ascii_case("stv") {
        return true;
    }
    // Older exports could mislabel an STV as POV when dem_usercmd packets were
    // present.  A supposed POV with no identified POV player and all-player
    // analysis cannot safely preserve a single recorded POV, so explicitly
    // focus the selected candidate just as an STV requires.
    candidate.demo_context.analysis_scope.eq_ignore_ascii_case("all_players")
        && candidate.demo_context.pov_player_user_id.is_none()
}

fn preview_vdm_text(candidate: &Candidate, target_tick: i64) -> String {
    let mut lines = vec!["demoactions".to_owned(), "{".to_owned()];
    let mut action = 1;
    add_vdm_action(
        &mut lines,
        &mut action,
        "SkipAhead",
        "TF2 Frag Demo Helper seek",
        1,
        Some(target_tick),
        "",
    );
    if candidate_needs_spectator_focus(candidate) {
        add_vdm_action(
            &mut lines,
            &mut action,
            "PlayCommands",
            "Focus selected STV candidate",
            target_tick + 1,
            None,
            &format!("spec_autodirector 0; spec_player #{}; spec_mode 4", candidate.attacker_user_id),
        );
    }
    lines.push("}".into());
    lines.join("\n")
}

fn manual_seek_targets(target_tick: i64) -> Vec<i64> {
    let target_tick = target_tick.max(0);
    let mut targets = Vec::new();
    let mut tick = MANUAL_SEEK_STEP_TICKS;
    while tick < target_tick {
        targets.push(tick);
        tick += MANUAL_SEEK_STEP_TICKS;
    }
    if target_tick > 0 {
        targets.push(target_tick);
    }
    targets
}

fn manual_hlae_vdm_text(candidate: &Candidate, target_tick: i64, end_tick: i64) -> String {
    let target_tick = target_tick.max(0);
    let end_tick = end_tick.max(target_tick + 1);
    let first_live_tick = target_tick + 1;
    let mut lines = vec!["demoactions".to_owned(), "{".to_owned()];
    let mut action = 1;
    let seek_targets = manual_seek_targets(target_tick);
    let mut previous_target = 0;
    for (index, seek_target) in seek_targets.iter().enumerate() {
        add_vdm_action(
            &mut lines,
            &mut action,
            "SkipAhead",
            &format!(
                "TF2 Frag Demo Helper safe seek {}/{}",
                index + 1,
                seek_targets.len()
            ),
            if previous_target == 0 {
                1
            } else {
                previous_target + 1
            },
            Some(*seek_target),
            "",
        );
        previous_target = *seek_target;
    }
    let focus = if candidate_needs_spectator_focus(candidate) {
        format!(
            "spec_autodirector 0; spec_player #{}; spec_mode 4; ",
            candidate.attacker_user_id
        )
    } else {
        String::new()
    };
    add_vdm_action(
        &mut lines,
        &mut action,
        "PlayCommands",
        "Focus candidate and pause after safe seek",
        target_tick + 1,
        None,
        &format!(
            "{focus}thirdperson; r_drawviewmodel 0; mirv_cmd clear; mirv_cmd enabled 1; mirv_cmd addCurves tick {first_live_tick} {end_tick} - interp=linear space=abs {first_live_tick} {first_live_tick} {end_tick} {end_tick} -- \"echo {DIRECTOR_TICK_MARKER_PREFIX} {{0}}\"; demo_pause; echo TF2FRAG_MANUAL_PAUSED_AT_START; echo {DIRECTOR_TICK_MARKER_PREFIX} {first_live_tick}"
        ),
    );
    lines.push("}".into());
    lines.join("\n")
}

fn manual_hotkey_cfg(
    candidate: &Candidate,
    target_tick: i64,
    staged_demo: &str,
    settings: &AppSettings,
) -> String {
    let mut shortcuts = settings.mirv_shortcuts.clone();
    shortcuts.normalize();
    let mut kill_ticks = candidate.point_of_kill_ticks.clone();
    kill_ticks.sort_unstable();
    kill_ticks.dedup();
    if kill_ticks.is_empty() {
        kill_ticks.push(target_tick);
    }
    let mut lines = vec![
        "// Temporary manual HLAE controls generated by TF2 Frag Demo Helper.".into(),
        "sv_cheats 1".into(),
        "con_enable 1".into(),
        "mirv_campath enabled 0".into(),
        "mirv_campath clear".into(),
        "mirv_input end".into(),
        "alias tf2frag_manual_start \"exec tf2fragdemohelper_manual_start\"".into(),
        "alias tf2frag_manual_stop \"exec tf2fragdemohelper_manual_stop\"".into(),
        "alias tf2frag_manual_save \"exec tf2fragdemohelper_manual_save\"".into(),
        format!(
            "alias tf2frag_manual_help \"echo TF2FRAG_KEYS {}_FORWARD_0.25_SECONDS {}_TOGGLE_HUD {}_HELP {}_BACK_1_SECOND {}_CLIP_START {}_NEXT_KILL {}_PAUSE {}_CAMERA {}_KEYFRAME {}_PLAY_PATH {}_RECORD {}_STOP {}_PRINT {}_SAVE\"",
            shortcuts.advance_time, shortcuts.toggle_hud, shortcuts.show_help,
            shortcuts.back_one_second, shortcuts.safe_restart, shortcuts.next_kill_tick,
            shortcuts.pause_resume, shortcuts.enter_camera, shortcuts.add_keyframe,
            shortcuts.play_campath, shortcuts.start_recording, shortcuts.stop_recording,
            shortcuts.print_keyframes, shortcuts.save_campath,
        ),
        format!("alias tf2frag_manual_clip_start \"playdemo {staged_demo}; echo TF2FRAG_MANUAL_SAFE_RESTART_FROM_ZERO TARGET {target_tick}\""),
    ];
    for (index, tick) in kill_ticks.iter().enumerate() {
        let current = index + 1;
        let next = if current == kill_ticks.len() { 1 } else { current + 1 };
        lines.push(format!(
            "alias tf2frag_manual_kill_{current} \"demo_gototick {tick}; alias tf2frag_manual_next_kill tf2frag_manual_kill_{next}; echo TF2FRAG_MANUAL_KILL {current}/{} TICK {tick}\"",
            kill_ticks.len()
        ));
    }
    lines.extend([
        "alias tf2frag_manual_hud_off \"cl_drawhud 0; alias tf2frag_manual_toggle_hud tf2frag_manual_hud_on; echo TF2FRAG_MANUAL_HUD_HIDDEN\"".into(),
        "alias tf2frag_manual_hud_on \"cl_drawhud 1; alias tf2frag_manual_toggle_hud tf2frag_manual_hud_off; echo TF2FRAG_MANUAL_HUD_VISIBLE\"".into(),
        "alias tf2frag_manual_toggle_hud tf2frag_manual_hud_off".into(),
        "alias tf2frag_manual_next_kill tf2frag_manual_kill_1".into(),
        format!("bind \"{}\" \"mirv_skip time 0.25\"", shortcuts.advance_time),
        format!("bind \"{}\" \"tf2frag_manual_toggle_hud\"", shortcuts.toggle_hud),
        format!("bind \"{}\" \"tf2frag_manual_help\"", shortcuts.show_help),
        format!("bind \"{}\" \"mirv_skip time -1\"", shortcuts.back_one_second),
        format!("bind \"{}\" \"tf2frag_manual_clip_start\"", shortcuts.safe_restart),
        format!("bind \"{}\" \"tf2frag_manual_next_kill\"", shortcuts.next_kill_tick),
        format!("bind \"{}\" \"demo_togglepause\"", shortcuts.pause_resume),
        format!("bind \"{}\" \"sv_cheats 1; thirdperson; r_drawviewmodel 0; spec_autodirector 0; mirv_input camera\"", shortcuts.enter_camera),
        format!("bind \"{}\" \"mirv_campath add; echo TF2FRAG_MANUAL_KEYFRAME_ADDED\"", shortcuts.add_keyframe),
        format!("bind \"{}\" \"mirv_input end; thirdperson; r_drawviewmodel 0; mirv_campath enabled 1; echo TF2FRAG_MANUAL_CAMPATH_ENABLED_THIRDPERSON\"", shortcuts.play_campath),
        format!("bind \"{}\" \"tf2frag_manual_start\"", shortcuts.start_recording),
        format!("bind \"{}\" \"tf2frag_manual_stop\"", shortcuts.stop_recording),
        format!("bind \"{}\" \"mirv_campath print\"", shortcuts.print_keyframes),
        format!("bind \"{}\" \"tf2frag_manual_save\"", shortcuts.save_campath),
        "echo TF2FRAG_MANUAL_READY".into(),
        "tf2frag_manual_help".into(),
    ]);
    format!("{}\n", lines.join("\n"))
}

fn vdm_text(
    clips: &[(usize, Candidate)],
    session_name: &str,
    next_demo: Option<&str>,
    settings: &AppSettings,
) -> Result<String> {
    let mut lines = vec!["demoactions".to_owned(), "{".to_owned()];
    let mut action = 1;
    add_vdm_action(
        &mut lines,
        &mut action,
        "PlayCommands",
        "Apply movie profile",
        1,
        None,
        &format!("mirv_campath enabled 0; mirv_campath clear; mirv_input end; exec tf2fragdemohelper_recording_profile; {}", clean_capture_screen_commands()),
    );
    let mut previous_finalize_tick = -1;
    for (order, candidate) in clips {
        let (start, end) = clip_window(candidate, settings);
        if previous_finalize_tick >= 0 && start <= previous_finalize_tick + VDM_ACTION_GAP_TICKS {
            bail!(
                "overlapping recording windows reached one VDM pass (candidate {}, start tick {}, previous finalize tick {})",
                candidate.candidate_id,
                start,
                previous_finalize_tick
            );
        }
        let base = format!("{:03}_{}_t{}-{}", order, sanitize(&candidate.candidate_id), candidate.clip_start_tick, candidate.clip_end_tick);
        let seek_at = if previous_finalize_tick < 0 { 2 } else { previous_finalize_tick + 2 };
        if start > seek_at {
            add_vdm_action(&mut lines, &mut action, "SkipAhead", "Batch seek", seek_at, Some(start), "");
        }
        let focus = if candidate_needs_spectator_focus(candidate) {
            format!("spec_autodirector 0; spec_player #{}; spec_mode 4; ", candidate.attacker_user_id)
        } else { String::new() };
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Start clip", start + 1, None, &format!("{focus}exec tf2fragdemohelper_batch/{session_name}/{base}_start"));
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Stop clip", end, None, &format!("exec tf2fragdemohelper_batch/{session_name}/{base}_stop"));
        previous_finalize_tick = end + RECORDING_FLUSH_TICKS;
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Finalize clip", previous_finalize_tick, None, &format!("echo TF2FRAG_RECORD_FINALIZED {base}"));
    }
    if let Some(demo) = next_demo {
        add_vdm_action(
            &mut lines,
            &mut action,
            "PlayCommands",
            "Continue batch",
            previous_finalize_tick + 2,
            None,
            &format!("echo TF2FRAG_PASS_FINISHED; playdemo {demo}"),
        );
    } else {
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Finish batch", previous_finalize_tick + 2, None, "echo TF2FRAG_BATCH_FINISHED");
        add_vdm_action(&mut lines, &mut action, "PlayCommands", "Close TF2", previous_finalize_tick + 3, None, "quit");
    }
    lines.push("}".into());
    Ok(lines.join("\n"))
}

fn add_vdm_action(
    lines: &mut Vec<String>,
    action: &mut i32,
    factory: &str,
    name: &str,
    tick: i64,
    skip_to: Option<i64>,
    commands: &str,
) {
    lines.extend([
        format!("    \"{}\"", *action),
        "    {".into(),
        format!("        factory \"{factory}\""),
        format!("        name \"{name}\""),
        format!("        starttick \"{tick}\""),
    ]);
    if let Some(target) = skip_to {
        lines.push(format!("        skiptotick \"{target}\""));
    }
    if !commands.is_empty() {
        lines.push(format!(
            "        commands \"{}\"",
            commands.replace('"', "\\\"")
        ));
    }
    lines.push("    }".into());
    *action += 1;
}

fn prepare_recording_clips(
    candidates: &[Candidate],
    session_name: &str,
    session: &Path,
    settings: &AppSettings,
    replace_existing: bool,
) -> Result<Vec<PreparedClip>> {
    let encoded = !settings.recording_format.contains("Image");
    let extension = encoded_extension(settings);
    let mut clips = Vec::new();
    let mut reserved_identifiers = HashSet::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let source = Path::new(&candidate.source_demo);
        if !source.is_file() {
            continue;
        }
        let order = index + 1;
        let (start_tick, end_tick) = clip_window(candidate, settings);
        let recording_key = recording_key(candidate)?;
        let demo_signature = portable_demo_signature(source)?;
        let demo_name = sanitize(
            source
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("demo"),
        );
        let candidate_name = sanitize(if candidate.candidate_id.trim().is_empty() {
            "candidate"
        } else {
            &candidate.candidate_id
        });
        let base_identifier = format!(
            "{demo_name}__{candidate_name}__t{start_tick}-{end_tick}__k{}",
            recording_key_token(&recording_key)
        );
        let output_category = candidate_output_category(candidate);
        let recording_identifier = unique_recording_identifier(
            &settings.recording_output_directory,
            &output_category,
            &base_identifier,
            encoded,
            extension,
            &reserved_identifiers,
        );
        reserved_identifiers.insert(recording_identifier.clone());
        let config_base = format!(
            "{:03}_{}_t{}-{}",
            order, candidate_name, candidate.clip_start_tick, candidate.clip_end_tick
        );
        let capture_base = format!("tf2frag_{session_name}_{order:03}");
        let (working_path, final_output_path, frames_path, audio_path) = if encoded {
            // HLAE writes into a short private folder. The descriptive output
            // name is applied only when the completed file is moved out.
            let working = session.join("working").join(format!("{order:03}"));
            let final_path = settings
                .recording_output_directory
                .join("Videos")
                .join(&output_category)
                .join(format!("{recording_identifier}.{extension}"));
            fs::create_dir_all(&working)?;
            (Some(working), final_path, None, None)
        } else {
            let sequence = settings
                .recording_output_directory
                .join("Image Sequences")
                .join(&recording_identifier);
            let frames = sequence.join("Frames");
            let audio = sequence.join("Audio");
            (None, sequence, Some(frames), Some(audio))
        };
        clips.push(PreparedClip {
            order,
            candidate: candidate.clone(),
            start_tick,
            end_tick,
            config_base,
            capture_base,
            recording_key,
            demo_signature,
            recording_identifier,
            working_path,
            final_output_path,
            frames_path,
            audio_path,
            replace_existing,
        });
    }
    Ok(clips)
}

fn unique_recording_identifier(
    root: &Path,
    output_category: &str,
    base: &str,
    encoded: bool,
    extension: &str,
    reserved: &HashSet<String>,
) -> String {
    for suffix in 1usize.. {
        let candidate = if suffix == 1 {
            base.to_owned()
        } else {
            format!("{base}_{suffix}")
        };
        let final_path = if encoded {
            root.join("Videos")
                .join(output_category)
                .join(format!("{candidate}.{extension}"))
        } else {
            root.join("Image Sequences").join(&candidate)
        };
        if !final_path.exists() && !reserved.contains(&candidate) {
            return candidate;
        }
    }
    unreachable!()
}

fn write_recording_manifest(
    session: &Path,
    clips: &[PreparedClip],
    settings: &AppSettings,
    game: &Path,
    batch_status: &str,
    error: Option<&str>,
) -> Result<()> {
    let clip_records = clips
        .iter()
        .map(|clip| {
            json!({
                "order": clip.order,
                "source_demo": clip.candidate.source_demo,
                "candidate_id": clip.candidate.candidate_id,
                "recording_key": clip.recording_key,
                "demo_content_signature": clip.demo_signature,
                "start_tick": clip.start_tick,
                "end_tick": clip.end_tick,
                "candidate_clip_start_tick": clip.candidate.clip_start_tick,
                "candidate_clip_end_tick": clip.candidate.clip_end_tick,
                "attacker_user_id": clip.candidate.attacker_user_id,
                "recording_identifier": clip.recording_identifier,
                "expected_output_path": clip.final_output_path,
                "working_path": clip.working_path,
                "frames_path": clip.frames_path,
                "audio_path": clip.audio_path,
                "native_capture_base": clip.capture_base,
                "replace_existing": clip.replace_existing,
                "status": "Pending",
                "actual_output_path": null,
                "output_fingerprint": null,
                "error": null,
            })
        })
        .collect::<Vec<_>>();
    let manifest = json!({
        "format": "tf2-hlae-recording-queue",
        "version": 9,
        "batch_status": batch_status,
        "batch_error": error,
        "offline_only": true,
        "output_format": settings.recording_format,
        "ffmpeg_executable": settings.ffmpeg_executable,
        "game_directory": game,
        "encoding": mp4_encoding_manifest(settings),
        "avi_encoding": avi_encoding_manifest(settings),
        "dnxhr_encoding": dnxhr_encoding_manifest(settings),
        "fps": settings.capture_fps,
        "jpg_quality": settings.jpg_quality,
        "lead_in_seconds": settings.lead_seconds,
        "outro_seconds": settings.outro_seconds,
        "updated_utc": Utc::now().to_rfc3339(),
        "clips": clip_records,
    });
    write_json_atomic(&session.join("recording_manifest.json"), &manifest)?;
    write_json_atomic(&session.join("recording_queue.json"), &manifest)
}

fn update_recording_manifest(
    session: &Path,
    config_base: Option<&str>,
    status: &str,
    output: Option<&Path>,
    fingerprint: Option<&str>,
    error: Option<&str>,
) {
    for name in ["recording_manifest.json", "recording_queue.json"] {
        let path = session.join(name);
        let Ok(bytes) = read_file_bounded(&path, MAX_RECOVERY_MANIFEST_BYTES) else {
            continue;
        };
        let Ok(mut root) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            continue;
        };
        root["updated_utc"] = json!(Utc::now().to_rfc3339());
        if let Some(config_base) = config_base {
            if let Some(clips) = root.get_mut("clips").and_then(|value| value.as_array_mut()) {
                if let Some(record) = clips.iter_mut().find(|record| {
                    record
                        .get("native_capture_base")
                        .and_then(|value| value.as_str())
                        .is_some_and(|value| value == config_base)
                        || record
                            .get("recording_identifier")
                            .and_then(|value| value.as_str())
                            .is_some_and(|value| value == config_base)
                        || record
                            .get("candidate_id")
                            .and_then(|value| value.as_str())
                            .is_some_and(|value| value == config_base)
                }) {
                    record["status"] = json!(status);
                    if let Some(output) = output {
                        record["actual_output_path"] = json!(output);
                    }
                    if let Some(fingerprint) = fingerprint {
                        record["output_fingerprint"] = json!(fingerprint);
                    }
                    if let Some(error) = error {
                        record["error"] = json!(error);
                    }
                }
            }
        } else {
            root["batch_status"] = json!(status);
            if let Some(error) = error {
                root["batch_error"] = json!(error);
            }
        }
        let _ = write_json_atomic(&path, &root);
    }
}

fn write_json_atomic(path: &Path, value: &serde_json::Value) -> Result<()> {
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
    if let Err(error) = fs::rename(&temporary, path) {
        if path.exists() {
            fs::remove_file(path)?;
            fs::rename(temporary, path)?;
        } else {
            return Err(error.into());
        }
    }
    Ok(())
}

fn finalize_recording_session(
    child: &mut std::process::Child,
    tf_process_name: &str,
    game: &Path,
    session: &Path,
    clips: &[PreparedClip],
    settings: &AppSettings,
    progress: Option<&RecordingProgressSink>,
    diagnostic_log: &Path,
) -> (usize, usize) {
    update_recording_manifest(session, None, "Running", None, None, None);
    let recording_log = game.join("tf2fragdemohelper_recording.log");
    let start_deadline = Instant::now() + Duration::from_secs(120);
    let mut tf2_started = false;
    let mut query_warning_logged = false;
    while Instant::now() < start_deadline {
        match windows_process_state(tf_process_name) {
            Some(true) => {
                tf2_started = true;
                break;
            }
            Some(false) => {}
            None => {
                if !query_warning_logged {
                    log_recording_diagnostic(diagnostic_log, "WARNING: Windows process query failed while waiting for TF2; monitoring will retry");
                    query_warning_logged = true;
                }
            }
        }
        if child.try_wait().ok().flatten().is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(250));
    }
    let mut completed = HashSet::new();
    let mut failed = HashSet::new();
    let mut processed_lines = 0usize;
    let mut marker_state = RecordingMarkerState::default();
    let mut absent_confirmations = 0u8;
    let mut batch_finished_at = None;
    let mut last_marker_progress = Instant::now();
    let mut last_marker_count = 0usize;
    let mut stalled_recording = false;
    if !tf2_started {
        log_recording_diagnostic(
            diagnostic_log,
            "ERROR: TF2 did not start within the launch window or HLAE exited before TF2 started",
        );
    } else {
        loop {
            process_recording_markers(
                &recording_log,
                &mut processed_lines,
                clips,
                game,
                session,
                settings,
                progress,
                diagnostic_log,
                &mut completed,
                &mut failed,
                &mut marker_state,
            );
            if marker_state.finalized_markers != last_marker_count {
                last_marker_count = marker_state.finalized_markers;
                last_marker_progress = Instant::now();
            }
            if marker_state.batch_finished && batch_finished_at.is_none() {
                batch_finished_at = Some(Instant::now());
                log_recording_diagnostic(
                    diagnostic_log,
                    "TF2 reported that the complete recording batch finished",
                );
            }

            match windows_process_state(tf_process_name) {
                Some(true) => absent_confirmations = 0,
                Some(false) => absent_confirmations = absent_confirmations.saturating_add(1),
                None => {
                    absent_confirmations = 0;
                    if !query_warning_logged {
                        log_recording_diagnostic(diagnostic_log, "WARNING: Windows process query failed during recording; the session remains active while monitoring retries");
                        query_warning_logged = true;
                    }
                }
            }
            if absent_confirmations >= TF2_ABSENT_CONFIRMATIONS {
                log_recording_diagnostic(
                    diagnostic_log,
                    format!("TF2 exit confirmed after {TF2_ABSENT_CONFIRMATIONS} consecutive successful absence checks"),
                );
                break;
            }

            if batch_finished_at
                .is_some_and(|finished| finished.elapsed() >= Duration::from_secs(5))
            {
                log_recording_diagnostic(diagnostic_log, "WARNING: TF2 remained open after the final batch marker; requesting bounded shutdown");
                if let Err(error) = stop_windows_process(tf_process_name) {
                    log_recording_diagnostic(
                        diagnostic_log,
                        format!("ERROR: could not stop TF2 after batch completion: {error}"),
                    );
                }
                batch_finished_at = None;
            } else if last_marker_progress.elapsed() >= Duration::from_secs(45 * 60) {
                log_recording_diagnostic(diagnostic_log, "ERROR: no recording marker was produced for 45 minutes; stopping the stalled offline recording session");
                if let Err(error) = stop_windows_process(tf_process_name) {
                    log_recording_diagnostic(
                        diagnostic_log,
                        format!("ERROR: could not stop stalled TF2 session: {error}"),
                    );
                }
                stalled_recording = true;
                break;
            }
            thread::sleep(Duration::from_millis(250));
        }
    }
    process_recording_markers(
        &recording_log,
        &mut processed_lines,
        clips,
        game,
        session,
        settings,
        progress,
        diagnostic_log,
        &mut completed,
        &mut failed,
        &mut marker_state,
    );
    let interrupted = !marker_state.batch_finished || stalled_recording;
    if interrupted {
        if let Some(sink) = progress {
            sink(RecordingProgress::Status(
                "TF2 closed early — consolidating completed recordings".into(),
            ));
        }
        log_recording_diagnostic(
            diagnostic_log,
            "TF2 closed before the final batch marker; waiting for HLAE to flush before consolidating completed captures",
        );
    }
    wait_for_hlae_shutdown(child, diagnostic_log);
    for clip in clips {
        if !completed.contains(&clip.config_base) && !failed.contains(&clip.config_base) {
            if capture_artifacts_exist(clip, game, settings) {
                if let Some(sink) = progress {
                    sink(RecordingProgress::Status(format!(
                        "Consolidating recorded candidate {} / {}: {}",
                        clip.order,
                        clips.len(),
                        clip.candidate.candidate_id
                    )));
                }
                finalize_one_clip(
                    clip,
                    game,
                    session,
                    settings,
                    progress,
                    diagnostic_log,
                    &mut completed,
                    &mut failed,
                );
            } else {
                let status = if interrupted { "Interrupted" } else { "Failed" };
                let error = if interrupted {
                    "recording ended before this clip produced capture artifacts"
                } else {
                    "recording batch finished but this clip produced no capture artifacts"
                };
                mark_unfinished_clip(clip, session, diagnostic_log, status, error, &mut failed);
            }
        }
    }
    let batch_status = if interrupted {
        "Interrupted"
    } else if failed.is_empty() {
        "Completed"
    } else if completed.is_empty() {
        "Failed"
    } else {
        "CompletedWithErrors"
    };
    let batch_error = interrupted
        .then_some("TF2 closed before the final batch marker; completed clips were preserved");
    update_recording_manifest(session, None, batch_status, None, None, batch_error);
    (completed.len(), failed.len())
}

fn process_recording_markers(
    log_path: &Path,
    processed_lines: &mut usize,
    clips: &[PreparedClip],
    game: &Path,
    session: &Path,
    settings: &AppSettings,
    progress: Option<&RecordingProgressSink>,
    diagnostic_log: &Path,
    completed: &mut HashSet<String>,
    failed: &mut HashSet<String>,
    marker_state: &mut RecordingMarkerState,
) {
    let Ok(text) = fs::read_to_string(log_path) else {
        return;
    };
    let lines = text.lines().collect::<Vec<_>>();
    if *processed_lines > lines.len() {
        *processed_lines = 0;
    }
    for line in &lines[*processed_lines..] {
        if line.contains("TF2FRAG_BATCH_FINISHED") {
            marker_state.batch_finished = true;
        }
        if let Some(marker) = line
            .split("TF2FRAG_RECORD_START ")
            .nth(1)
            .and_then(|value| value.split_whitespace().next())
        {
            let newly_started = marker_state.started_clips.insert(marker.to_owned());
            if newly_started {
                if let (Some(clip), Some(sink)) = (
                clips.iter().find(|clip| clip.config_base == marker),
                progress,
                ) {
                    sink(RecordingProgress::ClipStarted {
                        candidate_id: clip.candidate.candidate_id.clone(),
                        current: marker_state.started_clips.len(),
                        total: clips.len(),
                    });
                }
            }
        }
        let Some(marker) = line
            .split("TF2FRAG_RECORD_FINALIZED ")
            .nth(1)
            .and_then(|value| value.split_whitespace().next())
        else {
            continue;
        };
        marker_state.finalized_markers = marker_state.finalized_markers.saturating_add(1);
        if let Some(clip) = clips.iter().find(|clip| clip.config_base == marker) {
            finalize_one_clip(
                clip,
                game,
                session,
                settings,
                progress,
                diagnostic_log,
                completed,
                failed,
            );
        }
    }
    *processed_lines = lines.len();
}

fn capture_artifacts_exist(clip: &PreparedClip, game: &Path, settings: &AppSettings) -> bool {
    if settings.recording_format.contains("Image") {
        return fs::read_dir(game)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                entry.path().is_file()
                    && entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.starts_with(&clip.capture_base))
                    && fs::metadata(entry.path()).is_ok_and(|metadata| metadata.len() > 0)
            });
    }
    let Some(working) = &clip.working_path else {
        return false;
    };
    let media_name = encoded_media_name(settings);
    let take = find_encoded_take_directory(working, media_name);
    [take.join(media_name), take.join("audio.wav")]
        .iter()
        .any(|path| fs::metadata(path).is_ok_and(|metadata| metadata.len() > 0))
}

fn mark_unfinished_clip(
    clip: &PreparedClip,
    session: &Path,
    diagnostic_log: &Path,
    status: &str,
    error: &str,
    failed: &mut HashSet<String>,
) {
    failed.insert(clip.config_base.clone());
    update_recording_manifest(
        session,
        Some(&clip.recording_identifier),
        status,
        None,
        None,
        Some(error),
    );
    log_recording_diagnostic(
        diagnostic_log,
        format!("{status}: {}: {error}", clip.candidate.candidate_id),
    );
    log_recording_finalize(
        session,
        format!("{status}: {}: {error}", clip.candidate.candidate_id),
    );
}

fn wait_for_hlae_shutdown(child: &mut std::process::Child, diagnostic_log: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(status)) => {
                log_recording_diagnostic(
                    diagnostic_log,
                    format!("HLAE process exited with status {status}"),
                );
                return;
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(error) => {
                log_recording_diagnostic(
                    diagnostic_log,
                    format!("ERROR: could not query HLAE process: {error}"),
                );
                return;
            }
        }
    }
    log_recording_diagnostic(
        diagnostic_log,
        "WARNING: HLAE remained open after TF2 closed; terminating the launcher process",
    );
    if let Err(error) = child.kill() {
        log_recording_diagnostic(
            diagnostic_log,
            format!("ERROR: could not terminate HLAE: {error}"),
        );
        return;
    }
    match child.wait() {
        Ok(status) => log_recording_diagnostic(
            diagnostic_log,
            format!("HLAE process terminated with status {status}"),
        ),
        Err(error) => log_recording_diagnostic(
            diagnostic_log,
            format!("ERROR: could not reap HLAE process: {error}"),
        ),
    }
}

fn finalize_one_clip(
    clip: &PreparedClip,
    game: &Path,
    session: &Path,
    settings: &AppSettings,
    progress: Option<&RecordingProgressSink>,
    diagnostic_log: &Path,
    completed: &mut HashSet<String>,
    failed: &mut HashSet<String>,
) {
    if completed.contains(&clip.config_base) || failed.contains(&clip.config_base) {
        return;
    }
    update_recording_manifest(
        session,
        Some(&clip.recording_identifier),
        "Finalizing",
        None,
        None,
        None,
    );
    let result = if settings.recording_format.contains("Image") {
        finalize_image_sequence(clip, game)
    } else {
        finalize_encoded_video(clip, settings, diagnostic_log)
    };
    match result {
        Ok(output) => {
            let fingerprint = output_fingerprint(&output).unwrap_or_default();
            let mut index = RecordingIndex::load();
            let replaced_outputs = if clip.replace_existing {
                index.existing_outputs(&clip.candidate)
            } else {
                Vec::new()
            };
            if let Err(error) = index.register_with_fingerprint(
                &clip.candidate,
                output.clone(),
                Some(fingerprint.clone()),
            ) {
                failed.insert(clip.config_base.clone());
                update_recording_manifest(
                    session,
                    Some(&clip.recording_identifier),
                    "Failed",
                    None,
                    None,
                    Some(&error.to_string()),
                );
                log_recording_diagnostic(
                    diagnostic_log,
                    format!(
                        "ERROR: {} was finalized but could not be indexed: {error}",
                        clip.candidate.candidate_id
                    ),
                );
                return;
            }
            let mut removed = 0usize;
            for previous in replaced_outputs {
                if previous == output {
                    continue;
                }
                match remove_replaced_recording_output(&previous) {
                    Ok(true) => {
                        removed += 1;
                        log_recording_diagnostic(diagnostic_log, format!("Removed superseded recording output: {}", previous.display()));
                    }
                    Ok(false) => {}
                    Err(error) => log_recording_diagnostic(diagnostic_log, format!("WARNING: replacement is indexed, but the old output could not be removed ({}): {error}", previous.display())),
                }
            }
            completed.insert(clip.config_base.clone());
            update_recording_manifest(
                session,
                Some(&clip.recording_identifier),
                "Completed",
                Some(&output),
                Some(&fingerprint),
                None,
            );
            let replacement_note = (removed > 0)
                .then(|| format!("; replaced {removed} previous output(s)"))
                .unwrap_or_default();
            log_recording_diagnostic(
                diagnostic_log,
                format!(
                    "Completed {} -> {}{replacement_note}",
                    clip.candidate.candidate_id,
                    output.display()
                ),
            );
            log_recording_finalize(
                session,
                format!(
                    "COMPLETED {} -> {}{replacement_note}",
                    clip.candidate.candidate_id,
                    output.display()
                ),
            );
            if let Some(sink) = progress {
                sink(RecordingProgress::ClipCompleted {
                    candidate_id: clip.candidate.candidate_id.clone(),
                    output_path: output,
                });
            }
        }
        Err(error) => {
            failed.insert(clip.config_base.clone());
            update_recording_manifest(
                session,
                Some(&clip.recording_identifier),
                "Failed",
                None,
                None,
                Some(&error.to_string()),
            );
            log_recording_diagnostic(
                diagnostic_log,
                format!(
                    "ERROR: failed to finalize {}: {error}",
                    clip.candidate.candidate_id
                ),
            );
            log_recording_finalize(
                session,
                format!("FAILED {}: {error}", clip.candidate.candidate_id),
            );
        }
    }
}

fn remove_replaced_recording_output(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let parent = path.parent().map(Path::to_path_buf);
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    if let Some(parent) = parent.filter(|parent| {
        !parent
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, "Videos" | "Image Sequences"))
    }) {
        if parent.is_dir() && fs::read_dir(&parent)?.next().is_none() {
            fs::remove_dir(parent)?;
        }
    }
    Ok(true)
}

fn finalize_image_sequence(clip: &PreparedClip, game: &Path) -> Result<PathBuf> {
    let frames = clip
        .frames_path
        .as_ref()
        .context("image sequence has no Frames directory")?;
    let audio = clip
        .audio_path
        .as_ref()
        .context("image sequence has no Audio directory")?;
    let already_moved_frames = output_still_exists(frames);
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut sources = Vec::new();
    while Instant::now() < deadline {
        sources = fs::read_dir(game)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                path.file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.starts_with(&clip.capture_base))
            })
            .collect();
        if !sources.is_empty() {
            break;
        }
        thread::sleep(Duration::from_millis(250));
    }
    let has_frame = already_moved_frames
        || sources.iter().any(|source| {
            !source
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
        });
    if !has_frame {
        bail!(
            "TF2 produced no non-empty TGA/JPG frames for {}",
            clip.candidate.candidate_id
        );
    }
    let mut frame_count = if already_moved_frames { 1 } else { 0 };
    for source in sources {
        let name = source
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let suffix = name.strip_prefix(&clip.capture_base).unwrap_or(name);
        let extension = source
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let destination = if extension.eq_ignore_ascii_case("wav") {
            audio.join(format!("{}.wav", clip.recording_identifier))
        } else {
            frame_count += 1;
            frames.join(format!("frame{suffix}"))
        };
        move_file(&source, &destination)?;
    }
    if frame_count == 0 || !output_still_exists(frames) {
        bail!(
            "TF2 produced no non-empty TGA/JPG frames for {}",
            clip.candidate.candidate_id
        );
    }
    Ok(frames.parent().unwrap_or(frames).to_path_buf())
}

fn finalize_encoded_video(
    clip: &PreparedClip,
    settings: &AppSettings,
    diagnostic_log: &Path,
) -> Result<PathBuf> {
    let working = clip
        .working_path
        .as_ref()
        .context("encoded recording has no working directory")?;
    let media_name = encoded_media_name(settings);
    let take = wait_for_encoded_take_directory(working, media_name, Duration::from_secs(20))?;
    let video = take.join(media_name);
    let audio = take.join("audio.wav");
    wait_for_stable_file(&video, Duration::from_secs(20))?;
    wait_for_stable_file(&audio, Duration::from_secs(20))?;
    let muxing = take.join(encoded_muxing_name(settings));
    let mut command = Command::new(&settings.ffmpeg_executable);
    command
        .args(["-y", "-v", "error", "-i"])
        .arg(&video)
        .arg("-i")
        .arg(&audio)
        .args(["-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy"]);
    if settings.recording_format.contains("AVI") || settings.recording_format == "MOV - DNxHR" {
        command.args(["-c:a", "pcm_s16le", "-shortest"]);
    } else {
        let audio_bitrate = format!("{}k", settings.mp4_audio_bitrate_kbps);
        command
            .args(["-c:a", "aac", "-b:a"])
            .arg(audio_bitrate)
            .args(["-movflags", "+faststart", "-shortest"]);
    }
    command
        .arg(&muxing)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    log_recording_diagnostic(
        diagnostic_log,
        format!("[Recording] Final FFmpeg mux command: {command:?}"),
    );
    let output = command.output()?;
    if !output.status.success() {
        bail!(
            "FFmpeg audio mux failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    wait_for_stable_file(&muxing, Duration::from_secs(20))?;
    move_file(&muxing, &clip.final_output_path)?;
    let _ = fs::remove_file(&video);
    let _ = fs::remove_file(&audio);
    remove_empty_tree(working);
    if !output_still_exists(&clip.final_output_path) {
        bail!("the finalized video is missing or empty");
    }
    Ok(clip.final_output_path.clone())
}

fn find_encoded_take_directory(working: &Path, media_name: &str) -> PathBuf {
    if working.join(media_name).is_file() {
        return working.to_path_buf();
    }
    let mut takes = fs::read_dir(working)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.starts_with("take"))
        })
        .collect::<Vec<_>>();
    takes.sort();
    takes
        .into_iter()
        .find(|path| path.join(media_name).is_file())
        .unwrap_or_else(|| working.to_path_buf())
}

fn wait_for_encoded_take_directory(
    working: &Path,
    media_name: &str,
    timeout: Duration,
) -> Result<PathBuf> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let take = find_encoded_take_directory(working, media_name);
        if take.join(media_name).is_file() {
            return Ok(take);
        }
        thread::sleep(Duration::from_millis(250));
    }
    bail!("HLAE produced no encoded video in {}", working.display())
}

fn wait_for_stable_file(path: &Path, timeout: Duration) -> Result<u64> {
    let deadline = Instant::now() + timeout;
    let mut previous = 0u64;
    let mut stable = 0u8;
    while Instant::now() < deadline {
        let size = fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or_default();
        if size > 0 && size == previous {
            stable += 1;
            if stable >= 3 {
                return Ok(size);
            }
        } else {
            stable = 0;
        }
        previous = size;
        thread::sleep(Duration::from_millis(250));
    }
    bail!("output did not become stable: {}", path.display())
}

fn move_file(source: &Path, destination: &Path) -> Result<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(_) => {
            fs::copy(source, destination)?;
            fs::remove_file(source)?;
            Ok(())
        }
    }
}

fn remove_empty_tree(root: &Path) {
    if let Ok(entries) = fs::read_dir(root) {
        for path in entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_dir())
        {
            remove_empty_tree(&path);
        }
    }
    let _ = fs::remove_dir(root);
}

fn recording_start_cfg(clip: &PreparedClip, settings: &AppSettings) -> String {
    let fps = settings.capture_fps;
    if settings.recording_format.starts_with("TGA") {
        format!("echo TF2FRAG_RECORD_START {}; {}; host_framerate {fps}; startmovie {} raw; hideconsole\n", clip.config_base, clean_capture_screen_commands(), clip.capture_base)
    } else if settings.recording_format.starts_with("JPG") {
        format!("echo TF2FRAG_RECORD_START {}; {}; jpeg_quality {}; host_framerate {fps}; startmovie {} jpeg; hideconsole\n", clip.config_base, clean_capture_screen_commands(), settings.jpg_quality, clip.capture_base)
    } else {
        let output = clip
            .working_path
            .as_ref()
            .expect("encoded clip has a working path")
            .display()
            .to_string()
            .replace('\\', "/");
        let preset = recording_stream_preset(settings);
        format!("echo TF2FRAG_RECORD_START {}; {}; host_framerate {fps}; mirv_streams record fps {fps}; mirv_streams record screen enabled 1; mirv_streams record screen settings {preset}; mirv_streams record name \"{output}\"; mirv_streams record start; hideconsole\n", clip.config_base, clean_capture_screen_commands())
    }
}

fn recording_stream_preset(settings: &AppSettings) -> &'static str {
    if settings.recording_format.contains("Lossless") {
        "afxFfmpegLosslessBest"
    } else if settings.recording_format.contains("AVI") {
        if effective_avi_encoding(settings)
            .ok()
            .flatten()
            .is_some_and(|encoding| encoding.custom_hlae_preset)
        {
            "tf2FragAvi"
        } else {
            "afxFfmpegRaw"
        }
    } else if settings.recording_format == "MOV - DNxHR" {
        "tf2FragDnxhr"
    } else if effective_mp4_encoding(settings)
        .ok()
        .flatten()
        .is_some_and(|encoding| encoding.custom_hlae_preset)
    {
        "tf2FragMp4"
    } else {
        "afxFfmpeg"
    }
}

fn manual_recording_start_cfg(settings: &AppSettings, output_path: &Path) -> String {
    let fps = settings.capture_fps;
    let output = output_path.display().to_string().replace('\\', "/");
    if settings.recording_format.starts_with("TGA") {
        format!(
            "echo TF2FRAG_MANUAL_RECORD_START; {}; host_framerate {fps}; startmovie \"{output}/capture\" raw; hideconsole\n",
            clean_capture_screen_commands()
        )
    } else if settings.recording_format.starts_with("JPG") {
        format!(
            "echo TF2FRAG_MANUAL_RECORD_START; {}; jpeg_quality {}; host_framerate {fps}; startmovie \"{output}/capture\" jpeg; hideconsole\n",
            clean_capture_screen_commands(),
            settings.jpg_quality
        )
    } else {
        let preset = recording_stream_preset(settings);
        format!(
            "echo TF2FRAG_MANUAL_RECORD_START; {}; host_framerate {fps}; mirv_streams record fps {fps}; mirv_streams record screen enabled 1; mirv_streams record screen settings {preset}; mirv_streams record name \"{output}\"; mirv_streams record start; hideconsole\n",
            clean_capture_screen_commands()
        )
    }
}

fn manual_recording_stop_cfg(settings: &AppSettings) -> String {
    let stop = if settings.recording_format.contains("Image") {
        "endmovie"
    } else {
        "mirv_streams record end"
    };
    format!("echo TF2FRAG_MANUAL_RECORD_END; {stop}; host_framerate 0\n")
}

fn recording_stop_cfg(settings: &AppSettings, config_base: &str) -> String {
    let stop = if settings.recording_format.contains("Image") {
        "endmovie"
    } else {
        "mirv_streams record end"
    };
    format!("echo TF2FRAG_RECORD_END {config_base}; {stop}; host_framerate 0\n")
}

fn offline_cfg() -> &'static str {
    "// Generated by TF2 Frag Demo Helper. Offline demo playback only.\nsv_lan 1\nsv_allow_votes 0\ncl_allowdownload 0\ncl_downloadfilter none\ncl_chatfilters 0\nhud_saytext_time 0\ntv_nochat 1\ncl_showtextmsg 0\ncl_showpluginmessages 0\ncl_vote_ui_active_after_voting 0\ncl_vote_ui_show_notification 0\ntf_hud_notification_duration 0\ndeveloper 0\ncon_notifytime 0\ncontimes 0\nalias connect \"echo BLOCKED: recording mode cannot connect to servers\"\nalias retry \"echo BLOCKED: recording mode cannot reconnect to servers\"\nalias tf_party_join_request_mode \"echo BLOCKED: matchmaking is disabled in recording mode\"\nalias openserverbrowser \"echo BLOCKED: recording mode is offline only\"\ncon_logfile tf2fragdemohelper_recording.log\ncon_timestamp 1\nengine_no_focus_sleep 0\nsnd_mute_losefocus 0\necho TF2FRAG_RECORDER_INIT\necho TF2FRAG_RECORDER_READY\n"
}

fn clean_capture_screen_commands() -> &'static str {
    // TF2 starts a fresh client for the recording, but Steam/GameUI panels can
    // still appear while the demo initializes. Run this at startup and again
    // immediately before every capture so a loadout inspector, console, chat,
    // or other transient panel cannot be written into the output.
    "hideconsole; gameui_hide; cancelselect; cl_chatfilters 0; hud_saytext_time 0; tv_nochat 1; cl_showtextmsg 0; cl_showpluginmessages 0; cl_vote_ui_active_after_voting 0; cl_vote_ui_show_notification 0; tf_hud_notification_duration 0; developer 0; con_notifytime 0; contimes 0"
}

fn parse_resolution(value: &str) -> (u32, u32) {
    value
        .split_once('x')
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .unwrap_or((2560, 1440))
}

fn format_number(value: u64) -> String {
    let text = value.to_string();
    let mut output = String::new();
    for (index, character) in text.chars().enumerate() {
        if index > 0 && (text.len() - index) % 3 == 0 {
            output.push(',');
        }
        output.push(character);
    }
    output
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn candidate_output_category(candidate: &Candidate) -> String {
    let tag = candidate.inferred_primary_tag();
    let words = tag
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut characters = word.chars();
            match characters.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    characters.as_str().to_ascii_lowercase()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let safe = words
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    let safe = safe.trim().trim_end_matches(['.', ' ']);
    if safe.is_empty() {
        "Other".into()
    } else {
        safe.into()
    }
}

fn recording_profile_cfg(settings: &AppSettings) -> String {
    let mut lines = vec![
        "// Generated by TF2 Frag Demo Helper (Rust)".to_owned(),
        "sv_lan 1".to_owned(),
        "sv_master_legacy_mode 1".to_owned(),
        "sv_cheats 1".to_owned(),
        "fps_max 0".to_owned(),
        format!("mat_motion_blur_enabled {}", bool_num(settings.motion_blur)),
        format!(
            "mat_motion_blur_forward_enabled {}",
            bool_num(settings.motion_blur)
        ),
        format!(
            "mat_motion_blur_strength {}",
            bool_num(settings.motion_blur)
        ),
        format!("viewmodel_fov_demo {}", settings.viewmodel_fov),
        format!("hud_combattext {}", bool_num(!settings.disable_combat_text)),
        format!(
            "hud_combattext_healing {}",
            bool_num(!settings.disable_combat_text)
        ),
        format!(
            "tf_dingalingaling {}",
            bool_num(!settings.disable_hit_sounds)
        ),
        format!(
            "tf_dingalingaling_lasthit {}",
            bool_num(!settings.disable_hit_sounds)
        ),
        format!("voice_enable {}", bool_num(!settings.disable_voice_chat)),
        format!("cl_hud_minmode {}", bool_num(settings.minimal_hud)),
        format!(
            "cl_hud_playerclass_use_playermodel {}",
            bool_num(settings.hud_player_model)
        ),
        format!("crosshair {}", bool_num(!settings.disable_crosshair)),
    ];
    if settings.viewmodels.eq_ignore_ascii_case("On") {
        lines.push("r_drawviewmodel 1".into());
    } else if settings.viewmodels.eq_ignore_ascii_case("Off") {
        lines.push("r_drawviewmodel 0".into());
    }
    if settings.disable_crosshair_switching {
        lines.extend(
            [
                "alias cl_crosshair_file \"\"",
                "alias cl_crosshair_scale \"\"",
                "alias cl_crosshair_red \"\"",
                "alias cl_crosshair_green \"\"",
                "alias cl_crosshair_blue \"\"",
                "alias crosshair \"\"",
            ]
            .into_iter()
            .map(str::to_owned),
        );
    }
    if settings.maximum_graphics {
        lines.extend(
            [
                "cl_burninggibs 1",
                "cl_detaildist 8096",
                "cl_detailfade 0",
                "cl_maxrenderable_dist 8096",
                "cl_new_impact_effects 1",
                "cl_phys_props_max 1024",
                "cl_ragdoll_collide 1",
                "lod_transitiondist 6400",
                "mat_aaquality 2",
                "mat_antialias 8",
                "mat_bumpmap 1",
                "mat_compressedtextures 1",
                "mat_envmapsize 512",
                "mat_envmaptgasize 512",
                "mat_forceaniso 16",
                "mat_hdr_level 2",
                "mat_parallaxmap 1",
                "mat_picmip -1",
                "mat_postprocess_x 8",
                "mat_postprocess_y 8",
                "mat_reducefillrate 0",
                "mat_software_aa_quality 2",
                "mat_software_aa_strength 2",
                "mat_specular 1",
                "mat_vsync 0",
                "mat_wateroverlaysize 512",
                "mp_decals 4096",
                "mp_usehwmmodels 1",
                "mp_usehwmvcds 1",
                "r_avglight 3",
                "r_decals 4096",
                "r_eyeglintlodpixels 4",
                "r_lod 0",
                "r_maxmodeldecal 4096",
                "r_radiosity 3",
                "r_rainradius 2250",
                "r_rainsplashpercentage 100",
                "r_rootlod 0",
                "r_shadowmaxrendered 1024",
                "r_shadowrendertotexture 1",
                "r_shadows 1",
                "r_waterdrawreflection 1",
                "r_waterdrawrefraction 1",
                "r_waterforceexpensive 1",
                "r_waterforcereflectentities 1",
                "r_pixelfog 1",
                "mat_viewportscale 1",
                "mat_viewportupscale 1",
                "mat_queue_mode -1",
                "r_threaded_particles 1",
                "r_threaded_renderables 1",
                "r_threaded_client_shadow_manager 1",
            ]
            .into_iter()
            .map(str::to_owned),
        );
    }
    lines.extend(
        [
            "cl_showfps 0",
            "net_graph 0",
            "cl_chatfilters 0",
            "hud_saytext_time 0",
            "tv_nochat 1",
            "hideconsole",
            "gameui_hide",
            "cancelselect",
            "engine_no_focus_sleep 0",
            "snd_mute_losefocus 0",
            "echo TF2FRAG_MOVIE_PROFILE_READY",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    if let Ok(Some(encoding)) = effective_mp4_encoding(settings) {
        if let Some(command) = hlae_custom_mp4_preset_command(&encoding) {
            lines.push(command);
        }
    }
    if let Ok(Some(encoding)) = effective_avi_encoding(settings) {
        if let Some(command) = hlae_custom_avi_preset_command(&encoding) {
            lines.push(command);
        }
    }
    if let Ok(Some(encoding)) = effective_dnxhr_encoding(settings) {
        lines.push(hlae_custom_dnxhr_preset_command(&encoding));
    }
    lines.push(String::new());
    // Custom ffmpegEx presets must use the executable selected in Recording
    // Settings. Requiring a second copy under HLAE/ffmpeg/bin made a valid,
    // autosaved FFmpeg path look missing at launch time.
    lines.join("\n").replace(
        "{FFMPEG_PATH}",
        &settings.ffmpeg_executable.display().to_string(),
    )
}

fn hlae_custom_mp4_preset_command(encoding: &EffectiveMp4Encoding) -> Option<String> {
    if !encoding.custom_hlae_preset {
        return None;
    }
    let crf = encoding.crf?;
    let preset = encoding.encoder_preset.as_deref()?;
    Some(format!(
        "mirv_streams settings add ffmpegEx tf2FragMp4 \"{{QUOTE}}{{FFMPEG_PATH}}{{QUOTE}} -f rawvideo -pixel_format {{PIXEL_FORMAT}} -loglevel repeat+level+warning -framerate {{FRAMERATE}} -video_size {{WIDTH}}x{{HEIGHT}} -i pipe:0 -vf setsar=sar=1/1 -c:v libx264 -preset {preset} -crf {crf} -pix_fmt {} -profile:v {} {{QUOTE}}{{AFX_STREAM_PATH}}\\\\video.mp4{{QUOTE}}\"",
        encoding.pixel_format,
        encoding.ffmpeg_profile,
    ))
}

fn hlae_custom_avi_preset_command(encoding: &EffectiveAviEncoding) -> Option<String> {
    if !encoding.custom_hlae_preset {
        return None;
    }
    let codec = encoding.ffmpeg_codec.as_deref()?;
    let codec_options = if codec == "ffv1" {
        " -level 3 -coder range_tab -context 1 -slicecrc 1"
    } else {
        ""
    };
    Some(format!(
        "mirv_streams settings add ffmpegEx tf2FragAvi \"{{QUOTE}}{{FFMPEG_PATH}}{{QUOTE}} -f rawvideo -pixel_format {{PIXEL_FORMAT}} -loglevel repeat+level+warning -framerate {{FRAMERATE}} -video_size {{WIDTH}}x{{HEIGHT}} -i pipe:0 -vf setsar=sar=1/1 -c:v {codec}{codec_options} -pix_fmt {} {{QUOTE}}{{AFX_STREAM_PATH}}\\\\video.avi{{QUOTE}}\"",
        encoding.pixel_format,
    ))
}

fn hlae_custom_dnxhr_preset_command(encoding: &EffectiveDnxhrEncoding) -> String {
    format!(
        "mirv_streams settings add ffmpegEx tf2FragDnxhr \"{{QUOTE}}{{FFMPEG_PATH}}{{QUOTE}} -f rawvideo -pixel_format {{PIXEL_FORMAT}} -loglevel repeat+level+warning -framerate {{FRAMERATE}} -video_size {{WIDTH}}x{{HEIGHT}} -i pipe:0 -vf setsar=sar=1/1 -c:v dnxhd -profile:v {} -pix_fmt {} {{QUOTE}}{{AFX_STREAM_PATH}}\\\\video.mov{{QUOTE}}\"",
        encoding.ffmpeg_profile,
        encoding.pixel_format,
    )
}

fn validate_selected_encoder(settings: &AppSettings) -> Result<()> {
    let required_encoder =
        if effective_mp4_encoding(settings)?.is_some_and(|encoding| encoding.custom_hlae_preset) {
            Some("libx264".to_owned())
        } else if let Some(encoding) = effective_avi_encoding(settings)? {
            encoding.ffmpeg_codec.as_deref().map(str::to_owned)
        } else if effective_dnxhr_encoding(settings)?.is_some() {
            Some("dnxhd".into())
        } else {
            None
        };
    let Some(required_encoder) = required_encoder else {
        return Ok(());
    };
    let executable = settings.ffmpeg_executable.clone();
    if !executable.is_file() {
        bail!(
            "the selected advanced HLAE encoder requires the saved FFmpeg executable at {}",
            executable.display()
        );
    }
    let mut command = Command::new(&executable);
    command.args(["-hide_banner", "-encoders"]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let output = command.output().with_context(|| {
        format!(
            "could not inspect the selected FFmpeg executable at {}",
            executable.display()
        )
    })?;
    let encoder_list = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !output.status.success() {
        bail!("FFmpeg encoder check failed: {}", encoder_list.trim());
    }
    if !encoder_list.contains(required_encoder.as_str()) {
        bail!("the selected FFmpeg executable does not provide the required {required_encoder} encoder");
    }
    Ok(())
}

fn bool_num(value: bool) -> u8 {
    if value {
        1
    } else {
        0
    }
}

fn recording_key(candidate: &Candidate) -> Result<String> {
    let demo = portable_demo_signature(Path::new(&candidate.source_demo))?;
    let mut identity = vec![
        "v1".to_owned(),
        demo,
        candidate.attacker_user_id.to_string(),
    ];
    if candidate.point_of_kill_ticks.is_empty() {
        identity.push(candidate.clip_start_tick.to_string());
        identity.push(candidate.clip_end_tick.to_string());
    } else {
        identity.extend(
            candidate
                .point_of_kill_ticks
                .iter()
                .map(ToString::to_string),
        );
    }
    let mut hash = Sha256::new();
    hash.update(identity.join("|").as_bytes());
    Ok(hex::encode(hash.finalize()))
}

fn legacy_recording_key(candidate: &Candidate) -> Result<String> {
    let demo = demo_signature(Path::new(&candidate.source_demo))?;
    let mut hash = Sha256::new();
    hash.update(demo.as_bytes());
    hash.update(candidate.clip_start_tick.to_le_bytes());
    hash.update(candidate.clip_end_tick.to_le_bytes());
    hash.update(candidate.attacker_user_id.to_le_bytes());
    Ok(hex::encode(hash.finalize()))
}

fn recording_keys(candidate: &Candidate) -> [Option<String>; 2] {
    [
        recording_key(candidate).ok(),
        legacy_recording_key(candidate).ok(),
    ]
}

fn recording_key_token(key: &str) -> &str {
    key.get(..24).unwrap_or(key)
}

fn portable_demo_signature(path: &Path) -> Result<String> {
    let metadata =
        fs::metadata(path).with_context(|| format!("demo missing: {}", path.display()))?;
    let cache_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let modified = metadata.modified().ok();
    let cache = PORTABLE_DEMO_SIGNATURE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(entry) = cache.lock().get(&cache_key) {
        if entry.length == metadata.len() && entry.modified == modified {
            return Ok(entry.signature.clone());
        }
    }
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    let head_read = file.read(&mut buffer)?;
    hash.update(&buffer[..head_read]);
    if metadata.len() > buffer.len() as u64 {
        file.seek(SeekFrom::End(-(buffer.len() as i64)))?;
        let tail_read = file.read(&mut buffer)?;
        hash.update(&buffer[..tail_read]);
    }
    hash.update(metadata.len().to_string().as_bytes());
    let signature = format!("{}:{}", metadata.len(), hex::encode(hash.finalize()));
    cache.lock().insert(
        cache_key,
        DemoSignatureCacheEntry {
            length: metadata.len(),
            modified,
            signature: signature.clone(),
        },
    );
    Ok(signature)
}

fn demo_signature(path: &Path) -> Result<String> {
    let metadata =
        fs::metadata(path).with_context(|| format!("demo missing: {}", path.display()))?;
    let cache_key = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let modified = metadata.modified().ok();
    let cache = DEMO_SIGNATURE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(entry) = cache.lock().get(&cache_key) {
        if entry.length == metadata.len() && entry.modified == modified {
            return Ok(entry.signature.clone());
        }
    }
    let mut file = File::open(path)?;
    let mut head = vec![0u8; 1024 * 1024];
    let read = file.read(&mut head)?;
    let mut hash = Sha256::new();
    hash.update(metadata.len().to_le_bytes());
    hash.update(&head[..read]);
    let signature = hex::encode(hash.finalize());
    cache.lock().insert(
        cache_key,
        DemoSignatureCacheEntry {
            length: metadata.len(),
            modified,
            signature: signature.clone(),
        },
    );
    Ok(signature)
}

fn file_fingerprint(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(hex::encode(hash.finalize()))
}

fn output_still_exists(path: &Path) -> bool {
    if path.is_file() {
        return fs::metadata(path).is_ok_and(|metadata| metadata.len() > 0);
    }
    path.is_dir()
        && WalkDir::new(path)
            .max_depth(3)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|extension| {
                            matches!(
                                extension.to_ascii_lowercase().as_str(),
                                "tga" | "jpg" | "jpeg"
                            )
                        })
                    && fs::metadata(entry.path()).is_ok_and(|metadata| metadata.len() > 0)
            })
}

fn output_fingerprint(path: &Path) -> Result<String> {
    if path.is_file() {
        return file_fingerprint(path);
    }
    if !path.is_dir() {
        bail!("recording output is missing: {}", path.display());
    }
    let mut files = WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    files.sort();
    let mut hash = Sha256::new();
    for file in files {
        let relative = file
            .strip_prefix(path)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let metadata = fs::metadata(&file)?;
        hash.update(relative.as_bytes());
        hash.update(metadata.len().to_le_bytes());
        let mut input = File::open(&file)?;
        let mut sample = [0_u8; 4096];
        let read = input.read(&mut sample)?;
        hash.update(&sample[..read]);
        if metadata.len() > sample.len() as u64 {
            input.seek(SeekFrom::End(-(sample.len() as i64)))?;
            let read = input.read(&mut sample)?;
            hash.update(&sample[..read]);
        }
    }
    Ok(hex::encode(hash.finalize()))
}

fn recording_output_name(path: &Path) -> String {
    let named_path = if path
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("Frames"))
    {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    named_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn final_recording_outputs(root: &Path) -> Vec<PathBuf> {
    let mut outputs = Vec::new();
    let videos = root.join("Videos");
    if videos.is_dir() {
        outputs.extend(
            WalkDir::new(&videos)
                .min_depth(1)
                .max_depth(2)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.into_path())
                .filter(|path| path.is_file() && output_still_exists(path)),
        );
    }
    let sequences = root.join("Image Sequences");
    if let Ok(entries) = fs::read_dir(sequences) {
        outputs.extend(
            entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| output_still_exists(path)),
        );
    }
    outputs
}

fn application_data_root() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("TF2FragDemoHelper")
}

pub fn recording_sessions_root() -> PathBuf {
    application_data_root().join("Recording Sessions")
}

pub fn latest_recording_session() -> Option<PathBuf> {
    discover_recording_sessions_in(
        &recording_sessions_root(),
        true,
        MAX_RECOVERY_DIRECTORY_ENTRIES,
        MAX_RECOVERY_DIRECTORY_ENTRIES,
    )
    .sessions
    .pop()
}

pub fn recover_recording_sessions(settings: &AppSettings) -> RecordingRecoveryReport {
    let mut report = RecordingRecoveryReport::default();
    let discovery = discover_recording_sessions(false);
    report.deferred_sessions = discovery.deferred_sessions;
    report.disabled_sessions = discovery.disabled_sessions;
    for session in discovery.sessions {
        if !begin_automatic_recovery_attempt(&session, &mut report) {
            continue;
        }
        report.scanned_sessions += 1;
        let errors_before = report.errors.len();
        recover_one_recording_session(&session, settings, &mut report);
        if session.is_dir()
            && report.errors.len() > errors_before
            && automatic_recovery_attempts(&session) >= MAX_AUTOMATIC_RECOVERY_ATTEMPTS
            && !automatic_recovery_disabled(&session)
        {
            disable_automatic_recovery(
                &session,
                "Automatic recovery failed three times. The session was retained for manual inspection.",
            );
            report.disabled_sessions += 1;
        }
    }
    report
}

#[derive(Default)]
struct RecordingSessionDiscovery {
    sessions: Vec<PathBuf>,
    deferred_sessions: usize,
    disabled_sessions: usize,
}

fn discover_recording_sessions(include_disabled: bool) -> RecordingSessionDiscovery {
    let root = recording_sessions_root();
    discover_recording_sessions_in(
        &root,
        include_disabled,
        MAX_RECOVERY_DIRECTORY_ENTRIES,
        MAX_RECOVERY_SESSIONS_PER_STARTUP,
    )
}

fn discover_recording_sessions_in(
    root: &Path,
    include_disabled: bool,
    max_directory_entries: usize,
    max_sessions: usize,
) -> RecordingSessionDiscovery {
    let mut discovery = RecordingSessionDiscovery::default();
    let Ok(entries) = fs::read_dir(root) else {
        return discovery;
    };
    let mut directory_truncated = false;
    for (index, entry) in entries.enumerate() {
        if index >= max_directory_entries {
            directory_truncated = true;
            break;
        }
        let Some(path) = entry.ok().map(|entry| entry.path()) else {
            continue;
        };
        if !path.is_dir()
            || !path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.starts_with("tf2fragdemohelper_batch_"))
        {
            continue;
        }
        if automatic_recovery_disabled(&path) {
            discovery.disabled_sessions += 1;
            if !include_disabled {
                continue;
            }
        }
        discovery.sessions.push(path);
    }
    discovery.sessions.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .ok()
    });
    discovery.deferred_sessions = discovery.sessions.len().saturating_sub(max_sessions)
        + if directory_truncated { 1 } else { 0 };
    discovery.sessions.truncate(max_sessions);
    discovery
}

fn automatic_recovery_disabled(session: &Path) -> bool {
    session.join(RECOVERY_DISABLED_FILE).is_file()
}

fn automatic_recovery_attempts(session: &Path) -> u32 {
    let path = session.join(RECOVERY_ATTEMPTS_FILE);
    let Ok(file) = File::open(path) else { return 0 };
    let mut text = String::new();
    let _ = file.take(32).read_to_string(&mut text);
    text.trim().parse().unwrap_or_default()
}

fn disable_automatic_recovery(session: &Path, reason: &str) {
    let message = format!(
        "Automatic recovery was disabled at {}.\n\n{}\n\nThe session was retained and no recording output was deleted.\n",
        Utc::now().to_rfc3339(),
        reason,
    );
    let _ = fs::write(session.join(RECOVERY_DISABLED_FILE), message);
}

fn begin_automatic_recovery_attempt(session: &Path, report: &mut RecordingRecoveryReport) -> bool {
    if automatic_recovery_disabled(session) {
        report.disabled_sessions += 1;
        return false;
    }
    let attempts = automatic_recovery_attempts(session);
    if attempts >= MAX_AUTOMATIC_RECOVERY_ATTEMPTS {
        disable_automatic_recovery(
            session,
            "The retry limit was reached before this launch. No further automatic work was attempted.",
        );
        report.disabled_sessions += 1;
        return false;
    }
    let _ = fs::write(
        session.join(RECOVERY_ATTEMPTS_FILE),
        (attempts + 1).to_string(),
    );
    true
}

fn read_file_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let metadata =
        fs::metadata(path).with_context(|| format!("could not inspect {}", path.display()))?;
    if metadata.len() > max_bytes {
        bail!(
            "{} is too large ({} bytes; limit is {} bytes)",
            path.display(),
            metadata.len(),
            max_bytes
        );
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(path)?
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        bail!(
            "{} exceeded the {} byte read limit",
            path.display(),
            max_bytes
        );
    }
    Ok(bytes)
}

fn read_recovery_manifest(path: &Path) -> Result<StoredRecordingManifest> {
    let bytes = read_file_bounded(path, MAX_RECOVERY_MANIFEST_BYTES)?;
    let manifest = serde_json::from_slice::<StoredRecordingManifest>(&bytes)
        .with_context(|| format!("{} is not valid recording-session JSON", path.display()))?;
    if manifest.clips.len() > MAX_RECOVERY_CLIPS_PER_SESSION {
        bail!(
            "{} contains {} clips; the automatic recovery limit is {}",
            path.display(),
            manifest.clips.len(),
            MAX_RECOVERY_CLIPS_PER_SESSION,
        );
    }
    Ok(manifest)
}

fn recover_one_recording_session(
    session: &Path,
    settings: &AppSettings,
    report: &mut RecordingRecoveryReport,
) {
    let diagnostic_log = session.join("hlae_recording_diagnostics.log");
    let manifest_path = [
        session.join("recording_manifest.json"),
        session.join("recording_queue.json"),
    ]
    .into_iter()
    .find(|path| path.is_file());
    let Some(manifest_path) = manifest_path else {
        disable_automatic_recovery(session, "No recording manifest was found.");
        report.disabled_sessions += 1;
        report.retained_sessions += 1;
        report
            .errors
            .push(format!("{} has no recording manifest", session.display()));
        return;
    };
    let manifest = match read_recovery_manifest(&manifest_path) {
        Ok(manifest) => manifest,
        Err(error) => {
            disable_automatic_recovery(session, &error.to_string());
            report.disabled_sessions += 1;
            report.retained_sessions += 1;
            report.errors.push(error.to_string());
            return;
        }
    };
    if manifest.clips.is_empty() {
        disable_automatic_recovery(session, "The recording manifest contains no tracked clips.");
        report.disabled_sessions += 1;
        report.retained_sessions += 1;
        report
            .errors
            .push(format!("{} has no tracked clips", session.display()));
        return;
    }

    log_recording_diagnostic(
        &diagnostic_log,
        "Recovery scan started after a previous helper shutdown",
    );
    let mut recovery_settings = settings.clone();
    if !manifest.output_format.trim().is_empty() {
        recovery_settings.recording_format = manifest.output_format.clone();
    }
    if manifest.ffmpeg_executable.is_file() {
        recovery_settings.ffmpeg_executable = manifest.ffmpeg_executable.clone();
    }
    if let Some(encoding) = &manifest.encoding {
        recovery_settings.mp4_compatibility = encoding.compatibility.clone();
        recovery_settings.mp4_video_codec = encoding.video_codec.clone();
        recovery_settings.mp4_pixel_format = encoding.pixel_format.clone();
        recovery_settings.mp4_h264_profile = encoding.h264_profile.clone();
        recovery_settings.mp4_crf = encoding.crf;
        recovery_settings.mp4_encoder_preset = encoding.encoder_preset.clone();
        recovery_settings.mp4_audio_codec = encoding.audio_codec.clone();
        recovery_settings.mp4_audio_bitrate_kbps = encoding.audio_bitrate_kbps;
        recovery_settings.normalize_encoding_options();
    }
    if let Some(encoding) = &manifest.avi_encoding {
        recovery_settings.avi_video_codec = encoding.video_codec.clone();
        recovery_settings.avi_pixel_format = encoding.pixel_format.clone();
        recovery_settings.normalize_encoding_options();
    } else if recovery_settings.recording_format == "AVI - Raw" {
        recovery_settings.avi_video_codec = "Original HLAE Raw".into();
        recovery_settings.avi_pixel_format = "HLAE Native".into();
        recovery_settings.normalize_encoding_options();
    }
    if let Some(encoding) = &manifest.dnxhr_encoding {
        recovery_settings.dnxhr_profile = encoding.profile.clone();
        recovery_settings.normalize_encoding_options();
    }
    let game = if manifest.game_directory.is_dir() {
        manifest.game_directory.clone()
    } else {
        tf2_game_directory(&recovery_settings.tf2_executable).unwrap_or_default()
    };
    let mut index = RecordingIndex::load();
    let mut all_complete = true;

    for stored in &manifest.clips {
        let clip = prepared_clip_from_manifest(stored);
        let mut artifact_finalize_failed = false;
        let mut artifact_finalize_error = None;
        let existing_output = stored
            .actual_output_path
            .as_ref()
            .filter(|path| output_still_exists(path))
            .cloned()
            .or_else(|| {
                output_still_exists(&stored.expected_output_path)
                    .then(|| stored.expected_output_path.clone())
            });
        let image_sequence =
            recovery_settings.recording_format.contains("Image") || clip.working_path.is_none();
        let recoverable_artifacts = if image_sequence {
            game.is_dir() && capture_artifacts_exist(&clip, &game, &recovery_settings)
        } else {
            capture_artifacts_exist(&clip, &game, &recovery_settings)
        };

        let (output, finalized_now) = if recoverable_artifacts {
            let result = if image_sequence {
                finalize_image_sequence(&clip, &game)
            } else {
                finalize_encoded_video(&clip, &recovery_settings, &diagnostic_log)
            };
            match result {
                Ok(output) => (Some(output), true),
                Err(error) => {
                    artifact_finalize_failed = true;
                    all_complete = false;
                    artifact_finalize_error = Some(error.to_string());
                    log_recording_diagnostic(
                        &diagnostic_log,
                        format!(
                            "RECOVERY ERROR: {} could not be finalized: {error}",
                            stored.candidate_id
                        ),
                    );
                    report
                        .errors
                        .push(format!("{}: {}", stored.candidate_id, error));
                    (existing_output, false)
                }
            }
        } else {
            (existing_output, false)
        };

        let Some(output) = output else {
            all_complete = false;
            let error = "no complete output or recoverable HLAE capture artifacts were found";
            log_recording_diagnostic(
                &diagnostic_log,
                format!("RECOVERY PENDING: {}: {error}", stored.candidate_id),
            );
            report
                .errors
                .push(format!("{}: {error}", stored.candidate_id));
            update_recording_manifest(
                session,
                Some(&stored.recording_identifier),
                "RecoveryPending",
                None,
                None,
                Some(error),
            );
            continue;
        };
        let already_indexed = index.entries.get(&clip.recording_key).is_some_and(|entry| {
            entry.output_path == output && output_still_exists(&entry.output_path)
        });
        let fingerprint = stored
            .output_fingerprint
            .clone()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| output_fingerprint(&output).unwrap_or_default());
        let previous_output = clip
            .replace_existing
            .then(|| {
                index
                    .entries
                    .get(&clip.recording_key)
                    .map(|entry| entry.output_path.clone())
            })
            .flatten();
        if !already_indexed {
            if let Err(error) = index.register_recovered(&clip, output.clone(), fingerprint.clone())
            {
                all_complete = false;
                log_recording_diagnostic(
                    &diagnostic_log,
                    format!(
                        "RECOVERY ERROR: {} was finalized but could not be indexed: {error}",
                        stored.candidate_id
                    ),
                );
                report
                    .errors
                    .push(format!("{}: {error}", stored.candidate_id));
                continue;
            }
            if finalized_now {
                report.recovered_clips += 1;
            } else {
                report.indexed_clips += 1;
            }
        }
        if let Some(previous) = previous_output.filter(|previous| previous != &output) {
            if let Err(error) = remove_replaced_recording_output(&previous) {
                log_recording_diagnostic(&diagnostic_log, format!("RECOVERY WARNING: replacement was indexed, but {} could not be removed: {error}", previous.display()));
            }
        }
        let recovered_status = if artifact_finalize_failed {
            "RecoveryPending"
        } else {
            "Completed"
        };
        update_recording_manifest(
            session,
            Some(&stored.recording_identifier),
            recovered_status,
            Some(&output),
            Some(&fingerprint),
            artifact_finalize_error.as_deref(),
        );
        log_recording_diagnostic(
            &diagnostic_log,
            format!("RECOVERED {} -> {}", stored.candidate_id, output.display()),
        );
    }

    if all_complete {
        update_recording_manifest(session, None, "Completed", None, None, None);
        match remove_completed_recording_session(session) {
            Ok(()) => report.removed_sessions += 1,
            Err(error) => {
                report.retained_sessions += 1;
                report.errors.push(format!(
                    "{} could not be cleaned up: {error}",
                    session.display()
                ));
            }
        }
    } else {
        update_recording_manifest(
            session,
            None,
            "RecoveryPending",
            None,
            None,
            Some("one or more tracked clips still need recovery"),
        );
        report.retained_sessions += 1;
    }
}

fn prepared_clip_from_manifest(stored: &StoredRecordingClip) -> PreparedClip {
    let candidate_start_tick = stored
        .candidate_clip_start_tick
        .unwrap_or(stored.start_tick);
    let candidate_end_tick = stored.candidate_clip_end_tick.unwrap_or(stored.end_tick);
    PreparedClip {
        order: stored.order,
        candidate: Candidate {
            candidate_id: stored.candidate_id.clone(),
            source_demo: stored.source_demo.clone(),
            attacker_user_id: stored.attacker_user_id,
            clip_start_tick: candidate_start_tick,
            clip_end_tick: candidate_end_tick,
            ..Candidate::default()
        },
        start_tick: stored.start_tick,
        end_tick: stored.end_tick,
        config_base: stored.recording_identifier.clone(),
        capture_base: stored.native_capture_base.clone(),
        recording_key: stored.recording_key.clone(),
        demo_signature: stored.demo_content_signature.clone(),
        recording_identifier: stored.recording_identifier.clone(),
        working_path: stored.working_path.clone(),
        final_output_path: stored.expected_output_path.clone(),
        frames_path: stored.frames_path.clone(),
        audio_path: stored.audio_path.clone(),
        replace_existing: stored.replace_existing,
    }
}

fn remove_completed_recording_session(session: &Path) -> Result<()> {
    let root = recording_sessions_root();
    if !is_managed_recording_session(&root, session) {
        bail!(
            "refusing to remove an unexpected recording session path: {}",
            session.display()
        );
    }
    if session.is_dir() {
        fs::remove_dir_all(session)?;
    }
    if root.is_dir() && fs::read_dir(&root)?.next().is_none() {
        fs::remove_dir(root)?;
    }
    Ok(())
}

fn is_managed_recording_session(root: &Path, session: &Path) -> bool {
    session.parent().is_some_and(|parent| parent == root)
        && session
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.starts_with("tf2fragdemohelper_batch_"))
}

fn index_path() -> PathBuf {
    application_data_root().join("recorded_clip_index.ndjson")
}

#[cfg(test)]
mod recording_tests {
    use super::*;

    fn recovery_test_root(name: &str) -> PathBuf {
        let unique = Utc::now().timestamp_nanos_opt().unwrap_or_default();
        let root = std::env::temp_dir().join(format!(
            "tf2frag-recovery-{name}-{}-{unique}",
            std::process::id(),
        ));
        fs::create_dir_all(&root).expect("create recovery test root");
        root
    }

    #[test]
    fn primary_tag_creates_a_readable_output_category() {
        let candidate = Candidate {
            tags: vec!["confirmed_airshot".into()],
            primary_tag: "confirmed_airshot".into(),
            ..Candidate::default()
        };
        assert_eq!(candidate_output_category(&candidate), "Confirmed Airshot");
    }

    #[test]
    fn director_session_keeps_tick_tags_and_available_victim_names_together() {
        let candidate = Candidate {
            candidate_id: "r1-p2-t12000".into(),
            source_demo: r"C:\demos\match.dem".into(),
            map_name: "cp_process_final".into(),
            point_of_kill_ticks: vec![12_000, 12_060],
            tick_tags: vec![
                crate::models::TickTagGroup {
                    demo_tick: 12_000,
                    tags: vec!["confirmed_airshot".into()],
                    ..crate::models::TickTagGroup::default()
                },
                crate::models::TickTagGroup {
                    demo_tick: 12_060,
                    tags: vec!["medic_pick".into()],
                    ..crate::models::TickTagGroup::default()
                },
            ],
            sequence_tags: vec!["multi_kill".into()],
            kills: vec![
                json!({"demo_tick": 12_000, "victim_name": "Alice"}),
                json!({"demo_tick": 12_060}),
            ],
            ..Candidate::default()
        };
        let output = Path::new(r"C:\captures\candidate");
        let session = build_director_session(
            &candidate,
            11_000,
            13_000,
            output,
            Path::new(r"C:\Team Fortress 2\tf\tf2fragdemohelper_recording.log"),
            &AppSettings::default(),
        );

        session.validate().unwrap();
        assert_eq!(session.cues.len(), 2);
        assert_eq!(session.cues[0].tags, vec!["confirmed airshot"]);
        assert_eq!(session.cues[0].victims, vec!["Alice"]);
        assert!(session.cues[1].victims.is_empty());
        assert_eq!(session.whole_candidate_tags, vec!["multi kill"]);
        assert_eq!(session.shortcuts.len(), 15);
        assert_eq!(session.shortcuts[0].key, "[");
        assert_eq!(session.shortcuts[7].label, "MIRV camera");
        assert_eq!(session.campath_file, output.join("camera_path.xml"));
        assert_eq!(session.shortcuts[14].key, "S");
        assert_eq!(session.telemetry_marker_prefix, DIRECTOR_TICK_MARKER_PREFIX);
    }

    #[test]
    fn categorized_videos_are_found_during_recording_reconciliation() {
        let root = recovery_test_root("categorized-output");
        let video = root
            .join("Videos")
            .join("Confirmed Airshot")
            .join("clip.mp4");
        fs::create_dir_all(video.parent().unwrap()).expect("create category folder");
        fs::write(&video, b"video").expect("write categorized video");

        assert_eq!(final_recording_outputs(&root), vec![video]);
        fs::remove_dir_all(root).expect("remove categorized output root");
    }

    #[test]
    fn recovery_discovery_is_bounded_and_skips_disabled_sessions() {
        let root = recovery_test_root("discovery");
        for index in 0..5 {
            fs::create_dir_all(root.join(format!("tf2fragdemohelper_batch_{index:02}")))
                .expect("create retained session");
        }
        fs::create_dir_all(root.join("unrelated-folder")).expect("create unrelated folder");
        disable_automatic_recovery(&root.join("tf2fragdemohelper_batch_04"), "test quarantine");

        let active = discover_recording_sessions_in(&root, false, 32, 2);
        assert_eq!(active.sessions.len(), 2);
        assert_eq!(active.disabled_sessions, 1);
        assert_eq!(active.deferred_sessions, 2);

        let including_disabled = discover_recording_sessions_in(&root, true, 32, 32);
        assert_eq!(including_disabled.sessions.len(), 5);
        assert_eq!(including_disabled.disabled_sessions, 1);
        fs::remove_dir_all(root).expect("remove recovery test root");
    }

    #[test]
    fn malformed_manifest_is_retained_and_disabled_instead_of_retried_forever() {
        let root = recovery_test_root("malformed");
        let session = root.join("tf2fragdemohelper_batch_bad");
        fs::create_dir_all(&session).expect("create malformed session");
        fs::write(session.join("recording_manifest.json"), b"{not valid json")
            .expect("write malformed manifest");

        let mut report = RecordingRecoveryReport::default();
        recover_one_recording_session(&session, &AppSettings::default(), &mut report);
        assert!(automatic_recovery_disabled(&session));
        assert_eq!(report.retained_sessions, 1);
        assert_eq!(report.disabled_sessions, 1);
        assert_eq!(report.errors.len(), 1);
        fs::remove_dir_all(root).expect("remove recovery test root");
    }

    #[test]
    fn oversized_manifest_is_rejected_before_it_is_read_into_memory() {
        let root = recovery_test_root("oversized");
        let manifest = root.join("recording_manifest.json");
        let file = File::create(&manifest).expect("create oversized manifest");
        file.set_len(MAX_RECOVERY_MANIFEST_BYTES + 1)
            .expect("size oversized manifest");
        let error = read_recovery_manifest(&manifest).expect_err("oversized manifest must fail");
        assert!(error.to_string().contains("too large"));
        fs::remove_dir_all(root).expect("remove recovery test root");
    }

    #[test]
    fn repeatedly_failing_session_stops_after_three_automatic_attempts() {
        let root = recovery_test_root("retry-limit");
        let session = root.join("tf2fragdemohelper_batch_retry");
        fs::create_dir_all(&session).expect("create retry session");
        let mut report = RecordingRecoveryReport::default();
        for _ in 0..MAX_AUTOMATIC_RECOVERY_ATTEMPTS {
            assert!(begin_automatic_recovery_attempt(&session, &mut report));
        }
        assert!(!begin_automatic_recovery_attempt(&session, &mut report));
        assert!(automatic_recovery_disabled(&session));
        fs::remove_dir_all(root).expect("remove recovery test root");
    }

    #[test]
    fn recording_index_loader_has_an_entry_limit() {
        let root = recovery_test_root("index-limit");
        let path = root.join("recorded_clip_index.ndjson");
        let mut output = File::create(&path).expect("create index fixture");
        for index in 0..3 {
            let entry = RecordingEntry {
                recording_key: format!("key-{index}"),
                candidate_id: format!("candidate-{index}"),
                ..RecordingEntry::default()
            };
            writeln!(output, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
        }
        drop(output);
        let loaded = RecordingIndex::load_from_path(&path, 1024 * 1024, 2);
        assert_eq!(loaded.entries.len(), 2);
        fs::remove_dir_all(root).expect("remove recovery test root");
    }

    #[test]
    fn manual_hotkeys_cycle_every_distinct_kill_tick() {
        let candidate = Candidate {
            point_of_kill_ticks: vec![900, 900, 925, 970],
            ..Candidate::default()
        };
        let cfg = manual_hotkey_cfg(
            &candidate,
            500,
            "demos/tf2fragdemohelper_manual/session/candidate.dem",
            &AppSettings::default(),
        );
        assert!(cfg.contains(
            "playdemo demos/tf2fragdemohelper_manual/session/candidate.dem"
        ));
        assert!(!cfg.contains("demo_gototick 500"));
        assert!(cfg.contains("TF2FRAG_MANUAL_KILL 1/3 TICK 900"));
        assert!(cfg.contains("TF2FRAG_MANUAL_KILL 2/3 TICK 925"));
        assert!(cfg.contains("TF2FRAG_MANUAL_KILL 3/3 TICK 970"));
        assert!(cfg.contains("bind \"1\" \"tf2frag_manual_help\""));
        assert!(cfg.contains("bind \"[\" \"mirv_skip time 0.25\""));
        assert!(cfg.contains("bind \"]\" \"tf2frag_manual_toggle_hud\""));
        assert!(!cfg.contains("RIGHTARROW"));
        assert!(!cfg.contains("UPARROW"));
        assert!(cfg.contains("cl_drawhud 0"));
        assert!(cfg.contains("cl_drawhud 1"));
        assert!(cfg.contains("bind \"6\""));
        assert!(cfg.contains("mirv_input camera"));
        assert!(cfg.contains(
            "mirv_input end; thirdperson; r_drawviewmodel 0; mirv_campath enabled 1"
        ));
        assert!(cfg.contains("bind \"9\" \"tf2frag_manual_start\""));
        assert!(cfg.contains("bind \"0\" \"tf2frag_manual_stop\""));
        assert!(cfg.contains("bind \"=\" \"tf2frag_manual_save\""));
        assert!(!cfg.contains("bind \"F"));
    }

    #[test]
    fn manual_hotkeys_use_saved_custom_keys_and_never_bind_arrows() {
        let mut settings = AppSettings::default();
        settings.mirv_shortcuts.advance_time = "q".into();
        settings.mirv_shortcuts.toggle_hud = "F12".into();
        settings.mirv_shortcuts.safe_restart = "LEFTARROW".into();
        let cfg = manual_hotkey_cfg(
            &Candidate::default(),
            500,
            "demos/tf2fragdemohelper_manual/session/candidate.dem",
            &settings,
        );
        assert!(cfg.contains("bind \"q\" \"mirv_skip time 0.25\""));
        assert!(cfg.contains("bind \"F12\" \"tf2frag_manual_toggle_hud\""));
        assert!(cfg.contains("bind \"3\" \"tf2frag_manual_clip_start\""));
        assert!(!cfg.contains("bind \"LEFTARROW\""));
        assert!(!cfg.contains("bind \"RIGHTARROW\""));
        assert!(!cfg.contains("bind \"UPARROW\""));
        assert!(!cfg.contains("bind \"DOWNARROW\""));
    }

    #[test]
    fn manual_hlae_vdm_pauses_after_seeking_to_selected_start() {
        let vdm = manual_hlae_vdm_text(&Candidate::default(), 500, 530);
        assert!(vdm.contains("skiptotick \"500\""));
        assert!(vdm.contains("starttick \"501\""));
        assert!(vdm.contains("thirdperson; r_drawviewmodel 0; mirv_cmd clear"));
        assert!(vdm.contains("mirv_cmd enabled 1"));
        assert!(vdm.contains(
            "mirv_cmd addCurves tick 501 530 - interp=linear space=abs 501 501 530 530"
        ));
        assert!(vdm.contains("echo TF2FRAG_DIRECTOR_TICK {0}"));
        assert!(vdm.contains("demo_pause; echo TF2FRAG_MANUAL_PAUSED_AT_START"));
        assert!(vdm.contains("echo TF2FRAG_DIRECTOR_TICK 501"));
        assert_eq!(vdm.matches("TF2 MIRV Director live tick").count(), 0);
        assert!(!vdm.contains("exec tf2fragdemohelper_manual"));
        let pause = vdm.find("demo_pause").expect("pause command");
        let seek = vdm.find("skiptotick \"500\"").expect("safe seek command");
        assert!(pause > seek);
    }

    #[test]
    fn manual_hlae_seek_is_split_into_at_most_fifteen_thousand_tick_steps() {
        assert_eq!(
            manual_seek_targets(46_001),
            vec![15_000, 30_000, 45_000, 46_001]
        );
        assert_eq!(manual_seek_targets(15_000), vec![15_000]);
        assert!(manual_seek_targets(0).is_empty());

        let vdm = manual_hlae_vdm_text(&Candidate::default(), 46_001, 46_101);
        for tick in [15_000, 30_000, 45_000, 46_001] {
            assert!(vdm.contains(&format!("skiptotick \"{tick}\"")));
        }
        assert!(vdm.contains("starttick \"46002\""));
    }

    #[test]
    fn manual_encoded_recording_sets_both_frame_rates_and_selected_preset() {
        let settings = AppSettings {
            capture_fps: 240,
            recording_format: "MP4 - Standard".into(),
            ..AppSettings::default()
        };
        let cfg = manual_recording_start_cfg(&settings, Path::new("C:/manual/capture"));
        assert!(cfg.contains("host_framerate 240"));
        assert!(cfg.contains("mirv_streams record fps 240"));
        assert!(cfg.contains("mirv_streams record screen settings tf2FragMp4"));
        assert!(cfg.contains("mirv_streams record name \"C:/manual/capture\""));
    }

    fn recording_profile_suppresses_recorded_vote_and_server_message_panels() {
        let root = recovery_test_root("message-suppression");
        install_recording_message_suppression(&root).expect("install vote HUD suppression");
        let vote_hud = root
            .join(PROFILE_FOLDER)
            .join("resource/ui/votehud.res");
        let text = fs::read_to_string(&vote_hud).expect("read vote HUD suppression");
        assert!(text.contains("\"VoteActive\""));
        assert!(text.contains("\"VoteSetupDialog\""));
        assert!(text.matches("\"visible\" \"0\"").count() >= 2);
        assert!(text.matches("\"wide\" \"0\"").count() >= 2);
        assert!(offline_cfg().contains("sv_allow_votes 0"));
        assert!(offline_cfg().contains("cl_showtextmsg 0"));
        assert!(clean_capture_screen_commands().contains("cl_showpluginmessages 0"));
        fs::remove_dir_all(root).expect("remove message suppression test root");
    }

    #[test]
    fn output_identifier_does_not_reuse_batch_names() {
        let root = std::env::temp_dir().join(format!(
            "tf2frag-recording-identifier-test-{}",
            std::process::id()
        ));
        let base = "demo__candidate__t900-1200__k1234567890abcdef";
        let mut reserved = HashSet::new();

        let first = unique_recording_identifier(&root, "Confirmed Airshot", base, true, "mp4", &reserved);
        assert_eq!(first, base);
        assert!(!first.contains("__cam"));

        reserved.insert(first);
        let second = unique_recording_identifier(&root, "Confirmed Airshot", base, true, "mp4", &reserved);
        assert_eq!(second, format!("{base}_2"));
    }

    #[test]
    fn overlapping_windows_are_split_without_changing_ticks() {
        let windows = vec![(40_201, 40_934), (40_535, 41_268), (42_000, 42_200)];
        let passes = partition_recording_windows(&windows);

        assert_eq!(passes, vec![vec![0, 2], vec![1]]);
        for pass in passes {
            let mut previous_finalize: Option<i64> = None;
            for index in pass {
                let (start, end) = windows[index];
                if let Some(previous) = previous_finalize {
                    assert!(start > previous + VDM_ACTION_GAP_TICKS);
                }
                previous_finalize = Some(end + RECORDING_FLUSH_TICKS);
            }
        }
        assert_eq!(windows[1], (40_535, 41_268));
    }

    #[test]
    fn compatible_windows_stay_in_one_playback_pass() {
        let windows = vec![(1_000, 1_100), (1_300, 1_400), (1_600, 1_700)];
        assert_eq!(partition_recording_windows(&windows), vec![vec![0, 1, 2]]);
    }

    #[test]
    fn recording_estimate_includes_output_working_space_and_headroom() {
        let mut candidate = Candidate::default();
        candidate.point_of_kill_ticks = vec![1_000, 1_067];
        let mut settings = AppSettings::default();
        settings.recording_output_directory = std::env::temp_dir();
        settings.recording_format = "MP4 - Standard".into();
        settings.resolution = "1920x1080".into();
        settings.capture_fps = 120;

        let estimate =
            estimate_recording_space(&[candidate], &settings).expect("recording estimate");
        assert_eq!(estimate.clip_count, 1);
        assert!(estimate.frame_count > 0);
        assert!(estimate.final_output_bytes > 0);
        assert!(estimate.peak_working_bytes > estimate.final_output_bytes);
        assert!(estimate.required_free_bytes > estimate.peak_working_bytes);
    }

    #[test]
    fn completed_session_cleanup_accepts_only_direct_managed_children() {
        let root = PathBuf::from("app-data").join("Recording Sessions");
        assert!(is_managed_recording_session(
            &root,
            &root.join("tf2fragdemohelper_batch_20260825_120000_000")
        ));
        assert!(!is_managed_recording_session(
            &root,
            &root.join("unrelated")
        ));
        assert!(!is_managed_recording_session(
            &root,
            &root
                .join("nested")
                .join("tf2fragdemohelper_batch_20260825_120000_000")
        ));
    }

    #[test]
    fn retained_manifest_preserves_candidate_identity_for_reindexing() {
        let stored = StoredRecordingClip {
            order: 3,
            source_demo: "C:/demos/match.dem".into(),
            candidate_id: "candidate-7".into(),
            recording_key: "portable-recording-key".into(),
            demo_content_signature: "demo-signature".into(),
            start_tick: 9_900,
            end_tick: 10_400,
            candidate_clip_start_tick: Some(10_000),
            candidate_clip_end_tick: Some(10_300),
            attacker_user_id: 17,
            recording_identifier: "clip-identifier".into(),
            expected_output_path: PathBuf::from("C:/outputs/clip.mp4"),
            native_capture_base: "capture-base".into(),
            replace_existing: true,
            ..StoredRecordingClip::default()
        };

        let clip = prepared_clip_from_manifest(&stored);
        assert_eq!(clip.recording_key, "portable-recording-key");
        assert_eq!(clip.demo_signature, "demo-signature");
        assert_eq!(clip.candidate.clip_start_tick, 10_000);
        assert_eq!(clip.candidate.clip_end_tick, 10_300);
        assert_eq!(clip.start_tick, 9_900);
        assert_eq!(clip.end_tick, 10_400);
        assert_eq!(clip.final_output_path, PathBuf::from("C:/outputs/clip.mp4"));
        assert!(clip.replace_existing);
    }

    #[test]
    fn standard_mp4_defaults_to_resolve_compatible_encoding() {
        let settings = AppSettings::default();
        assert_eq!(settings.recording_format, "MP4 - Standard");
        let encoding = effective_mp4_encoding(&settings)
            .expect("valid encoding")
            .expect("MP4 encoding");
        assert_eq!(encoding.pixel_format, "yuv420p");
        assert_eq!(encoding.ffmpeg_profile, "high");
        assert_eq!(encoding.crf, Some(18));
        assert_eq!(encoding.encoder_preset.as_deref(), Some("medium"));
        assert_eq!(encoding.audio_bitrate_kbps, 192);
        assert!(encoding.custom_hlae_preset);
    }

    #[test]
    fn resolve_hlae_command_has_one_compatible_output_format_and_profile() {
        let encoding = effective_mp4_encoding(&AppSettings::default())
            .expect("valid encoding")
            .expect("MP4 encoding");
        let command = hlae_custom_mp4_preset_command(&encoding).expect("custom HLAE command");
        assert!(command.contains("-c:v libx264"));
        assert!(command.contains("-pix_fmt yuv420p"));
        assert!(command.contains("-profile:v high"));
        assert!(command.contains("-crf 18"));
        assert_eq!(command.matches("-pix_fmt").count(), 1);
        assert_eq!(command.matches("-profile:v").count(), 1);
        assert!(command.contains("{FRAMERATE}"));
    }

    #[test]
    fn custom_pixel_format_selects_matching_h264_profile() {
        let mut settings = AppSettings::default();
        settings.mp4_compatibility = "Custom".into();
        settings.mp4_pixel_format = "yuv422p".into();
        settings.normalize_encoding_options();
        let encoding = effective_mp4_encoding(&settings)
            .expect("valid encoding")
            .expect("MP4 encoding");
        assert_eq!(encoding.h264_profile, "High 4:2:2");
        assert_eq!(encoding.ffmpeg_profile, "high422");

        settings.mp4_pixel_format = "yuv444p".into();
        settings.normalize_encoding_options();
        let encoding = effective_mp4_encoding(&settings)
            .expect("valid encoding")
            .expect("MP4 encoding");
        assert_eq!(encoding.h264_profile, "High 4:4:4 Predictive");
        assert_eq!(encoding.ffmpeg_profile, "high444");
    }

    #[test]
    fn advanced_encoding_does_not_override_requested_recording_fps() {
        let mut settings = AppSettings::default();
        settings.capture_fps = 240;
        let clip = PreparedClip {
            working_path: Some(PathBuf::from("C:/working/clip")),
            ..prepared_clip_from_manifest(&StoredRecordingClip::default())
        };
        let command = recording_start_cfg(&clip, &settings);
        assert!(command.contains("host_framerate 240"));
        assert!(command.contains("mirv_streams record fps 240"));
        assert!(command.contains("screen settings tf2FragMp4"));
    }

    #[test]
    fn maximum_color_quality_keeps_original_hlae_preset() {
        let mut settings = AppSettings::default();
        settings.mp4_compatibility = "Maximum Color Quality".into();
        let clip = PreparedClip {
            working_path: Some(PathBuf::from("C:/working/clip")),
            ..prepared_clip_from_manifest(&StoredRecordingClip::default())
        };
        let command = recording_start_cfg(&clip, &settings);
        assert!(command.contains("screen settings afxFfmpeg"));
        assert!(!command.contains("screen settings tf2FragMp4"));
    }

    #[test]
    fn lossless_and_non_mp4_formats_ignore_lossy_advanced_options() {
        for format in [
            "MP4 - Lossless",
            "MOV - DNxHR",
            "AVI - Raw",
            "TGA Image Sequence",
            "JPG Image Sequence",
        ] {
            let mut settings = AppSettings::default();
            settings.recording_format = format.into();
            assert!(effective_mp4_encoding(&settings)
                .expect("valid non-standard format")
                .is_none());
        }
    }

    #[test]
    fn avi_original_preset_remains_the_default_and_advanced_codecs_are_wired() {
        let mut settings = AppSettings::default();
        settings.recording_format = "AVI - Raw".into();
        let original = effective_avi_encoding(&settings)
            .expect("valid AVI settings")
            .expect("AVI encoding");
        assert!(!original.custom_hlae_preset);
        assert!(hlae_custom_avi_preset_command(&original).is_none());

        settings.avi_video_codec = "FFV1 Lossless".into();
        settings.normalize_encoding_options();
        let ffv1 = effective_avi_encoding(&settings)
            .expect("valid FFV1 settings")
            .expect("AVI encoding");
        let command = hlae_custom_avi_preset_command(&ffv1).expect("custom FFV1 command");
        assert!(command.contains("-c:v ffv1"));
        assert!(command.contains("-level 3"));
        assert!(command.contains("-pix_fmt bgr0"));
        assert!(command.contains("video.avi"));

        settings.avi_video_codec = "HuffYUV Lossless".into();
        settings.avi_pixel_format = "rgb24".into();
        settings.normalize_encoding_options();
        let huffyuv = effective_avi_encoding(&settings)
            .expect("valid HuffYUV settings")
            .expect("AVI encoding");
        let command = hlae_custom_avi_preset_command(&huffyuv).expect("custom HuffYUV command");
        assert!(command.contains("-c:v huffyuv"));
        assert!(command.contains("-pix_fmt rgb24"));
    }

    #[test]
    fn dnxhr_profiles_force_ffmpeg_supported_pixel_formats() {
        for (profile, ffmpeg_profile, pixel_format, bit_depth) in [
            ("LB", "dnxhr_lb", "yuv422p", 8),
            ("SQ", "dnxhr_sq", "yuv422p", 8),
            ("HQ", "dnxhr_hq", "yuv422p", 8),
            ("HQX", "dnxhr_hqx", "yuv422p10le", 10),
            ("444", "dnxhr_444", "gbrp10le", 10),
        ] {
            let mut settings = AppSettings::default();
            settings.recording_format = "MOV - DNxHR".into();
            settings.dnxhr_profile = profile.into();
            let encoding = effective_dnxhr_encoding(&settings)
                .expect("valid DNxHR settings")
                .expect("DNxHR encoding");
            assert_eq!(encoding.ffmpeg_profile, ffmpeg_profile);
            assert_eq!(encoding.pixel_format, pixel_format);
            assert_eq!(encoding.bit_depth, bit_depth);
            let command = hlae_custom_dnxhr_preset_command(&encoding);
            let expected_profile = format!("-profile:v {ffmpeg_profile}");
            let expected_pixel_format = format!("-pix_fmt {pixel_format}");
            assert!(command.contains("-c:v dnxhd"));
            assert!(command.contains(expected_profile.as_str()));
            assert!(command.contains(expected_pixel_format.as_str()));
            assert!(command.contains("video.mov"));
        }
    }

    #[test]
    fn dnxhr_and_avi_keep_requested_recording_fps_and_select_their_presets() {
        let clip = PreparedClip {
            working_path: Some(PathBuf::from("C:/working/clip")),
            ..prepared_clip_from_manifest(&StoredRecordingClip::default())
        };
        let mut settings = AppSettings::default();
        settings.capture_fps = 480;
        settings.recording_format = "MOV - DNxHR".into();
        let command = recording_start_cfg(&clip, &settings);
        assert!(command.contains("host_framerate 480"));
        assert!(command.contains("mirv_streams record fps 480"));
        assert!(command.contains("screen settings tf2FragDnxhr"));
        assert_eq!(encoded_extension(&settings), "mov");

        settings.recording_format = "AVI - Raw".into();
        settings.avi_video_codec = "FFV1 Lossless".into();
        settings.normalize_encoding_options();
        let command = recording_start_cfg(&clip, &settings);
        assert!(command.contains("screen settings tf2FragAvi"));
        assert_eq!(encoded_extension(&settings), "avi");
    }
}
