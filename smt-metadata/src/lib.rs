use clap::{Parser, Subcommand};
use cue_lib::metadata::vorbis::VorbisTag;
use smt_common::VerboseLevel;
use std::path::PathBuf;

pub mod command;
pub mod context;
pub mod error;

pub const TOOL_NAME: &'static str = env!("CARGO_PKG_NAME");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
  /// Verbosity level
  #[arg(short, long)]
  pub verbose: Option<VerboseLevel>,

  /// Input file
  pub file: PathBuf,

  #[command(subcommand)]
  pub command: MetadataCliCommand,
}

#[derive(Subcommand, Debug)]
pub enum MetadataCliCommand {
  /// Cover image commands
  Cover {
    #[command(subcommand)]
    cmd: CoverImageCommand,
  },
  /// Metadata tag commands
  Tag {
    #[command(subcommand)]
    cmd: TagCommand,
  },
}

#[derive(Subcommand, Debug)]
pub enum CoverImageCommand {
  /// Set new cover image
  Set {
    /// Image file path
    path: PathBuf,
  },
  /// Remove cover image
  Remove,
  /// Extract cover image binary data
  Export {
    #[arg(long, short)]
    /// Target output path
    output: Option<PathBuf>,
  },
}

#[derive(Subcommand, Debug)]
pub enum TagCommand {
  /// Clear all metadata tags
  Clear,
  /// Set metadata values by overriding existing values.
  Set {
    #[arg(required = true)]
    key: VorbisTag,

    #[arg(required = true)]
    values: Vec<Box<str>>,
  },
  /// Append values to metadata tag without overriding existing values.
  Append {
    #[arg(required = true)]
    key: VorbisTag,

    #[arg(required = true)]
    values: Vec<Box<str>>,
  },
  /// Remove metadata tag.
  Remove { key: VorbisTag },
  /// List metadata
  List {
    /// Show metadata list in json format
    #[arg(long)]
    json: bool,
  },
  /// Edit metadata in interactive mode
  Edit {
    /// Set text editor
    #[arg(long)]
    editor: Option<PathBuf>,
  },
}
