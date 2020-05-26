#!/bin/bash

# use 127.0.0.1 or testnet-harbour.datahighway.com
# bootnode node-2 is QmTU8wBoSGWDX2Dd3sGE2bD9xb9cRjZTzaH3dG4BmoQWbD or
# sentry node id: QmRR5ipj6arL2rhfUsAUk9ndCQ6qYntjqDQSDD73mi2g7p

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-3 \
  --bootnodes /dns4/testnet-harbour.datahighway.com/tcp/30333/p2p/QmRR5ipj6arL2rhfUsAUk9ndCQ6qYntjqDQSDD73mi2g7p \
  --keystore-path "/tmp/polkadot-chains/node-3/keys" \
  --chain ../src/chain-definition-custom/chain_def_testnet_v0.1.0.json \
  --name "Example Validator Node 3" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
