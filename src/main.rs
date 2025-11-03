use otnvr::config::AppConfig;
use otnvr::recorder::RtspRecorder;
use std::fs;
use std::process;

fn main() {
    let mut args = std::env::args();
    let app = args.next().unwrap_or_else(|| "rtsp-recorder".to_string());

    let config_path = match args.next() {
        Some(path) => path,
        None => {
            print_usage(&app);
            process::exit(1);
        }
    };

    let config_contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("Failed to read config file {config_path}: {error}");
            process::exit(1);
        }
    };

    // Parse the configuration JSON that drives the recorder session.
    let config: AppConfig = match serde_json::from_str(&config_contents) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Invalid config JSON: {error}");
            process::exit(1);
        }
    };

    if config.recordings.is_empty() {
        eprintln!("No recordings defined in configuration.");
        process::exit(1);
    }

    let recorder = match RtspRecorder::new() {
        Ok(recorder) => recorder,
        Err(error) => {
            eprintln!("Failed to initialize FFmpeg recorder: {error}");
            process::exit(1);
        }
    };

    for (index, recording) in config.recordings.iter().enumerate() {
        let duration_limit = recording.duration();
        let duration_summary = duration_limit
            .as_ref()
            .map(|d| format!(" (captured for {} seconds)", d.as_secs()));
        let hls_output = recording.hls_output();

        println!(
            "Recording {}: capturing {} -> {}",
            index + 1,
            recording.rtsp_url,
            hls_output.playlist_path.display()
        );

        if let Err(error) = recorder.record(&recording.rtsp_url, &hls_output, duration_limit) {
            eprintln!(
                "Failed to record RTSP stream for {}: {error}",
                recording.rtsp_url
            );
            process::exit(1);
        }

        println!(
            "Recording {} complete: playlist at {}{}",
            index + 1,
            hls_output.playlist_path.display(),
            duration_summary.unwrap_or_default()
        );
    }
}

fn print_usage(app: &str) {
    eprintln!("Usage: {app} <config-file>");
}
