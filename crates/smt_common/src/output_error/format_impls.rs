use super::{ErrorFormat, ErrorFormatter};
use crate::VerboseLevel;
use std::io::Write as _;

impl ErrorFormat for cue_lib::error::CueLibError {
  fn fmt(
    &self,
    mut f: ErrorFormatter<'_>,
    input_context: &str,
    verbose_level: VerboseLevel,
  ) -> std::io::Result<()> {
    match self.kind() {
      cue_lib::error::CueLibErrorKind::ParseError(parse_error) => match verbose_level {
        VerboseLevel::Quiet => Ok(()),
        VerboseLevel::Default => f.writeln_error(format_args!("{}", &self)),
        VerboseLevel::Full => {
          let line_idx = parse_error.line();
          let line_no = line_idx + 1;
          let start = line_idx.saturating_sub(10);

          for (rel_idx, line) in input_context.lines().skip(start).take(20).enumerate() {
            if rel_idx + start == line_idx {
              let column_idx = if parse_error.column() > 0 {
                parse_error.column()
              } else {
                line.chars().take_while(|v| v.is_whitespace()).count()
              };
              let column = column_idx + 1;

              f.writeln_error(format_args!("{}", line))?;

              if column_idx > 0 {
                write!(f, "{:>column_idx$}", ' ')?;
              }

              f.writeln_warn(format_args!(
                "^{dash:->24}{self} at {line_no}:{column}",
                dash = ' ',
              ))?;
            } else {
              writeln!(f, "{line}")?;
            }
          }

          if let Some((_, line)) = input_context
            .lines()
            .enumerate()
            .find(|(idx, _)| *idx == line_idx)
          {
            f.writeln_info(format_args!("\nLine {line_no}: UTF-8 Character Breakdown",))?;
            writeln!(f, "\n  Column | UTF-8 Character")?;
            writeln!(f, "  -------|----------------")?;

            for (idx, char) in line.chars().enumerate() {
              write!(f, "  {column:<7}| ", column = idx + 1,)?;
              f.writeln_info(format_args!("{char:?}"))?;
            }
          }

          Ok(())
        }
      },
    }
  }
}

impl ErrorFormat for std::io::Error {
  fn fmt(
    &self,
    mut f: ErrorFormatter<'_>,
    _: &str,
    verbose_level: VerboseLevel,
  ) -> std::io::Result<()> {
    match verbose_level {
      VerboseLevel::Quiet => Ok(()),
      _ => f.writeln_error(format_args!("{}", &self)),
    }
  }
}

impl ErrorFormat for smt_ffmpeg::error::AvError {
  fn fmt(&self, mut f: ErrorFormatter<'_>, _: &str, verbose: VerboseLevel) -> std::io::Result<()> {
    match verbose {
      VerboseLevel::Quiet => Ok(()),
      _ => f.writeln_error(format_args!("{}", &self)),
    }
  }
}
