use std::{
  fs::File,
  io::{StdoutLock, Write},
};

pub enum OutputStream {
  File(File),
  Stdout(StdoutLock<'static>),
}

impl Write for OutputStream {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    match self {
      OutputStream::File(file) => file.write(buf),
      OutputStream::Stdout(stdout_lock) => stdout_lock.write(buf),
    }
  }

  fn flush(&mut self) -> std::io::Result<()> {
    match self {
      OutputStream::File(file) => file.flush(),
      OutputStream::Stdout(stdout_lock) => stdout_lock.flush(),
    }
  }
}

impl From<File> for OutputStream {
  #[inline]
  fn from(value: File) -> Self {
    Self::File(value)
  }
}

impl From<StdoutLock<'static>> for OutputStream {
  #[inline]
  fn from(value: StdoutLock<'static>) -> Self {
    Self::Stdout(value)
  }
}
