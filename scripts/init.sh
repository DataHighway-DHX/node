#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

rustup update stable
rustup update nightly
# revert to older version instead of 2022-03-07 to avoid this 'asm' error
# https://github.com/bytecodealliance/rustix/issues/142#issuecomment-1000056041
rustup toolchain install nightly-2021-12-15
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-12-15
rustup default nightly-2021-12-15
rustup override set nightly-2021-12-15
