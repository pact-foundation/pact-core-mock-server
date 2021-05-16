#!/bin/bash -xe

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/libpact_verifier_ffi.dylib > ../target/artifacts/libpact_verifier_ffi-osx-x86_64.dylib.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_verifier_ffi-osx-x86_64.dylib.gz > ../target/artifacts/libpact_verifier_ffi-osx-x86_64.dylib.gz.sha256
gzip -c ../target/release/libpact_verifier_ffi.a > ../target/artifacts/libpact_verifier_ffi-osx-x86_64.a.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_verifier_ffi-osx-x86_64.a.gz > ../target/artifacts/libpact_verifier_ffi-osx-x86_64.a.gz.sha256
