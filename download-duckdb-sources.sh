#!/bin/bash

set -eu

# When you want to upgrade version, delete duckdb-sources directory and
# run this script after bumping DUCKDB_RS_VERSION.
DUCKDB_RS_VERSION="0.9.1"

SCRIPT_DIR=$(cd $(dirname $0); pwd)

# We download duckdb sources from duckdb-rs/libduckdb-rs instead of original sources
# so we can use cc-crate to build duckdb lib and jfr-bridge together without cmake.
mkdir -p $SCRIPT_DIR/duckdb-sources
cd $SCRIPT_DIR/duckdb-sources

download_url="https://github.com/duckdb/duckdb-rs/archive/refs/tags/v${DUCKDB_RS_VERSION}.tar.gz"
source_filename="duckdb.tar.gz"
libduckdb_sys_path="duckdb-rs-${DUCKDB_RS_VERSION}/libduckdb-sys"

# exit if already downloaded
if [ -f ./duckdb/manifest.json ]; then
  exit 0
fi

curl -L $download_url | tar xf - "${libduckdb_sys_path}/${source_filename}"
mv "${libduckdb_sys_path}/${source_filename}" .

tar xf "${source_filename}"

# cleanup
rm -rf "duckdb-rs-${DUCKDB_RS_VERSION}" "${source_filename}"
