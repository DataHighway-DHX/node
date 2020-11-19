#!/bin/bash

echo "Docker Entrypoint Charlie"
echo "Node Key is ${NODE_KEY}"
echo "Node Env is ${NODE_ENV}"
echo "Chain Version is ${CHAIN_VERSION}"

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/charlie \
  --bootnodes /ip4/${ENDPOINT_DEVELOPMENT}/tcp/30333/p2p/${BOOT_NODE_LOCAL_1} \
  --keystore-path "/tmp/polkadot-chains/charlie/keys" \
  --chain ../node/src/chain-definition-custom/chain_def_${CHAIN_VERSION}.json \
  --charlie \
  --name "${NODE_ENV} Validator Charlie" \
  --port 30335 \
  --ws-port 9946 \
  --rpc-port 9934 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
