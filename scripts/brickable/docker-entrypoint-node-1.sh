#!/bin/bash

# FIXME - check security associated with each CLI option
/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-1 \
  --bootnodes /dns/${ENDPOINT_BRICKABLE}/tcp/30333/p2p/${BOOTNODE_NODE_ID_BRICKABLE} \
  --keystore-path "/tmp/polkadot-chains/node-1/keys" \
  # --chain /dhx/node/node/src/chain-built/chain_def_brickable.json \
  --chain brickable \
  --name "Brickable Validator Node 1" \
  --node-key ${NODE_KEY_BRICKABLE} \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
