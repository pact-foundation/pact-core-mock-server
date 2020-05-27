#!/bin/bash -xe

cargo clean

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_mock_server_cli > ../target/artifacts/pact_mock_server_cli-osx-x86_64.gz
cargo build --release --target x86_64-apple-ios
gzip -c ../target/x86_64-apple-ios/release/pact_mock_server_cli > ../target/artifacts/pact_mock_server_cli-ios-x86_64.gz
