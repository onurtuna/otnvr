use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use ffmpeg_next::{Dictionary, Rational, codec, encoder, format, log, media};

use super::{HlsOutput, RecorderError};

/// High-level orchestrator that translates RTSP input into a file-based HLS presentation.
pub struct RtspRecorder {
    _private: (),
}

impl RtspRecorder {
    /// Initializes FFmpeg (idempotent) and prepares a recorder instance.
    pub fn new() -> Result<Self, RecorderError> {
        ffmpeg_next::init()?;
        log::set_level(log::Level::Warning);
        Ok(Self { _private: () })
    }

    /// Copies packets from the provided RTSP source into an HLS muxer until the optional duration elapses.
    pub fn record(
        &self,
        rtsp_url: &str,
        hls_output: &HlsOutput,
        duration_limit: Option<Duration>,
    ) -> Result<(), RecorderError> {
        let mut ictx = format::input(&rtsp_url)?;
        let playlist_path = hls_output.playlist_path.as_path();

        if let Some(parent) = playlist_path.parent() {
            if !parent.as_os_str().is_empty() {
                // Ensure the target directory exists so FFmpeg can create playlist and segment files.
                fs::create_dir_all(parent)?;
            }
        }

        let mut format_options = Dictionary::new();

        if let Some(duration) = hls_output.segment_duration {
            format_options.set("hls_time", &duration.to_string());
        }

        if let Some(size) = hls_output.playlist_size {
            format_options.set("hls_list_size", &size.to_string());
        }

        let segment_template = derive_segment_template(hls_output);
        format_options.set("hls_segment_filename", &segment_template);

        let mut octx = format::output_as_with(&hls_output.playlist_path, "hls", format_options)?;

        let nb_streams = ictx.nb_streams() as usize;
        let mut stream_mapping = vec![-1; nb_streams];
        let mut ist_time_bases = vec![Rational(0, 1); nb_streams];
        let mut ost_index = 0usize;

        for (ist_index, ist) in ictx.streams().enumerate() {
            let medium = ist.parameters().medium();
            if medium != media::Type::Video && medium != media::Type::Audio {
                continue;
            }

            stream_mapping[ist_index] = ost_index as isize;
            ist_time_bases[ist_index] = ist.time_base();

            let mut ost = octx.add_stream(encoder::find(codec::Id::None))?;
            ost.set_parameters(ist.parameters());

            unsafe {
                (*ost.parameters().as_mut_ptr()).codec_tag = 0;
            }

            ost_index += 1;
        }

        if ost_index == 0 {
            return Err(RecorderError::MissingMediaStreams);
        }

        octx.set_metadata(ictx.metadata().to_owned());
        octx.write_header()?;

        let start = Instant::now();

        for (stream, mut packet) in ictx.packets() {
            let ist_index = stream.index();
            let mapping = stream_mapping[ist_index];
            if mapping < 0 {
                continue;
            }
            let mapping = mapping as usize;

            let ost = octx
                .stream(mapping)
                .ok_or(RecorderError::InvalidStreamMapping(mapping))?;
            packet.rescale_ts(ist_time_bases[ist_index], ost.time_base());
            packet.set_position(-1);
            packet.set_stream(mapping);
            packet.write_interleaved(&mut octx)?;

            if let Some(limit) = duration_limit {
                if start.elapsed() >= limit {
                    break;
                }
            }
        }

        octx.write_trailer()?;
        Ok(())
    }
}

pub fn derive_segment_template(hls_output: &HlsOutput) -> String {
    if let Some(template) = &hls_output.segment_filename {
        return template.clone();
    }

    let playlist_path = hls_output.playlist_path.as_path();
    let parent = playlist_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = playlist_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("segment");

    parent
        .join(format!("{stem}_%05d.ts"))
        .to_string_lossy()
        .to_string()
}
