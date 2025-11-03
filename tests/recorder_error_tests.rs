use std::{error::Error, io};

use ffmpeg_next::error::Error as FfmpegError;
use otnvr::recorder::RecorderError;

#[test]
fn from_ffmpeg_error_wraps_value() {
    let error = RecorderError::from(FfmpegError::Bug);

    match error {
        RecorderError::Ffmpeg(inner) => assert_eq!(inner, FfmpegError::Bug),
        _ => panic!("expected Ffmpeg variant"),
    }
}

#[test]
fn source_returns_underlying_io_error() {
    let io_error = io::Error::new(io::ErrorKind::Other, "test");
    let recorder_error = RecorderError::from(io_error);

    let source = recorder_error.source().unwrap();
    assert_eq!(source.to_string(), "test");
}
