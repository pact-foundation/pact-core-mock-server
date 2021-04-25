#!/bin/bash -x

cargo clean
mkdir -p ../target/artifacts
GENERATE_C_HEADER=true cargo build --release
gzip -c ../target/release/libpact_mock_server_ffi.so > ../target/artifacts/libpact_mock_server_ffi-linux-x86_64.so.gz
sha256sum -b ../target/artifacts/libpact_mock_server_ffi-linux-x86_64.a.gz > ../target/artifacts/libpact_mock_server_ffi-linux-x86_64.a.gz.sha256
gzip -c ../target/release/libpact_mock_server_ffi.a > ../target/artifacts/libpact_mock_server_ffi-linux-x86_64.a.gz
sha256sum -b ../target/artifacts/libpact_mock_server_ffi-linux-x86_64.a.gz > ../target/artifacts/libpact_mock_server_ffi-linux-x86_64.a.gz.sha256
