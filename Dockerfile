FROM debian:buster

ENV _CHAIN_VERSION=${CHAIN_VERSION}
ENV _RUST_VERSION=${RUST_VERSION}
RUN echo "DataHighway chain version ${_CHAIN_VERSION}"
RUN echo "DataHighway chain building with Rust Nightly version ${_RUST_VERSION}"

WORKDIR /dhx

COPY . .

RUN apt-get update && apt-get install -y build-essential wget cmake pkg-config libssl-dev \
    openssl git gcc clang libclang-dev curl vim unzip screen bash \
    && curl https://getsubstrate.io -sSf | bash -s -- --fast \
    && . ~/.cargo/env \
    && wget -O - https://sh.rustup.rs | sh -s -- -y --default-toolchain ${_RUST_VERSION} \
    && PATH=$PATH:/root/.cargo/bin \
    && rustup update stable \
    && rustup update nightly \
    && rustup toolchain install ${_RUST_VERSION} \
    && rustup target add wasm32-unknown-unknown --toolchain ${_RUST_VERSION} \
    && rustup default ${_RUST_VERSION} \
    && rustup override set ${_RUST_VERSION} \
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
