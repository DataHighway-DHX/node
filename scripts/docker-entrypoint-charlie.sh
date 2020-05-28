#!/bin/bash

# ./docker-prepare-local.sh

# touch ../src/chain-spec-templates/chain_spec_local_latest.json ../src/chain-definition-custom/chain_def_local_v0.1.0.json
# ../target/release/datahighway build-spec --chain local > ../src/chain-spec-templates/chain_spec_local_latest.json
# ../target/release/datahighway build-spec --chain ../src/chain-spec-templates/chain_spec_local_latest.json --raw > ../src/chain-definition-custom/chain_def_local_v0.1.0.json

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/charlie \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ \
  --keystore-path "/tmp/polkadot-chains/charlie/keys" \
  --chain ../src/chain-definition-custom/chain_def_local_v0.1.0.json \
  --charlie \
  --name "Validator 3" \
  --port 30335 \
  --ws-port 9946 \
  --rpc-port 9934 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
