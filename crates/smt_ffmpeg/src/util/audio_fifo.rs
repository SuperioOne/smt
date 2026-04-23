use crate::error::AvError;
use smt_ffmpeg_sys::{
  AVAudioFifo, AVSampleFormat, av_audio_fifo_alloc, av_audio_fifo_free, av_audio_fifo_read,
  av_audio_fifo_reset, av_audio_fifo_size, av_audio_fifo_write,
};

pub struct AudioFifo {
  inner: *mut AVAudioFifo,
}

impl AudioFifo {
  pub fn new(sample_format: AVSampleFormat, channel_count: i32) -> Self {
    Self {
      inner: unsafe { av_audio_fifo_alloc(sample_format, channel_count, 1) },
    }
  }

  pub fn with_capacity(sample_format: AVSampleFormat, channel_count: usize, len: usize) -> Self {
    Self {
      inner: unsafe { av_audio_fifo_alloc(sample_format, channel_count as i32, len as i32) },
    }
  }

  pub fn reset(&mut self) {
    unsafe {
      av_audio_fifo_reset(self.inner);
    }
  }

  pub fn len(&self) -> usize {
    let len = unsafe { av_audio_fifo_size(self.inner) };

    if len < 0 { 0 } else { len as usize }
  }

  pub fn is_empty(&self) -> bool {
    self.len() < 1
  }

  pub fn push(&mut self, data: *const *mut u8, frame_size: i32) -> Result<usize, AvError> {
    let result = unsafe { av_audio_fifo_write(self.inner, data.cast(), frame_size as i32) };
    AvError::from_raw_err_code(result)?;
    Ok(result as usize)
  }

  pub fn pop(&mut self, dst: *const *mut u8, len: i32) -> Result<usize, AvError> {
    let result = unsafe { av_audio_fifo_read(self.inner, dst.cast(), len) };
    AvError::from_raw_err_code(result)?;
    Ok(result as usize)
  }
}

impl Drop for AudioFifo {
  fn drop(&mut self) {
    unsafe {
      av_audio_fifo_free(self.inner);
    }
  }
}
