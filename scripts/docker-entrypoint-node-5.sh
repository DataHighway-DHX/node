#!/bin/bash

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-5 \
  --bootnodes /dns/${ENDPOINT_TESTNET}/tcp/30333/p2p/${BOOTNODE_ID_TESTNET} \
  --keystore-path "/tmp/polkadot-chains/node-5/keys" \
  --chain ${CHAIN_VERSION} \
  --name "${NODE_ENV} Validator Node 5" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
