#!/bin/bash

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_verifier_cli.dll > ../target/artifacts/libpact_verifier_cli-windows-x86_64.dll.gz
gzip -c ../target/release/pact_verifier_cli.dll.lib > ../target/artifacts/libpact_verifier_cli-windows-x86_64.dll.lib.gz
gzip -c ../target/release/pact_verifier_cli.lib > ../target/artifacts/libpact_verifier_cli-windows-x86_64.lib.gz
