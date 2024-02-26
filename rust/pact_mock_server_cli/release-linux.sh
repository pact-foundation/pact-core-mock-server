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

build_x86_64_gnu() {
    cargo build --target x86_64-unknown-linux-gnu "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/pact_mock_server_cli" \
            "$ARTIFACTS_DIR/pact_mock_server_cli-linux-x86_64.gz"
    fi
}

build_x86_64_musl() {
    sudo apt-get install -y musl-tools
    cargo build --target=x86_64-unknown-linux-musl "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/pact_mock_server_cli" \
            "$ARTIFACTS_DIR/pact_mock_server_cli-linux-x86_64-musl.gz"
    fi
}

install_cross() {
    cargo install cross@0.2.5
}

build_aarch64_gnu() {
    install_cross
    cross build --target aarch64-unknown-linux-gnu "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-unknown-linux-gnu/release/pact_mock_server_cli" \
            "$ARTIFACTS_DIR/pact_mock_server_cli-linux-aarch64.gz"
    fi
}

build_aarch64_musl() {
    install_cross
    cross build --target=aarch64-unknown-linux-musl "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-unknown-linux-musl/release/pact_mock_server_cli" \
            "$ARTIFACTS_DIR/pact_mock_server_cli-linux-aarch64-musl.gz"
    fi
}

build_x86_64_gnu
build_x86_64_musl
build_aarch64_gnu
build_aarch64_musl