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

ls -la ..
ls -la ../../target/debug

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
