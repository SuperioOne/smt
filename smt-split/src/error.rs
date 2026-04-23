use cue_lib::error::CueLibError;
use smt_common::{
  VerboseLevel,
  output_error::{ErrorFormat, ErrorFormatter},
};
use smt_ffmpeg::error::AvError;
use std::{io::Write as _, path::PathBuf};

pub enum SplitError {
  AvError(AvError),
  CueLibError(CueLibError),
  IOError(std::io::Error),
  InvalidFilePath(PathBuf),
  InvalidOutputDir(PathBuf),
  UnsupportedAvLibVersion,
  NothingToSplit,
  UnknownAudioContainer,
}

impl ErrorFormat for SplitError {
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
        Self::CueLibError(error) => ErrorFormat::fmt(error, f, input_buffer, verbose_level),
        Self::IOError(error) => ErrorFormat::fmt(error, f, input_buffer, verbose_level),
        Self::UnsupportedAvLibVersion => {
          write!(f, "linked avlib version on system is not supported")
        }
        Self::NothingToSplit => write!(f, "cuesheet has only one or none track, nothing to split."),
        Self::AvError(error) => write!(f, "{}", error),
        Self::UnknownAudioContainer => write!(f, "unable to detect audio codec"),
        Self::InvalidFilePath(path) => write!(f, "invalid file path: {:?}", path),
        Self::InvalidOutputDir(path) => write!(f, "invalid output directory: {:?}", path),
      }
    }
  }
}

impl From<CueLibError> for SplitError {
  #[inline]
  fn from(value: CueLibError) -> Self {
    Self::CueLibError(value)
  }
}

impl From<AvError> for SplitError {
  #[inline]
  fn from(value: AvError) -> Self {
    Self::AvError(value)
  }
}

impl From<std::io::Error> for SplitError {
  #[inline]
  fn from(value: std::io::Error) -> Self {
    Self::IOError(value)
  }
}
