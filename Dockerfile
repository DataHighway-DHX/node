# FROM rust:1.42 as builder

# RUN cd ~; pwd

# RUN mkdir -p /usr/src/datahighway-dhx
# COPY . /usr/src/datahighway-dhx
# WORKDIR /usr/src/datahighway-dhx

# RUN touch abc.md
# RUN echo "this is a new line" >> abc.md 

# RUN cat /usr/src/datahighway-dhx/abc.md

# FROM alpine:latest AS bundle
# COPY --from=builder /usr/src/datahighway-dhx/abc.md /usr/src/abc.md
# RUN echo "hi"
# RUN pwd
# RUN cd /usr/src/ && cat abc.md
# WORKDIR /usr/src/datahighway-dhx

# # .cargo/bin/datahighway



# Build stage
FROM rust:1.42 as builder

ARG CHAINSPEC_VERSION=0.1.0
ARG RUST_NIGHTLY_VERSION=2020-02-17

RUN rustup toolchain install nightly-${RUST_NIGHTLY_VERSION} && \
    rustup target add wasm32-unknown-unknown --toolchain nightly-${RUST_NIGHTLY_VERSION} && \
    rustup default nightly-${RUST_NIGHTLY_VERSION}

    # && rustup target add wasm32-unknown-unknown --toolchain nightly-${RUST_NIGHTLY_VERSION} \
    # && rustup target add x86_64-unknown-linux-musl --toolchain nightly-2020-02-17 \
    # && rustup target add x86_64-unknown-linux-musl --toolchain 1.42.0-x86_64-unknown-linux-gnu \
    # && curl https://getsubstrate.io -sSf | bash -s -- --fast

FROM builder as build1
# COPY . .
RUN curl https://getsubstrate.io -sSf | bash -s -- --fast && \
  . /usr/local/cargo/env

# RUN USER=root cargo new datahighway-dhx
# WORKDIR /usr/src/datahighway-dhx
# COPY Cargo.toml Cargo.lock ./runtime/
# COPY . .
# COPY /home/.cargo /home/.cargo
# RUN cargo build --release --target x86_64-unknown-linux-musl

# # Compile source and static linking using MUSL
# COPY . .
# RUN cargo install xargo && \
# RUN rustup target add x86_64-unknown-linux-musl && \
#   rustup target add x86_64-unknown-linux-musl --toolchain nightly-2020-02-17 && \
#   rustup target add x86_64-unknown-linux-musl --toolchain 1.42.0-x86_64-unknown-linux-gnu && \
#   cargo install --path . --locked

# FROM builder as build2

WORKDIR /usr/src/datahighway-dhx
COPY . /usr/src/datahighway-dhx
# Rust Nightly
COPY --from=builder /usr/local/cargo /usr/local/cargo

RUN . /usr/local/cargo/env && \
  rustup toolchain list && \
  rustup update nightly && \
  rustup update stable && \
#   rustup target add wasm32-unknown-unknown --toolchain nightly-${RUST_NIGHTLY_VERSION} && \
#   rustup default nightly-${RUST_NIGHTLY_VERSION} && \
  rustup target add wasm32-unknown-unknown --toolchain nightly && \
  rustup default stable && \
  cargo build --release
#   cargo install --path . --locked
#   CC_x86_64_unknown_linux_musl="x86_64-linux-musl-gcc" xargo install --target x86_64-unknown-linux-musl --path . --locked
#   CC_x86_64_unknown_linux_musl="x86_64-linux-musl-gcc" cargo install --target x86_64-unknown-linux-musl --path . --locked

# Generate the chain specification JSON file from src/chain_spec.rs and then
# Build "raw" chain definition for the new chain from it
RUN /usr/local/cargo/bin/datahighway build-spec --chain=testnet-latest > /var/tmp/chain_spec_testnet_latest.json && \
    /usr/local/cargo/bin/datahighway build-spec --chain /var/tmp/chain_spec_testnet_latest.json --raw > /var/tmp/chain_def_testnet_v${CHAINSPEC_VERSION}.json

# Bundle stage
FROM alpine:latest AS bundle
COPY --from=builder /usr/local/cargo/bin/datahighway /usr/local/bin/datahighway
COPY --from=builder /var/tmp/chain_def_testnet_v${CHAINSPEC_VERSION}.json /usr/local/chain_def_testnet_v${CHAINSPEC_VERSION}.json
ENV PATH "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
WORKDIR /usr/local/bin
RUN ./datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /var/tmp/polkadot-chains/alice \
  --keystore-path "/var/tmp/polkadot-chains/alice/keys" \
  --chain ../chain-definition-custom/chain_def_local_v${CHAINSPEC_VERSION}.json \
  --node-key 88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee \
  --alice \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --execution=native \
  -lruntime=debug