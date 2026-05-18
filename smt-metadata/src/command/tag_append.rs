use crate::{context::MetadataEditContext, error::MetadataError};
use cue_lib::metadata::vorbis::VorbisTag;
use smt_common::{Command, metadata::find_tag_from_str};
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

  fn run(mut self) -> Result<(), Self::Error> {
    self.values.dedup();

    let mut context = MetadataEditContext::open(self.path)?;
    context.copy_metadata_by_filter(|key, val| {
      match find_tag_from_str(key) {
        Some(tag) => {
          if tag == self.key {
            for new_value in self.values.iter() {
              if val == new_value.as_ref() {
                return false;
              }
            }
          }

          // catch-all case
          true
        }
        _ => true,
      }
    })?;

    let mut metadata_map = context.metadata_mut();

    for value in self.values {
      metadata_map.push(self.key, value.as_ref());
    }

    context.commit()
  }
}
