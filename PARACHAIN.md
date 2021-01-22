Datahighway runs as parachain collators , connecting up to Polkadot's relaychain.

During the test phase Polkadot's relaychain is called Rococo which provides the Validators.

To setup the DataHighway collator (full block authoring node)

### Run a collator node

* Fork and clone the repository

* Install or update Rust and dependencies. Build the WebAssembly binary from all code

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast && \
./scripts/init.sh && \
cargo build --release
```

* Check version and export genesis state and wasm 

```bash
./target/release/datahighway --version
./target/release/datahighway export-genesis-state --parachain-id <datahighway_registered_parachain_id> > genesis-state
./target/release/datahighway export-genesis-wasm > genesis-wasm
```

* Connect as a parachain collator node to the rococo relaychain

```bash
./target/release/datahighway --collator --parachain-id <datahighway_registered_parachain_id> --execution wasm --chain rococo
```



sources:

[Polkadot Builders Portal](https://wiki.polkadot.network/docs/en/build-parachains-rococo#rococo-v1-parachain-requirements)