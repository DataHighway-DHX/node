#!/bin/bash

echo "Docker Entrypoint Dave"
echo "Node Key is ${NODE_KEY}"
echo "Node Env is ${NODE_ENV}"
echo "Chain Version is ${CHAIN_VERSION}"

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/dave \
  --bootnodes /ip4/${ENDPOINT_DEVELOPMENT}/tcp/30333/p2p/${BOOTNODE_NODE_ID_LOCAL} \
  --keystore-path "/tmp/polkadot-chains/dave/keys" \
  --chain ${CHAIN_VERSION} \
  --dave \
  --name "${NODE_ENV} Validator Dave" \
  --port 30336 \
  --ws-port 9947 \
  --rpc-port 9935 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
