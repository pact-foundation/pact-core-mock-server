#!/bin/bash

mkdir -p ../release_artifacts
cargo build --release
gzip -c ../target/release/pact_verifier_cli.exe > ../release_artifacts/pact_verifier_cli-windows-x86_64.exe.gz
openssl dgst -sha256 -r ../release_artifacts/pact_verifier_cli-windows-x86_64.exe.gz > ../release_artifacts/pact_verifier_cli-windows-x86_64.exe.gz.sha256
