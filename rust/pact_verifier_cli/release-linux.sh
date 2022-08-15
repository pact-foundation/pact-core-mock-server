#!/bin/bash -xe

cargo clean

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_verifier_cli > ../target/artifacts/pact_verifier_cli-linux-x86_64.gz
openssl dgst -sha256 -r ../target/artifacts/pact_verifier_cli-linux-x86_64.gz > ../target/artifacts/pact_verifier_cli-linux-x86_64.gz.sha256

echo -- Build the aarch64 release artifacts --
cargo install cross
cross build --target aarch64-unknown-linux-gnu --release
gzip -c ../target/aarch64-unknown-linux-gnu/release/pact_verifier_cli > ../target/artifacts/pact_verifier_cli-linux-aarch64.gz
openssl dgst -sha256 -r ../target/artifacts/pact_verifier_cli-linux-aarch64.gz > ../target/artifacts/pact_verifier_cli-linux-aarch64.gz.sha256
