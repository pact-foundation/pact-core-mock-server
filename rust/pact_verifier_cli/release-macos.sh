#!/bin/bash

set -e
set -x

RUST_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd )"

source "$RUST_DIR/scripts/gzip-and-sum.sh"
ARTIFACTS_DIR=${ARTIFACTS_DIR:-"$RUST_DIR/release_artifacts"}
mkdir -p "$ARTIFACTS_DIR"
export CARGO_TARGET_DIR=${CARO_TARGET_DIR:-"$RUST_DIR/target"}

# We target the oldest supported version of macOS.
export MACOSX_DEPLOYMENT_TARGET=${MACOSX_DEPLOYMENT_TARGET:-12}

# All flags passed to this script are passed to cargo.
cargo_flags=( "$@" )

# Build the x86_64 darwin release
build_x86_64() {
    cargo build --target x86_64-apple-darwin "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-apple-darwin/release/pact_verifier_cli" \
            "$ARTIFACTS_DIR/pact_verifier_cli-osx-x86_64.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-apple-darwin/release/pact_verifier_cli" \
            "$ARTIFACTS_DIR/pact_verifier_cli-macos-x86_64.gz"
    fi
}

# Build the aarch64 darwin release
build_aarch64() {
    cargo build --target aarch64-apple-darwin "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR//aarch64-apple-darwin/release/pact_verifier_cli" \
            "$ARTIFACTS_DIR/pact_verifier_cli-osx-aarch64.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR//aarch64-apple-darwin/release/pact_verifier_cli" \
            "$ARTIFACTS_DIR/pact_verifier_cli-macos-aarch64.gz"
    fi
}

build_x86_64
build_aarch64
