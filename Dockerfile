# build stage
FROM rust as builder
# create a project folder
WORKDIR /dhx/node

COPY . .

# https://substrate.dev/docs/en/knowledgebase/getting-started
RUN apt-get update && apt-get install -y build-essential wget cmake pkg-config libssl-dev llvm \
    openssl git gcc clang libclang-dev curl vim unzip screen bash \
    && curl https://getsubstrate.io -sSf | bash -s -- --fast \
    && . /root/.cargo/env \
    && PATH=$PATH:/root/.cargo/bin \
    && ./scripts/init.sh && \
    && cargo version \
    && rustc --version \
    && cargo build --release

# runtime stage
FROM rust as runtime
# set path for docker scripts in case used, to override below default entrypoint
WORKDIR /dhx/node/scripts

COPY --from=builder /dhx/node/target/release/datahighway /usr/local/bin

ENTRYPOINT ["/usr/local/bin/datahighway"]
