#!/bin/bash

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/charlie \
  --bootnodes /ip4/172.31.1.212/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ \
  --keystore-path "/tmp/polkadot-chains/charlie/keys" \
  --chain ../src/chain-definition-custom/chain_def_testnet_v0.1.0.json \
  --charlie \
  # --name "Validator 3" \
  --port 30335 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
