FROM debian:buster as build

ARG CHAIN_VERSION
ENV _CHAIN_VERSION=${CHAIN_VERSION}
RUN echo "DataHighway chain version ${_CHAIN_VERSION}"

# Install dependencies. Combine update with other dependencies on same line so we
# do not use cache and bypass updating it if we change the dependencies
RUN apt-get update \
    && apt-get install -y \
        build-essential wget git curl vim unzip screen bash \
        cmake pkg-config libssl-dev openssl gcc clang libclang-dev \
    && echo "Installed Tools"

# Install Substrate
# Run `docker build --no-cache .` to update dependencies
RUN curl https://getsubstrate.io -sSf | bash -s -- --fast \
    && . /root/.cargo/env \
    && echo "Installed Substrate"

WORKDIR /dhx

# Copy entire project.
# This layer is rebuilt when a file changes in the project directory
COPY ./pallets/ /dhx/pallets/
COPY ./runtime/ /dhx/runtime/
# COPY ./runtime/Cargo.toml /dhx/runtime/
# COPY ./runtime/build.rs /dhx/runtime/
COPY ./scripts/ /dhx/scripts/
COPY ./src/ /dhx/src/
COPY ./target/ /dhx/target/
COPY Cargo.toml /dhx/
COPY build.rs /dhx/

# Install Rust
RUN wget -O - https://sh.rustup.rs | sh -s -- -y \
    && PATH=$PATH:/root/.cargo/bin \
    && rustup update stable \
    && rustup update nightly \
    && echo "Installed Rust" \
    # Build the project.
    && rustup toolchain install nightly-2020-02-17 \
    && rustup target add wasm32-unknown-unknown --toolchain nightly-2020-02-17 \
    && rustup default nightly-2020-02-17 \
    && rustup override set nightly-2020-02-17 \
    && cargo version \
    && rustc --version \
    && cargo build --release \
    && echo "Built DataHighway Chain" \
	# Generate the chain specification JSON file from src/chain_spec.rs
	&& ./target/release/datahighway build-spec \
  	    --chain=${_CHAIN_VERSION} > ./src/chain-spec-templates/chain_spec_${_CHAIN_VERSION}.json \
	# Build "raw" chain definition for the new chain from it
	&& ./target/release/datahighway build-spec \
        --chain ./src/chain-spec-templates/chain_spec_${_CHAIN_VERSION}.json \
        --raw > ./src/chain-definition-custom/chain_def_${_CHAIN_VERSION}.json

WORKDIR /dhx/scripts

# This results in a single layer image
FROM scratch
COPY --from=build /dhx/target/ /dhx/target/
COPY --from=build /dhx/scripts/ /dhx/scripts/
COPY --from=build /dhx/src/ /dhx/src/
COPY --from=build /dhx/build.rs /dhx/Cargo.toml /dhx/
COPY --from=build /root/.cargo/ /root/.cargo/
