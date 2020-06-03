FROM debian:buster

ARG CHAIN_VERSION
ENV _CHAIN_VERSION=${CHAIN_VERSION}
RUN echo "DataHighway chain version ${_CHAIN_VERSION}"

WORKDIR /dhx

# FIXME - only copy necessary files to reduce size of image, and try using intermediate stages again
COPY . .

RUN apt-get update && apt-get install -y build-essential wget cmake pkg-config libssl-dev \
    openssl git gcc clang libclang-dev curl vim unzip screen bash \
    && curl https://getsubstrate.io -sSf | bash -s -- --fast \
    && . ~/.cargo/env \
    && wget -O - https://sh.rustup.rs | sh -s -- -y \
    && PATH=$PATH:/root/.cargo/bin \
    && rustup update stable \
    && rustup update nightly \
    && rustup toolchain install nightly-2020-02-17 \
    && rustup target add wasm32-unknown-unknown --toolchain nightly-2020-02-17 \
    && rustup default nightly-2020-02-17 \
    && rustup override set nightly-2020-02-17 \
    && cargo version \
    && rustc --version \
    && cargo build --release \
	# Generate the chain specification JSON file from src/chain_spec.rs
	&& ./target/release/datahighway build-spec \
  	    --chain=${_CHAIN_VERSION} > ./src/chain-spec-templates/chain_spec_${_CHAIN_VERSION}.json \
	# Build "raw" chain definition for the new chain from it
	&& ./target/release/datahighway build-spec \
        --chain ./src/chain-spec-templates/chain_spec_${_CHAIN_VERSION}.json \
        --raw > ./src/chain-definition-custom/chain_def_${_CHAIN_VERSION}.json

WORKDIR /dhx/scripts
