// TODO: constant from avutil is not provided by bindgen?
pub const AV_NOPTS_VALUE: i64 = 0x8000000000000000u64 as i64;

#[macro_export]
macro_rules! ts_as_option {
  ($value:expr) => {{
    if $value == $crate::common::AV_NOPTS_VALUE {
      None
    } else {
      Some($value)
    }
  }};
}

#[macro_export]
macro_rules! unsafe_av_result {
  ($fn:expr) => {{
    let result_code: i32 = unsafe { $fn };
    $crate::error::AvError::from_raw_err_code(result_code)
  }};
}

pub use ts_as_option;
pub use unsafe_av_result;
