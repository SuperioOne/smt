use clap::{Parser, Subcommand};
use smt_common::VerboseLevel;
use std::path::PathBuf;

pub mod command;
pub mod error;

pub const TOOL_NAME: &'static str = env!("CARGO_PKG_NAME");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
  /// Cuesheet file path
  #[arg(short, long)]
  pub input: Option<PathBuf>,

  /// Verbosity level
  #[arg(short, long)]
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
  /// Split audio file into tracks via cuesheet
  Split {
    /// Root directory for the input file or, FILE path
    #[arg(short, long)]
    file_path: Option<PathBuf>,
    /// Output directory for the split tracks
    #[arg(short, long)]
    output_dir: Option<PathBuf>,
    /// Enables Vorbis metadata comments from remarks
    #[arg(short, long)]
    metadata: bool,
  },
}
