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
    && rustup toolchain install nightly-2021-03-03 \
    && rustup target add wasm32-unknown-unknown --toolchain nightly-2021-03-03 \
    && rustup default nightly-2021-03-03 \
    && rustup override set nightly-2021-03-03 \
    && cargo version \
    && rustc --version \
    && cargo update -p funty --precise 1.1.0 \
    && cargo build --release

WORKDIR /dhx/scripts
