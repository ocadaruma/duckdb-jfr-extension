use build_script::{cargo_rerun_if_changed, cargo_rustc_link_lib};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let duckdb_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("duckdb");
    let duckdb_include = duckdb_root.join("src/include");

    cargo_rerun_if_changed("src/bridge.hpp");
    cargo_rerun_if_changed("src/bridge.cpp");

    let mut builder = bindgen::builder()
        .header("src/bridge.hpp")
        .clang_arg(format!("-I{}", duckdb_include.display()))
        .clang_arg("-fvisibility=default")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));
    if env::var("TARGET").unwrap() == "wasm32-unknown-emscripten" {
        builder = builder.clang_arg(format!(
            "-I{}/upstream/emscripten/cache/sysroot/include",
            env::var("EMSDK").unwrap()
        ));
    }
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindgen.rs");
    builder
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .include(duckdb_include)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-std=c++11")
        .cpp(true)
        .file("src/bridge.cpp")
        .compile("bridge");
}
