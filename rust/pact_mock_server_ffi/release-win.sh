#!/bin/bash -e

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-win.sh version"
  exit 1
fi

cargo clean
cargo build --release --target x86_64-pc-windows-gnu
gzip -c ../target/x86_64-pc-windows-gnu/release/pact_mock_server_ffi.dll > ../target/release/libpact_mock_server_ffi-windows-x86_64-$1.dll.gz
gzip -c ../target/x86_64-pc-windows-gnu/release/pact_mock_server_ffi.lib > ../target/release/libpact_mock_server_ffi-windows-x86_64-$1.lib.gz
