#!/bin/bash

set -eu

SCRIPT_DIR=$(cd $(dirname $0); pwd)

mkdir -p $SCRIPT_DIR/duckdb-downloaded-lib
cd $SCRIPT_DIR/duckdb-downloaded-lib

download_url=""
lib_dir_suffix=""
case $(uname -s) in
  Darwin)
    download_url="https://artifacts.duckdb.org/latest/duckdb-binaries-osx.zip"
    lib_dir_suffix="osx-universal"
    ;;
  Linux)
    case $(uname -m) in
      x86_64)
        download_url="https://artifacts.duckdb.org/latest/duckdb-binaries-linux.zip"
        lib_dir_suffix="linux-amd64"
        ;;
      aarch64)
        download_url="https://artifacts.duckdb.org/latest/duckdb-binaries-linux-aarch64.zip"
        lib_dir_suffix="linux-aarch64"
        ;;
      *)
        ;;
    esac
    ;;
  *)
    ;;
esac

if [ -z "$download_url" ]; then
  echo "Unsupported platform: $(uname -s) or architecture: $(uname -m)"
  exit 1
fi

# exit if already downloaded
if [ -f ./duckdb ]; then
  exit 0
fi

curl -L -o duckdb.zip $download_url
unzip duckdb.zip
rm duckdb.zip

unzip libduckdb-$lib_dir_suffix.zip
unzip duckdb_cli-$lib_dir_suffix.zip
