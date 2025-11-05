use std::error::Error;
use std::fmt;

use ffmpeg_next::Error as FfmpegError;

use super::VideoCodec;

/// Errors that can surface while preparing or recording an RTSP stream to HLS.
#[derive(Debug)]
pub enum RecorderError {
    /// Wrapper around FFmpeg errors emitted by the `ffmpeg-next` bindings.
    Ffmpeg(FfmpegError),
    /// Triggered when the input stream lacks audio and video tracks worth remuxing.
    MissingMediaStreams,
    /// Emitted when a packet references an output stream that was never created.
    InvalidStreamMapping(usize),
    /// I/O failures such as creating directories or writing playlist/segment files.
    Io(std::io::Error),
    /// Requested video codec is unavailable or unsupported by the current FFmpeg build.
    UnsupportedVideoCodec(VideoCodec),
}

impl fmt::Display for RecorderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecorderError::Ffmpeg(err) => write!(f, "{err}"),
            RecorderError::MissingMediaStreams => {
                write!(f, "input did not contain audio or video streams")
            }
            RecorderError::InvalidStreamMapping(index) => {
                write!(f, "invalid stream mapping for output stream {index}")
            }
            RecorderError::Io(err) => write!(f, "{err}"),
            RecorderError::UnsupportedVideoCodec(codec) => {
                write!(f, "unsupported video codec requested: {:?}", codec)
            }
        }
    }
}

impl Error for RecorderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RecorderError::Ffmpeg(err) => Some(err),
            RecorderError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<FfmpegError> for RecorderError {
    fn from(value: FfmpegError) -> Self {
        RecorderError::Ffmpeg(value)
    }
}

impl From<std::io::Error> for RecorderError {
    fn from(value: std::io::Error) -> Self {
        RecorderError::Io(value)
    }
}
