fn main() {
    // println!("cargo:rustc-link-lib=/Users/hokada/develop/src/github.com/ocadaruma/duckdb-jfr-extension/build/debug/src/libduckdb.dylib");
    println!("cargo:rustc-link-lib=duckdb");
    println!("cargo:rustc-link-search=/Users/hokada/develop/src/github.com/ocadaruma/duckdb-jfr-extension/build/debug/src");

    println!("cargo:rustc-link-lib=static=wrapper");
    println!("cargo:rustc-link-search=/Users/hokada/develop/src/github.com/ocadaruma/duckdb-jfr-extension/src");
}
