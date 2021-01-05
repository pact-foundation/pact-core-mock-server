#!/bin/bash -xe

cargo clean

mkdir -p ../target/artifacts
GENERATE_C_HEADER=true cargo build --release
gzip -c ../target/release/libpact_verifier_ffi.so > ../target/artifacts/libpact_verifier_ffi-linux-x86_64.so.gz
gzip -c ../target/release/libpact_verifier_ffi.a > ../target/artifacts/libpact_verifier_ffi-linux-x86_64.a.gz
