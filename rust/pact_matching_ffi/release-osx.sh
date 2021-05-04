#!/bin/bash -e

cargo clean
cargo build --release
mkdir -p ../target/artifacts
gzip -c ../target/release/libpact_matching_ffi.dylib > ../target/artifacts/libpact_matching_ffi-osx-x86_64.dylib.gz
sha256sum -b ../target/artifacts/libpact_matching_ffi-osx-x86_64.dylib.gz > ../target/artifacts/libpact_matching_ffi-osx-x86_64.dylib.gz.sha256
gzip -c ../target/release/libpact_matching_ffi.a > ../target/artifacts/libpact_matching_ffi-osx-x86_64.a.gz
sha256sum -b ../target/artifacts/libpact_matching_ffi-osx-x86_64.a.gz > ../target/artifacts/libpact_matching_ffi-osx-x86_64.a.gz.sha256
