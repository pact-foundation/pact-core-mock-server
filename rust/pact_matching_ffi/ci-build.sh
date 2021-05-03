#!/bin/bash

set -e

cmake --version
cargo +nightly --version
cargo install cbindgen

rm -rf ./include

#####################################
# Build library with CMake
#####################################
mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE=Debug ..
cmake --build . -v
cd ..

#####################################
# Generate header with cbindgen
#####################################
rustup run nightly cbindgen \
  --config cbindgen.toml \
  --crate pact_matching_ffi \
  --output include/pact_matching.h

#####################################
# Make library available for examples
#####################################
cd build
cmake --install . --prefix ./install

#####################################
# Running examples
#####################################
cd ..
for i in examples/*; do
  pushd $i
  mkdir build
  cd build
  cmake ..
  cmake --build .
  popd
done
