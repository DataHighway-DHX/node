#!/bin/bash
CHAIN_VERSION=$1

echo "Build Chain Spec for ${CHAIN_VERSION}"

# Generate the chain specification JSON file from src/chain_spec.rs
# Note that this requires the native binary built first
# Pass network name as parameter 1 eg. ./build-chain-spec.sh dev
../target/release/datahighway build-spec \
  --chain=${CHAIN_VERSION} > ../res/chain_spec_${CHAIN_VERSION}.json
# Build "raw" chain definition for the new chain from it
../target/release/datahighway build-spec \
  --chain ../res/chain_spec_${CHAIN_VERSION}.json \
  --raw > ../res/chain_def_${CHAIN_VERSION}.json
