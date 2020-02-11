#!/usr/bin/env bash

set -eux

# Enable warnings about unused extern crates
export RUSTFLAGS=" -W unused-extern-crates"

# Install rustup and the specified rust toolchain.
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain=$RUST_TOOLCHAIN -y
# Load cargo environment. Specifically, put cargo into PATH.
source ~/.cargo/env

cargo install cargo-vendor
cargo vendor

rustc --version
rustup --version
cargo --version
echo $TARGET

case $TARGET in
	"native")

		sudo apt-get -y update
		sudo apt-get install -y cmake libclang-dev
		./scripts/init.sh
    cargo build --release &&
    cargo test -p node-runtime &&
    cargo test -p roaming-operators &&
    cargo test -p roaming-networks &&
    cargo test -p roaming-organizations &&
    cargo test -p roaming-network-servers &&
    cargo test -p roaming-devices &&
    cargo test -p roaming-routing-profiles &&
    cargo test -p roaming-service-profiles &&
    cargo test -p roaming-accounting-policies &&
    cargo test -p roaming-agreement-policies &&
    cargo test -p roaming-network-profiles &&
    cargo test -p roaming-device-profiles &&
    cargo test -p roaming-sessions &&
    cargo test -p roaming-billing-policies &&
    cargo test -p roaming-charging-policies &&
    cargo test -p roaming-packet-bundles
		;;

	"wasm")

		# Install prerequisites and build all wasm projects
		cargo install pwasm-utils-cli --bin wasm-prune --force

#		cd ./contracts/balances && ./build.sh && cargo test
#		cd ./contracts/cash && ./build.sh && cargo test
#		cd ../commitment && cargo test
#		cd ../deposit && cargo test
#		cd ../predicate && cargo test
		;;

esac
