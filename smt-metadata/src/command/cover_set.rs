use crate::{context::MetadataEditContext, error::MetadataError};
use smt_common::Command;
use smt_ffmpeg::format::context::AvInputContext;
use std::path::Path;

pub struct CmdCoverSet<P, C>
where
  P: AsRef<Path>,
  C: AsRef<Path>,
{
  path: P,
  cover: C,
}

impl<P, C> CmdCoverSet<P, C>
where
  P: AsRef<Path>,
  C: AsRef<Path>,
{
  #[inline]
  pub const fn new(path: P, cover: C) -> Self {
    Self { path, cover }
  }
}

impl<P, C> Command for CmdCoverSet<P, C>
where
  P: AsRef<Path>,
  C: AsRef<Path>,
{
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let cover_context = AvInputContext::open_path(self.cover)?;
    let mut context = MetadataEditContext::open(self.path)?;
    context.set_cover_source(cover_context.into());
    context.copy_all_metadata()?;
    context.commit()
  }
}
