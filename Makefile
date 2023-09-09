.PHONY: loadable-extension wasm download submodules start-duckdb lldb fmt test clean

PROJECT_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

export LD_LIBRARY_PATH := $(PROJECT_DIR)/duckdb-downloaded-lib
export DYLD_LIBRARY_PATH := $(PROJECT_DIR)/duckdb-downloaded-lib

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

fmt:
	cargo fmt --all
	cargo clippy --all
	cargo fix --all

test:
	cargo test

clean:
	cargo clean
