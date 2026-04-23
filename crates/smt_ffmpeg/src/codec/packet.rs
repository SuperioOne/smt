use crate::{common::unsafe_av_result, error::AvError};
use smt_ffmpeg_sys::{
  AVPacket, AVRational, av_packet_alloc, av_packet_clone, av_packet_free, av_packet_ref,
  av_packet_rescale_ts, av_packet_unref,
};
use std::ops::{Deref, DerefMut};

pub struct AvPacket {
  inner: *mut AVPacket,
}

impl AvPacket {
  pub fn try_new() -> Result<Self, AvError> {
    let packet = unsafe { av_packet_alloc() };

    if packet.is_null() {
      Err(AvError::AllocationError)
    } else {
      Ok(Self { inner: packet })
    }
  }

  pub fn reset(&mut self) {
    unsafe {
      av_packet_unref(self.inner);
    }
  }

  pub fn rescale(&mut self, tb_src: AVRational, tb_dst: AVRational) {
    unsafe {
      av_packet_rescale_ts(self.inner, tb_src, tb_dst);
    }
  }

  pub fn ref_from(&mut self, src: &AvPacket) -> Result<(), AvError> {
    unsafe_av_result!(av_packet_ref(self.inner, src.inner))
  }
}

impl Drop for AvPacket {
  fn drop(&mut self) {
    if !self.inner.is_null() {
      unsafe {
        av_packet_free(&mut self.inner);
      }
    }
  }
}

impl Deref for AvPacket {
  type Target = AVPacket;

  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.inner }
  }
}

impl DerefMut for AvPacket {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.inner }
  }
}

impl AsRef<AVPacket> for AvPacket {
  #[inline]
  fn as_ref(&self) -> &AVPacket {
    unsafe { &*self.inner }
  }
}

impl AsMut<AVPacket> for AvPacket {
  #[inline]
  fn as_mut(&mut self) -> &mut AVPacket {
    unsafe { &mut *self.inner }
  }
}

impl Clone for AvPacket {
  fn clone(&self) -> Self {
    Self {
      inner: unsafe { av_packet_clone(self.inner) },
    }
  }
}
