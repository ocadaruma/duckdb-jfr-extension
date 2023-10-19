PROJECT_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

.PHONY: duckdb-sources
duckdb-sources:
	$(PROJECT_DIR)/download-duckdb-sources.sh

.PHONY: submodules
submodules:
	git submodule update --init --recursive

.PHONY: loadable-extension-debug
loadable-extension-debug: duckdb-sources
	cargo rustc

.PHONY: loadable-extension-release
loadable-extension-release: duckdb-sources
	cargo rustc --release

.PHONY: wasm-debug
wasm-debug: duckdb-sources submodules
	cargo rustc --target wasm32-unknown-emscripten --no-default-features --crate-type staticlib

.PHONY: wasm-release
wasm-release: duckdb-sources submodules
	cargo rustc --release --target wasm32-unknown-emscripten --no-default-features --crate-type staticlib

.PHONY: fmt
fmt:
	cargo fix --allow-dirty --allow-staged --no-default-features
	cargo fmt
	cargo clippy --no-default-features

.PHONY: test
test:
	cargo test

.PHONY: clean
clean:
	cargo clean
