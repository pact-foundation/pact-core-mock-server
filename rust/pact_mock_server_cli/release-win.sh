#!/bin/bash

cargo clean
mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_mock_server_cli.exe > ../target/artifacts/pact_mock_server_cli-windows-x86_64.exe.gz
