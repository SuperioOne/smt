use cue_lib::{
  metadata::{Metadata, MetadataTag, avlib::AvLibTag, id3::Id3Tag, vorbis::VorbisTag},
  parse::{Cuesheet, TrackInfo},
};
use smt_ffmpeg::{
  ffmpeg::{
    AVCodecID_AV_CODEC_ID_FLAC, AVCodecID_AV_CODEC_ID_MP3, AVCodecID_AV_CODEC_ID_MP3ADU,
    AVCodecID_AV_CODEC_ID_MP3ON4,
  },
  format::{
    context::{AvContext, AvOutputContext},
    stream::StreamType,
  },
  util::dictionary::AvDictionaryMut,
};
use std::{borrow::Borrow, str::FromStr as _};

/// Simple dyn-compatible trait to map generic metadata tags to container specific tag names
pub trait CodecMetadataTagger: Sync {
  fn get_name(&self, tag: MetadataTag) -> Option<&str>;
}

pub struct VorbisTagger;
pub struct Id3Tagger;
pub struct AvLibTagger;

pub static VORBIS_TAGGER: &'static dyn CodecMetadataTagger = &VorbisTagger;
pub static ID3_TAGGER: &'static dyn CodecMetadataTagger = &Id3Tagger;
pub static AVLIB_TAGGER: &'static dyn CodecMetadataTagger = &AvLibTagger;

#[allow(nonstandard_style)]
pub fn find_tagger_by_codec_id(codec_id: u32) -> &'static dyn CodecMetadataTagger {
  match codec_id {
    AVCodecID_AV_CODEC_ID_MP3 | AVCodecID_AV_CODEC_ID_MP3ADU | AVCodecID_AV_CODEC_ID_MP3ON4 => {
      ID3_TAGGER
    }
    AVCodecID_AV_CODEC_ID_FLAC => VORBIS_TAGGER,
    _ => AVLIB_TAGGER,
  }
}

#[allow(nonstandard_style)]
pub fn find_tagger(context: &AvContext) -> Option<&'static dyn CodecMetadataTagger> {
  context
    .find_best_stream(StreamType::Audio)
    .map(|v| unsafe { v.codecpar.as_ref() })
    .flatten()
    .map(|v| find_tagger_by_codec_id(v.codec_id))
}

pub fn find_tag_from_str(value: &str) -> Option<MetadataTag> {
  if let Ok(av_tag) = AvLibTag::from_str(value) {
    Some(av_tag.into())
  } else if let Ok(id3_tag) = Id3Tag::from_str(value) {
    Some(id3_tag.into())
  } else if let Ok(vorbis) = VorbisTag::from_str(value) {
    Some(vorbis.into())
  } else {
    None
  }
}

/// Wrapper over [`AvDictionaryMut`] for codec aware tag mapping.
pub struct MetadataContainer<'a> {
  inner: AvDictionaryMut<'a>,
  tagger: &'a dyn CodecMetadataTagger,
}

impl<'a> MetadataContainer<'a> {
  #[inline]
  pub const fn new(dst: AvDictionaryMut<'a>, codec_tagger: &'a dyn CodecMetadataTagger) -> Self {
    Self {
      inner: dst,
      tagger: codec_tagger,
    }
  }

  pub fn from_context(context: &'a mut AvOutputContext) -> Self {
    let tagger = find_tagger(&context).unwrap_or(AVLIB_TAGGER);

    Self {
      tagger,
      inner: context.metadata_mut(),
    }
  }

  pub fn set<T, S>(&mut self, tag: T, value: S) -> bool
  where
    S: AsRef<str>,
    T: Borrow<MetadataTag>,
  {
    if let Some(tag) = self.tagger.get_name(*tag.borrow()) {
      self.inner.set(tag, value.as_ref()).is_ok()
    } else {
      false
    }
  }

  pub fn push<T, S>(&mut self, tag: T, value: S) -> bool
  where
    S: AsRef<str>,
    T: Borrow<MetadataTag>,
  {
    if let Some(tag) = self.tagger.get_name(*tag.borrow()) {
      self.inner.push(tag, value.as_ref()).is_ok()
    } else {
      false
    }
  }

  fn push_optional<T, S>(&mut self, tag: T, value: Option<S>) -> bool
  where
    S: AsRef<str>,
    T: Borrow<MetadataTag>,
  {
    if let Some(value) = value {
      self.push(tag, value)
    } else {
      false
    }
  }

  pub fn push_from_av_dict(&mut self, iter: smt_ffmpeg::util::dictionary::Iter<'a>) {
    for (tag, value) in iter {
      if let Some(tag) = find_tag_from_str(tag) {
        _ = self.push(tag, value);
      }
    }
  }

  pub fn push_from_cuesheet(&mut self, cuesheet: &'a Cuesheet) {
    for (tag, values) in cuesheet.remark_metadata.iter() {
      for value in values {
        self.push(tag, value.as_cow_str());
      }
    }

    self.push_optional(
      MetadataTag::OriginalFileName,
      cuesheet.file.map(|v| v.name.as_cow_str()),
    );
    self.push_optional(
      MetadataTag::Album,
      cuesheet.album_title.map(|v| v.as_cow_str()),
    );
    self.push_optional(
      MetadataTag::AlbumArtist,
      cuesheet.performer.map(|v| v.as_cow_str()),
    );
    self.push_optional(
      MetadataTag::CatalogNumber,
      cuesheet.catalog.map(|v| v.as_cow_str()),
    );
    self.push_optional(
      MetadataTag::Writer,
      cuesheet.songwriter.map(|v| v.as_cow_str()),
    );
    self.push(MetadataTag::TotalTracks, cuesheet.tracks.len().to_string());
  }

  pub fn push_from_track(&mut self, track: &'a TrackInfo<'a>) {
    for (tag, values) in track.remark_metadata.iter() {
      for value in values {
        _ = self.push(tag, value.as_cow_str());
      }
    }

    self.push_optional(MetadataTag::Title, track.title.map(|v| v.as_cow_str()));
    self.push_optional(MetadataTag::Artist, track.performer.map(|v| v.as_cow_str()));
    self.push_optional(
      MetadataTag::Writer,
      track.songwriter.map(|v| v.as_cow_str()),
    );
    self.push_optional(MetadataTag::Isrc, track.isrc.map(|v| v.to_string()));
    self.set(MetadataTag::TrackNumber, track.no.to_string());
  }
}

impl CodecMetadataTagger for VorbisTagger {
  fn get_name(&self, tag: MetadataTag) -> Option<&str> {
    if let Ok(tag) = VorbisTag::try_from(tag) {
      Some(tag.as_str())
    } else {
      None
    }
  }
}

impl CodecMetadataTagger for Id3Tagger {
  fn get_name(&self, tag: MetadataTag) -> Option<&str> {
    if let Ok(tag) = Id3Tag::try_from(tag) {
      Some(tag.as_str())
    } else {
      None
    }
  }
}

impl CodecMetadataTagger for AvLibTagger {
  fn get_name(&self, tag: MetadataTag) -> Option<&str> {
    if let Ok(tag) = AvLibTag::try_from(tag) {
      Some(tag.as_str())
    } else {
      None
    }
  }
}
