# OtNvr

OtNvr is a Rust command-line application that uses FFmpeg to capture one or more RTSP streams and remux them to HTTP Live Streaming (HLS) assets (an `.m3u8` playlist and `.ts` segments).

## Features

- JSON configuration file describes any number of RTSP sources, optional per-stream durations, and HLS output options.
- HLS output supports custom segment duration, playlist window size, segment filename patterns, and selectable H.264/H.265 encoding.

## Prerequisites

- Rust toolchain (via [rustup](https://rustup.rs/)).
- FFmpeg libraries available on the system.

## Configuration

Create a JSON file that matches the structure in [`config.example.json`](config.example.json). Example:

```json
{
  "recordings": [
    {
      "rtsp_url": "rtsp://user:password@camera-one.example.com/stream",
      "duration_seconds": 60,
      "hls": {
        "playlist_path": "output/camera-one/stream.m3u8",
        "segment_duration_seconds": 4,
        "playlist_size": 10,
        "segment_filename": "output/camera-one/stream_%05d.ts",
        "video_codec": "h264"
      }
    },
    {
      "rtsp_url": "rtsp://user:password@camera-two.example.com/stream",
      "hls": {
        "playlist_path": "output/camera-two/stream.m3u8",
        "segment_duration_seconds": 6,
        "playlist_size": 5,
        "video_codec": "h265"
      }
    }
  ]
}
```

Omit `video_codec` to default to H.264, or set it to `"h265"` to transcode the video stream to HEVC with fragmented MP4 segments.

## Usage

```bash
cargo run --release path/to/config.json
```

The application initializes FFmpeg, then iterates over each configured recording, attaching to the RTSP source and writing HLS files according to the per-recording settings. If `duration_seconds` is omitted for a recording, that stream runs until interrupted (Ctrl+C).
