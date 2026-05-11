use std::{env, path::PathBuf, process::ExitCode};

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

macro_rules! has_feature {
  ($feat:literal) => {{
    let name = format!(
      "CARGO_FEATURE_{}",
      $feat.trim().to_uppercase().replace("-", "_")
    );
    env::var(name).is_ok_and(|v| v == "1")
  }};
}

fn main() -> ExitCode {
  let options = if has_feature!("vendored_static") {
    LinkOptions {
      src: LibSource::Local,
      kind: LinkageKind::Static,
    }
  } else if has_feature!("vendored_dynamic") {
    LinkOptions {
      src: LibSource::Local,
      kind: LinkageKind::Dynamic,
    }
  } else {
    LinkOptions {
      src: LibSource::System,
      kind: LinkageKind::Dynamic,
    }
  };

  println!("cargo::rustc-link-lib={}=avutil", options.kind);
  println!("cargo::rustc-link-lib={}=avformat", options.kind);
  println!("cargo::rustc-link-lib={}=avcodec", options.kind);

  let out_path = env::var("OUT_DIR")
    .map(|v| PathBuf::from(v).canonicalize().unwrap())
    .unwrap();

  let bindgen = bindgen::Builder::default()
    .header("wrapper.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
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

  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("couldn't write ffmpeg bindings");

  println!("cargo::rerun-if-changed=wrapper.h");
  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=src");

  ExitCode::SUCCESS
}
