use crate::{common::unsafe_av_result, error::AvError};
use smt_ffmpeg_sys::{
  AVChannelLayout, AVFrame, av_channel_layout_copy, av_frame_alloc, av_frame_clone, av_frame_free,
  av_frame_get_buffer, av_frame_ref, av_frame_unref,
};
use std::ops::{Deref, DerefMut};

pub struct AvFrame {
  inner: *mut AVFrame,
}

impl AvFrame {
  pub fn try_new() -> Result<Self, AvError> {
    let frame = unsafe { av_frame_alloc() };

    if frame.is_null() {
      Err(AvError::AllocationError)
    } else {
      Ok(Self { inner: frame })
    }
  }

  pub fn reset(&mut self) {
    unsafe {
      av_frame_unref(self.inner);
    }
  }

  pub fn ref_from(&mut self, src: &AvFrame) -> Result<(), AvError> {
    unsafe_av_result!(av_frame_ref(self.inner, src.inner))
  }

  pub fn copy_channel_layout(&mut self, layout: &AVChannelLayout) -> Result<(), AvError> {
    unsafe_av_result!(av_channel_layout_copy(&mut self.ch_layout, layout))
  }

  pub fn alloc_buffer(&mut self) -> Result<(), AvError> {
    unsafe_av_result!(av_frame_get_buffer(self.inner, 0))
  }
}

impl Drop for AvFrame {
  fn drop(&mut self) {
    if !self.inner.is_null() {
      unsafe {
        av_frame_free(&mut self.inner);
      }
    }
  }
}

impl Deref for AvFrame {
  type Target = AVFrame;

  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.inner }
  }
}

impl DerefMut for AvFrame {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.inner }
  }
}

impl AsRef<AVFrame> for AvFrame {
  #[inline]
  fn as_ref(&self) -> &AVFrame {
    unsafe { &*self.inner }
  }
}

impl AsMut<AVFrame> for AvFrame {
  #[inline]
  fn as_mut(&mut self) -> &mut AVFrame {
    unsafe { &mut *self.inner }
  }
}

impl Clone for AvFrame {
  fn clone(&self) -> Self {
    Self {
      inner: unsafe { av_frame_clone(self.inner) },
    }
  }
}
