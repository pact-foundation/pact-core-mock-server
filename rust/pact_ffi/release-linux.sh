#!/bin/bash -x

cargo clean
mkdir -p ../target/artifacts
cargo build --release
rustup run nightly cbindgen \
  --config cbindgen.toml \
  --crate pact_ffi \
  --output include/pact.h
rustup run nightly cbindgen \
  --config cbindgen-c++.toml \
  --crate pact_ffi \
  --output include/pact-cpp.h
cp include/*.h ../target/artifacts
gzip -c ../target/release/libpact_ffi.so > ../target/artifacts/libpact_ffi-linux-x86_64.so.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_ffi-linux-x86_64.so.gz > ../target/artifacts/libpact_ffi-linux-x86_64.so.gz.sha256
gzip -c ../target/release/libpact_ffi.a > ../target/artifacts/libpact_ffi-linux-x86_64.a.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_ffi-linux-x86_64.a.gz > ../target/artifacts/libpact_ffi-linux-x86_64.a.gz.sha256
