use clap::Parser;
use smt_common::{Command, input_reader::read_input, output_error::OutputError};
use smt_parse::{
  Args, Commands,
  command::{convert::CmdConvert, test::CmdTest},
};
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
  };

  _ = stderr.close();
  code
}
