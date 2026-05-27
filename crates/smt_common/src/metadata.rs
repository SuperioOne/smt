use self::tagger::Tagger;
use cue_lib::{
  metadata::{MetadataTag, avlib::AvLibTag, id3::Id3Tag, vorbis::VorbisTag},
  parse::{Cuesheet, TrackInfo},
};
use smt_ffmpeg::{format::context::AvOutputContext, util::dictionary::AvDictionaryMut};
use std::{borrow::Borrow, str::FromStr as _};

pub mod tag_name;
mod tagger;

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
  tagger: Tagger,
}

impl<'a> MetadataContainer<'a> {
  pub fn new(context: &'a mut AvOutputContext) -> Self {
    let tagger = Tagger::from_av_context(&context).unwrap_or_default();
    Self {
      inner: context.metadata_mut(),
      tagger,
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
