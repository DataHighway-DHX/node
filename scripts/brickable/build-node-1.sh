#!/bin/bash

echo "Build Chain Spec for brickable"

# Generate the chain specification JSON file from src/chain_spec.rs
# Note that this requires the native binary built first
/usr/local/bin/datahighway build-spec \
  --chain=brickable > /dhx/node/node/src/chain-built/chain_spec_brickable.json
# Build "raw" chain definition for the new chain from it
/usr/local/bin/datahighway build-spec \
  --chain /dhx/node/node/src/chain-built/chain_spec_brickable.json \
  --raw > /dhx/node/node/src/chain-built/chain_def_brickable.json
