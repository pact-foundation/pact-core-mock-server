#!/bin/bash

set -e

cmake --version
cargo +nightly --version
cargo install cbindgen

rm -rf ./include

echo #####################################
echo # Build library with CMake
echo #####################################
mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE=Debug ..
cmake --build . -v
cd ..

rustup run nightly cbindgen \
  --config cbindgen.toml
  --crate pact_matching_ffi
  --output include/pact_matching.h

echo #####################################
echo # Make library available for examples
echo #####################################
cmake --install . --prefix ./install


echo #####################################
echo # Running examples
echo #####################################
cd ..
for i in examples/*; do
  pushd $i
  mkdir build
  cd build
  cmake ..
  cmake --build .
  popd
done
