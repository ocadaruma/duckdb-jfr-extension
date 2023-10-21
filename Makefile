PROJECT_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

.PHONY: duckdb-sources
duckdb-sources:
	$(PROJECT_DIR)/download-duckdb-sources.sh

.PHONY: submodules
submodules:
	git submodule update --init --recursive

.PHONY: loadable-extension
loadable-extension: duckdb-sources
	CXXFLAGS="-D_GLIBCXX_USE_CXX11_ABI=0" ./build-extension.sh

.PHONY: wasm
wasm: duckdb-sources submodules
	cargo rustc --release --target wasm32-unknown-emscripten --no-default-features --crate-type staticlib

.PHONY: duckdb-wasm
duckdb-wasm: wasm
	$(MAKE) -C $(PROJECT_DIR)/wasm/duckdb-wasm \
		DUCKDB_SKIP_BUILD_EH=1 DUCKDB_SKIP_BUID_COI=1 \
		CUSTOM_EXTENSION_DIRS=$(PROJECT_DIR)/wasm \
		wasm wasmpack js_release

.PHONY: fmt
fmt:
	cargo fix --allow-dirty --allow-staged --no-default-features
	cargo fmt
	cargo clippy --no-default-features

.PHONY: test
test: loadable-extension
	cargo test
	cd $(PROJECT_DIR)/jdbc && ./gradlew test

.PHONY: clean
clean:
	cargo clean
	cd $(PROJECT_DIR)/jdbc && ./gradlew clean
