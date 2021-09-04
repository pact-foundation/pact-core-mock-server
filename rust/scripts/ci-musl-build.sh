#!/bin/bash

set -ex

sudo apk add shared-mime-info

rustc --print cfg
cargo build
cargo test
