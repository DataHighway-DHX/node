#!/bin/bash

echo "Docker Build Chain Spec"

# Generate the chain specification JSON file from src/chain_spec.rs
/usr/local/bin/datahighway build-spec \
  --chain=${CHAIN_VERSION} > /dhx/node/node/src/chain-built/chain_spec_${CHAIN_VERSION}.json
# Build "raw" chain definition for the new chain from it
/usr/local/bin/datahighway build-spec \
  --chain /dhx/node/node/src/chain-built/chain_spec_${CHAIN_VERSION}.json \
  --raw > /dhx/node/node/src/chain-built/chain_def_${CHAIN_VERSION}.json
