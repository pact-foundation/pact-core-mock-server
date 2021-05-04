#!/bin/bash -x

cargo clean
mkdir -p ../target/artifacts
cargo build --release
rustup run nightly cbindgen \
  --config cbindgen.toml \
  --crate pact_matching_ffi \
  --output include/pact_matching.h
cp include/pact_matching.h ../target/artifacts
gzip -c ../target/release/libpact_matching_ffi.so > ../target/artifacts/libpact_matching_ffi-linux-x86_64.so.gz
sha256sum -b ../target/artifacts/libpact_matching_ffi-linux-x86_64.a.gz > ../target/artifacts/libpact_matching_ffi-linux-x86_64.a.gz.sha256
gzip -c ../target/release/libpact_matching_ffi.a > ../target/artifacts/libpact_matching_ffi-linux-x86_64.a.gz
sha256sum -b ../target/artifacts/libpact_matching_ffi-linux-x86_64.a.gz > ../target/artifacts/libpact_matching_ffi-linux-x86_64.a.gz.sha256
