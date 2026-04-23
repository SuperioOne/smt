use smt_ffmpeg_sys::{
  AV_TIME_BASE, AVRational, AVRounding, AVRounding_AV_ROUND_DOWN, AVRounding_AV_ROUND_INF,
  AVRounding_AV_ROUND_NEAR_INF, AVRounding_AV_ROUND_PASS_MINMAX, AVRounding_AV_ROUND_UP,
  AVRounding_AV_ROUND_ZERO, av_rescale_q, av_rescale_q_rnd,
};
use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

pub const AV_TIME_BASE_Q: AVRational = AVRational {
  num: 1,
  den: AV_TIME_BASE as i32,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
  Zero,
  Inf,
  Down,
  Up,
  NearInf,
  PassMinMax,
}

/// Wrapper struct to make pts/dts operations a little bit more fail-safe at the expense of carrying
/// around additional 8-byte timebase info.
///
/// It can be safely used for comparing/manipulating timestamps with different timebases,
/// as it automatically rescales the right hand side operand to the left hand side's timebase.
#[derive(Copy, Clone, Debug)]
pub struct AvTimestamp {
  timestamp: i64,
  time_base: AVRational,
}

impl Into<AVRounding> for Rounding {
  #[inline]
  fn into(self) -> AVRounding {
    match self {
      Rounding::Zero => AVRounding_AV_ROUND_ZERO,
      Rounding::Inf => AVRounding_AV_ROUND_INF,
      Rounding::Down => AVRounding_AV_ROUND_DOWN,
      Rounding::Up => AVRounding_AV_ROUND_UP,
      Rounding::NearInf => AVRounding_AV_ROUND_NEAR_INF,
      Rounding::PassMinMax => AVRounding_AV_ROUND_PASS_MINMAX,
    }
  }
}

impl AvTimestamp {
  #[inline]
  pub const fn new(value: i64, time_base: AVRational) -> Self {
    Self {
      timestamp: value,
      time_base,
    }
  }

  pub const fn from_duration(duration: Duration) -> Self {
    let timestamp = duration.as_secs() as i64 * AV_TIME_BASE as i64;

    Self {
      timestamp,
      time_base: AV_TIME_BASE_Q,
    }
  }

  pub fn to_duration(&self) -> Duration {
    if self.time_base == AV_TIME_BASE_Q {
      Duration::from_micros(self.timestamp as u64)
    } else {
      let scaled = self.rescale(AV_TIME_BASE_Q);
      Duration::from_micros(scaled.timestamp as u64)
    }
  }

  /// Rescale timestamp with the new time base
  pub fn rescale(self, target_time_base: AVRational) -> Self {
    let timestamp = unsafe { av_rescale_q(self.timestamp, self.time_base, target_time_base) };

    Self {
      time_base: target_time_base,
      timestamp,
    }
  }

  /// Rescale timestamp with the new time base and custom rounding
  pub fn rescale_rounding(self, target_time_base: AVRational, rounding: Rounding) -> Self {
    let timestamp = unsafe {
      av_rescale_q_rnd(
        self.timestamp,
        self.time_base,
        target_time_base,
        rounding.into(),
      )
    };

    Self {
      time_base: target_time_base,
      timestamp,
    }
  }

  #[inline]
  pub const fn as_i64(&self) -> i64 {
    self.timestamp
  }
}

impl PartialEq for AvTimestamp {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}

impl Eq for AvTimestamp {}

impl PartialOrd for AvTimestamp {
  #[inline]
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for AvTimestamp {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    let other = if self.time_base == other.time_base {
      *other
    } else {
      other.rescale(self.time_base)
    };

    self.timestamp.cmp(&other.timestamp)
  }
}

impl Default for AvTimestamp {
  #[inline]
  fn default() -> Self {
    Self {
      timestamp: 0,
      time_base: AV_TIME_BASE_Q,
    }
  }
}

impl From<Duration> for AvTimestamp {
  #[inline]
  fn from(value: Duration) -> Self {
    Self::from_duration(value)
  }
}

impl Into<Duration> for AvTimestamp {
  #[inline]
  fn into(self) -> Duration {
    self.to_duration()
  }
}

impl Into<Duration> for &AvTimestamp {
  #[inline]
  fn into(self) -> Duration {
    self.to_duration()
  }
}

impl Sub for AvTimestamp {
  type Output = AvTimestamp;

  #[inline]
  fn sub(mut self, rhs: Self) -> Self::Output {
    self.sub_assign(rhs);
    self
  }
}

impl Add for AvTimestamp {
  type Output = AvTimestamp;

  #[inline]
  fn add(mut self, rhs: Self) -> Self::Output {
    self.add_assign(rhs);
    self
  }
}

impl SubAssign for AvTimestamp {
  fn sub_assign(&mut self, rhs: Self) {
    let value = if self.time_base == rhs.time_base {
      rhs.timestamp
    } else {
      rhs.rescale(self.time_base).timestamp
    };

    self.timestamp = self.timestamp.saturating_sub(value);
  }
}

impl AddAssign for AvTimestamp {
  fn add_assign(&mut self, rhs: Self) {
    let value = if self.time_base == rhs.time_base {
      rhs.timestamp
    } else {
      rhs.rescale(self.time_base).timestamp
    };

    self.timestamp = self.timestamp.saturating_add(value);
  }
}
