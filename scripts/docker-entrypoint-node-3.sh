#!/bin/bash

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-3 \
  --bootnodes /dns4/${ENDPOINT_TESTNET}/tcp/30333/p2p/${BOOTNODE_ID_TESTNET} \
  --keystore-path "/tmp/polkadot-chains/node-3/keys" \
  --chain ../node/src/chain-built/chain_def_${CHAIN_VERSION}.json \
  --name "${NODE_ENV} Validator Node 3" \
  --port 30335 \
  --ws-port 9946 \
  --rpc-port 9934 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
