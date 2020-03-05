FROM debian:buster AS development

WORKDIR /dhx
COPY . /dhx
RUN apt-get update && apt-get install -y build-essential wget cmake pkg-config libssl-dev openssl git clang libclang-dev curl vim unzip screen \
	&& wget -O - https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2020-02-17 \
	&& curl https://sh.rustup.rs -sSf | sh -s -- -y \
	&& echo 'PATH="$/root/.cargo/bin:$PATH";' >> ~/.bash_profile && . ~/.bash_profile && . /root/.cargo/env \
	&& rustup update stable && rustup toolchain install nightly-2020-02-17 && rustup target add wasm32-unknown-unknown --toolchain nightly-2020-02-17 \
	&& cargo version \
	&& cargo build --release

WORKDIR /dhx/scripts
ENTRYPOINT ["./docker-entrypoint.sh"]


