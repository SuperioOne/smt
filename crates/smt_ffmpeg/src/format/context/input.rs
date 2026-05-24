use super::AvContext;
use crate::{codec::packet::AvPacket, common::unsafe_av_result, error::AvError};
use smt_ffmpeg_sys::{
  av_dump_format, av_read_frame, avformat_close_input, avformat_find_stream_info,
  avformat_open_input,
};
use std::{
  ffi::CString,
  ops::{Deref, DerefMut},
  path::Path,
  ptr::{null, null_mut},
};

pub struct AvInputContext {
  context: AvContext,
}

impl AvInputContext {
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

    unsafe_av_result!(avformat_open_input(
      &mut ctx.inner,
      path.as_bytes_with_nul().as_ptr().cast(),
      null(),
      null_mut(),
    ))?;

    unsafe_av_result!(avformat_find_stream_info(ctx.inner, null_mut()))?;

    if ctx.inner.is_null() {
      Err(AvError::AllocationError)
    } else {
      Ok(Self { context: ctx })
    }
  }

  pub fn read_frame(&mut self, pkt: &mut AvPacket) -> Result<(), AvError> {
    unsafe_av_result!(av_read_frame(self.inner, pkt.deref_mut()))
  }

  pub fn dump(&self) -> Result<(), AvError> {
    unsafe {
      if !self.inner.is_null() && !(*self.inner).url.is_null() {
        av_dump_format(self.inner, 0, (*self.inner).url, 0);
        Ok(())
      } else {
        Err(AvError::DumpError)
      }
    }
  }
}

impl Deref for AvInputContext {
  type Target = AvContext;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.context
  }
}

impl DerefMut for AvInputContext {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.context
  }
}

impl AsRef<AvContext> for AvInputContext {
  #[inline]
  fn as_ref(&self) -> &AvContext {
    &self.context
  }
}

impl AsMut<AvContext> for AvInputContext {
  #[inline]
  fn as_mut(&mut self) -> &mut AvContext {
    &mut self.context
  }
}

impl Drop for AvInputContext {
  fn drop(&mut self) {
    unsafe {
      avformat_close_input(&mut self.context.inner);
    }
  }
}
