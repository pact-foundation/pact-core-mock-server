#!/bin/bash -xe

mkdir -p ../target/artifacts
cargo build --release
gzip -c ../target/release/libpact_verifier_cli.dylib > ../target/artifacts/libpact_verifier_cli-osx-x86_64.dylib.gz
gzip -c ../target/release/libpact_verifier_cli.a > ../target/artifacts/libpact_verifier_cli-osx-x86_64.a.gz
