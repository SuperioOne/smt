use crate::error::MetadataError;
use smt_common::Command;
use smt_ffmpeg::format::context::AvInputContext;
use std::path::{Path, PathBuf};

pub struct CmdTagList {
  path: PathBuf,
  json_encoding: bool,
}

impl CmdTagList {
  pub fn new<P>(path: P) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      path: path.as_ref().to_path_buf(),
      json_encoding: false,
    }
  }

  #[inline]
  pub const fn use_json_encoding(mut self, value: bool) -> Self {
    self.json_encoding = value;
    self
  }
}

impl Command for CmdTagList {
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let media = AvInputContext::open_path(self.path)?;
    let metadata = media.metadata();

    for (key, value) in metadata.iter() {
      match (key.to_str(), value.to_str()) {
        (Ok(_), Ok(_)) => todo!(),
        _ => {
          continue;
        }
      }
    }

    Ok(())
  }
}
