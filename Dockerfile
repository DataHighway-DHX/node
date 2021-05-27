# build stage
FROM rust as builder
# create a project folder
WORKDIR /dhx/node

COPY . .

RUN apt-get update && apt-get install -y build-essential wget cmake pkg-config libssl-dev \
    openssl git gcc clang libclang-dev curl vim unzip screen bash \
    && curl https://getsubstrate.io -sSf | bash -s -- --fast \
    && wget -O - https://sh.rustup.rs | sh -s -- -y \
    && PATH=$PATH:/root/.cargo/bin \
    && rustup update stable \
    && rustup update nightly \
    && rustup toolchain install nightly-2021-03-10 \
    && rustup target add wasm32-unknown-unknown --toolchain nightly-2021-03-10 \
    && rustup default nightly-2021-03-10 \
    && rustup override set nightly-2021-03-10 \
    && cargo version \
    && rustc --version \
    && cargo build --release

# runtime stage
FROM rust as runtime
# set path for docker scripts in case used, to override below default entrypoint
WORKDIR /dhx/node/scripts

COPY --from=builder /dhx/node/target/release/datahighway /usr/local/bin

ENTRYPOINT ["/usr/local/bin/datahighway"]
