#!/bin/bash

./docker-prepare-local.sh

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/charlie \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ \
  --keystore-path "/tmp/polkadot-chains/charlie/keys" \
  --chain ../src/chain-definition-custom/chain_def_local_v0.1.0.json \
  --charlie \
  # --name "Validator 3" \
  --port 30335 \
  --ws-port 9946 \
  --rpc-port 9934 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
