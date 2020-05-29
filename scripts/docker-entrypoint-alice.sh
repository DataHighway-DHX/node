#!/bin/bash

# ./docker-prepare-local.sh

# touch ../src/chain-spec-templates/chain_spec_local_latest.json ../src/chain-definition-custom/chain_def_local_latest.json
# ../target/release/datahighway build-spec --chain local > ../src/chain-spec-templates/chain_spec_local_latest.json
# ../target/release/datahighway build-spec --chain ../src/chain-spec-templates/chain_spec_local_latest.json --raw > ../src/chain-definition-custom/chain_def_local_latest.json

echo "part124"
cat ../src/chain-spec-templates/chain_spec_local_latest.json

../target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/alice \
  --keystore-path "/tmp/polkadot-chains/alice/keys" \
  --chain ../src/chain-definition-custom/chain_def_local_latest.json \
  --node-key 88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee \
  --alice \
  --name "Validator 1" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug
