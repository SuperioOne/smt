#![allow(unused)]
use std::{env, path::PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
enum LinkageKind {
  Dynamic,
  Static,
}

#[derive(Clone, Copy)]
enum LibSource {
  Local,
  System,
}

impl core::fmt::Display for LinkageKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      LinkageKind::Dynamic => f.write_str("dylib"),
      LinkageKind::Static => f.write_str("static"),
    }
  }
}

#[derive(Clone, Copy)]
struct LinkOptions {
  src: LibSource,
  kind: LinkageKind,
}

fn main() {
  cfg_select! {
    feature = "vendored_static" => {
      let options = LinkOptions {
        src: LibSource::Local,
        kind: LinkageKind::Static,
      };
    },
    feature = "vendored_dynamic" => {
      let options = LinkOptions {
        src: LibSource::Local,
        kind: LinkageKind::Dynamic,
      };
    },
    _ => {
      let options = LinkOptions {
        src: LibSource::System,
        kind: LinkageKind::Dynamic,
      };
    },
  };

  let out_path = env::var("OUT_DIR")
    .map(|v| PathBuf::from(v).canonicalize().unwrap())
    .unwrap();

  let bindgen = bindgen::Builder::default()
    .header("wrapper.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    .impl_debug(true)
    .blocklist_item("FP_INFINITE")
    .blocklist_item("FP_NAN")
    .blocklist_item("FP_ZERO")
    .blocklist_item("FP_SUBNORMAL")
    .blocklist_item("FP_NORMAL");

  let bindgen = match options.src {
    LibSource::Local => {
      let is_shared = options.kind == LinkageKind::Dynamic;
      let artifacts = smt_ffmpeg_src::AVLibBuilder::new()
        .enable_shared(is_shared)
        .enable_static(!is_shared)
        .build(&out_path)
        .unwrap();

      println!("cargo::rustc-link-search={}", artifacts.lib_dir.display());

      bindgen
        .clang_arg("-I")
        .clang_arg(artifacts.inc_dir.to_string_lossy())
    }
    LibSource::System => bindgen,
  };

  let bindings = bindgen.generate().expect("unable to generate bindings");
  let bindings_path = out_path.join("bindings.rs");

  bindings
    .write_to_file(&bindings_path)
    .expect("couldn't write ffmpeg bindings");

  println!(
    "cargo::rustc-env=FFMPEG_BINDINGS_PATH={}",
    bindings_path.display()
  );
  println!("cargo::rustc-link-lib={}=avutil", options.kind);
  println!("cargo::rustc-link-lib={}=avformat", options.kind);
  println!("cargo::rustc-link-lib={}=avcodec", options.kind);
  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=wrapper.h");
  println!("cargo::rerun-if-changed=src");
}
