use std::path::PathBuf;

use otnvr::recorder::{HlsOutput, VideoCodec, derive_segment_template};

fn base_hls_output() -> HlsOutput {
    HlsOutput {
        playlist_path: PathBuf::from("output/stream.m3u8"),
        segment_duration: Some(4),
        playlist_size: Some(10),
        segment_filename: None,
        video_codec: VideoCodec::H264,
    }
}

#[test]
fn derive_segment_template_uses_custom_pattern() {
    let mut hls = base_hls_output();
    hls.segment_filename = Some("custom/segment_%03d.ts".to_string());

    let template = derive_segment_template(&hls);

    assert_eq!(template, "custom/segment_%03d.ts");
}

#[test]
fn derive_segment_template_builds_default_pattern_next_to_playlist() {
    let hls = base_hls_output();

    let template = derive_segment_template(&hls);

    assert_eq!(template, "output/stream_%05d.ts");
}

#[test]
fn derive_segment_template_handles_playlist_without_parent() {
    let hls = HlsOutput {
        playlist_path: PathBuf::from("stream.m3u8"),
        segment_duration: None,
        playlist_size: None,
        segment_filename: None,
        video_codec: VideoCodec::H264,
    };

    let template = derive_segment_template(&hls);

    assert_eq!(template, "stream_%05d.ts");
}

#[test]
fn derive_segment_template_switches_extension_for_h265() {
    let mut hls = base_hls_output();
    hls.video_codec = VideoCodec::H265;

    let template = derive_segment_template(&hls);

    assert_eq!(template, "output/stream_%05d.m4s");
}

#[test]
fn derive_segment_template_overrides_custom_extension_for_h265() {
    let mut hls = base_hls_output();
    hls.segment_filename = Some("custom/segment_%03d.ts".to_string());
    hls.video_codec = VideoCodec::H265;

    let template = derive_segment_template(&hls);

    assert_eq!(template, "custom/segment_%03d.m4s");
}
