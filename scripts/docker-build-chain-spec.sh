#!/bin/bash

echo "Docker Build Chain Spec"

# Generate the chain specification JSON file from src/chain_spec.rs
../target/release/datahighway build-spec \
  --chain=${CHAIN_VERSION} > ../node/src/chain-spec-templates/chain_spec_${CHAIN_VERSION}.json
# Build "raw" chain definition for the new chain from it
../target/release/datahighway build-spec \
  --chain ../node/src/chain-spec-templates/chain_spec_${CHAIN_VERSION}.json \
  --raw > ../node/src/chain-built/chain_def_${CHAIN_VERSION}.json
