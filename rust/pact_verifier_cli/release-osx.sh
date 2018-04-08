#!/bin/bash -xe

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-osx.sh version"
  exit 1
fi

cargo clean
cargo build --release
cargo build --release --target x86_64-apple-ios
gzip -c target/release/pact_verifier_cli > target/release/pact_verifier_cli-osx-x86_64-$1.gz
cargo build --release --target x86_64-apple-ios
gzip -c target/x86_64-apple-ios/release/pact_verifier_cli > target/x86_64-apple-ios/release/pact_verifier_cli-ios-x86_64-$1.gz
