#!/bin/bash -x

set -e

echo -- Setup directories --
cargo clean
mkdir -p ../release_artifacts

echo -- Build the Docker build image --
docker build -f Dockerfile.linux-build -t pact-ffi-build .

echo -- Build the release artifacts --
docker run -t --rm --user "$(id -u)":"$(id -g)" -v $(pwd)/..:/workspace -w /workspace/pact_ffi pact-ffi-build -c 'cargo build --release'
gzip -c ../target/release/libpact_ffi.so > ../release_artifacts/libpact_ffi-linux-x86_64.so.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-x86_64.so.gz > ../release_artifacts/libpact_ffi-linux-x86_64.so.gz.sha256
gzip -c ../target/release/libpact_ffi.a > ../release_artifacts/libpact_ffi-linux-x86_64.a.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-x86_64.a.gz > ../release_artifacts/libpact_ffi-linux-x86_64.a.gz.sha256

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
cp include/*.h ../release_artifacts

echo -- Build the musl release artifacts --
sudo apt install musl-tools
rustup target add x86_64-unknown-linux-musl
cargo build --release --target=x86_64-unknown-linux-musl
gzip -c ../target/x86_64-unknown-linux-musl/release/libpact_ffi.a > ../release_artifacts/libpact_ffi-linux-x86_64-musl.a.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-x86_64-musl.a.gz > ../release_artifacts/libpact_ffi-linux-x86_64-musl.a.gz.sha256

echo -- Build the aarch64 release artifacts --
cargo install cross --git https://github.com/cross-rs/cross
cargo clean
cross build --target aarch64-unknown-linux-gnu --release
gzip -c ../target/aarch64-unknown-linux-gnu/release/libpact_ffi.so > ../release_artifacts/libpact_ffi-linux-aarch64.so.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-aarch64.so.gz > ../release_artifacts/libpact_ffi-linux-aarch64.so.gz.sha256
gzip -c ../target/aarch64-unknown-linux-gnu/release/libpact_ffi.a > ../release_artifacts/libpact_ffi-linux-aarch64.a.gz
openssl dgst -sha256 -r ../release_artifacts/libpact_ffi-linux-aarch64.a.gz > ../release_artifacts/libpact_ffi-linux-aarch64.a.gz.sha256
