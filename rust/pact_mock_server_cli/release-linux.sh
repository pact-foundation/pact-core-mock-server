#!/bin/bash -xe

cargo clean

mkdir -p ../release_artifacts
cargo build --release
gzip -c ../target/release/pact_mock_server_cli > ../release_artifacts/pact_mock_server_cli-linux-x86_64.gz
openssl dgst -sha256 -r ../release_artifacts/pact_mock_server_cli-linux-x86_64.gz > ../release_artifacts/pact_mock_server_cli-linux-x86_64.gz.sha256

echo -- Build the aarch64 release artifacts --
cargo install cross@0.2.5
cargo clean
cross build --target aarch64-unknown-linux-gnu --release
gzip -c ../target/aarch64-unknown-linux-gnu/release/pact_mock_server_cli > ../release_artifacts/pact_mock_server_cli-linux-aarch64.gz
openssl dgst -sha256 -r ../release_artifacts/pact_mock_server_cli-linux-aarch64.gz > ../release_artifacts/pact_mock_server_cli-linux-aarch64.gz.sha256
