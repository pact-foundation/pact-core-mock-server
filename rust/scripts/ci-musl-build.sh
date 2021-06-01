#!/bin/bash

cargo build
cd pact_models && cargo test
