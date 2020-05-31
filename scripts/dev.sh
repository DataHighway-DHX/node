#!/bin/bash

echo "Running scripts/dev.sh"
echo pwd
# Generate the chain specification JSON file from src/chain_spec.rs
../target/release/datahighway build-spec \
  --chain=${_CHAIN_VERSION} > ../src/chain-spec-templates/chain_spec_${_CHAIN_VERSION}.json
# Build "raw" chain definition for the new chain from it
../target/release/datahighway build-spec \
  --chain ../src/chain-spec-templates/chain_spec_${_CHAIN_VERSION}.json \
  --raw > ../src/chain-definition-custom/chain_def_${_CHAIN_VERSION}.json
