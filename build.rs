use build_script::cargo_rerun_if_changed;
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let duckdb_sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("duckdb-sources");
    let duckdb_include = duckdb_sources.join("duckdb/src/include");
    let bridge_header = "src/bridge.hpp";

    let mut builder = bindgen::builder()
        .header(bridge_header)
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

    let mut cpp_files = HashSet::new();
    let mut include_dirs = HashSet::new();

    include_dirs.insert(duckdb_include.clone());

    let bridge_sources = ["src/bridge.cpp", "src/bridge_table_function.cpp"];
    cpp_files.extend(bridge_sources.iter().map(PathBuf::from));

    #[cfg(feature = "build-duckdb")]
    build_duckdb::configure(&duckdb_sources, &mut cpp_files, &mut include_dirs);

    cargo_rerun_if_changed(&duckdb_include);
    cargo_rerun_if_changed(bridge_header);
    for source in bridge_sources {
        cargo_rerun_if_changed(source);
    }

    // We use same flags as in libduckdb-sys basically
    cc::Build::new()
        .includes(include_dirs)
        .files(cpp_files)
        .flag_if_supported("-std=c++11")
        .flag_if_supported("-stdlib=libc++")
        // .flag_if_supported("-stdlib=libstdc++") // Except this. If we specify this, em++ spits an error
        .flag_if_supported("/bigobj")
        .flag_if_supported("-w")
        // https://discord.com/channels/909674491309850675/921100573732909107/1110164344525832192
        .define("NDEBUG", None)
        .warnings(false)
        .cpp(true)
        .compile("duckdb-cc");
}

#[cfg(feature = "build-duckdb")]
mod build_duckdb {
    use build_script::cargo_rerun_if_changed;
    use std::collections::{HashMap, HashSet};
    use std::fs::File;
    use std::path::PathBuf;

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

    pub fn configure(
        duckdb_sources: &PathBuf,
        cpp_files: &mut HashSet<PathBuf>,
        include_dirs: &mut HashSet<PathBuf>,
    ) {
        let manifest_file =
            File::open(duckdb_sources.join("duckdb/manifest.json")).expect("manifest file");
        let manifest: Manifest =
            serde_json::from_reader(manifest_file).expect("reading manifest file");

        cpp_files.extend(
            manifest
                .base
                .cpp_files
                .iter()
                .map(|f| duckdb_sources.join(f)),
        );
        include_dirs.extend(
            manifest
                .base
                .include_dirs
                .iter()
                .map(|d| duckdb_sources.join("duckdb").join(d)),
        );

        cargo_rerun_if_changed(duckdb_sources);
    }
}
