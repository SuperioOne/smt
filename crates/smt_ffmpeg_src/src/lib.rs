use std::path::{Path, PathBuf};

pub const SOURCE_DIR: &'static str = env!("FFMPEG_SRC_DIR");

#[derive(Debug)]
pub enum BuildError {
  InvalidOutDir,
  BuildFailure(String),
  IO(std::io::Error),
}

#[derive(Debug)]
pub struct BuildInfo {
  pub lib_dir: PathBuf,
  pub inc_dir: PathBuf,
}

pub struct AVLibBuilder {
  enable_static: bool,
  enable_shared: bool,
  arch: Option<Box<str>>,
  os: Option<Box<str>>,
}

fn copy_recursive<S, D>(src: S, dst: D) -> Result<(), std::io::Error>
where
  S: AsRef<Path>,
  D: AsRef<Path>,
{
  let dir = src.as_ref().read_dir()?;
  let dst = dst.as_ref();

  if !dst.exists() {
    std::fs::create_dir_all(&dst)?;
  }

  for entry in dir.flatten() {
    let metadata = entry.metadata()?;
    let name = entry.file_name();

    if metadata.is_file() {
      std::fs::copy(entry.path(), dst.join(name))?;
    } else if metadata.is_dir() {
      let src_dir = entry.path();
      let dst_dir = dst.join(name);
      copy_recursive(src_dir, &dst_dir)?;
    }
    // Ignoring hypothetical symlink case
  }

  Ok(())
}

impl AVLibBuilder {
  #[inline]
  pub const fn new() -> Self {
    Self {
      enable_static: true,
      enable_shared: false,
      arch: None,
      os: None,
    }
  }

  #[inline]
  pub const fn enable_static(mut self, value: bool) -> Self {
    self.enable_static = value;
    self
  }

  #[inline]
  pub const fn enable_shared(mut self, value: bool) -> Self {
    self.enable_shared = value;
    self
  }

  #[inline]
  pub fn set_os<V>(mut self, value: V) -> Self
  where
    V: AsRef<str>,
  {
    self.os = Some(Box::from(value.as_ref()));
    self
  }

  #[inline]
  pub fn set_arch<V>(mut self, value: V) -> Self
  where
    V: AsRef<str>,
  {
    self.arch = Some(Box::from(value.as_ref()));
    self
  }

  pub fn build<P>(self, path: P) -> Result<BuildInfo, BuildError>
  where
    P: AsRef<Path>,
  {
    let path = path.as_ref();

    if path.is_file() {
      return Err(BuildError::InvalidOutDir);
    }

    let src_dir = path.join("ffmpeg_src");
    copy_recursive(SOURCE_DIR, &src_dir)?;

    let lib_dir = path.join("lib");
    let inc_dir = path.join("include");

    let mut configure = std::process::Command::new(src_dir.join("configure"));

    configure
      .current_dir(&src_dir)
      .arg("--enable-gpl")
      .arg(format!("--prefix={}", path.display()))
      .arg("--disable-programs")
      .arg("--disable-doc")
      .arg("--disable-avdevice")
      .arg("--disable-network")
      .arg("--disable-avfilter")
      .arg("--disable-swscale")
      .arg("--disable-swresample")
      .arg("--disable-lsp")
      .arg("--disable-debug");

    if self.enable_shared {
      configure.arg("--enable-shared");
    }

    if !self.enable_static {
      configure.arg("--disable-static");
    }

    if let Some(arch) = self.arch {
      configure.arg(format!("--arch={arch}"));
    }

    if let Some(os) = self.os {
      configure.arg(format!("--target-os={os}"));
    }

    let config_output = configure.output()?;

    if !config_output.status.success() {
      let build_error = String::from_utf8(config_output.stdout).map_err(|_| {
        BuildError::BuildFailure("./configure script failed without any readable error".to_owned())
      })?;
      return Err(BuildError::BuildFailure(build_error));
    }

    let mut make = std::process::Command::new("make");
    make.current_dir(&src_dir).arg("install");
    let make_output = make.output()?;

    if !make_output.status.success() {
      let build_error = String::from_utf8(make_output.stderr).map_err(|_| {
        BuildError::BuildFailure("make install failed without any readable error.".to_owned())
      })?;
      return Err(BuildError::BuildFailure(build_error));
    }

    Ok(BuildInfo { lib_dir, inc_dir })
  }
}

impl From<std::io::Error> for BuildError {
  #[inline]
  fn from(value: std::io::Error) -> Self {
    Self::IO(value)
  }
}
