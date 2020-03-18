FROM debian:buster

WORKDIR /dhx

RUN apt-get update && apt-get install -y build-essential wget cmake pkg-config libssl-dev openssl git clang libclang-dev curl vim unzip screen bash \
	&& wget -O - https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2020-02-17 \
	&& curl https://sh.rustup.rs -sSf | sh -s -- -y \
	&& PATH=$PATH:/root/.cargo/bin \
	&& rustup update stable && rustup toolchain install nightly-2020-02-17 && rustup target add wasm32-unknown-unknown --toolchain nightly-2020-02-17 \
	&& cargo version 

COPY . .

RUN PATH=$PATH:/root/.cargo/bin \
	&& curl https://getsubstrate.io -sSf | bash -s -- --fast \
	&& ./scripts/init.sh \
  && cargo build --release \
	# Generate the chain specification JSON file from src/chain_spec.rs
	&& mkdir -p ./src/chain-spec-templates \
	&& ./target/release/datahighway build-spec \
  	--chain=testnet-latest > ./src/chain-spec-templates/chain_spec_testnet_latest.json \
	# Build "raw" chain definition for the new chain from it
	&& mkdir -p ./src/chain-definition-custom \
	&& ./target/release/datahighway build-spec \
  	--chain ./src/chain-spec-templates/chain_spec_testnet_latest.json \
  	--raw > ./src/chain-definition-custom/chain_def_testnet_v0.1.0.json

WORKDIR /dhx/scripts
