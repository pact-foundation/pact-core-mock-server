#!/bin/bash -x

echo -- Setup directories --
cargo clean
mkdir -p ../target/artifacts

echo -- Build the release artifacts --
cargo build --release
gzip -c ../target/release/libpact_ffi.so > ../target/artifacts/libpact_ffi-linux-x86_64.so.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_ffi-linux-x86_64.so.gz > ../target/artifacts/libpact_ffi-linux-x86_64.so.gz.sha256
gzip -c ../target/release/libpact_ffi.a > ../target/artifacts/libpact_ffi-linux-x86_64.a.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_ffi-linux-x86_64.a.gz > ../target/artifacts/libpact_ffi-linux-x86_64.a.gz.sha256

echo -- Generate the header files --
rustup toolchain install nightly
rustup component add rustfmt --toolchain nightly
rustup run nightly cbindgen \
  --config cbindgen.toml \
  --crate pact_ffi \
  --output include/pact.h
rustup run nightly cbindgen \
  --config cbindgen-c++.toml \
  --crate pact_ffi \
  --output include/pact-cpp.h
cp include/*.h ../target/artifacts

echo -- Build the musl release artifacts --
sudo apt install musl-tools
rustup target add x86_64-unknown-linux-musl
cargo build --release --target=x86_64-unknown-linux-musl
gzip -c ../target/x86_64-unknown-linux-musl/release/libpact_ffi.a > ../target/artifacts/libpact_ffi-linux-x86_64-musl.a.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_ffi-linux-x86_64-musl.a.gz > ../target/artifacts/libpact_ffi-linux-x86_64-musl.a.gz.sha256
