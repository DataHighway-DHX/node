#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

rustup default stable
rustup update stable
rustup update nightly
rustup toolchain install nightly-2021-03-10
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-03-10
rustup default nightly-2021-03-10
rustup override set nightly-2021-03-10
