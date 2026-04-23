use self::command::{convert::CmdConvert, test::CmdTest};
use clap::{Parser, Subcommand};
use smt_common::{Command, VerboseLevel, input_reader::read_input, output_error::OutputError};
use std::{path::PathBuf, process::ExitCode};

mod command;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
  /// Cuesheet file path
  #[arg(short, long)]
  pub input: Option<PathBuf>,

  /// Verbosity level
  #[arg(long)]
  pub verbose: Option<VerboseLevel>,

  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
  /// Verifies input cuesheet syntax
  Test,
  /// Parses cuesheet and serializes data as structured JSON string
  ConvertJson {
    #[arg(short, long)]
    output_file: Option<PathBuf>,
    /// Enables Vorbis metadata comments from remarks
    #[arg(short, long)]
    metadata: bool,
    /// Formats JSON output
    #[arg(short, long)]
    pretty_print: bool,
  },
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
