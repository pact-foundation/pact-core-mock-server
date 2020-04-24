#!/bin/bash

cargo clean
cargo build --release
gzip -c ../target/release/libpact_mock_server_ffi.so > ../target/release/libpact_mock_server_ffi-linux-x86_64.so.gz
gzip -c ../target/release/libpact_mock_server_ffi.a > ../target/release/libpact_mock_server_ffi-linux-x86_64.a.gz
