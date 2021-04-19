#!/usr/bin/env bash

set -e

cargo clean

WASM_BUILD_TYPE=release cargo run -- build-spec --chain local > ./node/src/chain-spec-templates/chain_spec_local.json
WASM_BUILD_TYPE=release cargo run -- build-spec --chain ./node/src/chain-spec-templates/chain_spec_local.json --raw > ./node/src/chain-built/chain_def_local.json
