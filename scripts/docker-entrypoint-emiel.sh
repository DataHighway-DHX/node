#!/bin/bash

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/emiel \
  --bootnodes /ip4/172.31.1.212/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ \
  --keystore-path "/tmp/polkadot-chains/emiel/keys" \
  --chain ../src/chain-definition-custom/chain_def_testnet_v0.1.0.json \
  --name "Emiel DataHighway Validator" \
  --port 30336 \
  --ws-port 9947 \
  --rpc-port 9934 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
