#!/bin/bash -xe

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_verifier_cli > ../target/artifacts/pact_verifier_cli-osx-x86_64.gz
gzip -c ../target/release/pact_verifier_cli.dylib > ../target/artifacts/pact_verifier_cli-osx-x86_64.dylib.gz
gzip -c ../target/release/pact_verifier_cli.a > ../target/artifacts/pact_verifier_cli-osx-x86_64.a.gz
