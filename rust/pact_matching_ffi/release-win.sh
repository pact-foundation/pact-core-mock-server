#!/bin/bash -e

cargo clean
cargo build --release
mkdir -p ../target/artifacts
gzip -c ../target/release/libpact_matching_ffi.dll > ../target/artifacts/liblibpact_matching_ffi-windows-x86_64.dll.gz
openssl dgst -sha256 -r ../target/artifacts/liblibpact_matching_ffi-windows-x86_64.dll.gz > ../target/artifacts/liblibpact_matching_ffi-windows-x86_64.dll.gz.sha256
gzip -c ../target/release/libpact_matching_ffi.dll.lib > ../target/artifacts/liblibpact_matching_ffi-windows-x86_64.dll.lib.gz
openssl dgst -sha256 -r ../target/artifacts/liblibpact_matching_ffi-windows-x86_64.dll.lib.gz > ../target/artifacts/liblibpact_matching_ffi-windows-x86_64.dll.lib.gz.sha256
gzip -c ../target/release/libpact_matching_ffi.lib > ../target/artifacts/liblibpact_matching_ffi-windows-x86_64.lib.gz
openssl dgst -sha256 -r ../target/artifacts/liblibpact_matching_ffi-windows-x86_64.lib.gz > ../target/artifacts/liblibpact_matching_ffi-windows-x86_64.lib.gz.sha256
