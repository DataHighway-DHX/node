#!/bin/bash

../target/release/datahighway --base-path /tmp/alice   --chain=local   --alice   --node-key 0000000000000000000000000000000000000000000000000000000000000001   --telemetry-url ws://telemetry.polkadot.io:1024   --validator
