#!/bin/bash

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-win.sh version"
  exit 1
fi

cargo clean
cargo build --release
gzip -c ../target/release/pact_mock_server_ffi.dll > ../target/release/pact_mock_server_ffi-windows-x86_64-$1.dll.gz
gzip -c ../target/release/pact_mock_server_ffi.lib > ../target/release/pact_mock_server_ffi-windows-x86_64-$1.lib.gz
