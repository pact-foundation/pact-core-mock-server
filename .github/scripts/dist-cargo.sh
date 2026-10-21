#!/usr/bin/env bash
# The cargo that dist runs in the musl build jobs (dist reads `CARGO`; see
# .github/build-setup.yml).
#
# dist treats aarch64-musl in an x86_64 container as a cross build and runs
# `cargo zigbuild`. The cross-rs image already sets up its musl toolchain,
# so this runs a plain `cargo build`.
set -euo pipefail

args=("$@")
[[ ${args[0]-} == zigbuild ]] && args[0]=build

exec cargo "${args[@]}"
