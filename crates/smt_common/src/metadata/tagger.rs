use cue_lib::metadata::{Metadata, MetadataTag, avlib::AvLibTag, id3::Id3Tag, vorbis::VorbisTag};
use smt_ffmpeg::{
  ffmpeg::{
    AVCodecID_AV_CODEC_ID_FLAC, AVCodecID_AV_CODEC_ID_MP3, AVCodecID_AV_CODEC_ID_MP3ADU,
    AVCodecID_AV_CODEC_ID_MP3ON4,
  },
  format::{context::AvContext, stream::StreamType},
};

#[derive(Clone, Copy, Debug, Default)]
pub enum Tagger {
  #[default]
  AvLib,
  Vorbis,
  Id3,
}

impl Tagger {
  #[inline]
  #[allow(nonstandard_style)]
  pub const fn from_codec_id(codec_id: u32) -> Self {
    match codec_id {
      AVCodecID_AV_CODEC_ID_MP3 | AVCodecID_AV_CODEC_ID_MP3ADU | AVCodecID_AV_CODEC_ID_MP3ON4 => {
        Tagger::Id3
      }
      AVCodecID_AV_CODEC_ID_FLAC => Tagger::Vorbis,
      _ => Tagger::AvLib,
    }
  }

  pub fn from_av_context(context: &AvContext) -> Option<Self> {
    context
      .find_best_stream(StreamType::Audio)
      .map(|v| unsafe { v.codecpar.as_ref() })
      .flatten()
      .map(|v| Tagger::from_codec_id(v.codec_id))
  }

  pub fn get_name(&self, tag: MetadataTag) -> Option<&'static str> {
    match self {
      Tagger::AvLib => AvLibTag::try_from(tag).map(|v| v.as_str()).ok(),
      Tagger::Vorbis => VorbisTag::try_from(tag).map(|v| v.as_str()).ok(),
      Tagger::Id3 => Id3Tag::try_from(tag).map(|v| v.as_str()).ok(),
    }
  }
}
