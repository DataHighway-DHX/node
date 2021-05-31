#!/bin/bash

echo "Docker Entrypoint Alice"
echo "Node Key is ${NODE_KEY}"
echo "Chain Version is ${CHAIN_VERSION}"

/usr/local/bin/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/alice \
  --chain ${CHAIN_VERSION} \
  --node-key ${NODE_KEY} \
  --alice \
  --name "${CHAIN_VERSION} Validator Alice" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
