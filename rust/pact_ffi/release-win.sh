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

    # If --release in cargo flags, then gzip and sum the release artifacts
    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/release/pact_ffi.dll" \
            "$ARTIFACTS_DIR/pact_ffi-windows-x86_64.dll.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/release/pact_ffi.dll.lib" \
            "$ARTIFACTS_DIR/pact_ffi-windows-x86_64.dll.lib.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-pc-windows-msvc/release/pact_ffi.lib" \
            "$ARTIFACTS_DIR/pact_ffi-windows-x86_64.lib.gz"
    fi
}

# Build the aarch64 windows release
build_aarch64() {
    cargo build --target aarch64-pc-windows-msvc "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-pc-windows-msvc/release/pact_ffi.dll" \
            "$ARTIFACTS_DIR/pact_ffi-windows-aarch64.dll.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-pc-windows-msvc/release/pact_ffi.dll.lib" \
            "$ARTIFACTS_DIR/pact_ffi-windows-aarch64.dll.lib.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-pc-windows-msvc/release/pact_ffi.lib" \
            "$ARTIFACTS_DIR/pact_ffi-windows-aarch64.lib.gz"
    fi
}

build_x86_64
build_aarch64