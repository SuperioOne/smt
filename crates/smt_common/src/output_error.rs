use crate::VerboseLevel;
use std::{
  fmt::Arguments,
  io::{IsTerminal, Stderr, StderrLock, Write, stderr},
};

mod format_impls;

pub trait ErrorFormat {
  fn fmt(
    &self,
    f: ErrorFormatter<'_>,
    input_buffer: &str,
    verbose: VerboseLevel,
  ) -> std::io::Result<()>;
}

pub struct AnsiCodes {
  error: &'static str,
  reset: &'static str,
  warn: &'static str,
  info: &'static str,
}

pub struct OutputError<'a> {
  ansi_codes: AnsiCodes,
  input_buffer: &'a str,
  output_stream: Stderr,
  verbose_level: VerboseLevel,
}

pub struct ErrorFormatter<'a> {
  ansi_codes: &'a AnsiCodes,
  writer: StderrLock<'a>,
}

impl<'a> ErrorFormatter<'a> {
  #[inline]
  pub fn flush(mut self) -> std::io::Result<()> {
    self.writer.flush()
  }

  fn write_line_feed(&mut self) -> std::io::Result<()> {
    cfg_select! {
      target_os = "windows" => {
        const LF: [u8; 2] = [b'\r', b'\n'];
      }
      _ => {
        const LF: [u8; 1] = [b'\n'];
      }
    };
    self.writer.write(&LF)?;
    Ok(())
  }

  pub fn write_error(&mut self, args: Arguments<'_>) -> std::io::Result<()> {
    self.writer.write(self.ansi_codes.error.as_bytes())?;
    self.writer.write_fmt(args)?;
    self.writer.write(self.ansi_codes.reset.as_bytes())?;
    Ok(())
  }

  pub fn write_warn(&mut self, args: Arguments<'_>) -> std::io::Result<()> {
    self.writer.write(self.ansi_codes.warn.as_bytes())?;
    self.writer.write_fmt(args)?;
    self.writer.write(self.ansi_codes.reset.as_bytes())?;
    Ok(())
  }

  pub fn write_info(&mut self, args: Arguments<'_>) -> std::io::Result<()> {
    self.writer.write(self.ansi_codes.info.as_bytes())?;
    self.writer.write_fmt(args)?;
    self.writer.write(self.ansi_codes.reset.as_bytes())?;
    Ok(())
  }

  pub fn writeln_error(&mut self, args: Arguments<'_>) -> std::io::Result<()> {
    self.write_error(args)?;
    self.write_line_feed()
  }

  pub fn writeln_warn(&mut self, args: Arguments<'_>) -> std::io::Result<()> {
    self.write_warn(args)?;
    self.write_line_feed()
  }

  pub fn writeln_info(&mut self, args: Arguments<'_>) -> std::io::Result<()> {
    self.write_info(args)?;
    self.write_line_feed()
  }
}

impl<'a> OutputError<'a> {
  pub fn new() -> Self {
    let output = stderr();
    let ansi_codes = if output.is_terminal() {
      AnsiCodes {
        error: "\x1B[1;91m",
        info: "\x1B[1;96m",
        warn: "\x1B[1;93m",
        reset: "\x1B[0m",
      }
    } else {
      AnsiCodes::default()
    };

    Self {
      ansi_codes,
      input_buffer: "",
      output_stream: output,
      verbose_level: VerboseLevel::Default,
    }
  }

  #[inline]
  pub const fn set_verbosity(&mut self, verbose_level: VerboseLevel) {
    self.verbose_level = verbose_level;
  }

  #[inline]
  pub const fn set_input_buffer(&mut self, input: &'a str) {
    self.input_buffer = input;
  }

  pub fn write_error<E>(&mut self, error: E) -> std::io::Result<()>
  where
    E: ErrorFormat,
  {
    let formatter = ErrorFormatter {
      writer: self.output_stream.lock(),
      ansi_codes: &self.ansi_codes,
    };
    error.fmt(formatter, self.input_buffer, self.verbose_level)
  }

  pub fn write_str(&mut self, input: &str) -> std::io::Result<()> {
    match self.verbose_level {
      VerboseLevel::Quiet => Ok(()),
      _ => self.output_stream.lock().write_all(input.as_bytes()),
    }
  }

  pub fn write_fmt(&mut self, args: Arguments<'_>) -> std::io::Result<()> {
    match self.verbose_level {
      VerboseLevel::Quiet => Ok(()),
      _ => self.output_stream.lock().write_fmt(args),
    }
  }

  #[inline]
  pub fn flush(&mut self) -> std::io::Result<()> {
    self.output_stream.flush()
  }

  #[inline]
  pub fn close(mut self) {
    _ = self.output_stream.flush();
  }
}

impl Default for AnsiCodes {
  fn default() -> Self {
    Self {
      error: Default::default(),
      reset: Default::default(),
      warn: Default::default(),
      info: Default::default(),
    }
  }
}

impl Write for ErrorFormatter<'_> {
  #[inline]
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.writer.write(buf)
  }

  #[inline]
  fn flush(&mut self) -> std::io::Result<()> {
    self.writer.flush()
  }
}

