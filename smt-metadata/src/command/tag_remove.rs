use crate::{context::MetadataEditContext, error::MetadataError};
use cue_lib::metadata::vorbis::VorbisTag;
use smt_common::{Command, metadata::find_tag_from_str};
use std::path::Path;

pub struct CmdTagRemove<P>
where
  P: AsRef<Path>,
{
  path: P,
  key: VorbisTag,
}

impl<P> CmdTagRemove<P>
where
  P: AsRef<Path>,
{
  #[inline]
  pub const fn new(path: P, key: VorbisTag) -> Self {
    Self { path, key }
  }
}

impl<P> Command for CmdTagRemove<P>
where
  P: AsRef<Path>,
{
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let mut context = MetadataEditContext::open(self.path)?;
    context.copy_metadata_by_filter(|key, _| match find_tag_from_str(key) {
      Some(tag) => tag != self.key,
      _ => true,
    })?;

    context.commit()
  }
}
