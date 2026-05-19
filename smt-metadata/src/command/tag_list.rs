use crate::error::MetadataError;
use cue_lib::metadata::{Metadata, vorbis::VorbisTag};
use smt_common::{Command, metadata::find_tag_from_str};
use smt_ffmpeg::{format::context::AvInputContext, util::dictionary::AvDictionaryRef};
use std::{
  borrow::Cow,
  collections::BTreeMap,
  io::{BufWriter, stdout},
  path::Path,
};

pub struct CmdTagList<P>
where
  P: AsRef<Path>,
{
  path: P,
  json_output: bool,
}

impl<P> CmdTagList<P>
where
  P: AsRef<Path>,
{
  #[inline]
  pub const fn new(path: P) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      path,
      json_output: false,
    }
  }

  #[inline]
  pub const fn set_json_output(mut self, value: bool) -> Self {
    self.json_output = value;
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

impl<P> Command for CmdTagList<P>
where
  P: AsRef<Path>,
{
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let media = AvInputContext::open_path(self.path)?;
    let metadata = media.metadata();

    if self.json_output {
      display_json(metadata)?;
    } else {
      display_text(metadata);
    }

    Ok(())
  }
}
