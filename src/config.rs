use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;

use crate::recorder::VideoCodec;

/// Top-level configuration describing one or more RTSP-to-HLS capture jobs.
#[derive(Deserialize)]
pub struct AppConfig {
    /// Collection of recordings that should be executed by the application.
    #[serde(default)]
    pub recordings: Vec<RecordingConfig>,
}

/// Parameters for an individual RTSP recording job.
#[derive(Deserialize)]
pub struct RecordingConfig {
    /// Network location of the RTSP source, including credentials if required.
    pub rtsp_url: String,
    /// Optional wall-clock duration (seconds) to limit how long the recorder runs.
    #[serde(default)]
    pub duration_seconds: Option<u64>,
    /// Parameters that control details of the generated HLS output.
    pub hls: HlsConfig,
}

/// Nested configuration block for HLS muxer options.
#[derive(Deserialize)]
pub struct HlsConfig {
    /// Destination path for the `.m3u8` playlist; segment paths are derived from this.
    pub playlist_path: String,
    /// Optional segment duration, in seconds. The FFmpeg muxer rounds as needed.
    #[serde(default)]
    pub segment_duration_seconds: Option<u32>,
    /// Optional clamp for how many segment URIs remain in the sliding playlist window.
    #[serde(default)]
    pub playlist_size: Option<u32>,
    /// Optional custom segment filename pattern. Supports FFmpeg printf-style counters.
    #[serde(default)]
    pub segment_filename: Option<String>,
    /// Desired codec for the encoded video stream within the HLS segments.
    #[serde(default = "default_video_codec")]
    pub video_codec: VideoCodec,
}

impl RecordingConfig {
    /// Returns the optional duration limit as a `Duration`.
    pub fn duration(&self) -> Option<Duration> {
        self.duration_seconds.map(Duration::from_secs)
    }

    /// Converts the configuration into an `HlsOutput` suitable for the recorder.
    pub fn hls_output(&self) -> crate::recorder::HlsOutput {
        crate::recorder::HlsOutput {
            playlist_path: PathBuf::from(&self.hls.playlist_path),
            segment_duration: self.hls.segment_duration_seconds,
            playlist_size: self.hls.playlist_size,
            segment_filename: self.hls.segment_filename.clone(),
            video_codec: self.hls.video_codec,
        }
    }
}

fn default_video_codec() -> VideoCodec {
    VideoCodec::default()
}
