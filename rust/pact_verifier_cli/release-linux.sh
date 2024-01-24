#!/bin/bash -xe

cargo clean

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_verifier_cli > ../target/artifacts/pact_verifier_cli-linux-x86_64.gz
openssl dgst -sha256 -r ../target/artifacts/pact_verifier_cli-linux-x86_64.gz > ../target/artifacts/pact_verifier_cli-linux-x86_64.gz.sha256

# aarch64 is failing to build on Rust 1.70+, and the dependencies need Rust 1.70+
#echo -- Build the aarch64 release artifacts --

#cargo install cross@0.2.5
#cross build --target aarch64-unknown-linux-gnu --release
#gzip -c ../target/aarch64-unknown-linux-gnu/release/pact_verifier_cli > ../target/artifacts/pact_verifier_cli-linux-aarch64.gz
#openssl dgst -sha256 -r ../target/artifacts/pact_verifier_cli-linux-aarch64.gz > ../target/artifacts/pact_verifier_cli-linux-aarch64.gz.sha256
