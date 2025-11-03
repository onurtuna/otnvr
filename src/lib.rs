pub mod config;
pub mod recorder;

pub use config::{AppConfig, HlsConfig};
pub use recorder::{HlsOutput, RecorderError, RtspRecorder, derive_segment_template};
