use std::path::PathBuf;
use std::time::Duration;

use otnvr::{AppConfig, HlsConfig};

fn build_config() -> AppConfig {
    AppConfig {
        rtsp_url: "rtsp://example.com/stream".to_string(),
        duration_seconds: Some(42),
        hls: HlsConfig {
            playlist_path: "out/stream.m3u8".to_string(),
            segment_duration_seconds: Some(6),
            playlist_size: Some(5),
            segment_filename: Some("out/segments_%04d.ts".to_string()),
        },
    }
}

#[test]
fn duration_converts_seconds_to_duration() {
    let config = build_config();

    let duration = config.duration().expect("missing duration");

    assert_eq!(duration, Duration::from_secs(42));
}

#[test]
fn hls_output_translates_config_into_struct() {
    let config = build_config();

    let hls = config.hls_output();

    assert_eq!(hls.playlist_path, PathBuf::from("out/stream.m3u8"));
    assert_eq!(hls.segment_duration, Some(6));
    assert_eq!(hls.playlist_size, Some(5));
    assert_eq!(
        hls.segment_filename.as_deref(),
        Some("out/segments_%04d.ts")
    );
}
