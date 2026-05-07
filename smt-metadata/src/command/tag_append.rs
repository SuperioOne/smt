use crate::{context::MetadataEditContext, error::MetadataError};
use cue_lib::metadata::vorbis::VorbisTag;
use smt_common::Command;
use std::path::{Path, PathBuf};

pub struct CmdTagAppend {
  path: PathBuf,
  key: VorbisTag,
  values: Vec<Box<str>>,
}

impl CmdTagAppend {
  pub fn new<P>(path: P, key: VorbisTag, values: Vec<Box<str>>) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      path: path.as_ref().to_path_buf(),
      key,
      values,
    }
  }
}

impl Command for CmdTagAppend {
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let mut context = MetadataEditContext::open(self.path)?;
    context.copy_all_metadata()?;

    let mut metadata_map = context.metadata_mut();

    for value in self.values {
      metadata_map.push(self.key, value);
    }

    context.commit()
  }
}
