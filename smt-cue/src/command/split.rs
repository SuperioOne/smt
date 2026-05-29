use self::demux::SplitDemuxer;
use crate::error::SplitError;
use cue_lib::parse::{Cuesheet, CuesheetParser};
use smt_ffmpeg::{
  avlib_version,
  ffmpeg::{LIBAVCODEC_VERSION_MAJOR, LIBAVFORMAT_VERSION_MAJOR},
};
use std::path::{Path, PathBuf};

mod demux;

pub struct CmdSplit<'a> {
  cuesheet: &'a str,
  output_dir: Option<PathBuf>,
  file_path: Option<PathBuf>,
  vorbis_remarks: bool,
}

impl<'a> CmdSplit<'a> {
  #[inline]
  pub const fn new(cuesheet: &'a str) -> Self {
    Self {
      cuesheet,
      output_dir: None,
      file_path: None,
      vorbis_remarks: false,
    }
  }

  #[inline]
  pub fn set_file_path(mut self, value: Option<PathBuf>) -> Self {
    self.file_path = value;
    self
  }

  #[inline]
  pub const fn set_vorbis_remarks(mut self, value: bool) -> Self {
    self.vorbis_remarks = value;
    self
  }

  #[inline]
  pub fn set_output_dir(mut self, value: Option<PathBuf>) -> Self {
    self.output_dir = value;
    self
  }

  #[inline]
  fn resolve_file_path(&self, cuesheet: &Cuesheet<'_>) -> Result<PathBuf, SplitError> {
    match self.file_path.as_ref() {
      Some(path) => {
        if path.is_dir() {
          if let Some(file_cmd) = cuesheet.file {
            Ok(path.join(file_cmd.name.to_string()))
          } else {
            Err(SplitError::InvalidFilePath(path.clone()))
          }
        } else if path.is_file() {
          Ok(path.clone())
        } else {
          Err(SplitError::InvalidFilePath(path.clone()))
        }
      }
      None => {
        if let Some(file_cmd) = cuesheet.file {
          let file_name = file_cmd.name.to_string();
          let file_path = Path::new(&file_name);

          if file_path.is_file() {
            Ok(file_path.to_path_buf())
          } else {
            Err(SplitError::InvalidFilePath(file_path.to_path_buf()))
          }
        } else {
          Err(SplitError::InvalidFilePath(PathBuf::new()))
        }
      }
    }
  }

  pub fn run(self) -> Result<(), SplitError> {
    let av_version = avlib_version();

    if av_version.avformat.major() != LIBAVFORMAT_VERSION_MAJOR as u8
      || av_version.avcodec.major() != LIBAVCODEC_VERSION_MAJOR as u8
    {
      return Err(SplitError::UnsupportedAvLibVersion);
    }

    let cuesheet = CuesheetParser::new()
      .allow_vorbis_remarks(self.vorbis_remarks)
      .parse(self.cuesheet)?;

    let input_path = self.resolve_file_path(&cuesheet)?;

    let output_dir = if let Some(output_dir) = self.output_dir {
      output_dir
    } else {
      PathBuf::from(".")
    };

    SplitDemuxer::init(input_path, output_dir, &cuesheet)?.split()
  }
}
