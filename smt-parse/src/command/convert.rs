use cue_lib::{error::CueLibError, parse::CuesheetParser};
use smt_common::{
  Command, VerboseLevel,
  output_error::{ErrorFormat, ErrorFormatter},
  output_writer::OutputStream,
};
use std::{
  fs::OpenOptions,
  io::{BufWriter, Write as _, stdout},
  path::PathBuf,
};

pub struct CmdConvert<'a> {
  cuesheet: &'a str,
  vorbis_remarks: bool,
  output_file: Option<PathBuf>,
  pretty_print: bool,
}

impl<'a> CmdConvert<'a> {
  #[inline]
  pub const fn new(cuesheet: &'a str) -> Self {
    Self {
      cuesheet,
      pretty_print: false,
      vorbis_remarks: false,
      output_file: None,
    }
  }

  #[inline]
  pub const fn set_pretty_print(mut self, value: bool) -> Self {
    self.pretty_print = value;
    self
  }

  #[inline]
  pub const fn set_vorbis_remarks(mut self, value: bool) -> Self {
    self.vorbis_remarks = value;
    self
  }

  #[inline]
  pub fn set_output_file(mut self, value: Option<PathBuf>) -> Self {
    self.output_file = value;
    self
  }
}

impl<'a> Command for &'a CmdConvert<'a> {
  type Error = ConvertError;

  fn run(self) -> Result<(), ConvertError> {
    let cuesheet = CuesheetParser::new()
      .allow_vorbis_remarks(self.vorbis_remarks)
      .parse(self.cuesheet)?;

    let mut ouput_stream = {
      let stream = match self.output_file.as_ref() {
        Some(path) => {
          let fd = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

          OutputStream::from(fd)
        }
        None => OutputStream::from(stdout().lock()),
      };

      BufWriter::new(stream)
    };

    if self.pretty_print {
      serde_json::to_writer_pretty(&mut ouput_stream, &cuesheet)?
    } else {
      serde_json::to_writer(&mut ouput_stream, &cuesheet)?
    }

    Ok(())
  }
}

pub enum ConvertError {
  CueLibError(CueLibError),
  IOError(std::io::Error),
  JsonSerializeError(serde_json::error::Error),
}

impl ErrorFormat for ConvertError {
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
        ConvertError::CueLibError(error) => ErrorFormat::fmt(error, f, input_buffer, verbose_level),
        ConvertError::IOError(error) => write!(f, "{}", error),
        ConvertError::JsonSerializeError(error) => write!(f, "{}", error),
      }
    }
  }
}

impl From<CueLibError> for ConvertError {
  #[inline]
  fn from(value: CueLibError) -> Self {
    Self::CueLibError(value)
  }
}

impl From<serde_json::error::Error> for ConvertError {
  #[inline]
  fn from(value: serde_json::error::Error) -> Self {
    Self::JsonSerializeError(value)
  }
}

impl From<std::io::Error> for ConvertError {
  #[inline]
  fn from(value: std::io::Error) -> Self {
    Self::IOError(value)
  }
}
