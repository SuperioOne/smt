use crate::{context::MetadataEditContext, error::MetadataError};
use smt_common::Command;
use std::path::{Path, PathBuf};

pub struct CmdTagClear {
  path: PathBuf,
}

impl CmdTagClear {
  pub fn new<P>(path: P) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      path: path.as_ref().to_path_buf(),
    }
  }
}

impl Command for CmdTagClear {
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let context = MetadataEditContext::open(self.path)?;
    context.commit()?;
    Ok(())
  }
}
