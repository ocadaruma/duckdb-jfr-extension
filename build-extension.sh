#!/bin/bash

set -eu

SCRIPT_DIR=$(cd $(dirname $0); pwd)
BUILD_DIR=$SCRIPT_DIR/build
FILENAME=libduckdb_jfr_extension.so

cd $SCRIPT_DIR

case $(uname -s) in
  Darwin)
    mkdir -p $BUILD_DIR/osx-universal
    cargo rustc --release --target aarch64-apple-darwin
    cargo rustc --release --target x86_64-apple-darwin
    lipo -create -output $BUILD_DIR/osx-universal/$FILENAME \
      target/aarch64-apple-darwin/release/libduckdb_jfr_extension.dylib \
      target/x86_64-apple-darwin/release/libduckdb_jfr_extension.dylib
    ;;
  Linux)
    out_dir=""
    target=""
    case $(uname -m) in
      x86_64)
        out_dir="$BUILD_DIR/linux-amd64"
        mkdir -p $out_dir
        target="x86_64-unknown-linux-musl"
        ;;
      aarch64)
        out_dir="$BUILD_DIR/linux-arm64"
        mkdir -p $out_dir
        target="aarch64-unknown-linux-musl"
        ;;
      *)
        echo "Unsupported architecture: $(uname -m)" >&2
        exit 1
        ;;
    esac
    cargo rustc --release --target $target
    cp -f target/$target/release/libduckdb_jfr_extension.so $out_dir/$filename
    ;;
  *)
    echo "Unsupported OS: $(uname -s)" >&2
    exit 1
    ;;
esac
