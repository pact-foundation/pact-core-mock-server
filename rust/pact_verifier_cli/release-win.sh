#!/bin/bash

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_verifier_cli.exe > ../target/artifacts/pact_verifier_cli-windows-x86_64.exe.gz
openssl dgst -sha256 -r ../target/artifacts/pact_verifier_cli-windows-x86_64.exe.gz > ../target/artifacts/pact_verifier_cli-windows-x86_64.exe.gz.sha256
