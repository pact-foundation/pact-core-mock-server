#!/bin/bash -e

cargo clean
cargo build --release
mkdir -p ../target/artifacts
gzip -c ../target/release/pact_mock_server_ffi.dll > ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.dll.gz
gzip -c ../target/release/pact_mock_server_ffi.dll.lib > ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.dll.lib.gz
gzip -c ../target/release/pact_mock_server_ffi.lib > ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.lib.gz
