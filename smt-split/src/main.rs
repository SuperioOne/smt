use clap::Parser as _;
use smt_common::{input_reader::read_input, output_error::OutputError};
use smt_ffmpeg::avlib_log_level;
use smt_split::{Args, split::CmdSplit};
use std::process::ExitCode;

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

  avlib_log_level(verbosity.into());

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
