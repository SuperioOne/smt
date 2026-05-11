use crate::{context::MetadataEditContext, error::MetadataError};
use cue_lib::metadata::vorbis::VorbisTag;
use smt_common::{Command, metadata::guess_metadata_from_str};
use std::path::{Path, PathBuf};

pub struct CmdTagSet {
  path: PathBuf,
  key: VorbisTag,
  values: Vec<Box<str>>,
}

impl CmdTagSet {
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

impl Command for CmdTagSet {
  type Error = MetadataError;

  fn run(mut self) -> Result<(), Self::Error> {
    // self.values.dedup();

    println!("tags {:?}", self.values);

    let mut context = MetadataEditContext::open(self.path)?;
    context.copy_metadata_by_filter(|key, _| {
      match key.to_str().map(|v| guess_metadata_from_str(v)) {
        Ok(Some(tag)) => tag != self.key,
        _ => true,
      }
    })?;

    let mut metadata_map = context.metadata_mut();

    for value in self.values {
      metadata_map.push(self.key, value);
    }

    context.commit()
  }
}
