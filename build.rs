use build_script::cargo_rerun_if_changed;
use std::path::Path;

fn main() {
    // println!("cargo:rustc-link-lib=/Users/hokada/develop/src/github.com/ocadaruma/duckdb-jfr-extension/build/debug/src/libduckdb.dylib");
    // println!("cargo:rustc-link-lib=duckdb");
    // println!("cargo:rustc-link-search=/Users/hokada/develop/src/github.com/ocadaruma/duckdb-jfr-extension/build/release/src");
    //
    // println!("cargo:rustc-link-lib=static=wrapper");
    // println!("cargo:rustc-link-search=/Users/hokada/develop/src/github.com/ocadaruma/duckdb-jfr-extension/src");

    let duckdb_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("duckdb");

    cargo_rerun_if_changed("src/wrapper.hpp");
    cargo_rerun_if_changed("src/wrapper.cpp");

    cc::Build::new()
        .include(duckdb_root.join("src/include"))
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-std=c++11")
        .opt_level(0) // TODO
        .cpp(true)
        .file("src/wrapper.cpp")
        .compile("wrapper");
}
