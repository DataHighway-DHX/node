#!/bin/bash

echo "Docker Entrypoint Eve"
echo "Node Key is ${NODE_KEY}"
echo "Chain Version is ${CHAIN_VERSION}"

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/eve \
  --bootnodes /ip4/${BOOTNODE_ENDPOINT_DEV}/tcp/30333/p2p/${BOOTNODE_ID_LOCAL} \
  --chain ${CHAIN_VERSION} \
  --eve \
  --name "${CHAIN_VERSION} Validator Eve" \
  --port 30337 \
  --ws-port 9948 \
  --rpc-port 9936 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
