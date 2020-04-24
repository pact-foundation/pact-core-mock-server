#!/bin/bash -e

cargo clean
cargo build --release
gzip -c ../target/release/libpact_mock_server_ffi.dylib > ../target/release/libpact_mock_server_ffi-osx-x86_64.dylib.gz
gzip -c ../target/release/libpact_mock_server_ffi.a > ../target/release/libpact_mock_server_ffi-osx-x86_64.a.gz
