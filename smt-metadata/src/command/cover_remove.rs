use crate::{
  context::{CoverSource, MetadataEditContext},
  error::MetadataError,
};
use smt_common::Command;
use std::path::Path;

pub struct CmdCoverRemove<P>
where
  P: AsRef<Path>,
{
  path: P,
}

impl<P> CmdCoverRemove<P>
where
  P: AsRef<Path>,
{
  #[inline]
  pub const fn new(path: P) -> Self {
    Self { path }
  }
}

impl<P> Command for CmdCoverRemove<P>
where
  P: AsRef<Path>,
{
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let mut context = MetadataEditContext::open(self.path)?;
    context.set_cover_source(CoverSource::NoCover);
    context.copy_all_metadata()?;
    context.commit()
  }
}
