#!/bin/bash

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/charlie \
  --bootnodes /ip4/${ENDPOINT_DEVELOPMENT}/tcp/30333/p2p/${BOOT_NODE_LOCAL_1} \
  --keystore-path "/tmp/polkadot-chains/charlie/keys" \
  --chain ../src/chain-definition-custom/chain_def_${CHAIN_VERSION}.json \
  --charlie \
  --name "${NODE_ENV} Validator Charlie" \
  --port 30335 \
  --ws-port 9946 \
  --rpc-port 9934 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
