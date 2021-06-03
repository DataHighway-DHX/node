#!/bin/bash

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-5 \
  --bootnodes /ip4/${BOOTNODE_ENDPOINT_TESTNET}/tcp/30333/p2p/${BOOTNODE_ID_TESTNET} \
  --chain ${CHAIN_VERSION} \
  --name "${CHAIN_VERSION} Validator Node 5" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
