#!/bin/bash -x

cargo clean
cargo build --release
mkdir -p ../target/artifacts
gzip -c ../target/release/libpact_mock_server_ffi.so > ../target/artifacts/libpact_mock_server_ffi-linux-x86_64.so.gz
gzip -c ../target/release/libpact_mock_server_ffi.a > ../target/artifacts/libpact_mock_server_ffi-linux-x86_64.a.gz
