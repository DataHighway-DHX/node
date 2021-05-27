#!/bin/bash

echo "Docker Entrypoint Eve"
echo "Node Key is ${NODE_KEY}"
echo "Node Env is ${NODE_ENV}"
echo "Chain Version is ${CHAIN_VERSION}"

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/eve \
  --bootnodes /dns4/${ENDPOINT_DEVELOPMENT}/tcp/30333/p2p/${BOOTNODE_ID_LOCAL} \
  --keystore-path "/tmp/polkadot-chains/eve/keys" \
  --chain /dhx/node/node/src/chain-built/chain_def_${CHAIN_VERSION}.json \
  --eve \
  --name "${NODE_ENV} Validator Eve" \
  --port 30337 \
  --ws-port 9948 \
  --rpc-port 9936 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
