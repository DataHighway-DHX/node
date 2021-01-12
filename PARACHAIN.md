Datahighway runs as a parachain collator, connecting up to Polkadot's relaychain.

During the test phase Polkadot's relaychain is called Rococo which provides the Validators.

To setup the DataHighway collator (full block authoring node)

```
cd node
cargo build --release
./target/release/datahighway --version
./target/release/datahighway export-genesis-state --parachain-id <datahighway_registered_parachain_id> > genesis-state
./target/release/datahighway export-genesis-wasm > genesis-wasm
./target/release/datahighway --collator --parachain-id <datahighway_registered_parachain_id> --execution wasm --chain rococo
```

sources:

[Polkadot Builders Portal](https://wiki.polkadot.network/docs/en/build-parachains-rococo#rococo-v1-parachain-requirements)