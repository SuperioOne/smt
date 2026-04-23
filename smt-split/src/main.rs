use self::split::CmdSplit;
use clap::Parser;
use smt_common::{VerboseLevel, input_reader::read_input, output_error::OutputError};
use smt_ffmpeg::{AvLogLevel, avlib_log_level};
use std::{path::PathBuf, process::ExitCode};

mod demux;
mod error;
mod metadata;
mod split;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
  /// Cuesheet file path
  #[arg(short, long)]
  input: Option<PathBuf>,
  /// Verbosity level
  #[arg(short, long)]
  verbose: Option<VerboseLevel>,
  /// Root directory for the input file or, FILE path
  #[arg(short, long)]
  file_path: Option<PathBuf>,
  /// Output directory for the split tracks
  #[arg(short, long)]
  output_dir: Option<PathBuf>,
  /// Enables Vorbis metadata comments from remarks
  #[arg(short, long)]
  metadata: bool,
}

fn main() -> ExitCode {
  let args = Args::parse();
  let mut stderr = OutputError::new();
  let verbosity = args.verbose.unwrap_or_default();
  stderr.set_verbosity(verbosity);

  let cuesheet = match read_input(args.input.as_ref()) {
    Ok(buffer) => buffer,
    Err(err) => {
      _ = stderr.write_error(err);
      return ExitCode::FAILURE;
    }
  };

  let root_dir =
    args
      .file_path
      .or_else(|| match args.input.as_ref().map(|v| v.parent()).flatten() {
        Some(p) => Some(p.to_owned()),
        _ => None,
      });

  let av_verbosity = match verbosity {
    VerboseLevel::Default => AvLogLevel::Error,
    VerboseLevel::Full => AvLogLevel::Info,
    VerboseLevel::Quiet => AvLogLevel::Quiet,
  };

  avlib_log_level(av_verbosity);

  stderr.set_input_buffer(cuesheet.as_str());

  let cmd = CmdSplit::new(cuesheet.as_str())
    .set_vorbis_remarks(args.metadata)
    .set_file_path(root_dir)
    .set_output_dir(args.output_dir);

  let code = match cmd.run() {
    Ok(_) => ExitCode::SUCCESS,
    Err(err) => {
      _ = stderr.write_error(err);
      ExitCode::FAILURE
    }
  };

  stderr.close();
  code
}
