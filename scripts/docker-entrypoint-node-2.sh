#!/bin/bash

# use 127.0.0.1 or testnet-harbour.datahighway.com
# bootnode node-1 is QmTci9Adif5gUXhgPUZk4bEd3U1E1Tvr1PwKBjDbL5UGn7 or
# sentry node id: QmVuryfE427VRqrqqXsGuWpwBk4g8mGXgYmnt3f1v6j78r

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/node-2 \
  --bootnodes /dns4/testnet-harbour.datahighway.com/tcp/30333/p2p/QmVuryfE427VRqrqqXsGuWpwBk4g8mGXgYmnt3f1v6j78r \
  --keystore-path "/tmp/polkadot-chains/node-2/keys" \
  --chain ../src/chain-definition-custom/chain_def_testnet_v0.1.0.json \
  --name "Example Validator Node 2" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
