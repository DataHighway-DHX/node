#!/bin/bash

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-3 \
  --bootnodes /dns/${ENDPOINT_BRICKABLE}/tcp/30333/p2p/${BOOTNODE_NODE_ID_BRICKABLE} \
  --keystore-path "/tmp/polkadot-chains/node-3/keys" \
  # --chain /dhx/node/node/src/chain-built/chain_def_brickable.json \
  --chain brickable \
  --name "Brickable Validator Node 3" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
