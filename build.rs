use std::collections::{HashMap, HashSet};
use build_script::{
    cargo_rerun_if_changed, cargo_rerun_if_env_changed, cargo_rustc_link_lib,
    cargo_rustc_link_search,
};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let duckdb_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("duckdb-wasm/submodules/duckdb");
    let duckdb_include = duckdb_root.join("src/include");
    let sources = [
        "src/bridge.cpp",
        "src/bridge_scalar_function.cpp",
        "src/bridge_table_function.cpp",
    ];

    cargo_rerun_if_changed("src/bridge.hpp");
    for source in sources {
        cargo_rerun_if_changed(source);
    }
    cargo_rerun_if_changed(duckdb_root);
    cargo_rerun_if_env_changed("LD_LIBRARY_PATH");

    // cargo_rustc_link_lib("duckdb");
    // if let Ok(ld_library_path) = env::var("LD_LIBRARY_PATH") {
    //     cargo_rustc_link_search(ld_library_path);
    // }

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

    let manifest_file = std::fs::File::open("duckdb/manifest.json").expect("manifest file");
    let manifest: Manifest = serde_json::from_reader(manifest_file).expect("reading manifest file");

    let mut cpp_files = HashSet::new();
    let mut include_dirs = HashSet::new();

    cpp_files.extend(manifest.base.cpp_files.clone());
    // otherwise clippy will remove the clone here...
    // https://github.com/rust-lang/rust-clippy/issues/9011
    #[allow(clippy::all)]
    include_dirs.extend(manifest.base.include_dirs.clone());

    let mut cfg = cc::Build::new();
    cfg.include("duckdb");
    cfg.includes(include_dirs.iter().map(|x| format!("{}/{}", "duckdb", x)));
    for f in cpp_files {
        cfg.file(f);
    }

    cfg
        .include(duckdb_include)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-std=c++11")
        .flag_if_supported("-stdlib=libc++")
        .flag_if_supported("-stdlib=libstdc++")
        .flag_if_supported("/bigobj")
        .flag_if_supported("-w")
        // .flag_if_supported("-frtti")
        // .flag_if_supported("-fvisibility=default")
        // https://discord.com/channels/909674491309850675/921100573732909107/1110164344525832192
        .define("NDEBUG", None)
        .warnings(false)
        .cpp(true)
        .files(sources)
        .compile("bridge");
}

#[derive(serde::Deserialize)]
struct Sources {
    cpp_files: HashSet<String>,
    include_dirs: HashSet<String>,
}

#[derive(serde::Deserialize)]
struct Manifest {
    base: Sources,

    #[allow(unused)]
    extensions: HashMap<String, Sources>,
}
