#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

rustup update stable
rustup update nightly
rustup toolchain install nightly-2021-08-31
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-08-31
rustup default nightly-2021-08-31
rustup override set nightly-2021-08-31
