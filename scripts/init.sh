#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

rustup update stable
rustup update nightly
rustup toolchain install nightly-2020-12-12
rustup target add wasm32-unknown-unknown --toolchain nightly-2020-12-12
rustup default nightly-2020-12-12
rustup override set nightly-2020-12-12
