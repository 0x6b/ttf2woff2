use std::{env::var, error::Error, path::PathBuf};

use pkg_config::probe_library;

fn main() -> Result<(), Box<dyn Error>> {
    let lib = probe_library("libbrotlienc").expect("Could not find brotli");

    let root = PathBuf::from(var("CARGO_MANIFEST_DIR")?);
    let woff2_root = root.join("woff2");

    let mut build = cc::Build::new();

    build
        .cpp(true)
        .warnings(false)
        .define("__STDC_FORMAT_MACROS", None)
        .define("BUILD_SHARED_LIBS", "OFF")
        .flag("-fno-omit-frame-pointer")
        .flag("-no-canonical-prefixes")
        .flag("-std=c++11")
        .include(woff2_root.join("include"));

    lib.include_paths.iter().for_each(|path| {
        build.include(path);
    });

    #[cfg(target_os = "macos")]
    build.define("OS_MACOSX", None);
    #[cfg(not(target_os = "macos"))]
    build.flag("-fno-tree-vrp");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-lib=brotlienc");
    lib.link_paths.iter().for_each(|path| {
        println!("rustc-link-search=native={}", path.display());
    });

    [
        "font.cc",
        "glyph.cc",
        "normalize.cc",
        "table_tags.cc",
        "transform.cc",
        "woff2_enc.cc",
        "woff2_common.cc",
        "variable_length.cc",
    ]
    .iter()
    .for_each(|file| {
        build.file(woff2_root.join("src").join(file));
    });

    build.compile("woff2");

    cpp_build::Config::new()
        .include(woff2_root.join("include"))
        .build(root.join("src").join("lib.rs"));

    Ok(())
}
