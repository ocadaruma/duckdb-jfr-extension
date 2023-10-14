.PHONY: loadable-extension wasm download submodules start-duckdb lldb fmt test clean

PROJECT_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

export LD_LIBRARY_PATH := $(PROJECT_DIR)/duckdb-downloaded-lib
export DYLD_LIBRARY_PATH := $(PROJECT_DIR)/duckdb-downloaded-lib

BUILD_FLAGS=-DEXTENSION_STATIC_BUILD=1 ${OSX_BUILD_UNIVERSAL_FLAG}

download:
	$(PROJECT_DIR)/download-duckdb-lib.sh

submodules:
	git submodule update --init --recursive

loadable-extension: download submodules
	cargo rustc --release --crate-type cdylib

wasm: download submodules
	cargo rustc --release --target wasm32-unknown-emscripten --crate-type staticlib

start-duckdb: loadable-extension
	$(PROJECT_DIR)/duckdb-downloaded-lib/duckdb -unsigned -init .duckdbrc

lldb: loadable-extension
	lldb $(PROJECT_DIR)/duckdb-downloaded-lib/duckdb --local-lldbinit -- -unsigned -init .duckdbrc

static:
	mkdir -p build/release && \
	cd build/release && \
	cmake $(GENERATOR) $(FORCE_COLOR) -DCMAKE_BUILD_TYPE=RelWithDebInfo ${BUILD_FLAGS} ../../duckdb/CMakeLists.txt -DEXTERNAL_EXTENSION_DIRECTORIES=../../duckdb-jfr-extension -B. && \
	cmake --build . --config Release

fmt:
	cargo fix
	cargo fmt
	cargo clippy

test:
	cargo test

clean:
	cargo clean
