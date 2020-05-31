
# FROM debian:buster AS build
FROM debian:buster

COPY . .

WORKDIR /dhx

RUN apt-get update \
    && apt-get install -y \
        build-essential wget git curl vim unzip screen bash \
        cmake pkg-config libssl-dev openssl gcc clang libclang-dev \
    && echo "Installed Tools" \
    && curl https://getsubstrate.io -sSf | bash -s -- --fast \
    && echo "Installed Substrate" \
    && wget -O - https://sh.rustup.rs | sh -s -- -y \
    && PATH=$PATH:/root/.cargo/bin \
    && rustup update stable \
    && rustup update nightly \
    && echo "Installed Rust" \
    # Build the project.
    && rustup toolchain install nightly-2020-02-17 \
    && rustup target add wasm32-unknown-unknown --toolchain nightly-2020-02-17 \
    && . ~/.cargo/env \
    && rustup default nightly-2020-02-17 \
    && rustup override set nightly-2020-02-17 \
    && cargo version \
    && rustc --version \
    && echo "Building DataHighway" \
    && cargo build --release

WORKDIR /dhx/scripts




# # FROM debian:buster AS build
# FROM debian:buster

# # FIXME - unable to use arguments or environment variables in commands since
# # get error like `error: invalid toolchain name: '"nightly-2020-02-17"'`
# # in code like `rustup default ${_RUST_VERSION} \`.
# # In the interim the data is being hard-coded

# # ARG CHAIN_VERSION
# # ARG RUST_VERSION

# # ENV _CHAIN_VERSION=${CHAIN_VERSION}
# # ENV _RUST_VERSION=${RUST_VERSION}
# # RUN echo "DataHighway chain version ${_CHAIN_VERSION}"
# # RUN echo "DataHighway chain building with Rust Nightly version ${_RUST_VERSION}"

# # Install dependencies. Combine update with other dependencies on same line so we
# # do not use cache and bypass updating it if we change the dependencies
# RUN apt-get update \
#     && apt-get install -y \
#         build-essential wget git curl vim unzip screen bash \
#         cmake pkg-config libssl-dev openssl gcc clang libclang-dev \
#     && echo "Installed Tools"

# # Install Substrate
# # Run `docker build --no-cache .` to update dependencies
# RUN curl https://getsubstrate.io -sSf | bash -s -- --fast \
#     && echo "Installed Substrate"

# # # Copy entire project.
# # # This layer is rebuilt when a file changes in the project directory
# # COPY ./pallets/* /dhx/pallets/
# # COPY ./runtime/* /dhx/runtime/
# # COPY ./runtime/Cargo.toml ./runtime/build.rs /dhx/runtime/
# # COPY ./scripts/* /dhx/scripts/
# # COPY ./src/* /dhx/src/
# # COPY build.rs Cargo.toml /dhx/
# COPY . .

# WORKDIR /dhx

# # Install Rust
# RUN wget -O - https://sh.rustup.rs | sh -s -- -y \
#     && PATH=$PATH:/root/.cargo/bin \
#     && rustup update stable \
#     && rustup update nightly \
#     && echo "Installed Rust" \
#     # Build the project.
#     && rustup toolchain install nightly-2020-02-17 \
#     && rustup target add wasm32-unknown-unknown --toolchain nightly-2020-02-17 \
#     && . ~/.cargo/env \
#     && rustup default nightly-2020-02-17 \
#     && rustup override set nightly-2020-02-17 \
#     && cargo version \
#     && rustc --version \
#     && cargo build --release \
#     && echo "Built DataHighway Chain"

# #     && rustup toolchain install ${_RUST_VERSION} \
# #     && echo "here542" \
# #     && rustup target add wasm32-unknown-unknown --toolchain ${_RUST_VERSION} \
# #     && echo "here3455" \
# #     && . ~/.cargo/env \
# #     && echo "here12345" \
# #     && rustup default ${_RUST_VERSION} \
# #     && echo "here123456" \
# #     # && rustup override set ${_RUST_VERSION} \
# #     && cargo version \
# #     && rustc --version \
# #     && cargo build --release \
# # 	# Generate the chain specification JSON file from src/chain_spec.rs
# # 	&& ./target/release/datahighway build-spec \
# #   	    --chain=${_CHAIN_VERSION} > ./src/chain-spec-templates/chain_spec_${_CHAIN_VERSION}.json \
# # 	# Build "raw" chain definition for the new chain from it
# # 	&& ./target/release/datahighway build-spec \
# #         --chain ./src/chain-spec-templates/chain_spec_${_CHAIN_VERSION}.json \
# #         --raw > ./src/chain-definition-custom/chain_def_${_CHAIN_VERSION}.json

# WORKDIR /dhx/scripts

# # FIXME - unable to resolve error when using staged builds
# # no targets specified in the manifest
# # either src/lib.rs, src/main.rs, a [lib] section, or [[bin]] section must be present

# # # This results in a single layer image
# # FROM scratch
# # COPY --from=build /dhx/target/* /dhx/target/
# # COPY --from=build /dhx/scripts/* /dhx/scripts/
# # COPY --from=build /dhx/src/* /dhx/src/
# # COPY --from=build build.rs Cargo.toml /dhx/
