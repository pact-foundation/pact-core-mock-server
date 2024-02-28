#!/bin/bash

set -e
set -x

RUST_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd )"

source "$RUST_DIR/scripts/gzip-and-sum.sh"
ARTIFACTS_DIR=${ARTIFACTS_DIR:-"$RUST_DIR/release_artifacts"}
mkdir -p "$ARTIFACTS_DIR"
export CARGO_TARGET_DIR=${CARO_TARGET_DIR:-"$RUST_DIR/target"}

# All flags passed to this script are passed to cargo.
cargo_flags=( "$@" )

# Build the x86_64 windows release
build_x86_64() {
    cargo build --target x86_64-pc-windows-msvc "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/release/pact_verifier_cli.exe" \
            "$ARTIFACTS_DIR/pact_verifier_cli-windows-x86_64.exe.gz"
    fi
}

# Build the aarch64 windows release
build_aarch64() {
    cargo build --target aarch64-pc-windows-msvc "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-pc-windows-msvc/release/pact_verifier_cli.exe" \
            "$ARTIFACTS_DIR/pact_verifier_cli-windows-aarch64.exe.gz"
    fi
}

build_x86_64
build_aarch64