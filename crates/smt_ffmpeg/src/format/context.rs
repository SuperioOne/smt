use crate::{
  common::{ts_as_option, unsafe_av_result},
  error::{AvError, AvLibError},
  format::stream::{StreamIter, StreamMutIter, StreamType},
  util::{
    dictionary::{AvDictionaryMut, AvDictionaryRef},
    timestamp::AvTimestamp,
  },
};
use smt_ffmpeg_sys::{
  AV_DISPOSITION_ATTACHED_PIC, AVFormatContext, AVSEEK_FLAG_BACKWARD, AVStream,
  av_find_best_stream, av_seek_frame, avformat_new_stream,
};
use std::{
  ffi::CStr,
  ops::{Deref, DerefMut},
  ptr::{null, null_mut},
  time::Duration,
};

mod input;
mod output;

pub use input::*;
pub use output::*;

macro_rules! static_cstr {
  ($value:literal) => {
    unsafe { &CStr::from_bytes_with_nul_unchecked(concat!($value, "\0").as_bytes()) }
  };
}

pub const COVER_IMAGE_KEY: &'static CStr = static_cstr!("comment");
pub const COVER_IMAGE_VALUE: &'static CStr = static_cstr!("Cover (front)");

pub struct AvContext {
  inner: *mut AVFormatContext,
}

impl AvContext {
  pub const fn duration(&self) -> Option<Duration> {
    match ts_as_option!(unsafe { (*self.inner).duration }) {
      Some(v) => Some(Duration::from_micros(v as u64)),
      None => None,
    }
  }

  #[inline]
  pub fn seek_forward(&mut self, stream_index: u32, timestamp: Duration) -> Result<(), AvError> {
    self.internal_seek(stream_index, timestamp, 0)
  }

  #[inline]
  pub fn seek_backward(&mut self, stream_index: u32, timestamp: Duration) -> Result<(), AvError> {
    self.internal_seek(stream_index, timestamp, AVSEEK_FLAG_BACKWARD as i32)
  }

  fn internal_seek(
    &mut self,
    stream_index: u32,
    timestamp: Duration,
    flag: i32,
  ) -> Result<(), AvError> {
    if stream_index < self.nb_streams {
      let av_timestamp = AvTimestamp::from(timestamp);

      unsafe_av_result!(av_seek_frame(
        self.inner,
        stream_index as i32,
        av_timestamp.as_i64(),
        flag
      ))
    } else {
      Err(AvLibError::StreamNotFound.into())
    }
  }

  pub fn get_stream(&self, idx: usize) -> Option<&AVStream> {
    if idx >= self.nb_streams as usize {
      None
    } else {
      let stream = unsafe { self.streams.add(idx).read() };
      unsafe { stream.as_ref() }
    }
  }

  pub fn get_mut_stream(&self, idx: usize) -> Option<&mut AVStream> {
    if idx >= self.nb_streams as usize {
      None
    } else {
      let stream = unsafe { self.streams.add(idx).read() };
      unsafe { stream.as_mut() }
    }
  }

  pub fn find_best_stream(&self, stream_type: StreamType) -> Option<&AVStream> {
    let index =
      unsafe { av_find_best_stream(self.inner, stream_type.as_i32(), -1, -1, null_mut(), 0) };

    if index >= 0 {
      self.get_stream(index as usize)
    } else {
      None
    }
  }

  pub fn find_cover_image_stream(&self) -> Option<&AVStream> {
    for stream in self.stream_iter() {
      if (stream.disposition as u32 & AV_DISPOSITION_ATTACHED_PIC) == AV_DISPOSITION_ATTACHED_PIC {
        return Some(stream);
      }
    }

    None
  }

  pub fn create_stream(&mut self) -> Result<&mut AVStream, AvError> {
    let stream = unsafe { avformat_new_stream(self.inner, null()) };

    if let Some(value) = unsafe { stream.as_mut() } {
      Ok(value)
    } else {
      Err(AvError::AllocationError)
    }
  }

  #[inline]
  pub fn stream_iter(&self) -> StreamIter<'_> {
    StreamIter::from_context(self)
  }

  #[inline]
  pub fn stream_mut_iter(&mut self) -> StreamMutIter<'_> {
    StreamMutIter::from_context(self)
  }

  #[inline]
  pub const fn metadata(&self) -> AvDictionaryRef<'_> {
    AvDictionaryRef::from_ptr_ref(&unsafe { &mut *self.inner }.metadata)
  }

  #[inline]
  pub const fn metadata_mut(&mut self) -> AvDictionaryMut<'_> {
    AvDictionaryMut::from_ptr_ref(&mut unsafe { &mut *self.inner }.metadata)
  }

  #[inline]
  pub const fn as_ptr(&self) -> *const AVFormatContext {
    self.inner
  }
}

impl Deref for AvContext {
  type Target = AVFormatContext;

  #[inline]
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.inner }
  }
}

impl DerefMut for AvContext {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.inner }
  }
}

impl AsRef<AVFormatContext> for AvContext {
  #[inline]
  fn as_ref(&self) -> &AVFormatContext {
    unsafe { &*self.inner }
  }
}

impl AsMut<AVFormatContext> for AvContext {
  #[inline]
  fn as_mut(&mut self) -> &mut AVFormatContext {
    unsafe { &mut *self.inner }
  }
}
