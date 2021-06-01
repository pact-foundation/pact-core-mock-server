#!/bin/bash

set -ex

cargo build
cd pact_models && cargo test
