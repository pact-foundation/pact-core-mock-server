#!/bin/bash -e

cargo clean
cargo build --release
mkdir -p target/artifacts
gzip -c ../target/release/libpact_mock_server_ffi.dylib > ../target/artifacts/libpact_mock_server_ffi-osx-x86_64.dylib.gz
gzip -c ../target/release/libpact_mock_server_ffi.a > ../target/artifacts/libpact_mock_server_ffi-osx-x86_64.a.gz
