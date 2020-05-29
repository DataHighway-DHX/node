#!/bin/bash

# ./docker-prepare-testnet.sh

# touch ../src/chain-spec-templates/chain_spec_testnet_latest.json ../src/chain-definition-custom/chain_def_testnet_latest.json
# ../target/release/datahighway build-spec --chain testnet_latest > ../src/chain-spec-templates/chain_spec_testnet_latest.json
# ../target/release/datahighway build-spec --chain ../src/chain-spec-templates/chain_spec_testnet_latest.json --raw > ../src/chain-definition-custom/chain_def_testnet_latest.json

# use 127.0.0.1 or testnet-harbour.datahighway.com
# bootnode node-2 is QmTU8wBoSGWDX2Dd3sGE2bD9xb9cRjZTzaH3dG4BmoQWbD or
# sentry node id: QmVuryfE427VRqrqqXsGuWpwBk4g8mGXgYmnt3f1v6j78r

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-3 \
  --bootnodes /dns4/testnet-harbour.datahighway.com/tcp/30333/p2p/QmVuryfE427VRqrqqXsGuWpwBk4g8mGXgYmnt3f1v6j78r \
  --keystore-path "/tmp/polkadot-chains/node-3/keys" \
  --chain ../src/chain-definition-custom/chain_def_testnet_latest.json \
  --name "Example Validator Node 3" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
