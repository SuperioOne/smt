use clap::ValueEnum;
use clap::builder::PossibleValue;
use cue_lib::metadata::{Metadata, MetadataTag, vorbis::VorbisTag};

#[derive(Clone, Copy, Debug)]
pub struct TagName {
  inner: VorbisTag,
}

macro_rules! tag {
  ($variant:ident) => {
    TagName {
      inner: VorbisTag::$variant,
    }
  };
}

static TAG_VARIANTS: &[TagName] = &[
  tag!(Barcode),
  tag!(AcoustidFingerprint),
  tag!(AcoustidId),
  tag!(Album),
  tag!(AlbumArtist),
  tag!(AlbumArtistSort),
  tag!(AlbumSort),
  tag!(Arranger),
  tag!(Artist),
  tag!(Artists),
  tag!(ArtistSort),
  tag!(Asin),
  tag!(Barcode),
  tag!(Bpm),
  tag!(CatalogNumber),
  tag!(Comment),
  tag!(Compilation),
  tag!(Composer),
  tag!(ComposerSort),
  tag!(Conductor),
  tag!(Copyright),
  tag!(Date),
  tag!(Director),
  tag!(DiscNumber),
  tag!(DiscSubtitle),
  tag!(TotalDiscs),
  tag!(DjMixer),
  tag!(EncodedBy),
  tag!(EncoderSettings),
  tag!(Engineer),
  tag!(Genre),
  tag!(Grouping),
  tag!(Isrc),
  tag!(Key),
  tag!(Label),
  tag!(Language),
  tag!(License),
  tag!(Lyricist),
  tag!(Lyrics),
  tag!(Media),
  tag!(Mixer),
  tag!(Mood),
  tag!(MovementNumber),
  tag!(Movement),
  tag!(MovementTotal),
  tag!(OriginalDate),
  tag!(OriginalFileName),
  tag!(OriginalYear),
  tag!(Performer),
  tag!(Producer),
  tag!(Rating),
  tag!(ReleaseCountry),
  tag!(ReleaseStatus),
  tag!(ReleaseType),
  tag!(Remixer),
  tag!(ReplayGainAlbumGain),
  tag!(ReplayGainAlbumPeak),
  tag!(ReplayGainAlbumRange),
  tag!(ReplayGainReferenceLoudness),
  tag!(ReplayGainTrackGain),
  tag!(ReplayGainTrackPeak),
  tag!(ReplayGainTrackRange),
  tag!(Script),
  tag!(ShowMovement),
  tag!(Subtitle),
  tag!(Title),
  tag!(TitleSort),
  tag!(TrackNumber),
  tag!(TotalTracks),
  tag!(Website),
  tag!(Work),
  tag!(Writer),
];

impl ValueEnum for TagName {
  fn value_variants<'a>() -> &'a [Self] {
    TAG_VARIANTS
  }

  fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
    Some(PossibleValue::new(self.inner.as_str()))
  }
}

impl Into<MetadataTag> for TagName {
  #[inline]
  fn into(self) -> MetadataTag {
    self.inner.into()
  }
}

impl Into<VorbisTag> for TagName {
  #[inline]
  fn into(self) -> VorbisTag {
    self.inner
  }
}
