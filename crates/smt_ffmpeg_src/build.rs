use std::path::PathBuf;

fn main() {
  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=src");

  let ffmpeg_src_dir = PathBuf::from("ffmpeg").canonicalize().unwrap();
  println!(
    "cargo::rustc-env=FFMPEG_SRC_DIR={}",
    ffmpeg_src_dir.display()
  );
}
