#!/bin/bash

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_verifier_ffi.dll > ../target/artifacts/libpact_verifier_ffi-windows-x86_64.dll.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_verifier_ffi-windows-x86_64.dll.gz > ../target/artifacts/libpact_verifier_ffi-windows-x86_64.dll.gz.sha256
gzip -c ../target/release/pact_verifier_ffi.dll.lib > ../target/artifacts/libpact_verifier_ffi-windows-x86_64.dll.lib.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_verifier_ffi-windows-x86_64.dll.lib.gz > ../target/artifacts/libpact_verifier_ffi-windows-x86_64.dll.lib.gz.sha256
gzip -c ../target/release/pact_verifier_ffi.lib > ../target/artifacts/libpact_verifier_ffi-windows-x86_64.lib.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_verifier_ffi-windows-x86_64.lib.gz > ../target/artifacts/libpact_verifier_ffi-windows-x86_64.lib.gz.sha256
