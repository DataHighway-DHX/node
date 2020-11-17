#!/usr/bin/env bash

set -e

cargo clean

WASM_BUILD_TYPE=release cargo run -- build-spec --chain testnet_latest > ./node/src/chain-spec-templates/chain_spec_testnet_latest.json
WASM_BUILD_TYPE=release cargo run -- build-spec --chain ./node/src/chain-spec-templates/chain_spec_testnet_latest.json --raw > ./node/src/chain-definition-custom/chain_def_testnet_latest.json
