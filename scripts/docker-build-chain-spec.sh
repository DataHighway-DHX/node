#!/bin/bash

echo "Docker Build Chain Spec"
echo "Node Key is ${NODE_KEY}"
echo "Node Env is ${NODE_ENV}"
echo "Chain Version is ${CHAIN_VERSION}"

# Generate the chain specification JSON file from src/chain_spec.rs
../target/release/datahighway build-spec \
  --chain=${CHAIN_VERSION} > ../src/chain-spec-templates/chain_spec_${CHAIN_VERSION}.json
# Build "raw" chain definition for the new chain from it
../target/release/datahighway build-spec \
  --chain=${CHAIN_VERSION} ../src/chain-spec-templates/chain_spec_${CHAIN_VERSION}.json \
  --raw > ../src/chain-definition-custom/chain_def_${CHAIN_VERSION}.json
