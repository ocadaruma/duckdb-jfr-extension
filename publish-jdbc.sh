#!/bin/bash

set -e

version="$1"
if [ -z "$version" ]; then
    echo "Usage: $0 VERSION" >&2
    exit 1
fi

cd "$(dirname $0)"

# download built extensions
rm -rf build/*
cd build
targets="linux-amd64 osx-universal"
for target in $targets; do
    filename="duckdb-jfr-extension-${target}.tar.gz"
    curl --fail -L "https://github.com/ocadaruma/duckdb-jfr-extension/releases/download/v${version}/${filename}" -o $filename
    tar -xzf $filename
    rm -f $filename
done

cd ../jdbc
./gradlew -Pversion=$version -Psnapshot=false clean publish
