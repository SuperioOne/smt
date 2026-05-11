use crate::error::MetadataError;
use smt_common::metadata::MetadataContainer;
use smt_ffmpeg::{
  codec::packet::AvPacket,
  error::AvLibError,
  format::{
    context::{AvInputContext, AvOutputContext},
    stream::copy_stream_properties,
  },
};
use std::{
  ffi::{CStr, OsString},
  fs, io,
  path::{Path, PathBuf},
};

pub struct MetadataEditContext {
  in_context: AvInputContext,
  out_context: AvOutputContext,
  in_path: PathBuf,
  out_tmp_path: PathBuf,
}

impl MetadataEditContext {
  pub fn open(input: PathBuf) -> Result<Self, MetadataError> {
    let out_tmp_path = get_tmp_path(&input)?;
    let in_context = AvInputContext::open_path(&input)?;
    let mut out_context = AvOutputContext::open_path(&out_tmp_path)?;

    for in_stream in in_context.stream_iter() {
      let mut out_stream = out_context.create_stream()?;
      copy_stream_properties(in_stream, &mut out_stream)?;
    }

    Ok(Self {
      out_tmp_path,
      in_path: input,
      in_context,
      out_context,
    })
  }

  pub fn metadata_mut(&mut self) -> MetadataContainer<'_> {
    MetadataContainer::from_output_context(&mut self.out_context)
  }

  pub fn copy_all_metadata(&mut self) -> Result<(), MetadataError> {
    let src_metadata = self.in_context.metadata();
    let mut out_metadata = self.out_context.metadata_mut();
    out_metadata.try_copy_from(&src_metadata)?;
    Ok(())
  }

  pub fn copy_metadata_by_filter<F>(&mut self, predicate: F) -> Result<(), MetadataError>
  where
    F: Fn(&CStr, &CStr) -> bool,
  {
    let src_metadata = self.in_context.metadata();
    let mut out_metadata = self.out_context.metadata_mut();

    for (key, val) in src_metadata.iter() {
      if predicate(key, val) {
        println!("what {:?} {:?}", key, val);
        out_metadata.push_cstr(key, val)?;
      }
    }

    Ok(())
  }

  pub fn commit(self) -> Result<(), MetadataError> {
    let Self {
      out_context,
      in_context,
      in_path,
      out_tmp_path,
    } = self;

    match write_all_packets(in_context, out_context) {
      Ok(()) => match out_tmp_path.try_exists()? {
        true => {
          fs::rename(out_tmp_path, in_path)?;
          Ok(())
        }
        false => Err(
          io::Error::new(
            io::ErrorKind::NotFound,
            "temporary output file no longer exists",
          )
          .into(),
        ),
      },
      Err(err) => {
        _ = fs::remove_file(out_tmp_path);
        Err(err)
      }
    }
  }

  pub fn discard(self) -> Result<(), MetadataError> {
    fs::remove_file(self.out_tmp_path)?;
    Ok(())
  }
}

#[inline]
fn write_all_packets(src: AvInputContext, dst: AvOutputContext) -> Result<(), MetadataError> {
  let mut packet = AvPacket::try_new()?;
  let mut writer = dst.start_writer()?;
  writer.write_header()?;

  loop {
    match src.read_frame(&mut packet) {
      Ok(()) => writer.write_frame(&mut packet)?,
      Err(smt_ffmpeg::error::AvError::AvLibError(AvLibError::Eof)) => {
        break;
      }
      Err(err) => return Err(err.into()),
    }
  }

  writer.finish()?;
  Ok(())
}

#[inline]
fn get_tmp_path<P>(path: P) -> Result<PathBuf, io::Error>
where
  P: AsRef<Path>,
{
  const PREFIX: &str = ".tmp_";
  let mut output: PathBuf = path.as_ref().to_path_buf();

  match output.file_name() {
    Some(name) => {
      let mut target = OsString::with_capacity(name.len() + PREFIX.len());
      target.push(PREFIX);
      target.push(name);

      output.set_file_name(target);

      Ok(output)
    }
    None => Err(io::ErrorKind::NotFound.into()),
  }
}
