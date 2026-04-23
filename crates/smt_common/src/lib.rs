use self::output_error::ErrorFormat;
use clap::ValueEnum;

pub mod input_reader;
pub mod output_error;
pub mod output_writer;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum VerboseLevel {
  Default,
  Full,
  Quiet,
}

impl Default for VerboseLevel {
  #[inline]
  fn default() -> Self {
    Self::Default
  }
}

pub trait Command
where
  Self::Error: ErrorFormat,
{
  type Error;

  fn run(self) -> Result<(), Self::Error>;
}
