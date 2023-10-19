cmake_minimum_required(VERSION 3.10)

register_custom_extension(
    "libduckdb_jfr_extension"
    "${CMAKE_CURRENT_SOURCE_DIR}/../target/wasm32-unknown-emscripten/release/libduckdb_jfr_extension.a")
