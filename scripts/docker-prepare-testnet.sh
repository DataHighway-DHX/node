#!/usr/bin/env bash

set -e

rm -rf /tmp/polkadot-chains/node-1 /tmp/polkadot-chains/node-2 /tmp/polkadot-chains/node-3

rm -rf ../src/chain-spec-templates/chain_spec_testnet_latest.json ../src/chain-definition-custom/chain_def_testnet_v0.1.0.json

touch ../src/chain-spec-templates/chain_spec_testnet_latest.json ../src/chain-definition-custom/chain_def_testnet_v0.1.0.json

../target/release/datahighway build-spec --chain testnet-latest > ../src/chain-spec-templates/chain_spec_testnet_latest.json
../target/release/datahighway build-spec --chain ../src/chain-spec-templates/chain_spec_testnet_latest.json --raw > ../src/chain-definition-custom/chain_def_testnet_v0.1.0.json

# WASM_BUILD_TYPE=release cargo run -- build-spec --chain testnet-latest > ./src/chain-spec-templates/chain_spec_testnet_latest.json
# WASM_BUILD_TYPE=release cargo run -- build-spec --chain ./src/chain-spec-templates/chain_spec_testnet_latest.json --raw > ./src/chain-definition-custom/chain_def_testnet_v0.1.0.json
