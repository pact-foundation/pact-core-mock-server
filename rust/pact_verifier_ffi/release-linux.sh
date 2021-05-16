#!/bin/bash -xe

cargo clean

mkdir -p ../target/artifacts
GENERATE_C_HEADER=true cargo build --release
gzip -c ../target/release/libpact_verifier_ffi.so > ../target/artifacts/libpact_verifier_ffi-linux-x86_64.so.gz
openssl dgst -sha256 -r ../target/release/libpact_verifier_ffi.so > ../target/release/libpact_verifier_ffi.so.sha256
gzip -c ../target/release/libpact_verifier_ffi.a > ../target/artifacts/libpact_verifier_ffi-linux-x86_64.a.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_verifier_ffi-linux-x86_64.a.gz > ../target/artifacts/libpact_verifier_ffi-linux-x86_64.a.gz.sha256