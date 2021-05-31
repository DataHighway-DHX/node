#!/bin/bash

echo "Docker Entrypoint Bob"
echo "Node Key is ${NODE_KEY}"
echo "Chain Version is ${CHAIN_VERSION}"

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/bob \
  --bootnodes /ip4/${BOOTNODE_ENDPOINT_DEV}/tcp/30333/p2p/${BOOTNODE_ID_LOCAL} \
  --chain ${CHAIN_VERSION} \
  --bob \
  --name "${CHAIN_VERSION} Validator Bob" \
  --port 30334 \
  --ws-port 9945 \
  --rpc-port 9934 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
