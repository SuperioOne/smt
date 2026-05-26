use clap::Parser;
use smt_common::VerboseLevel;
use std::path::PathBuf;

pub mod demux;
pub mod error;
pub mod split;

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
  /// Root directory for the input file or, FILE path
  #[arg(short, long)]
  pub file_path: Option<PathBuf>,
  /// Output directory for the split tracks
  #[arg(short, long)]
  pub output_dir: Option<PathBuf>,
  /// Enables Vorbis metadata comments from remarks
  #[arg(short, long)]
  pub metadata: bool,
}
