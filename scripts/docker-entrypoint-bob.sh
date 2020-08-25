#!/bin/bash

echo "Docker Entrypoint Bob"
echo "Node Key is ${NODE_KEY}"
echo "Node Env is ${NODE_ENV}"
echo "Chain Version is ${CHAIN_VERSION}"

./docker-build-chain-spec.sh

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/bob \
  --keystore-path "/tmp/polkadot-chains/bob/keys" \
  --bootnodes /ip4/${ENDPOINT_DEVELOPMENT}/tcp/30333/p2p/${BOOT_NODE_LOCAL_1} \
  --chain ../src/chain-definition-custom/chain_def_${CHAIN_VERSION}.json \
  --bob \
  --name "${NODE_ENV} Validator Bob" \
  --port 30334 \
  --ws-port 9945 \
  --rpc-port 9934 \
  --telemetry-url wss://telemetry.polkadot.io/submit/ \
  --execution=native \
  -lruntime=debug
