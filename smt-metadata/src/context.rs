use crate::error::MetadataError;
use smt_common::metadata::MetadataContainer;
use smt_ffmpeg::{
  codec::packet::AvPacket,
  error::{AvError, AvLibError},
  ffmpeg::{
    AV_DISPOSITION_ATTACHED_PIC, AVCodecID_AV_CODEC_ID_JPEG2000, AVCodecID_AV_CODEC_ID_JPEGLS,
    AVCodecID_AV_CODEC_ID_JPEGXL, AVCodecID_AV_CODEC_ID_LJPEG, AVCodecID_AV_CODEC_ID_MJPEG,
    AVCodecID_AV_CODEC_ID_MJPEGB, AVCodecID_AV_CODEC_ID_PNG, AVCodecID_AV_CODEC_ID_SMVJPEG,
  },
  format::{
    context::{AvInputContext, AvOutputContext, AvOutputWriter},
    stream::{StreamType, copy_stream_properties},
  },
  util::dictionary::{AvDictionaryMut, AvDictionaryRef},
};
use std::{
  ffi::OsString,
  fs, io,
  path::{Path, PathBuf},
};

pub struct MetadataEditContext<P> {
  in_context: AvInputContext,
  out_context: AvOutputContext,
  in_path: P,
  out_tmp_path: PathBuf,
  cover_src: CoverSource,
}

pub enum CoverSource {
  UseInputCover,
  External(AvInputContext),
  NoCover,
}

#[derive(Copy, Clone)]
struct StreamIndexMap {
  src: i32,
  dst: i32,
}

impl<P> MetadataEditContext<P>
where
  P: AsRef<Path>,
{
  pub fn open(input: P) -> Result<Self, MetadataError> {
    let out_tmp_path = get_tmp_path(&input)?;
    let in_context = AvInputContext::open_path(&input)?;
    let out_context = AvOutputContext::open_path(&out_tmp_path)?;

    Ok(Self {
      out_tmp_path,
      in_path: input,
      in_context,
      out_context,
      cover_src: CoverSource::UseInputCover,
    })
  }

  #[inline]
  pub fn set_cover_source(&mut self, src: CoverSource) {
    self.cover_src = src;
  }

  #[inline]
  pub fn file_path(&self) -> &Path {
    self.in_path.as_ref()
  }

  #[inline]
  pub fn metadata(&self) -> AvDictionaryRef<'_> {
    self.in_context.metadata()
  }

  #[inline]
  pub fn metadata_mut(&mut self) -> MetadataContainer<'_> {
    MetadataContainer::new(&mut self.out_context)
  }

  pub fn copy_all_metadata(&mut self) -> Result<(), MetadataError> {
    let src = self.in_context.metadata();
    let mut dst = self.out_context.metadata_mut();

    for (key, value) in src.iter() {
      dst.push(key, value)?;
    }

    Ok(())
  }

  pub fn copy_metadata_by_filter<F>(&mut self, predicate: F) -> Result<(), MetadataError>
  where
    F: Fn(&str, &str) -> bool,
  {
    let src = self.in_context.metadata();
    let mut dst = self.out_context.metadata_mut();

    for (key, value) in src.iter() {
      if predicate(key, value.as_ref()) {
        dst.push(key, value)?;
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
      cover_src,
    } = self;

    match copy_packets(in_context, out_context, cover_src) {
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
    _ = fs::remove_file(self.out_tmp_path);
    Ok(())
  }
}

fn copy_packets(
  mut src: AvInputContext,
  mut dst: AvOutputContext,
  cover_src: CoverSource,
) -> Result<(), MetadataError> {
  let mut mappings = Vec::with_capacity(src.nb_streams as usize);
  let skip_cover = match &cover_src {
    CoverSource::UseInputCover => false,
    _ => true,
  };

  for stream in src.stream_iter() {
    if (stream.disposition as u32 & AV_DISPOSITION_ATTACHED_PIC) == AV_DISPOSITION_ATTACHED_PIC
      && skip_cover
    {
      continue;
    } else {
      let mut dst_stream = dst.create_stream()?;
      copy_stream_properties(stream, &mut dst_stream)?;

      mappings.push(StreamIndexMap {
        src: stream.index,
        dst: dst_stream.index,
      });
    }
  }

  match cover_src {
    CoverSource::External(mut av_context) => {
      let img_stream = av_context
        .find_best_stream(StreamType::Video)
        .ok_or(MetadataError::UnsupportedImageFormat)?;

      #[allow(nonstandard_style)]
      match unsafe { *img_stream.codecpar }.codec_id {
        AVCodecID_AV_CODEC_ID_PNG
        | AVCodecID_AV_CODEC_ID_JPEGXL
        | AVCodecID_AV_CODEC_ID_JPEG2000
        | AVCodecID_AV_CODEC_ID_JPEGLS
        | AVCodecID_AV_CODEC_ID_LJPEG
        | AVCodecID_AV_CODEC_ID_MJPEGB
        | AVCodecID_AV_CODEC_ID_SMVJPEG
        | AVCodecID_AV_CODEC_ID_MJPEG => Ok(()),
        _ => Err(MetadataError::UnsupportedImageFormat),
      }?;

      let cover_src_idx = img_stream.index;
      let cover_dst_idx = {
        let mut dst_cover = dst.create_stream()?;
        copy_stream_properties(img_stream, &mut dst_cover)?;
        dst_cover.disposition = AV_DISPOSITION_ATTACHED_PIC as i32;

        let mut metadata = AvDictionaryMut::from_ptr_ref(&mut dst_cover.metadata);
        metadata.push("comment", "Cover (front)")?;

        dst_cover.index
      };

      let mut writer = dst.start_writer()?;
      writer.write_header()?;
      write_packets_by(&mut src, &mut writer, |pkt| {
        for StreamIndexMap { src, dst } in mappings.iter() {
          if pkt.stream_index == *src {
            return Some(*dst);
          }
        }

        None
      })?;

      write_packets_by(&mut av_context, &mut writer, |pkt| {
        if pkt.stream_index == cover_src_idx {
          Some(cover_dst_idx)
        } else {
          None
        }
      })?;

      writer.finish()?;
      Ok(())
    }
    _ => {
      let mut writer = dst.start_writer()?;
      writer.write_header()?;
      write_packets_by(&mut src, &mut writer, |pkt| {
        for StreamIndexMap { src, dst } in mappings.iter() {
          if pkt.stream_index == *src {
            return Some(*dst);
          }
        }

        None
      })?;

      writer.finish()?;
      Ok(())
    }
  }
}

fn write_packets_by<F>(
  src: &mut AvInputContext,
  writer: &mut AvOutputWriter,
  predicate: F,
) -> Result<(), MetadataError>
where
  F: Fn(&AvPacket) -> Option<i32>,
{
  let mut packet = AvPacket::try_new()?;
  loop {
    match src.read_frame(&mut packet) {
      Ok(()) => {
        if let Some(index) = predicate(&packet) {
          packet.stream_index = index;
          writer.write_frame(&mut packet)?;
        }

        packet.reset();
      }
      Err(AvError::AvLibError(AvLibError::Eof)) => return Ok(()),
      Err(err) => return Err(err.into()),
    }
  }
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

impl From<AvInputContext> for CoverSource {
  #[inline]
  fn from(value: AvInputContext) -> Self {
    Self::External(value)
  }
}
