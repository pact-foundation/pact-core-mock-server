#!/bin/bash -e

cargo clean
cargo build --release
mkdir -p ../target/artifacts
gzip -c ../target/release/libpact_ffi.dylib > ../target/artifacts/libpact_ffi-osx-x86_64.dylib.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_ffi-osx-x86_64.dylib.gz > ../target/artifacts/libpact_ffi-osx-x86_64.dylib.gz.sha256
gzip -c ../target/release/libpact_ffi.a > ../target/artifacts/libpact_ffi-osx-x86_64.a.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_ffi-osx-x86_64.a.gz > ../target/artifacts/libpact_ffi-osx-x86_64.a.gz.sha256

# M1
#export SDKROOT=$(xcrun -sdk macosx11.1 --show-sdk-path)
#export MACOSX_DEPLOYMENT_TARGET=$(xcrun -sdk macosx11.1 --show-sdk-platform-version)
#cargo build --target aarch64-apple-darwin --release
cargo install cross --git https://github.com/cross-rs/cross
cross build --target aarch64-apple-darwin --release

gzip -c ../target/aarch64-apple-darwin/release/libpact_ffi.dylib > ../target/artifacts/libpact_ffi-osx-aarch64-apple-darwin.dylib.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_ffi-osx-aarch64-apple-darwin.dylib.gz > ../target/artifacts/libpact_ffi-osx-aarch64-apple-darwin.dylib.gz.sha256
gzip -c ../target/aarch64-apple-darwin/release/libpact_ffi.a > ../target/artifacts/libpact_ffi-osx-aarch64-apple-darwin.a.gz
openssl dgst -sha256 -r ../target/artifacts/libpact_ffi-osx-aarch64-apple-darwin.a.gz > ../target/artifacts/libpact_ffi-osx-aarch64-apple-darwin.a.gz.sha256
