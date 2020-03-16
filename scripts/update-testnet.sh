#!/usr/bin/env bash

set -e

cargo clean
WASM_BUILD_TYPE=release cargo run -- build-spec --chain testnet-latest > ./src/chain-spec-templates/chain_spec_testnet_latest.json
WASM_BUILD_TYPE=release cargo run -- build-spec --chain ./src/chain-spec-templates/chain_spec_testnet_latest.json --raw > ./src/chain-definition-custom/chain_def_testnet_v0.1.0.json
