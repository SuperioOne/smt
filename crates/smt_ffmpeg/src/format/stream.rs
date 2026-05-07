use smt_ffmpeg_sys::{
  AVFormatContext, AVMediaType_AVMEDIA_TYPE_ATTACHMENT, AVMediaType_AVMEDIA_TYPE_AUDIO,
  AVMediaType_AVMEDIA_TYPE_DATA, AVMediaType_AVMEDIA_TYPE_NB, AVMediaType_AVMEDIA_TYPE_SUBTITLE,
  AVMediaType_AVMEDIA_TYPE_UNKNOWN, AVMediaType_AVMEDIA_TYPE_VIDEO, AVStream, av_dict_copy,
  av_packet_ref, avcodec_parameters_copy,
};
use std::marker::PhantomData;

use crate::error::AvError;
use crate::unsafe_av_result;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StreamType {
  Audio,
  Video,
  Subtitle,
  Data,
  Attachment,
  NB,
  Unknown,
}

pub struct StreamIter<'a> {
  streams: *const *const AVStream,
  len: usize,
  current: usize,
  _phantom: PhantomData<&'a AVStream>,
}

pub struct StreamMutIter<'a> {
  streams: *const *mut AVStream,
  len: usize,
  current: usize,
  _phantom: PhantomData<&'a AVStream>,
}

pub fn copy_stream_properties(src: &AVStream, dst: &mut AVStream) -> Result<(), AvError> {
  dst.disposition = src.disposition;
  dst.time_base = src.time_base;
  dst.duration = 0;
  dst.start_time = 0;

  if !src.metadata.is_null() {
    unsafe_av_result!(av_dict_copy(&mut dst.metadata, src.metadata, 0))?;
  }

  unsafe_av_result!(av_packet_ref(&mut dst.attached_pic, &src.attached_pic))?;

  if !src.codecpar.is_null() {
    unsafe_av_result!(avcodec_parameters_copy(dst.codecpar, src.codecpar))?;
  }

  unsafe {
    (*dst.codecpar).codec_tag = 0;
  }

  Ok(())
}

#[allow(nonstandard_style)]
impl StreamType {
  pub const fn from_i32(value: i32) -> Self {
    match value {
      AVMediaType_AVMEDIA_TYPE_AUDIO => Self::Audio,
      AVMediaType_AVMEDIA_TYPE_NB => Self::NB,
      AVMediaType_AVMEDIA_TYPE_DATA => Self::Data,
      AVMediaType_AVMEDIA_TYPE_VIDEO => Self::Video,
      AVMediaType_AVMEDIA_TYPE_SUBTITLE => Self::Subtitle,
      AVMediaType_AVMEDIA_TYPE_ATTACHMENT => Self::Attachment,
      _ => Self::Unknown,
    }
  }

  pub const fn as_i32(&self) -> i32 {
    match self {
      Self::Audio => AVMediaType_AVMEDIA_TYPE_AUDIO,
      Self::NB => AVMediaType_AVMEDIA_TYPE_NB,
      Self::Data => AVMediaType_AVMEDIA_TYPE_DATA,
      Self::Video => AVMediaType_AVMEDIA_TYPE_VIDEO,
      Self::Subtitle => AVMediaType_AVMEDIA_TYPE_SUBTITLE,
      Self::Attachment => AVMediaType_AVMEDIA_TYPE_ATTACHMENT,
      Self::Unknown => AVMediaType_AVMEDIA_TYPE_UNKNOWN,
    }
  }
}

impl PartialEq<i32> for StreamType {
  #[inline]
  fn eq(&self, other: &i32) -> bool {
    self.as_i32() == *other
  }
}

impl PartialEq<StreamType> for i32 {
  #[inline]
  fn eq(&self, other: &StreamType) -> bool {
    *self == other.as_i32()
  }
}

impl<'a> StreamIter<'a> {
  #[inline]
  pub const fn from_context(context: &'a AVFormatContext) -> Self {
    let len = if context.streams.is_null() {
      0
    } else {
      context.nb_streams as usize
    };

    Self {
      streams: context.streams.cast(),
      len,
      current: 0,
      _phantom: PhantomData,
    }
  }
}

impl<'a> StreamMutIter<'a> {
  #[inline]
  pub const fn from_context(context: &'a mut AVFormatContext) -> Self {
    let len = if context.streams.is_null() {
      0
    } else {
      context.nb_streams as usize
    };

    Self {
      streams: context.streams,
      len,
      current: 0,
      _phantom: PhantomData,
    }
  }
}

impl<'a> Iterator for StreamMutIter<'a> {
  type Item = &'a mut AVStream;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.current >= self.len {
        return None;
      } else {
        let next_item = unsafe { self.streams.add(self.current).read() };
        self.current += 1;

        match unsafe { next_item.as_mut() } {
          Some(item) => return Some(item),
          None => continue,
        };
      }
    }
  }
}

impl<'a> Iterator for StreamIter<'a> {
  type Item = &'a AVStream;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.current >= self.len {
        return None;
      } else {
        let next_item = unsafe { self.streams.add(self.current).read() };
        self.current += 1;

        match unsafe { next_item.as_ref() } {
          Some(item) => return Some(item),
          None => continue,
        };
      }
    }
  }
}
