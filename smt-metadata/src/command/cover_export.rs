use crate::error::MetadataError;
use smt_common::Command;
use smt_ffmpeg::{
  codec::packet::AvPacket,
  error::AvError,
  ffmpeg::{
    AV_DISPOSITION_ATTACHED_PIC, AVCodecID_AV_CODEC_ID_JPEG2000, AVCodecID_AV_CODEC_ID_JPEGLS,
    AVCodecID_AV_CODEC_ID_JPEGXL, AVCodecID_AV_CODEC_ID_LJPEG, AVCodecID_AV_CODEC_ID_MJPEG,
    AVCodecID_AV_CODEC_ID_MJPEGB, AVCodecID_AV_CODEC_ID_PNG, AVCodecID_AV_CODEC_ID_SMVJPEG,
  },
  format::{
    context::{AvInputContext, AvOutputContext},
    stream::copy_stream_properties,
  },
};
use std::env::current_dir;
use std::path::{Path, PathBuf};

const DEFAULT_EXPORT_NAME: &'static str = "cover";
const EXT_JPG: &'static str = "jpg";
const EXT_PNG: &'static str = "png";

pub struct CmdCoverExport<P>
where
  P: AsRef<Path>,
{
  path: P,
  output: Option<PathBuf>,
}

impl<P> CmdCoverExport<P>
where
  P: AsRef<Path>,
{
  #[inline]
  pub const fn new(path: P) -> Self {
    Self { path, output: None }
  }

  #[inline]
  pub fn set_output_path(&mut self, path: PathBuf) -> &mut Self {
    self.output = Some(path);
    self
  }
}

impl<P> Command for CmdCoverExport<P>
where
  P: AsRef<Path>,
{
  type Error = MetadataError;

  fn run(self) -> Result<(), Self::Error> {
    let mut input = AvInputContext::open_path(self.path)?;
    let cover = input
      .find_stream_by_disposition(AV_DISPOSITION_ATTACHED_PIC)
      .ok_or(MetadataError::NoCoverImage)?;

    #[allow(nonstandard_style)]
    let extension = match unsafe { *cover.codecpar }.codec_id {
      AVCodecID_AV_CODEC_ID_PNG => Ok(EXT_PNG),
      AVCodecID_AV_CODEC_ID_JPEGXL
      | AVCodecID_AV_CODEC_ID_JPEG2000
      | AVCodecID_AV_CODEC_ID_JPEGLS
      | AVCodecID_AV_CODEC_ID_LJPEG
      | AVCodecID_AV_CODEC_ID_MJPEGB
      | AVCodecID_AV_CODEC_ID_SMVJPEG
      | AVCodecID_AV_CODEC_ID_MJPEG => Ok(EXT_JPG),
      _ => Err(MetadataError::UnsupportedImageFormat),
    }?;

    let output_path = match self.output {
      Some(path) => {
        if path.is_dir() {
          path.join(DEFAULT_EXPORT_NAME).with_extension(extension)
        } else {
          path
        }
      }
      None => {
        let current_dir = current_dir()?;
        current_dir
          .join(DEFAULT_EXPORT_NAME)
          .with_extension(extension)
      }
    };

    let mut output = AvOutputContext::open_path(output_path)?;
    let mut image_stream = output.create_stream()?;
    copy_stream_properties(cover, &mut image_stream)?;
    image_stream.disposition = 0;

    let src_index = cover.index;
    let dst_index = image_stream.index;
    let mut packet = AvPacket::try_new()?;
    let mut writer = output.start_writer()?;
    writer.write_header()?;

    loop {
      match input.read_frame(&mut packet) {
        Ok(()) => {
          if packet.stream_index == src_index {
            packet.stream_index = dst_index;
            writer.write_frame(&mut packet)?;
            packet.reset();
          }
        }
        Err(AvError::AvLibError(smt_ffmpeg::error::AvLibError::Eof)) => break,
        Err(err) => return Err(err.into()),
      }
    }

    writer.finish()?;
    Ok(())
  }
}
