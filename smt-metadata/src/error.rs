use smt_common::{
  VerboseLevel,
  output_error::{ErrorFormat, ErrorFormatter},
};
use smt_ffmpeg::error::AvError;
use std::io::Write as _;

pub enum MetadataError {
  IOError(std::io::Error),
  AvError(AvError),
  NoAudioStream,
}

impl ErrorFormat for MetadataError {
  fn fmt(
    &self,
    mut f: ErrorFormatter<'_>,
    input_buffer: &str,
    verbose_level: VerboseLevel,
  ) -> std::io::Result<()> {
    if verbose_level == VerboseLevel::Quiet {
      Ok(())
    } else {
      match self {
        Self::IOError(error) => ErrorFormat::fmt(error, f, input_buffer, verbose_level),
        Self::AvError(error) => write!(f, "{}", error),
        Self::NoAudioStream => write!(f, "media file has no audio stream"),
      }
    }
  }
}

impl From<AvError> for MetadataError {
  #[inline]
  fn from(value: AvError) -> Self {
    Self::AvError(value)
  }
}

impl From<std::io::Error> for MetadataError {
  #[inline]
  fn from(value: std::io::Error) -> Self {
    Self::IOError(value)
  }
}
