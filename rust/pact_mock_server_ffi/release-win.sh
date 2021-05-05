#!/bin/bash -e

cargo clean
cargo build --release
mkdir -p ../target/artifacts
gzip -c ../target/release/pact_mock_server_ffi.dll > ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.dll.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.dll.gz > ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.dll.gz.sha256
gzip -c ../target/release/pact_mock_server_ffi.dll.lib > ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.dll.lib.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.dll.lib.gz > ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.dll.lib.gz.sha256
gzip -c ../target/release/pact_mock_server_ffi.lib > ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.lib.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.lib.gz > ../target/artifacts/libpact_mock_server_ffi-windows-x86_64.lib.gz.sha256
