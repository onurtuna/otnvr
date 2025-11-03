pub mod config;
pub mod recorder;

pub use config::{AppConfig, HlsConfig, RecordingConfig};
pub use recorder::{HlsOutput, RecorderError, RtspRecorder, derive_segment_template};
