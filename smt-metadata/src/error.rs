use smt_common::{
  VerboseLevel,
  output_error::{ErrorFormat, ErrorFormatter},
};
use smt_ffmpeg::error::AvError;
use std::path::PathBuf;

pub enum MetadataError {
  IOError(std::io::Error),
  AvError(AvError),
  EditError { kind: EditErrorKind },
  UnsupportedImageFormat,
  NoCoverImage,
}

pub enum EditErrorKind {
  EditorNotAvailable(PathBuf),
  InvalidEditMessage,
  InvalidTagName(Box<str>),
  EditDiscarded,
}

impl ErrorFormat for MetadataError {
  fn fmt(
    &self,
    mut f: ErrorFormatter<'_>,
    input_buffer: &str,
    verbose: VerboseLevel,
  ) -> std::io::Result<()> {
    if verbose == VerboseLevel::Quiet {
      Ok(())
    } else {
      match self {
        Self::NoCoverImage => f.writeln_warn(format_args!("no cover image found")),
        Self::UnsupportedImageFormat => f.writeln_error(format_args!("unsupported image format")),
        Self::IOError(error) => ErrorFormat::fmt(error, f, input_buffer, verbose),
        Self::AvError(error) => ErrorFormat::fmt(error, f, input_buffer, verbose),
        Self::EditError { kind } => ErrorFormat::fmt(kind, f, input_buffer, verbose),
      }
    }
  }
}

impl ErrorFormat for EditErrorKind {
  fn fmt(&self, mut f: ErrorFormatter<'_>, _: &str, verbose: VerboseLevel) -> std::io::Result<()> {
    match verbose {
      VerboseLevel::Quiet => Ok(()),
      _ => match self {
        Self::InvalidEditMessage => {
          f.writeln_error(format_args!("invalid edit message format"))
        }
        Self::InvalidTagName(tag) => {
          f.writeln_error(format_args!("invalid tag name '{tag}' in edit message "))
        }
        Self::EditDiscarded => f.writeln_warn(format_args!("edit message discarded")),
        Self::EditorNotAvailable(editor) => f.writeln_warn(
          format_args!(
          "Text editor '{}' not found. Set the 'EDITOR' environment variable or use the '--editor' argument to specify an alternative",
          editor.display()
        )),
      },
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

impl From<EditErrorKind> for MetadataError {
  #[inline]
  fn from(value: EditErrorKind) -> Self {
    Self::EditError { kind: value }
  }
}
