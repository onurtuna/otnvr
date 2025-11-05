use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use ffmpeg_next::{
    Dictionary, Packet, Rational, codec, decoder, encoder, format, frame, log, media, picture,
};

use super::{HlsOutput, RecorderError, VideoCodec};

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
        if matches!(hls_output.video_codec, VideoCodec::H265) {
            format_options.set("hls_segment_type", "fmp4");
        }

        let mut octx = format::output_as_with(&hls_output.playlist_path, "hls", format_options)?;

        let nb_streams = ictx.nb_streams() as usize;
        let mut stream_mapping = vec![-1; nb_streams];
        let mut ist_time_bases = vec![Rational(0, 1); nb_streams];
        let mut ost_time_bases = vec![Rational(0, 1); nb_streams];
        let mut video_transcoders: HashMap<usize, VideoTranscoder> = HashMap::new();
        let mut ost_index = 0usize;

        for (ist_index, ist) in ictx.streams().enumerate() {
            let medium = ist.parameters().medium();
            if medium != media::Type::Video && medium != media::Type::Audio {
                continue;
            }

            stream_mapping[ist_index] = ost_index as isize;
            ist_time_bases[ist_index] = ist.time_base();

            if medium == media::Type::Video {
                let transcoder = VideoTranscoder::new(
                    &ist,
                    &mut octx,
                    ost_index,
                    hls_output.video_codec,
                )?;
                video_transcoders.insert(ist_index, transcoder);
            } else {
                let mut ost = octx.add_stream(encoder::find(codec::Id::None))?;
                ost.set_parameters(ist.parameters());

                unsafe {
                    (*ost.parameters().as_mut_ptr()).codec_tag = 0;
                }
            }

            ost_index += 1;
        }

        if ost_index == 0 {
            return Err(RecorderError::MissingMediaStreams);
        }

        ost_time_bases.truncate(ost_index);

        octx.set_metadata(ictx.metadata().to_owned());
        octx.write_header()?;

        for index in 0..ost_index {
            let stream = octx
                .stream(index)
                .ok_or(RecorderError::InvalidStreamMapping(index))?;
            ost_time_bases[index] = stream.time_base();
        }

        let start = Instant::now();

        for (stream, mut packet) in ictx.packets() {
            let ist_index = stream.index();
            let mapping = stream_mapping[ist_index];
            if mapping < 0 {
                continue;
            }
            let mapping = mapping as usize;

            let ost_time_base = ost_time_bases[mapping];

            if let Some(transcoder) = video_transcoders.get_mut(&ist_index) {
                transcoder.send_packet_to_decoder(&packet)?;
                transcoder.receive_and_process_decoded_frames(&mut octx, ost_time_base)?;
            } else {
                packet.rescale_ts(ist_time_bases[ist_index], ost_time_base);
                packet.set_position(-1);
                packet.set_stream(mapping);
                packet.write_interleaved(&mut octx)?;
            }

            if let Some(limit) = duration_limit {
                if start.elapsed() >= limit {
                    break;
                }
            }
        }

        for (ist_index, transcoder) in video_transcoders.iter_mut() {
            let mapping = stream_mapping[*ist_index];
            if mapping < 0 {
                continue;
            }
            let mapping = mapping as usize;
            let ost_time_base = ost_time_bases[mapping];

            transcoder.send_eof_to_decoder()?;
            transcoder.receive_and_process_decoded_frames(&mut octx, ost_time_base)?;
            transcoder.send_eof_to_encoder()?;
            transcoder.receive_and_process_encoded_packets(&mut octx, ost_time_base)?;
        }

        octx.write_trailer()?;
        Ok(())
    }
}

struct VideoTranscoder {
    decoder: decoder::Video,
    encoder: encoder::Video,
    input_time_base: Rational,
    ost_index: usize,
}

impl VideoTranscoder {
    fn new(
        ist: &format::stream::Stream,
        octx: &mut format::context::Output,
        ost_index: usize,
        codec: VideoCodec,
    ) -> Result<Self, RecorderError> {
        let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);
        let decoder = ffmpeg_next::codec::context::Context::from_parameters(ist.parameters())?
            .decoder()
            .video()?;

        let codec_id = match codec {
            VideoCodec::H264 => codec::Id::H264,
            VideoCodec::H265 => codec::Id::HEVC,
        };

        let encoder_codec = encoder::find(codec_id)
            .ok_or(RecorderError::UnsupportedVideoCodec(codec))?;

        let mut ost = octx.add_stream(Some(encoder_codec))?;
        let mut encoder_context =
            ffmpeg_next::codec::context::Context::new_with_codec(encoder_codec)
                .encoder()
                .video()?;

        encoder_context.set_height(decoder.height());
        encoder_context.set_width(decoder.width());
        encoder_context.set_aspect_ratio(decoder.aspect_ratio());
        encoder_context.set_format(decoder.format());
        encoder_context.set_frame_rate(decoder.frame_rate());
        encoder_context.set_time_base(ist.time_base());

        if global_header {
            encoder_context.set_flags(codec::Flags::GLOBAL_HEADER);
        }

        ost.set_parameters(&encoder_context);
        let options = encoder_options(codec);
        let opened_encoder = encoder_context.open_with(options)?;
        ost.set_parameters(&opened_encoder);

        Ok(Self {
            decoder,
            encoder: opened_encoder,
            input_time_base: ist.time_base(),
            ost_index,
        })
    }

    fn send_packet_to_decoder(&mut self, packet: &Packet) -> Result<(), RecorderError> {
        self.decoder.send_packet(packet)?;
        Ok(())
    }

    fn send_eof_to_decoder(&mut self) -> Result<(), RecorderError> {
        self.decoder.send_eof()?;
        Ok(())
    }

    fn send_eof_to_encoder(&mut self) -> Result<(), RecorderError> {
        self.encoder.send_eof()?;
        Ok(())
    }

    fn receive_and_process_decoded_frames(
        &mut self,
        octx: &mut format::context::Output,
        ost_time_base: Rational,
    ) -> Result<(), RecorderError> {
        let mut frame = frame::Video::empty();
        while self.decoder.receive_frame(&mut frame).is_ok() {
            let timestamp = frame.timestamp();
            frame.set_pts(timestamp);
            frame.set_kind(picture::Type::None);
            self.encoder.send_frame(&frame)?;
            self.receive_and_process_encoded_packets(octx, ost_time_base)?;
        }
        Ok(())
    }

    fn receive_and_process_encoded_packets(
        &mut self,
        octx: &mut format::context::Output,
        ost_time_base: Rational,
    ) -> Result<(), RecorderError> {
        let mut encoded = Packet::empty();
        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(self.ost_index);
            encoded.rescale_ts(self.input_time_base, ost_time_base);
            encoded.set_position(-1);
            encoded.write_interleaved(octx)?;
        }
        Ok(())
    }
}

fn encoder_options(codec: VideoCodec) -> Dictionary<'static> {
    let mut options = Dictionary::new();
    match codec {
        VideoCodec::H264 => {
            options.set("preset", "veryfast");
            options.set("crf", "23");
        }
        VideoCodec::H265 => {
            options.set("preset", "medium");
            options.set("crf", "28");
        }
    }
    options
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

    let extension = match hls_output.video_codec {
        VideoCodec::H264 => "ts",
        VideoCodec::H265 => "m4s",
    };

    parent
        .join(format!("{stem}_%05d.{extension}"))
        .to_string_lossy()
        .to_string()
}
