use std::path::PathBuf;

fn main() {
  let ffmpeg_src_dir = PathBuf::from("ffmpeg").canonicalize().unwrap();

  println!(
    "cargo::rustc-env=FFMPEG_SRC_DIR={}",
    ffmpeg_src_dir.display()
  );

  println!("cargo::rerun-if-changed=ffmpeg");
  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=src");
}
