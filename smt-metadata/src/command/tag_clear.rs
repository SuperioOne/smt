use crate::{context::MetadataEditContext, error::MetadataError};
use smt_common::Command;
use std::path::Path;

pub struct CmdTagClear<P>
where
  P: AsRef<Path>,
{
  path: P,
}

impl<P> CmdTagClear<P>
where
  P: AsRef<Path>,
{
  pub const fn new(path: P) -> Self
  where
    P: AsRef<Path>,
  {
    Self { path }
  }
}

impl<P> Command for CmdTagClear<P>
where
  P: AsRef<Path>,
{
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let context = MetadataEditContext::open(self.path)?;
    context.commit()?;
    Ok(())
  }
}
