use crate::{context::MetadataEditContext, error::MetadataError};
use cue_lib::metadata::vorbis::VorbisTag;
use smt_common::{Command, metadata::find_tag_from_str};
use std::path::{Path, PathBuf};

pub struct CmdTagRemove {
  path: PathBuf,
  key: VorbisTag,
}

impl CmdTagRemove {
  pub fn new<P>(path: P, key: VorbisTag) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      path: path.as_ref().to_path_buf(),
      key,
    }
  }
}

impl Command for CmdTagRemove {
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
