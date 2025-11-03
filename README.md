# OtNvr

OtNvr is a Rust command-line application that uses FFmpeg to capture an RTSP stream and remux it to HTTP Live Streaming (HLS) assets (an `.m3u8` playlist and `.ts` segments).

## Features

- JSON configuration file describes the RTSP source, optional recording duration, and HLS output options.
- HLS output supports custom segment duration, playlist window size, and segment filename patterns.

## Prerequisites

- Rust toolchain (via [rustup](https://rustup.rs/)).
- FFmpeg libraries available on the system.

Run `./scripts/install-deps.sh` for a guided dependency setup on macOS or common Linux distributions.

## Configuration

Create a JSON file that matches the structure in [`config.example.json`](config.example.json). Example:

```json
{
  "rtsp_url": "rtsp://user:password@camera.example.com/stream",
  "duration_seconds": 60,
  "hls": {
    "playlist_path": "output/stream.m3u8",
    "segment_duration_seconds": 4,
    "playlist_size": 10,
    "segment_filename": "output/stream_%05d.ts"
  }
}
```

## Usage

```bash
cargo run --release path/to/config.json
```

The application initializes FFmpeg, attaches to the RTSP source, and writes HLS files according to the configuration. If `duration_seconds` is omitted, the program runs until interrupted (Ctrl+C).
