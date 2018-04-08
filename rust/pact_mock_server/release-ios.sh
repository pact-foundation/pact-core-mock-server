#!/bin/bash -x
# Requires all architectures to be added to in order to produce universal library for iOS.
# ```rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios
#    cargo install cargo-lipo```

if [ "" = "$1" ]; then
  echo "Usage: "
  echo "  ./release-ios.sh version"
  exit 1
fi

cargo clean
cargo lipo --release
gzip -c ../target/universal/release/libpact_mock_server.a > ../target/universal/release/libpact_mock_server-ios-universal-$1.a.gz
