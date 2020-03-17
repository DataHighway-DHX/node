# Data Highway [![GitHub license](https://img.shields.io/github/license/paritytech/substrate)](LICENSE) <a href="https://github.com/DataHighway-DHX/node/actions?query=workflow%3ACI+branch%3Adevelop" target="_blank"><img src="https://github.com/DataHighway-DHX/node/workflows/CI/badge.svg?branch=develop"></a>

The Data Highway Substrate-based blockchain node.

__WARNING__: This implementation is a proof-of-concept prototype and is not ready for production use.

# Table of contents

* [Contributing](#chapter-cb8b82)
* [Documentation](#chapter-888ccd)
* [Build and run blockchain](#chapter-5f0881)
* [Interact with blockchain using Polkadot.js Apps UI](#chapter-6d9058)
* [Maintain dependencies, rebuild, and add new runtime modules](#chapter-e16e68)
* [Debugging](#chapter-93c645)
* [Create custom blockchain configuration](#chapter-b1b53c)
* [Run multiple node PoS testnet using custom blockchain configuration](#chapter-f21efd)
* [Linting](#chapter-c345d7)

Note: Generate a new chapter with `openssl rand -hex 3`

## Contributing <a id="chapter-cb8b82"></a>

Refer to [CONTRIBUTING.md](./CONTRIBUTING.md) for contributing instructions, including:
* Pull Requests
* FAQ
* Continuous Integration

## Documentation <a id="chapter-888ccd"></a>

Refer to the [DataHighway Developer Hub](https://github.com/DataHighway-DHX/documentation).

## Build and run blockchain <a id="chapter-5f0881"></a>

### Build blockchain node

* Fork and clone the repository

* Install Rust and dependencies. Build the WebAssembly binary from all code

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast && \
./scripts/init.sh && \
cargo build --release
```

### Run blockchain node (full node)

* Remove all existing blockchain testnet database and keys

```bash
./target/release/datahighway purge-chain --dev --base-path /tmp/polkadot-chains/alice
```

* Connect to development testnet (`--chain development`)

```bash
./target/release/datahighway \
  --base-path /tmp/polkadot-chains/alice \
  --name "Data Highway Testnet" \
  --dev \
  --telemetry-url ws://telemetry.polkadot.io:1024
```

Important: Setup Custom Types prior to submitting extrinsics otherwise StorageMap values from storage will not be displayed.

Detailed logs output by prefixing the above with: `RUST_LOG=debug RUST_BACKTRACE=1`

## Interact with blockchain using Polkadot.js Apps UI <a id="chapter-6d9058"></a>

* Interact with node when running it:
  * Go to Polkadot.js Apps "Settings" tab at https://polkadot.js.org/apps/#/settings
  * General > remote node/endpoint to connect to > Local Node (127.0.0.1:9944)

* Important:
  * Input parameter quirk: Sometimes it is necessary to modify the value of one of the input parameters to allow you to click "Submit Transaction" (i.e. if the first arguments input value is already 0 and appears valid, but the "Submit Transaction" button appears disabled, just delete the 0 value and re-enter 0 again)
  * Prior to being able to submit extrinics at https://polkadot.js.org/apps/#/extrinsics (i.e. roaming > createNetwork()) or to view StorageMap values, it is necessary to add the Custom Types to https://polkadot.js.org/apps/#/settings/developer, which are included in [custom_types.json](./custom_types.json), otherwise the "Submit Transaction" button will not work.

### Troubleshooting

If you encounter any UI errors or any errors in the browser console using https://polkadot.js.org/apps, then you may be able to contribute to Polkadot.js Apps. If you run Polkadot.js Apps locally from your machine then the errors are easier to debug.

Follow the instructions at https://github.com/polkadot-js/apps, including cloning it, and running it.
Try to identify and fix the error, and raise an issue in that repository if necessary.  

## Maintain dependencies, rebuild, and add new runtime modules <a id="chapter-e16e68"></a>

### Add new runtime module

```bash
substrate-module-new <module-name> <author>
```

### Update Rust and dependencies

```bash
curl https://getsubstrate.io -sSf | bash && \
./scripts/init.sh
```

### Re-build runtime after purge chain database of all blocks.

```bash
./target/release/datahighway purge-chain --dev
cargo build --release
```

### All Tests

```bash
cargo test -p datahighway-runtime &&
cargo test -p roaming-operators &&
cargo test -p roaming-networks &&
cargo test -p roaming-organizations &&
cargo test -p roaming-network-servers &&
cargo test -p roaming-devices &&
cargo test -p roaming-routing-profiles &&
cargo test -p roaming-service-profiles &&
cargo test -p roaming-accounting-policies &&
cargo test -p roaming-agreement-policies &&
cargo test -p roaming-network-profiles &&
cargo test -p roaming-device-profiles &&
cargo test -p roaming-sessions &&
cargo test -p roaming-billing-policies &&
cargo test -p roaming-charging-policies &&
cargo test -p roaming-packet-bundles &&
cargo test -p mining-speed-boosts-configuration-token-mining &&
cargo test -p mining-speed-boosts-configuration-hardware-mining &&
cargo test -p mining-speed-boosts-rates-token-mining &&
cargo test -p mining-speed-boosts-rates-hardware-mining &&
cargo test -p mining-speed-boosts-sampling-token-mining &&
cargo test -p mining-speed-boosts-sampling-hardware-mining &&
cargo test -p mining-speed-boosts-eligibility-token-mining &&
cargo test -p mining-speed-boosts-eligibility-hardware-mining &&
cargo test -p mining-speed-boosts-lodgements-token-mining &&
cargo test -p mining-speed-boosts-lodgements-hardware-mining
```

## Integration Tests

```
cargo test -p datahighway-runtime
```

#### Specific Integration Tests

Example
```
cargo test -p datahighway-runtime --test cli_integration_tests_mining_tokens
```

### Check

```
cargo check
```

### Install Specific Dependencies

```
cargo install cargo-edit
cargo add ...
```

### Upgrade runtime

https://www.youtube.com/watch?v=0aTnxHrV_j4&list=PLOyWqupZ-WGt3mA_d9wu74vVe0bM37-39&index=9&t=0s

## Debugging <a id="chapter-93c645"></a>

### Simple Debugging

**TODO** - Replace with use of log::debug with native::debug. See https://github.com/DataHighway-DHX/node/issues/41

* Add to Cargo.toml of runtime module:
```yaml
...
    'log/std',
...
[dependencies.log]
version = "0.4.8"
```

* Add to my-module/src/lib.rs
```rust
use log::{error, info, debug, trace};
...
log::debug!("hello {:?}", world); // Only shows in terminal in debug mode
log::info!("hello {:?}", world); // Shows in terminal in release mode
```

### Detailed Debugging

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/datahighway ...
```

## Create custom blockchain configuration <a id="chapter-b1b53c"></a>

* Create latest chain specification code changes of <CHAIN_ID> (i.e. dev, local, testnet, or testnet-latest)

> Remember to build your chain first with:

```bash
cargo build --release
```

```bash
mkdir -p ./src/chain-spec-templates
./target/release/datahighway build-spec \
  --chain=testnet-latest > ./src/chain-spec-templates/chain_spec_testnet_latest.json
```

* Build "raw" chain definition for the new chain

```bash
mkdir -p ./src/chain-definition-custom
./target/release/datahighway build-spec \
  --chain ./src/chain-spec-templates/chain_spec_testnet_latest.json \
  --raw > ./src/chain-definition-custom/chain_def_testnet_v0.1.0.json
```

> Remember to purge the chain state if you change anything

## Run multiple nodes in PoS testnet using custom blockchain configuration <a id="chapter-f21efd"></a>

* Run custom Substrate-based blockchain on local machine testnet with multiple terminals:
  * Imported custom chain definition for custom testnet
  * Use default accounts Alice and Bob as the two initial authorities of the genesis configuration that have been endowed with testnet units that will run validator nodes
  * Multiple authority nodes using the Aura consensus to produce blocks

Terminal 1: Alice's Substrate-based node on default TCP port 30333 with her chain database stored locally at `/tmp/polkadot-chains/alice` and where the bootnode ID of her node is `Local node identity is: Qma68PCzu2xt2SctTBk6q6pLep6wAxRr6FpziQYwhsMCK6` (peer id), which is generated from the `--node-key` value specified below and shown when the node is running. Note that `--alice` provides Alice's session key that is shown when you run `subkey -e inspect //Alice`, alternatively you could provide the private key to that is necessary to produce blocks with `--key "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice"`. In production the session keys are provided to the node using RPC calls `author_insertKey` and `author_rotateKeys`.
If you explicitly specify a `--node-key` (i.e. `--node-key 88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee`) when you start your validator node, the logs will still display your peer id with `Local node identity is: Qxxxxxx`, and you could then include it in the chainspec.json file under "bootNodes". Also the peer id is listed when you go to view the list of full nodes and authority nodes at Polkadot.js Apps https://polkadot.js.org/apps/#/explorer/datahighway:

```bash
SKIP_WASM_BUILD= ./target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/alice \
  --keystore-path "/tmp/polkadot-chains/alice/keys" \
  --chain ./src/chain-definition-custom/chain_def_testnet_v0.1.0.json \
  --node-key 88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee \
  --alice \
  --rpc-port 9933 \
  --port 30333 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --ws-port 9944 \
  --execution=native \
  -lruntime=debug
```

When the node is started, copy the address of the node, and paste in the `bootNodes` of chain_def_testnet_v0.1.0.json.

Terminal 2: Bob's Substrate-based node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/polkadot-chains/alice`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
SKIP_WASM_BUILD= ./target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ \
  --chain ./src/chain-definition-custom/chain_def_testnet_v0.1.0.json \
  --bob \
  --rpc-port 9933 \
  --port 30334 \
  --telemetry-url ws://telemetry.polkadot.io:1024 \
  --ws-port 9944 \
  --execution=native \
  -lruntime=debug
```

* Generate session keys for Alice
```bash
$ subkey --ed25519 inspect "//Alice"
Secret Key URI `//Alice` is account:
  Secret seed:      0xabf8e5bdbe30c65656c0a3cbd181ff8a56294a69dfedd27982aace4a76909115
  Public key (hex): 0x88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee
  Account ID:       0x88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee
  SS58 Address:     5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu

$ subkey --sr25519 inspect "//Alice"//aura
Secret Key URI `//Alice//aura` is account:
  Secret seed:      0x153d8db5f7ef35f18a456c049d6f6e2c723d6c18d1f9f6c9fbee880c2a171c73
  Public key (hex): 0x408f99b525d90cce76288245cb975771282c2cefa89d693b9da2cdbed6cd9152
  Account ID:       0x408f99b525d90cce76288245cb975771282c2cefa89d693b9da2cdbed6cd9152
  SS58 Address:     5DXMabRsSpaMwfNivWjWEnzYtiHsKwQnP4aAKB85429ZQU6v

$ subkey --sr25519 inspect "//Alice"//babe
Secret Key URI `//Alice//babe` is account:
  Secret seed:      0x7bc0e13f128f3f3274e407de23057efe043c2e12d8ed72dc5c627975755c9620
  Public key (hex): 0x46ffa3a808850b2ad55732e958e781146ed1e6436ffb83290e0cb810aacf5070
  Account ID:       0x46ffa3a808850b2ad55732e958e781146ed1e6436ffb83290e0cb810aacf5070
  SS58 Address:     5Dfo9eF9C7Lu5Vbc8LbaMXi1Us2yi5VGTTA7radKoxb7M9HT

$ subkey --sr25519 inspect "//Alice"//imonline
Secret Key URI `//Alice//imonline` is account:
  Secret seed:      0xf54dc00d41d0ea7929ac00a08ed1e111eb8c35d669b011c649cea23997f5d8d9
  Public key (hex): 0xee725cf87fa2d6f264f26d7d8b84b1054d2182cdcce51fdea95ec868be9d1e17
  Account ID:       0xee725cf87fa2d6f264f26d7d8b84b1054d2182cdcce51fdea95ec868be9d1e17
  SS58 Address:     5HTME6o2DqEuoNCxE5263j2dNzFGxspeP8wswenPA3WerfmA

$ subkey --ed25519 inspect "//Alice"//grandpa
Secret Key URI `//Alice//grandpa` is account:
  Secret seed:      0x03bee0237d4847732404fde7539e356da44bce9cd69f26f869883419371a78ab
  Public key (hex): 0x6e2de2e5087b56ed2370359574f479d7e5da1973e17ca1b55882c4773f154d2f
  Account ID:       0x6e2de2e5087b56ed2370359574f479d7e5da1973e17ca1b55882c4773f154d2f
  SS58 Address:     5EZAkmxARDqRz5z5ojuTjacTs2rTd7WRL1A9ZeLvwgq2STA2
```

* Add session keys for account(s) to be configured as authorities (validators). Run cURL to insert session key for each key type (i.e. "aura"), by providing the associated secret key, and associated Public key (hex) 
```bash
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["aura", "//Alice//aura", "0x408f99b525d90cce76288245cb975771282c2cefa89d693b9da2cdbed6cd9152"],"id":1 }' 127.0.0.1:9933
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["babe", "//Alice//babe", "0x46ffa3a808850b2ad55732e958e781146ed1e6436ffb83290e0cb810aacf5070"],"id":1 }' 127.0.0.1:9933
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["imon", "//Alice//imonline", "0xee725cf87fa2d6f264f26d7d8b84b1054d2182cdcce51fdea95ec868be9d1e17"],"id":1 }' 127.0.0.1:9933
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["gran", "//Alice//grandpa", "0x6e2de2e5087b56ed2370359574f479d7e5da1973e17ca1b55882c4773f154d2f"],"id":1 }' 127.0.0.1:9933
```

* Check that the output from each cURL request is `{"jsonrpc":"2.0","result":"0x...","id":1}` and that the following folder is not empty /tmp/polkadot-chains/alice/keys

* Check that the chain is finalizing blocks (i.e. finalized is non-zero `Idle (1 peers), best: #58 (0x1aac…05f1), finalized #0 (0x3947…2716), ⬇ 0 ⬆ 0`)

* Configure settings to view at [Polkadot.js Apps](#chapter-6d9058)

* View on [Polkadot Telemetry](https://telemetry.polkadot.io/#list/DataHighway%20Local%20PoA%20Testnet%20v0.1.0)

* Distribute the custom chain definition (i.e. chain_def_testnet_v0.1.0.json) to allow others to synchronise and validate if they are an authority

## Linting<a id="chapter-c345d7"></a>

### Clippy

#### Run Manually

##### Stable
```rust
cargo clippy --release -- -D warnings
```

##### Nightly

The following is a temporary fix. See https://github.com/rust-lang/rust-clippy/issues/5094#issuecomment-579116431

```
rustup component add clippy --toolchain nightly-2020-02-17-x86_64-unknown-linux-gnu
rustup component add clippy-preview --toolchain nightly-2020-02-17-x86_64-unknown-linux-gnu
cargo +nightly-2020-02-17 clippy-preview -Zunstable-options
```

#### Continuous Integration (CI)

Clippy is currently disabled in CI for the following reasons.

A configuration file clippy.toml to accept or ignore different types of Clippy errors
is not available (see https://github.com/rust-lang/cargo/issues/5034). So it
currenty takes a long time to manually ignore each type of Clippy error in each file.

To manually ignore a clippy error it is necessary to do the following,
where `redundant_pattern_matching` is the clippy error type in this example:

```rust
#![cfg_attr(feature = "cargo-clippy", allow(clippy::redundant_pattern_matching))]
```

### Rust Format

[RustFmt](https://github.com/rust-lang/rustfmt) should be used for styling Rust code.
The styles are defined in the rustfmt.toml configuration file, which was generated by running `rustfmt --print-config default rustfmt.toml` and making some custom tweaks according to https://rust-lang.github.io/rustfmt/

### Install RustFmt

```bash
rustup component add rustfmt --toolchain nightly
```

### Check Formating Changes that RustFmt before applying them

Check that you agree with all the formating changes that RustFmt will apply to identify anything that you do not agree with.

```bash
cargo +nightly fmt --all -- --check
```

### Apply Formating Changes

```bash
cargo +nightly fmt --all
```

### Add Vertical Rulers in VS Code

Add the following to settings.json `"editor.rulers": [80,120]`, as recommended here https://stackoverflow.com/a/45951311/3208553

### EditorConfig

Install an [EditorConfig Plugin](https://editorconfig.org/) for your code editor to detect and apply the configuration in .editorconfig.
