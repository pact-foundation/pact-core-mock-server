#!/bin/bash

set -e

echo #####################################
echo # Build library with CMake
echo #####################################
mkdir build
cd build
cmake ..
cmake --build .

echo #####################################
echo # Make library available for examples
echo #####################################
cmake --install . --prefix ./install


echo #####################################
echo # Running examples
echo #####################################
for i in examples/*; do
  pushd $i
  mkdir build
  cd build
  cmake ..
  cmake --build .
  popd
done
