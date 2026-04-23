use super::AvContext;
use crate::{codec::packet::AvPacket, common::unsafe_av_result, error::AvError};
use smt_ffmpeg_sys::{
  AVFMT_NOFILE, AVFormatContext, AVIO_FLAG_WRITE, av_dump_format, av_interleaved_write_frame,
  av_write_trailer, avformat_alloc_output_context2, avformat_free_context, avformat_write_header,
  avio_closep, avio_open,
};
use std::{
  ffi::CString,
  ops::{Deref, DerefMut},
  path::Path,
  ptr::{null, null_mut},
};

pub struct AvOutputContext {
  context: AvContext,
}

pub struct AvOutputWriter {
  context: AvOutputContext,
}

impl AvOutputWriter {
  pub fn write_frame(&mut self, pkt: &mut AvPacket) -> Result<(), AvError> {
    unsafe_av_result!(av_interleaved_write_frame(
      self.context.inner,
      pkt.deref_mut()
    ))
  }

  pub fn write_header(&mut self) -> Result<(), AvError> {
    unsafe_av_result!(avformat_write_header(self.context.inner, null_mut()))
  }

  pub fn finish(self) -> Result<(), AvError> {
    unsafe_av_result!(av_write_trailer(self.context.inner))?;
    Ok(())
  }
}

impl AvOutputContext {
  pub fn open_path<T>(path: T) -> Result<Self, AvError>
  where
    T: AsRef<Path>,
  {
    match path.as_ref().to_str() {
      Some(v) => Self::open(v),
      None => Err(std::io::ErrorKind::NotFound.into()),
    }
  }

  pub fn open(path: &str) -> Result<Self, AvError> {
    let path = CString::new(path).map_err(|_| std::io::ErrorKind::InvalidData)?;
    let mut ctx = AvContext { inner: null_mut() };

    unsafe_av_result!(avformat_alloc_output_context2(
      &mut ctx.inner,
      null(),
      null(),
      path.as_bytes_with_nul().as_ptr().cast(),
    ))?;

    if ctx.inner.is_null() {
      Err(AvError::AllocationError)
    } else {
      Ok(Self { context: ctx })
    }
  }

  pub fn start_writer(mut self) -> Result<AvOutputWriter, AvError> {
    if self.flags as u32 & AVFMT_NOFILE != AVFMT_NOFILE && !self.url.is_null() {
      unsafe_av_result!(avio_open(&mut self.pb, self.url, AVIO_FLAG_WRITE as i32))?;
    }

    Ok(AvOutputWriter { context: self })
  }

  pub fn dump(&self) -> Result<(), AvError> {
    unsafe {
      if !self.inner.is_null() && !(*self.inner).url.is_null() {
        av_dump_format(self.inner, 0, (*self.inner).url, 1);
        Ok(())
      } else {
        Err(AvError::DumpError)
      }
    }
  }

  #[inline]
  pub const fn as_ptr(&self) -> *const AVFormatContext {
    self.context.inner
  }

  #[inline]
  pub const fn as_mut_ptr(&self) -> *mut AVFormatContext {
    self.context.inner
  }
}

impl Deref for AvOutputContext {
  type Target = AvContext;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.context
  }
}

impl AsRef<AvContext> for AvOutputContext {
  #[inline]
  fn as_ref(&self) -> &AvContext {
    &self.context
  }
}

impl DerefMut for AvOutputContext {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.context
  }
}

impl AsMut<AvContext> for AvOutputContext {
  #[inline]
  fn as_mut(&mut self) -> &mut AvContext {
    &mut self.context
  }
}

impl Drop for AvOutputWriter {
  fn drop(&mut self) {
    if !self.context.pb.is_null() && (self.context.flags as u32 & AVFMT_NOFILE) != AVFMT_NOFILE {
      _ = unsafe_av_result!(avio_closep(&mut self.context.pb));
    }
  }
}

impl Drop for AvOutputContext {
  fn drop(&mut self) {
    if !self.context.inner.is_null() {
      unsafe {
        avformat_free_context(self.context.inner);
      }
    }
  }
}
