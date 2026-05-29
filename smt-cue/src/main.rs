use clap::Parser as _;
use smt_common::{Command, input_reader::read_input, output_error::OutputError};
use smt_cue::{
  Args, Commands,
  command::{convert::CmdConvert, split::CmdSplit, test::CmdTest},
};
use smt_ffmpeg::avlib_log_level;
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
  stderr.set_input_buffer(cuesheet.as_str());

  macro_rules! run {
    ($cmd:expr) => {
      match $cmd.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
          _ = stderr.write_error(err);
          ExitCode::FAILURE
        }
      }
    };
  }

  let code = match args.command {
    Commands::Test => {
      let cmd = CmdTest::new(cuesheet.as_str());
      run!(cmd)
    }
    Commands::ConvertJson {
      output_file,
      metadata,
      pretty_print,
    } => {
      let cmd = CmdConvert::new(cuesheet.as_str())
        .set_vorbis_remarks(metadata)
        .set_output_file(output_file)
        .set_pretty_print(pretty_print);

      run!(cmd)
    }
    Commands::Split {
      file_path,
      output_dir,
      metadata,
    } => {
      avlib_log_level(verbosity.into());

      let root_dir =
        file_path.or_else(|| match args.input.as_ref().map(|v| v.parent()).flatten() {
          Some(p) => Some(p.to_owned()),
          _ => None,
        });

      let cmd = CmdSplit::new(cuesheet.as_str())
        .set_vorbis_remarks(metadata)
        .set_file_path(root_dir)
        .set_output_dir(output_dir);

      run!(cmd)
    }
  };

  stderr.close();
  code
}
