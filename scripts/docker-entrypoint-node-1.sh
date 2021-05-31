#!/bin/bash

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-1 \
  --bootnodes /ip4/${BOOTNODE_ENDPOINT_TESTNET}/tcp/30333/p2p/${BOOTNODE_NODE_ID_TESTNET} \
  --chain ${CHAIN_VERSION} \
  --name "${CHAIN_VERSION} Validator Node 1" \
  --node-key ${NODE_KEY_TESTNET} \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
