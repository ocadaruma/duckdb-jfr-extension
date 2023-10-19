#!/bin/bash

set -eu

SCRIPT_DIR=$(cd $(dirname $0); pwd)
BUILD_DIR=$SCRIPT_DIR/build

mkdir -p $BUILD_DIR
cd $SCRIPT_DIR

CARGO_CMD="cargo rustc --release"

case $(uname -s) in
  Darwin)
    cargo rustc --release --target aarch64-apple-darwin
    cargo rustc --release --target x86_64-apple-darwin
    lipo -create -output $BUILD_DIR/libduckdb_jfr_extension_osx-universal.so \
      target/aarch64-apple-darwin/release/libduckdb_jfr_extension.dylib \
      target/x86_64-apple-darwin/release/libduckdb_jfr_extension.dylib
    ;;
  Linux)
    filename=""
    target=""
    case $(uname -m) in
      x86_64)
        filename=libduckdb_jfr_extension_linux-amd64.so
        target="x86_64-unknown-linux-musl"
        ;;
      aarch64)
        filename=libduckdb_jfr_extension_linux-arm64.so
        target="aarch64-unknown-linux-musl"
        ;;
      *)
        echo "Unsupported architecture: $(uname -m)" >&2
        exit 1
        ;;
    esac
    cargo rustc --release --target $target
    cp -f target/$target/release/libduckdb_jfr_extension.so $BUILD_DIR/$filename
    ;;
  *)
    echo "Unsupported OS: $(uname -s)" >&2
    exit 1
    ;;
esac
