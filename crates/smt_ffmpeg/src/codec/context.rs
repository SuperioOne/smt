use crate::{
  codec::{frame::AvFrame, packet::AvPacket},
  error::AvError,
  unsafe_av_result,
};
use smt_ffmpeg_sys::{
  AVCodec, AVCodecContext, AVCodecParameters, avcodec_alloc_context3, avcodec_find_decoder,
  avcodec_find_encoder, avcodec_free_context, avcodec_is_open, avcodec_open2,
  avcodec_parameters_from_context, avcodec_parameters_to_context, avcodec_receive_frame,
  avcodec_receive_packet, avcodec_send_frame, avcodec_send_packet,
};
use std::{
  ops::{Deref, DerefMut},
  ptr::{null, null_mut},
};

pub struct AvCodecContext {
  inner: *mut AVCodecContext,
  codec: *const AVCodec,
}

impl AvCodecContext {
  pub fn new_encoder(codec_id: u32) -> Self {
    let codec = unsafe { avcodec_find_encoder(codec_id) };
    Self {
      inner: unsafe { avcodec_alloc_context3(codec) },
      codec,
    }
  }

  pub fn new_decoder(codec_id: u32) -> Self {
    let codec = unsafe { avcodec_find_decoder(codec_id) };
    Self {
      inner: unsafe { avcodec_alloc_context3(codec) },
      codec,
    }
  }

  pub fn copy_params_from(&mut self, src: &AVCodecParameters) -> Result<(), AvError> {
    unsafe_av_result!(avcodec_parameters_to_context(self.inner, src))
  }

  pub fn copy_params_to(&self, dst: &mut AVCodecParameters) -> Result<(), AvError> {
    unsafe_av_result!(avcodec_parameters_from_context(dst, self.inner))
  }

  pub fn open(&mut self) -> Result<(), AvError> {
    unsafe_av_result!(avcodec_open2(self.inner, self.codec, null_mut()))
  }

  pub fn is_open(&self) -> bool {
    unsafe { avcodec_is_open(self.inner) > 0 }
  }

  pub fn send_packet(&mut self, packet: &AvPacket) -> Result<(), AvError> {
    unsafe_av_result!(avcodec_send_packet(self.inner, core::ptr::from_ref(packet)))
  }

  pub fn send_frame(&mut self, frame: &AvFrame) -> Result<(), AvError> {
    unsafe_av_result!(avcodec_send_frame(self.inner, frame.deref()))
  }

  pub fn flush(&mut self) -> Result<(), AvError> {
    unsafe_av_result!(avcodec_send_frame(self.inner, null()))
  }

  pub fn receive_frame(&mut self, frame: &mut AvFrame) -> Result<(), AvError> {
    unsafe_av_result!(avcodec_receive_frame(self.inner, frame.deref_mut()))
  }

  pub fn receive_packet(&mut self, packet: &mut AvPacket) -> Result<(), AvError> {
    unsafe_av_result!(avcodec_receive_packet(self.inner, packet.deref_mut()))
  }
}

impl Deref for AvCodecContext {
  type Target = AVCodecContext;

  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.inner }
  }
}

impl DerefMut for AvCodecContext {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.inner }
  }
}

impl Drop for AvCodecContext {
  fn drop(&mut self) {
    unsafe {
      avcodec_free_context(&mut self.inner);
    }
  }
}
