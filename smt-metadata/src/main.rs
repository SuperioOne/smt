use self::command::{
  cover_export::CmdCoverExport, cover_remove::CmdCoverRemove, cover_set::CmdCoverSet,
  tag_append::CmdTagAppend, tag_clear::CmdTagClear, tag_edit::CmdTagEdit, tag_list::CmdTagList,
  tag_remove::CmdTagRemove, tag_set::CmdTagSet,
};
use clap::{Parser, Subcommand};
use cue_lib::metadata::vorbis::VorbisTag;
use smt_common::{Command, VerboseLevel, output_error::OutputError};
use smt_ffmpeg::avlib_log_level;
use std::{path::PathBuf, process::ExitCode};

mod command;
mod context;
mod error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  /// Verbosity level
  #[arg(short, long)]
  verbose: Option<VerboseLevel>,

  /// Input file
  file: PathBuf,

  #[command(subcommand)]
  command: MetadataCliCommand,
}

#[derive(Subcommand, Debug)]
enum MetadataCliCommand {
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
enum CoverImageCommand {
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
enum TagCommand {
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

fn main() -> ExitCode {
  let args = Args::parse();
  let mut stderr = OutputError::new();
  let verbosity = args.verbose.unwrap_or_default();
  stderr.set_verbosity(verbosity);

  avlib_log_level(verbosity.into());

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

  match args.command {
    MetadataCliCommand::Cover { cmd } => match cmd {
      CoverImageCommand::Set { path } => {
        run!(CmdCoverSet::new(args.file, path))
      }
      CoverImageCommand::Remove => {
        run!(CmdCoverRemove::new(args.file))
      }
      CoverImageCommand::Export { output } => {
        let mut cmd = CmdCoverExport::new(args.file);

        if let Some(output_path) = output {
          cmd.set_output_path(output_path);
        }
        run!(cmd)
      }
    },
    MetadataCliCommand::Tag { cmd } => match cmd {
      TagCommand::Clear => {
        run!(CmdTagClear::new(args.file))
      }
      TagCommand::Set { key, values } => {
        run!(CmdTagSet::new(args.file, key, values))
      }
      TagCommand::Append { key, values } => {
        run!(CmdTagAppend::new(args.file, key, values))
      }
      TagCommand::Remove { key } => {
        run!(CmdTagRemove::new(args.file, key))
      }
      TagCommand::List { json } => {
        run!(CmdTagList::new(args.file).set_json_output(json))
      }
      TagCommand::Edit { editor } => {
        let mut cmd = CmdTagEdit::new(args.file);

        if let Some(editor) = editor {
          cmd.set_editor(editor);
        }

        run!(cmd)
      }
    },
  }
}
