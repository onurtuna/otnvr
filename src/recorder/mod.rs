mod error;
mod hls_output;
mod rtsp_recorder;

pub use error::RecorderError;
pub use hls_output::{HlsOutput, VideoCodec};
pub use rtsp_recorder::{RtspRecorder, derive_segment_template};
