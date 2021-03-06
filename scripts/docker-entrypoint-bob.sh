#!/bin/bash

echo "Docker Entrypoint Bob"
echo "Node Key is ${NODE_KEY}"
echo "Node Env is ${NODE_ENV}"
echo "Chain Version is ${CHAIN_VERSION}"

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/bob \
  --keystore-path "/tmp/polkadot-chains/bob/keys" \
  --bootnodes /dns4/${ENDPOINT_DEVELOPMENT}/tcp/30333/p2p/${BOOTNODE_ID_LOCAL} \
  --chain ../node/src/chain-definition-custom/chain_def_${CHAIN_VERSION}.json \
  --bob \
  --name "${NODE_ENV} Validator Bob" \
  --port 30334 \
  --ws-port 9945 \
  --rpc-port 9934 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
