use std::{env, path::PathBuf};

fn main() {
  let mut cmake = cmake::Config::new("src/flac");

  cmake
    .define("BUILD_CXXLIBS", "OFF")
    .define("BUILD_DOCS", "OFF")
    .define("BUILD_EXAMPLES", "OFF")
    .define("BUILD_PROGRAMS", "OFF")
    .define("INSTALL_CMAKE_CONFIG_MODULE", "OFF")
    .define("INSTALL_MANPAGES", "OFF")
    .define("INSTALL_PKGCONFIG_MODULES", "OFF")
    .define("WITH_OGG", "OFF");

  let install_dir = cmake.build();
  let libdir = install_dir.join("lib");

  println!("cargo:rustc-link-search=native={}", libdir.display());
  println!("cargo:rustc-link-lib=static=FLAC");

  let bindings = bindgen::Builder::default()
    .header("src/flac/include/FLAC/all.h")
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}
