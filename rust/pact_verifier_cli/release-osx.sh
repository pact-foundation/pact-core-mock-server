#!/bin/bash -xe

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/pact_verifier_cli > ../target/artifacts/pact_verifier_cli-osx-x86_64.gz
