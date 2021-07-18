#!/bin/bash

set -e

which cargo

cargo install cbindgen
rm -rf ./include

#echo -------------------------------------
#echo - Build library with CMake
#echo -------------------------------------
#mkdir -p build
#cd build
#cmake -DCMAKE_BUILD_TYPE=Debug ..
#cmake --build . -v
#cd ..
cargo build

echo -------------------------------------
echo - Generate header with cbindgen
echo -------------------------------------
rustup run nightly cbindgen \
  --config cbindgen.toml \
  --crate pact_ffi \
  --output include/pact.h

echo -------------------------------------
echo - Make library available for examples
echo -------------------------------------
cd build
cmake --install . --prefix ./install

echo -------------------------------------
echo - Running examples
echo -------------------------------------
cd ..
for i in examples/*; do
  pushd $i
  mkdir -p build
  cd build
  cmake ..
  cmake --build .
  echo Running example
  ./example
  popd
done
