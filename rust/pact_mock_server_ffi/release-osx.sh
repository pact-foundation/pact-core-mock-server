#!/bin/bash -e

#if [ "" = "$OSXCROSS" ]; then
#  echo "This script needs OSXCROSS set to the home of osxcross"
#  exit 1
#fi
#
#export PATH="$OSXCROSS/target/bin:$PATH"
#export CC=o64-clang
#export CXX=o64-clang++

cargo clean
cargo build --release
gzip -c ../target/x86_64-apple-darwin/release/libpact_mock_server_ffi.dylib > ../target/release/libpact_mock_server_ffi-osx-x86_64.dylib.gz
gzip -c ../target/x86_64-apple-darwin/release/libpact_mock_server_ffi.a > ../target/release/libpact_mock_server_ffi-osx-x86_64.a.gz
