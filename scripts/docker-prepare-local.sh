#!/usr/bin/env bash

set -e

rm -rf /tmp/polkadot-chains/alice /tmp/polkadot-chains/bob /tmp/polkadot-chains/charlie

rm -rf ../src/chain-spec-templates/chain_spec_local_latest.json ../src/chain-definition-custom/chain_def_local_v0.1.0.json

touch ../src/chain-spec-templates/chain_spec_local_latest.json ../src/chain-definition-custom/chain_def_local_v0.1.0.json

../target/release/datahighway build-spec --chain local > ../src/chain-spec-templates/chain_spec_local_latest.json
../target/release/datahighway build-spec --chain ../src/chain-spec-templates/chain_spec_local_latest.json --raw > ../src/chain-definition-custom/chain_def_local_v0.1.0.json
