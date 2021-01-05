#!/bin/bash

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_verifier_ffi.dll > ../target/artifacts/libpact_verifier_ffi-windows-x86_64.dll.gz
gzip -c ../target/release/pact_verifier_ffi.dll.lib > ../target/artifacts/libpact_verifier_ffi-windows-x86_64.dll.lib.gz
gzip -c ../target/release/pact_verifier_ffi.lib > ../target/artifacts/libpact_verifier_ffi-windows-x86_64.lib.gz
