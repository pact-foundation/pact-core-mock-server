#!/bin/bash -e

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-osx.sh version"
  exit 1
fi

if [ "" = "$OSXCROSS" ]; then
  echo "This script needs OSXCROSS set to the home of osxcross"
  exit 1
fi

export PATH="$OSXCROSS/target/bin:$PATH"
export CC=o64-clang
export CXX=o64-clang++

cargo clean
cargo build --release --target x86_64-apple-darwin
gzip -c ../target/x86_64-apple-darwin/release/libpact_mock_server_ffi.dylib > ../target/release/libpact_mock_server_ffi-osx-x86_64-$1.dylib.gz
gzip -c ../target/x86_64-apple-darwin/release/libpact_mock_server_ffi.a > ../target/release/libpact_mock_server_ffi-osx-x86_64-$1.a.gz
