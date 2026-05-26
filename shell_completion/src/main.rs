use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use std::{io, str::FromStr};

#[derive(Debug, Clone, Copy)]
struct ToolParseError;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SmtTool {
  Metadata,
  Split,
  Parse,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  shell: Option<Shell>,
  tool: SmtTool,
}

fn main() {
  let args = Args::parse();
  let shell = args.shell.unwrap_or(Shell::Bash);

  let (name, mut command) = match args.tool {
    SmtTool::Metadata => (smt_metadata::TOOL_NAME, smt_metadata::Args::command()),
    SmtTool::Split => (smt_split::TOOL_NAME, smt_split::Args::command()),
    SmtTool::Parse => (smt_parse::TOOL_NAME, smt_parse::Args::command()),
  };

  generate(shell, &mut command, name, &mut io::stdout());
}

impl FromStr for SmtTool {
  type Err = ToolParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    const TOOLS: &'static [(SmtTool, &'static str)] = &[
      (SmtTool::Split, smt_split::TOOL_NAME),
      (SmtTool::Metadata, smt_metadata::TOOL_NAME),
      (SmtTool::Parse, smt_parse::TOOL_NAME),
    ];

    for (variant, name) in TOOLS {
      if s.eq_ignore_ascii_case(name) {
        return Ok(*variant);
      }
    }

    Err(ToolParseError)
  }
}

impl core::fmt::Display for SmtTool {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SmtTool::Metadata => f.write_str(smt_metadata::TOOL_NAME),
      SmtTool::Split => f.write_str(smt_split::TOOL_NAME),
      SmtTool::Parse => f.write_str(smt_parse::TOOL_NAME),
    }
  }
}

impl core::fmt::Display for ToolParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("unknown tool name")
  }
}

impl core::error::Error for ToolParseError {}
