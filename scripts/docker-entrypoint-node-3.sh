#!/bin/bash

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-3 \
  --bootnodes /dns4/${ENDPOINT_TESTNET}/tcp/30333/p2p/${BOOTNODE_ID_NODE_TESTNET_1} \
  --keystore-path "/tmp/polkadot-chains/node-3/keys" \
  --chain ../src/chain-definition-custom/chain_def_${CHAIN_VERSION}.json \
  --name "${NODE_ENV} Validator Node 3" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
