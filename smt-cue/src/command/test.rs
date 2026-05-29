use cue_lib::probe::CuesheetProbe;
use smt_common::Command;

pub struct CmdTest<'a> {
  cuesheet: &'a str,
}

impl<'a> CmdTest<'a> {
  #[inline]
  pub const fn new(cuesheet: &'a str) -> Self {
    Self { cuesheet }
  }
}

impl<'a> Command for &'a CmdTest<'a> {
  type Error = cue_lib::error::CueLibError;

  #[inline]
  fn run(self) -> Result<(), cue_lib::error::CueLibError> {
    CuesheetProbe::verify(self.cuesheet)
  }
}
