use crate::{
  context::MetadataEditContext,
  error::{EditErrorKind, MetadataError},
};
use cue_lib::metadata::{Metadata as _, vorbis::VorbisTag};
use smt_common::{Command, metadata::find_tag_from_str};
use std::{
  env,
  fs::{self, File, OpenOptions},
  io::{self, BufRead, BufReader, BufWriter, IsTerminal, Read, Seek, Write as _},
  path::{Path, PathBuf},
  process,
  str::FromStr,
  time::SystemTime,
};

const DEFAULT_EDITOR: &'static str = "vi";
const EDITMSG_COMMENT: char = '#';
const EDITMSG_DELIMITER: char = ':';
const EDITMSG_FILE_NAME: &'static str = "EDITMSG";
const ENV_EDITOR: &'static str = "EDITOR";
const ENV_SYSTEM_PATH: &'static str = "PATH";

pub struct CmdTagEdit<P>
where
  P: AsRef<Path>,
{
  path: P,
  editor: PathBuf,
}

struct EditMessage {
  path: PathBuf,
  fd: File,
  io_time: SystemTime,
}

struct EditMessageReader<T>
where
  T: Read,
{
  reader: BufReader<T>,
  line_buf: String,
}

impl<P> CmdTagEdit<P>
where
  P: AsRef<Path>,
{
  pub fn new(path: P) -> Self {
    let editor = env::var(ENV_EDITOR).map_or_else(|_| DEFAULT_EDITOR.into(), |v| PathBuf::from(v));
    Self { path, editor }
  }

  #[inline]
  pub fn set_editor<E>(&mut self, editor: E) -> &mut Self
  where
    E: AsRef<Path>,
  {
    self.editor = editor.as_ref().to_path_buf();
    self
  }
}

impl<P> Command for CmdTagEdit<P>
where
  P: AsRef<Path>,
{
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let mut context = MetadataEditContext::open(self.path.as_ref())?;

    if io::stdin().is_terminal() {
      let editor = find_editor(self.editor)?;
      let editmsg = EditMessage::open(&context)?;
      let mut metadata = context.metadata_mut();

      match process::Command::new(editor).arg(&editmsg.path).status() {
        Ok(_) => {
          if let Ok(true) = editmsg.is_modified() {
            let mut reader = editmsg.reader();

            while let Some((tag, value)) = reader.next()? {
              _ = metadata.push(tag, value);
            }

            context.commit()?;
            Ok(())
          } else {
            Err(EditErrorKind::EditDiscarded.into())
          }
        }
        Err(err) => Err(err.into()),
      }
    } else {
      let mut input = String::new();
      io::stdin().read_to_string(&mut input)?;
      let mut reader = EditMessageReader {
        reader: BufReader::new(input.as_bytes()),
        line_buf: String::new(),
      };
      let mut metadata = context.metadata_mut();

      while let Some((tag, value)) = reader.next()? {
        _ = metadata.push(tag, value);
      }

      context.commit()?;
      Ok(())
    }
  }
}

fn find_editor<P>(editor: P) -> Result<PathBuf, MetadataError>
where
  P: AsRef<Path>,
{
  let path = editor.as_ref();

  if path.is_absolute() {
    if path.is_file() {
      Ok(path.to_path_buf())
    } else {
      Err(EditErrorKind::EditorNotAvailable(path.to_owned()).into())
    }
  } else {
    if path.is_file() {
      let absolute = path.canonicalize()?;
      Ok(absolute)
    } else {
      let system_paths = env::var(ENV_SYSTEM_PATH)
        .map_err(|_| EditErrorKind::EditorNotAvailable(path.to_owned()))?;

      for search_dir in system_paths.split(':').map(|v| Path::new(v.trim())) {
        let target = search_dir.join(path);

        if target.is_file() {
          return Ok(target);
        }
      }

      Err(EditErrorKind::EditorNotAvailable(path.to_owned()).into())
    }
  }
}

impl EditMessage {
  pub fn open<P>(context: &MetadataEditContext<P>) -> Result<Self, io::Error>
  where
    P: AsRef<Path>,
  {
    let tmp_dir = env::temp_dir();
    let mut tmp_file = tmp_dir.join(EDITMSG_FILE_NAME);

    if tmp_file.exists() {
      tmp_file = Self::find_alternative(tmp_file)?;
    }

    let mut fd = OpenOptions::new()
      .create(true)
      .read(true)
      .write(true)
      .truncate(true)
      .open(&tmp_file)?;

    {
      let mut writer = BufWriter::new(&mut fd);

      writeln!(writer, "# Target file: {}", context.file_path().display())?;
      writeln!(
        writer,
        "# Edit metadata tags and their values with <TAG>{}VALUE> format.",
        EDITMSG_DELIMITER
      )?;
      writeln!(writer, "")?; // Intentional blank line

      for (key, value) in context.metadata().iter() {
        let tag = match find_tag_from_str(key).and_then(|v| VorbisTag::try_from(v).ok()) {
          Some(tag) => tag.as_str(),
          _ => key,
        };

        writeln!(writer, "{:<20} {} {}", tag, EDITMSG_DELIMITER, value)?;
      }

      writer.flush()?;
    }

    fd.seek(io::SeekFrom::Start(0))?;
    let io_time = fd.metadata()?.modified()?;

    Ok(Self {
      path: tmp_file,
      fd,
      io_time,
    })
  }

  fn find_alternative(mut path: PathBuf) -> Result<PathBuf, io::Error> {
    for i in 0..256 {
      if path.set_extension(i.to_string()) && !path.exists() {
        return Ok(path);
      }
    }

    Err(io::ErrorKind::TimedOut.into())
  }

  fn is_modified(&self) -> Result<bool, io::Error> {
    let modified = self.fd.metadata()?.modified()?;
    Ok(modified > self.io_time)
  }

  pub fn reader(&self) -> EditMessageReader<&File> {
    EditMessageReader {
      reader: BufReader::new(&self.fd),
      line_buf: String::new(),
    }
  }
}

impl<T> EditMessageReader<T>
where
  T: Read,
{
  pub fn next(&mut self) -> Result<Option<(VorbisTag, &str)>, MetadataError> {
    loop {
      self.line_buf.clear();
      match self.reader.read_line(&mut self.line_buf) {
        Ok(0) => return Ok(None),
        Ok(_) => {
          let line = self.line_buf.trim();

          if line.is_empty() || line.starts_with(EDITMSG_COMMENT) {
            continue;
          }

          match self.line_buf.trim().split_once(EDITMSG_DELIMITER) {
            Some((tag_str, value)) => {
              let tag = VorbisTag::from_str(tag_str.trim())
                .map_err(|_| EditErrorKind::InvalidTagName(tag_str.into()))?;

              return Ok(Some((tag, value.trim())));
            }
            None => {
              return Err(EditErrorKind::InvalidEditMessage.into());
            }
          }
        }
        Err(err) => {
          if err.kind() == io::ErrorKind::WouldBlock {
            continue;
          } else {
            return Err(err.into());
          }
        }
      }
    }
  }
}

impl Drop for EditMessage {
  fn drop(&mut self) {
    _ = fs::remove_file(&self.path);
  }
}
