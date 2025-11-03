mod error;
mod hls_output;
mod rtsp_recorder;

pub use error::RecorderError;
pub use hls_output::HlsOutput;
pub use rtsp_recorder::{RtspRecorder, derive_segment_template};
