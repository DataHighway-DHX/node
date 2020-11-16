#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ -z $CI_PROJECT_NAME ] ; then
  rustup update stable
  rustup toolchain install nightly-2020-10-06
  rustup default nightly-2020-10-06
fi

rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-06
