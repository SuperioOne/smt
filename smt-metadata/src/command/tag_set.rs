use crate::{context::MetadataEditContext, error::MetadataError};
use cue_lib::metadata::vorbis::VorbisTag;
use smt_common::{Command, metadata::find_tag_from_str};
use std::path::Path;

pub struct CmdTagSet<P>
where
  P: AsRef<Path>,
{
  path: P,
  key: VorbisTag,
  values: Vec<Box<str>>,
}

impl<P> CmdTagSet<P>
where
  P: AsRef<Path>,
{
  #[inline]
  pub const fn new(path: P, key: VorbisTag, values: Vec<Box<str>>) -> Self {
    Self { path, key, values }
  }
}

impl<P> Command for CmdTagSet<P>
where
  P: AsRef<Path>,
{
  type Error = MetadataError;

  fn run(mut self) -> Result<(), Self::Error> {
    self.values.dedup();

    let mut context = MetadataEditContext::open(self.path)?;
    context.copy_metadata_by_filter(|key, _| match find_tag_from_str(key) {
      Some(tag) => tag != self.key,
      _ => true,
    })?;

    let mut metadata_map = context.metadata_mut();

    for value in self.values {
      metadata_map.push(self.key, value);
    }

    context.commit()
  }
}
