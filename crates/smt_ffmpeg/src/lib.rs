use smt_ffmpeg_sys::{av_log_set_level, avcodec_version, avformat_version, avutil_version};

pub mod codec;
pub mod common;
pub mod error;
pub mod format;
pub mod util;

pub mod ffmpeg {
  pub use smt_ffmpeg_sys::*;
}

pub enum AvLogLevel {
  /// Print no output.
  Quiet = -8,

  /// Something went really wrong and we will crash now.
  Panic = 0,

  ///  Something went wrong and recovery is not possible.
  ///  For example, no header was found for a format which depends
  ///  on headers or an illegal combination of parameters is used.
  Fatal = 8,

  ///  Something went wrong and cannot losslessly be recovered.
  ///  However, not all future data is affected.
  Error = 16,

  /// Something somehow does not look correct. This may or may not
  /// lead to problems. An example would be the use of '-vstrict -2'.
  Warning = 24,

  /// Standard information.
  Info = 32,

  /// Detailed information.
  Verbose = 40,

  /// Stuff which is only useful for libav* developers.
  Debug = 48,

  /// Extremely verbose debugging, useful for libav* development.
  Trace = 56,
}

pub fn avlib_log_level(level: AvLogLevel) {
  unsafe {
    av_log_set_level(level as i32);
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VersionInfo {
  pub avutil: Version,
  pub avformat: Version,
  pub avcodec: Version,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version(u32);

pub fn avlib_version() -> VersionInfo {
  VersionInfo {
    avutil: Version(unsafe { avutil_version() }),
    avformat: Version(unsafe { avformat_version() }),
    avcodec: Version(unsafe { avcodec_version() }),
  }
}

impl Version {
  #[inline]
  pub const fn minor(&self) -> u8 {
    (self.0 >> 8 & 0xFF) as u8
  }

  #[inline]
  pub const fn major(&self) -> u8 {
    (self.0 >> 16 & 0xFF) as u8
  }

  #[inline]
  pub const fn micro(&self) -> u8 {
    (self.0 & 0xFF) as u8
  }

  #[inline]
  pub const fn as_u32(&self) -> u32 {
    self.0
  }
}

impl std::fmt::Display for Version {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "{}.{}.{}",
      self.major(),
      self.minor(),
      self.micro()
    ))
  }
}
