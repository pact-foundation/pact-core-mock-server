#!/bin/bash

set -ex

rustc --print cfg
cargo build
RUST_LOG=trace cargo test
