#!/bin/bash

set -e
set -x

RUST_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd )"

source "$RUST_DIR/scripts/gzip-and-sum.sh"
ARTIFACTS_DIR=${ARTIFACTS_DIR:-"$RUST_DIR/release_artifacts"}
mkdir -p "$ARTIFACTS_DIR"
CARGO_TARGET_DIR=${CARO_TARGET_DIR:-"$RUST_DIR/target"}

# All flags passed to this script are passed to cargo.
cargo_flags=( "$@" )

build_x86_64() {
    sudo apt-get install -y musl-tools
    cargo build --target=x86_64-unknown-linux-musl "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/pact_verifier_cli" \
            "$ARTIFACTS_DIR/pact_verifier_cli-linux-x86_64.gz"
    fi
}

install_cross() {
    cargo install cross@0.2.5
}

build_aarch64() {
    install_cross
    cross build --target aarch64-unknown-linux-musl "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum "$CARGO_TARGET_DIR/aarch64-unknown-linux-musl/release/pact_verifier_cli" \
        "$ARTIFACTS_DIR/pact_verifier_cli-linux-aarch64.gz"
    fi
}

build_x86_64
build_aarch64
