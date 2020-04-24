#!/bin/bash -e

cargo clean
cargo build --release
gzip -c ../target/release/pact_mock_server_ffi.dll > ../target/release/libpact_mock_server_ffi-windows-x86_64.dll.gz
gzip -c ../target/release/pact_mock_server_ffi.lib > ../target/release/libpact_mock_server_ffi-windows-x86_64.lib.gz
