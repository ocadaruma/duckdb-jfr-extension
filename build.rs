use build_script::cargo_rerun_if_changed;
use std::path::Path;

fn main() {
    let duckdb_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("duckdb");

    cargo_rerun_if_changed("src/bridge.hpp");
    cargo_rerun_if_changed("src/bridge.cpp");

    cc::Build::new()
        .include(duckdb_root.join("src/include"))
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-std=c++11")
        .cpp(true)
        .file("src/bridge.cpp")
        .compile("bridge");
}
