#!/bin/bash

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-win.sh version"
  exit 1
fi

cargo clean
cargo build --release
gzip -c ../target/release/pact_mock_server_cli.exe > ../target/release/pact_mock_server_cli-windows-x86_64-$1.exe.gz

