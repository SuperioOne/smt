use clap::Parser as _;
use smt_common::{Command, output_error::OutputError};
use smt_ffmpeg::avlib_log_level;
use smt_metadata::command::{
  cover_export::CmdCoverExport, cover_remove::CmdCoverRemove, cover_set::CmdCoverSet,
  tag_append::CmdTagAppend, tag_clear::CmdTagClear, tag_edit::CmdTagEdit, tag_list::CmdTagList,
  tag_remove::CmdTagRemove, tag_set::CmdTagSet,
};
use smt_metadata::{Args, CoverImageCommand, MetadataCliCommand, TagCommand};
use std::process::ExitCode;

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
