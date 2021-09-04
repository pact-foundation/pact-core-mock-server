#!/bin/bash

set -ex

apk add shared-mime-info

rustc --print cfg
cargo build
cargo test
