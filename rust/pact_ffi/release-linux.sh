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

# Build the x86_64 GNU linux release
build_x86_64_gnu() {
    install_cross
    cargo clean
    cross build --target x86_64-unknown-linux-gnu "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/libpact_ffi.a" \
            "$ARTIFACTS_DIR/libpact_ffi-linux-x86_64.a.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/release/libpact_ffi.so" \
            "$ARTIFACTS_DIR/libpact_ffi-linux-x86_64.so.gz"
    fi
}

build_x86_64_musl() {
    sudo apt-get install -y musl-tools
    cargo clean
    cargo build --target x86_64-unknown-linux-musl "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        BUILD_SCRIPT=$(cat <<EOM
apk add --no-cache musl-dev gcc && \
cd /scratch && \
ar -x libpact_ffi.a && \
gcc -shared *.o -o libpact_ffi.so && \
rm -f *.o
EOM
        )

        docker run \
            --platform=linux/amd64 \
            --rm \
            -v "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release:/scratch" \
            alpine \
            /bin/sh -c "$BUILD_SCRIPT"

        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/libpact_ffi.a" \
            "$ARTIFACTS_DIR/libpact_ffi-linux-x86_64-musl.a.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/libpact_ffi.so" \
            "$ARTIFACTS_DIR/libpact_ffi-linux-x86_64-musl.so.gz"
    fi
}

install_cross() {
    cargo install cross@0.2.5
}

build_aarch64_gnu() {
    install_cross
    cargo clean
    cross build --target aarch64-unknown-linux-gnu "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-unknown-linux-gnu/release/libpact_ffi.a" \
            "$ARTIFACTS_DIR/libpact_ffi-linux-aarch64.a.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-unknown-linux-gnu/release/libpact_ffi.so" \
            "$ARTIFACTS_DIR/libpact_ffi-linux-aarch64.so.gz"
    fi
}

build_aarch64_musl() {
    install_cross
    cargo clean
    cross build --target aarch64-unknown-linux-musl "${cargo_flags[@]}"

    if [[ "${cargo_flags[*]}" =~ "--release" ]]; then
        BUILD_SCRIPT=$(cat <<EOM
apk add --no-cache musl-dev gcc && \
cd /scratch && \
ar -x libpact_ffi.a && \
gcc -shared *.o -o libpact_ffi.so && \
rm -f *.o
EOM
        )

        docker run \
            --platform=linux/arm64 \
            --rm \
            -v "$CARGO_TARGET_DIR/aarch64-unknown-linux-musl/release:/scratch" \
            alpine \
            /bin/sh -c "$BUILD_SCRIPT"

        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-unknown-linux-musl/release/libpact_ffi.a" \
            "$ARTIFACTS_DIR/libpact_ffi-linux-aarch64-musl.a.gz"
        gzip_and_sum \
            "$CARGO_TARGET_DIR/aarch64-unknown-linux-musl/release/libpact_ffi.so" \
            "$ARTIFACTS_DIR/libpact_ffi-linux-aarch64-musl.so.gz"
    fi
}

build_header() {
    rustup toolchain install nightly
    rustup run nightly cbindgen \
        --config cbindgen.toml \
        --crate pact_ffi \
        --output "$ARTIFACTS_DIR/pact.h"
    rustup run nightly cbindgen \
        --config cbindgen-c++.toml \
        --crate pact_ffi \
        --output "$ARTIFACTS_DIR/pact-cpp.h"
}

build_x86_64_gnu
build_x86_64_musl
build_aarch64_gnu
build_aarch64_musl
build_header