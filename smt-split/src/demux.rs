#![allow(nonstandard_style)]

// Inspired from FFmpeg's transcode_aac.c example.
//
// Original Copyright (c) 2013-2022 Andreas Unterweger
//
// Disclaimer:
// Some brain cells were harmed while encoding flacs with proper duration header.

use super::{
  error::SplitError,
  metadata::{AvLibTagger, CodecMetadataTagger, Id3Tagger, MetadataContainer, VorbisTagger},
};
use cue_lib::{core::CueStr, parse::Cuesheet};
use smt_ffmpeg::{
  codec::{context::AvCodecContext, frame::AvFrame, packet::AvPacket},
  error::{AvError, AvLibError},
  ffmpeg::{
    AV_CODEC_FLAG_GLOBAL_HEADER, AVCodecID_AV_CODEC_ID_FLAC, AVCodecID_AV_CODEC_ID_MP3,
    AVCodecID_AV_CODEC_ID_MP3ADU, AVCodecID_AV_CODEC_ID_MP3ON4, AVFMT_GLOBALHEADER, AVStream,
  },
  format::{
    context::{AvContext, AvInputContext, AvOutputContext},
    stream::{StreamType, copy_stream_properties},
  },
  util::{audio_fifo::AudioFifo, dictionary::AvDictionaryRef, timestamp::AvTimestamp},
};
use std::{ffi::CStr, io::ErrorKind, path::Path};

macro_rules! static_cstr {
  ($value:literal) => {
    unsafe { &CStr::from_bytes_with_nul_unchecked(concat!($value, "\0").as_bytes()) }
  };
}

const COVER_IMAGE_KEY: &'static CStr = static_cstr!("comment");
const COVER_IMAGE_VALUE: &'static CStr = static_cstr!("Cover (front)");
const EXT_FLAC: &'static str = "flac";
const EXT_MP3: &'static str = "mp3";
const UNTITLED_TRACK: CueStr<'static> = CueStr::Text("untitled");

static AV_LIB_TAGGER: AvLibTagger = AvLibTagger;
static ID3_TAGGER: Id3Tagger = Id3Tagger;
static VORBIS_TAGGER: VorbisTagger = VorbisTagger;

struct SplitOutput {
  start_time: AvTimestamp,
  end_time: Option<AvTimestamp>,
  context: AvOutputContext,
  encoder: AvCodecContext,
  samples: AudioFifo,
}

pub struct SplitDemuxer {
  outputs: Vec<SplitOutput>,
  cover_image_packets: Vec<AvPacket>,
}

impl SplitDemuxer {
  pub fn init<I, O>(
    input_path: I,
    output_dir: O,
    cuesheet: &Cuesheet<'_>,
  ) -> Result<Self, SplitError>
  where
    I: AsRef<Path>,
    O: AsRef<Path>,
  {
    if cuesheet.tracks.len() < 2 {
      return Err(SplitError::NothingToSplit);
    }

    if output_dir.as_ref().exists() {
      if !output_dir.as_ref().is_dir() {
        return Err(SplitError::InvalidOutputDir(
          output_dir.as_ref().to_path_buf(),
        ));
      }
    } else {
      std::fs::create_dir_all(&output_dir)?;
    }

    let input = AvInputContext::open_path(input_path.as_ref())?;
    let audio_stream = input
      .find_best_stream(StreamType::Audio)
      .ok_or(SplitError::NothingToSplit)
      .and_then(|v| {
        if v.codecpar.is_null() {
          Err(SplitError::UnknownAudioContainer)
        } else {
          Ok(v)
        }
      })?;

    let audio_codec = unsafe { &*audio_stream.codecpar };

    let (file_extension, tagger) = match audio_codec.codec_id {
      AVCodecID_AV_CODEC_ID_MP3 | AVCodecID_AV_CODEC_ID_MP3ADU | AVCodecID_AV_CODEC_ID_MP3ON4 => {
        (EXT_MP3, &ID3_TAGGER as &dyn CodecMetadataTagger)
      }
      AVCodecID_AV_CODEC_ID_FLAC => (EXT_FLAC, &VORBIS_TAGGER as &dyn CodecMetadataTagger),
      _ => input_path
        .as_ref()
        .extension()
        .map(|v| v.to_str())
        .flatten()
        .map(|v| (v, &AV_LIB_TAGGER as &dyn CodecMetadataTagger))
        .ok_or(SplitError::UnknownAudioContainer)?,
    };

    let mut decoder = AvCodecContext::new_decoder(audio_codec.codec_id);
    decoder.copy_params_from(audio_codec)?;
    decoder.pkt_timebase = audio_stream.time_base;
    decoder.time_base.den = audio_codec.sample_rate;
    decoder.time_base.num = 1;
    decoder.open()?;

    let input_cover_stream = find_cover_image_stream(&input);
    let mut outputs = Vec::with_capacity(cuesheet.tracks.len());

    for track_info in cuesheet.tracks.iter() {
      let file_name = format!(
        "{no} {title}",
        no = track_info.no,
        title = track_info.title.unwrap_or(UNTITLED_TRACK),
      );

      let output_path = output_dir
        .as_ref()
        .join(file_name)
        .with_added_extension(file_extension);

      let mut context = AvOutputContext::open_path(output_path)?;
      let flags = context.flags;
      let output_audio = context.create_stream()?;
      let mut encoder = AvCodecContext::new_encoder(audio_codec.codec_id);
      encoder.copy_params_from(audio_codec)?;
      encoder.frame_size = decoder.frame_size;
      encoder.time_base.den = audio_codec.sample_rate;
      encoder.time_base.num = 1;

      if (flags & AVFMT_GLOBALHEADER as i32) == AVFMT_GLOBALHEADER as i32 {
        encoder.flags |= AV_CODEC_FLAG_GLOBAL_HEADER as i32;
      }

      encoder.copy_params_to(unsafe { &mut *output_audio.codecpar })?;

      if let Some(cover) = input_cover_stream {
        let output_cover_stream = context.create_stream()?;
        copy_stream_properties(cover, output_cover_stream)?;
      }

      let mut output_metadata = MetadataContainer::new(context.metadata_mut(), tagger);
      let input_metadata = input.metadata();

      output_metadata.insert_from_av_dict(input_metadata.iter());
      output_metadata.insert_from_cuesheet(cuesheet);
      output_metadata.insert_from_track(track_info);

      outputs.push(SplitOutput {
        context,
        encoder,
        end_time: track_info.time_info.end.map(|v| v.as_duration().into()),
        samples: AudioFifo::new(decoder.sample_fmt, decoder.ch_layout.nb_channels),
        start_time: track_info.time_info.start.as_duration().into(),
      });
    }

    let mut pkt = AvPacket::try_new()?;
    let mut cover_image_packets = Vec::new();

    'DECODER: loop {
      pkt.reset();

      match input.read_frame(&mut pkt) {
        Ok(()) => {
          if pkt.stream_index == audio_stream.index {
            let frame_ts = AvTimestamp::new(pkt.pts, decoder.pkt_timebase);

            for output in outputs.iter_mut() {
              if frame_ts >= output.start_time && output.end_time.is_none_or(|v| frame_ts <= v) {
                decoder.send_packet(&pkt)?;
                let mut frame = AvFrame::try_new()?;

                match decoder.receive_frame(&mut frame) {
                  Ok(()) => {
                    output.samples.push(frame.data.as_ptr(), frame.nb_samples)?;
                    break;
                  }
                  Err(AvError::IOError(err)) if err.kind() == ErrorKind::WouldBlock => {
                    continue 'DECODER;
                  }
                  Err(other) => return Err(other.into()),
                }
              }
            }
          } else if let Some(cover_stream) = input_cover_stream
            && pkt.stream_index == cover_stream.index
          {
            cover_image_packets.push(pkt.clone());
          }
        }
        Err(AvError::AvLibError(AvLibError::Eof)) => {
          break 'DECODER;
        }
        Err(err) => {
          return Err(err.into());
        }
      }
    }

    Ok(Self {
      cover_image_packets,
      outputs,
    })
  }

  pub fn split(self) -> Result<(), SplitError> {
    for mut output in self.outputs.into_iter() {
      let cover_stream_idx = find_cover_image_stream(&output.context).map(|v| v.index);
      let mut pts: i64 = 0;
      let mut writer = output.context.start_writer()?;

      output.encoder.open()?;
      writer.write_header()?;

      if let Some(cover_idx) = cover_stream_idx {
        let mut cover_packet = AvPacket::try_new()?;

        for pkt in self.cover_image_packets.iter() {
          cover_packet.ref_from(pkt)?;
          cover_packet.stream_index = cover_idx;
          writer.write_frame(&mut cover_packet)?;
          cover_packet.reset();
        }
      }

      let mut audio_packet = AvPacket::try_new()?;
      let mut frame = AvFrame::try_new()?;

      while !output.samples.is_empty() {
        frame.reset();
        let frame_size = {
          let estimated = output.samples.len().min(output.encoder.frame_size as usize) as i32;
          if estimated <= 0 { 4096 } else { estimated }
        };

        frame.nb_samples = frame_size;
        frame.copy_channel_layout(&output.encoder.ch_layout)?;
        frame.format = output.encoder.sample_fmt;
        frame.sample_rate = output.encoder.sample_rate;
        frame.alloc_buffer()?;

        let read = output.samples.pop(frame.data.as_ptr(), frame_size)?;

        frame.pts = pts;
        pts += read as i64;

        output.encoder.send_frame(&frame)?;

        audio_packet.reset();
        match output.encoder.receive_packet(&mut audio_packet) {
          Ok(()) => {
            writer.write_frame(&mut audio_packet)?;
          }
          Err(AvError::AvLibError(AvLibError::Eof)) => break,
          Err(AvError::IOError(err)) if err.kind() == ErrorKind::WouldBlock => {
            continue;
          }
          Err(err) => return Err(err.into()),
        }
      }

      output.encoder.flush()?;

      // remaining packets from encoder
      loop {
        audio_packet.reset();
        match output.encoder.receive_packet(&mut audio_packet) {
          Ok(()) => {
            writer.write_frame(&mut audio_packet)?;
          }
          Err(AvError::AvLibError(AvLibError::Eof)) => break,
          Err(AvError::IOError(err)) if err.kind() == ErrorKind::WouldBlock => {
            continue;
          }
          Err(err) => return Err(err.into()),
        }
      }

      writer.finish()?;
    }

    Ok(())
  }
}

fn find_cover_image_stream(input: &AvContext) -> Option<&AVStream> {
  if let Some(stream) = input.find_best_stream(StreamType::Video) {
    let metadata = AvDictionaryRef::from_ptr_ref(&stream.metadata);

    if let Some(value) = metadata.get(COVER_IMAGE_KEY) {
      if value == COVER_IMAGE_VALUE {
        return Some(stream);
      }
    }
  }

  None
}
