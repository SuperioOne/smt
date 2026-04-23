use std::{env, path::PathBuf};

fn main() {
  println!("cargo:rustc-link-lib=avutil");
  println!("cargo:rustc-link-lib=avformat");
  println!("cargo:rustc-link-lib=avcodec");

  let bindings = bindgen::Builder::default()
    .header("wrapper.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .blocklist_item("FP_INFINITE")
    .blocklist_item("FP_NAN")
    .blocklist_item("FP_ZERO")
    .blocklist_item("FP_SUBNORMAL")
    .blocklist_item("FP_NORMAL")
    .generate()
    .expect("unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("couldn't write ffmpeg bindings");
}
