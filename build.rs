use build_script::{
    cargo_rerun_if_changed, cargo_rerun_if_env_changed, cargo_rustc_link_lib,
    cargo_rustc_link_search,
};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let duckdb_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("duckdb-wasm/submodules/duckdb");
    let duckdb_include = duckdb_root.join("src/include");

    cargo_rerun_if_changed("src/bridge.hpp");
    cargo_rerun_if_changed("src/bridge.cpp");
    cargo_rerun_if_changed(duckdb_root);
    cargo_rerun_if_env_changed("LD_LIBRARY_PATH");

    cargo_rustc_link_lib("duckdb");
    if let Ok(ld_library_path) = env::var("LD_LIBRARY_PATH") {
        cargo_rustc_link_search(ld_library_path);
    }

    let mut builder = bindgen::builder()
        .header("src/bridge.hpp")
        .clang_arg(format!("-I{}", duckdb_include.display()))
        // https://github.com/rust-lang/rust-bindgen/issues/751#issuecomment-496891269
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
