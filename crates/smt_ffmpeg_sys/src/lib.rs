#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unnecessary_transmutes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl PartialEq for AVRational {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
    self.num == other.num && self.den == other.den
  }
}

impl Eq for AVRational {}

impl PartialOrd for AVRational {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    let a = self.num as f32 / self.den as f32;
    let b = other.num as f32 / other.den as f32;

    a.partial_cmp(&b)
  }
}
