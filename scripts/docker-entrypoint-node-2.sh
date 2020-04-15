#!/bin/bash

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-2 \
  --bootnodes /dns4/testnet-frankfurt-v0.1.0-alpha.datahighway.com/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ \
  --keystore-path "/tmp/polkadot-chains/node-2/keys" \
  --chain ../src/chain-definition-custom/chain_def_testnet_v0.1.0.json \
  --name "Example Validator Node 2" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
