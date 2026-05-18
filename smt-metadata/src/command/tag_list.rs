use crate::error::MetadataError;
use cue_lib::metadata::{Metadata, vorbis::VorbisTag};
use smt_common::{Command, metadata::find_tag_from_str};
use smt_ffmpeg::{format::context::AvInputContext, util::dictionary::AvDictionaryRef};
use std::{
  borrow::Cow,
  collections::BTreeMap,
  io::{BufWriter, stdout},
  path::{Path, PathBuf},
};

pub struct CmdTagList {
  path: PathBuf,
  json_encoding: bool,
}

impl CmdTagList {
  pub fn new<P>(path: P) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      path: path.as_ref().to_path_buf(),
      json_encoding: false,
    }
  }

  #[inline]
  pub const fn use_json_encoding(mut self, value: bool) -> Self {
    self.json_encoding = value;
    self
  }
}

fn display_text(metadata: AvDictionaryRef<'_>) {
  for (key, value) in metadata.iter() {
    let tag = match find_tag_from_str(key).and_then(|v| VorbisTag::try_from(v).ok()) {
      Some(tag) => tag.as_str(),
      _ => key,
    };

    println!("{}:{}", tag, value);
  }
}

fn display_json(metadata: AvDictionaryRef<'_>) -> Result<(), std::io::Error> {
  let mut obj: BTreeMap<&str, Vec<Cow<'_, str>>> = BTreeMap::new();

  for (key, value) in metadata.iter() {
    let tag = match find_tag_from_str(key).and_then(|v| VorbisTag::try_from(v).ok()) {
      Some(tag) => tag.as_str(),
      _ => key,
    };

    let entry = obj.entry(tag).or_default();
    entry.push(value);
  }

  let mut writer = BufWriter::new(stdout().lock());
  serde_json::to_writer(&mut writer, &obj)?;

  Ok(())
}

impl Command for CmdTagList {
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let media = AvInputContext::open_path(self.path)?;
    let metadata = media.metadata();

    if self.json_encoding {
      display_json(metadata)?;
    } else {
      display_text(metadata);
    }

    Ok(())
  }
}
