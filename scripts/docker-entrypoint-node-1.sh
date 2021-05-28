#!/bin/bash

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-1 \
  --keystore-path "/tmp/polkadot-chains/node-1/keys" \
  --chain ${CHAIN_VERSION} \
  --name "${NODE_ENV} Validator Node 1" \
  --node-key ${NODE_KEY_TESTNET} \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
