use cue_lib::{
  core::CueStr,
  metadata::{Metadata, MetadataTag, avlib::AvLibTag, id3::Id3Tag, vorbis::VorbisTag},
  parse::{Cuesheet, TrackInfo},
};
use smt_ffmpeg::util::dictionary::AvDictionaryMut;
use std::{borrow::Borrow, str::FromStr as _};

/// Simple dyn-compatible trait to map generic metadata tags to container specific tag names
pub trait CodecMetadataTagger {
  fn get_name(&self, tag: MetadataTag) -> Option<&str>;
}

pub struct VorbisTagger;
pub struct Id3Tagger;
pub struct AvLibTagger;

pub struct MetadataContainer<'a> {
  inner: AvDictionaryMut<'a>,
  codec_tagger: &'a dyn CodecMetadataTagger,
}

impl<'a> MetadataContainer<'a> {
  #[inline]
  pub const fn new(dst: AvDictionaryMut<'a>, codec_tagger: &'a dyn CodecMetadataTagger) -> Self {
    Self {
      inner: dst,
      codec_tagger,
    }
  }

  pub fn override_entry<T, S>(&mut self, tag: T, value: S) -> bool
  where
    S: AsRef<str>,
    T: Borrow<MetadataTag>,
  {
    if let Some(tag) = self.codec_tagger.get_name(*tag.borrow()) {
      self.inner.override_entry(tag, value.as_ref()).is_ok()
    } else {
      false
    }
  }

  pub fn insert<T, S>(&mut self, tag: T, value: S) -> bool
  where
    S: AsRef<str>,
    T: Borrow<MetadataTag>,
  {
    if let Some(tag) = self.codec_tagger.get_name(*tag.borrow()) {
      self.inner.set(tag, value.as_ref()).is_ok()
    } else {
      false
    }
  }

  fn insert_optional<T, S>(&mut self, tag: T, value: Option<S>) -> bool
  where
    S: AsRef<str>,
    T: Borrow<MetadataTag>,
  {
    if let Some(value) = value {
      self.insert(tag, value)
    } else {
      false
    }
  }

  pub fn insert_from_av_dict(&mut self, iter: smt_ffmpeg::util::dictionary::Iter<'a>) {
    for (tag, value) in iter {
      match (tag.to_str(), CueStr::try_from(value)) {
        (Ok(tag), Ok(value)) => {
          let tag: Option<MetadataTag> = {
            if let Ok(av_tag) = AvLibTag::from_str(tag) {
              Some(av_tag.into())
            } else if let Ok(id3_tag) = Id3Tag::from_str(tag) {
              Some(id3_tag.into())
            } else if let Ok(vorbis) = VorbisTag::from_str(tag) {
              Some(vorbis.into())
            } else {
              None
            }
          };

          if let Some(tag) = tag {
            _ = self.insert(tag, value.as_cow_str());
          }
        }
        _ => continue,
      };
    }
  }

  pub fn insert_from_cuesheet(&mut self, cuesheet: &'a Cuesheet) {
    for (tag, values) in cuesheet.remark_metadata.iter() {
      for value in values {
        self.insert(tag, value.as_cow_str());
      }
    }

    self.insert_optional(
      MetadataTag::OriginalFileName,
      cuesheet.file.map(|v| v.name.as_cow_str()),
    );
    self.insert_optional(
      MetadataTag::Album,
      cuesheet.album_title.map(|v| v.as_cow_str()),
    );
    self.insert_optional(
      MetadataTag::AlbumArtist,
      cuesheet.performer.map(|v| v.as_cow_str()),
    );
    self.insert_optional(
      MetadataTag::CatalogNumber,
      cuesheet.catalog.map(|v| v.as_cow_str()),
    );
    self.insert_optional(
      MetadataTag::Writer,
      cuesheet.songwriter.map(|v| v.as_cow_str()),
    );
    self.insert(MetadataTag::TotalTracks, cuesheet.tracks.len().to_string());
  }

  pub fn insert_from_track(&mut self, track: &'a TrackInfo<'a>) {
    for (tag, values) in track.remark_metadata.iter() {
      for value in values {
        _ = self.insert(tag, value.as_cow_str());
      }
    }

    self.insert_optional(MetadataTag::Title, track.title.map(|v| v.as_cow_str()));
    self.insert_optional(MetadataTag::Artist, track.performer.map(|v| v.as_cow_str()));
    self.insert_optional(
      MetadataTag::Writer,
      track.songwriter.map(|v| v.as_cow_str()),
    );
    self.insert_optional(MetadataTag::Isrc, track.isrc.map(|v| v.to_string()));
    self.override_entry(MetadataTag::TrackNumber, track.no.to_string());
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
