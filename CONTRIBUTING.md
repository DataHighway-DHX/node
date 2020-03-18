# Table of contents

* [Pull Requests](#chapter-4a9b69)
* [Continuous Integration](#chapter-7a8301)
* [Linting](#chapter-c345d7)
* [Debugging](#chapter-93c645)
* [Testing](#chapter-e146ec)
* [Code Editor Configuration](#chapter-d5a9de)
* [Interact with blockchain using Polkadot.js Apps UI](#chapter-6d9058)
* [Create new runtime modules](#chapter-18873f)
* [FAQ](#chapter-f078a2)

Note: Generate a new chapter with `openssl rand -hex 3`

## Pull Requests <a id="chapter-4a9b69"></a>

All Pull Requests should first be made into the 'develop' branch, since the Github Actions CI badge build status that is shown in the README depends on the outcome of building Pull Requests from the 'develop' branch.

### Skipping CI

To skip running the CI unnecessarily for simple changes such as updating the documentation, include `[ci skip]` or `[skip ci]` in your Git commit message.

### Linting

Please apply Rust Format on your changes prior to creating a PR. See [Linting](#chapter-c345d7).

```bash
cargo +nightly fmt --all
```

Optionally run Clippy

```bash
cargo clippy --release -- -D warnings
```

Optionally run check
```
cargo check
```

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
debug::native::info!("hello {:?}", world);
```

### Detailed Debugging

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/datahighway ... \
  ... \
  -lruntime=debug
```

## Testing <a id="chapter-e146ec"></a>

### Run All Tests

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

### Run Integration Tests Only

```
cargo test -p datahighway-runtime
```

#### Run Specific Integration Tests

Example
```
cargo test -p datahighway-runtime --test cli_integration_tests_mining_tokens
```

## Continuous Integration <a id="chapter-7a8301"></a>

Github Actions are used for Continuous Integration.
View the latest [CI Build Status](https://github.com/DataHighway-DHX/node/actions?query=workflow%3ACI+branch%3Adevelop) of the 'develop' branch, from which all Pull Requests are made into the 'master' branch.

Note: We do not watch Pull Requests from the 'master' branch, as they would come from Forked repos.

Reference: https://help.github.com/en/actions/configuring-and-managing-workflows/configuring-a-workflow

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

#### Clippy and Continuous Integration (CI)

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

#### Install RustFmt

```bash
rustup component add rustfmt --toolchain nightly
```

#### Check Formating Changes that RustFmt before applying them

Check that you agree with all the formating changes that RustFmt will apply to identify anything that you do not agree with.

```bash
cargo +nightly fmt --all -- --check
```

#### Apply Formating Changes

```bash
cargo +nightly fmt --all
```

## Code Editor Configuration <a id="chapter-d5a9de"></a>

### Add Vertical Rulers in VS Code

Add the following to settings.json `"editor.rulers": [80,120]`, as recommended here https://stackoverflow.com/a/45951311/3208553

### EditorConfig

Install an [EditorConfig Plugin](https://editorconfig.org/) for your code editor to detect and apply the configuration in .editorconfig.

## Interact with blockchain using Polkadot.js Apps UI <a id="chapter-6d9058"></a>

* Interact with the node when running it:
  * Go to Polkadot.js Apps "Settings" tab at https://polkadot.js.org/apps/#/settings
  * General > remote node/endpoint to connect to > Local Node (127.0.0.1:9944)

* Important:
  * Input parameter quirk: Sometimes it is necessary to modify the value of one of the input parameters to allow you to click "Submit Transaction" (i.e. if the first arguments input value is already 0 and appears valid, but the "Submit Transaction" button appears disabled, just delete the 0 value and re-enter 0 again)
  * Prior to being able to submit extrinics at https://polkadot.js.org/apps/#/extrinsics (i.e. roaming > createNetwork()) or to view StorageMap values, it is necessary to add the Custom Types to https://polkadot.js.org/apps/#/settings/developer, which are included in [custom_types.json](./custom_types.json), otherwise the "Submit Transaction" button will not work.

### Troubleshooting

If you encounter any UI errors or any errors in the browser console using https://polkadot.js.org/apps, then you may be able to contribute to Polkadot.js Apps. If you run Polkadot.js Apps locally from your machine then the errors are easier to debug.

Follow the instructions at https://github.com/polkadot-js/apps, including cloning it, and running it.
Try to identify and fix the error, and raise an issue in that repository if necessary.  

### Create new runtime modules <a id="chapter-18873f"></a>

```bash
substrate-module-new <module-name> <author>
```

## FAQ <a id="chapter-f078a2"></a>

* Question: Why do we need to install Rust Stable and Rust Nightly?
	* Answer: In .github/workflows/rust.yml, we need to run the following,
	because Substrate builds two binaries: 1) Wasm binary of your Runtime;
	and 2) Native executable containing all your other Substrate components
	including your runtimes too. The Wasm build requires rust nightly and
	wasm32-unknown-unknown to be installed. Note that we do not use
	`rustup update nightly` since the latest Rust Nightly may break our build,
	so we must manually change this to the latest Rust Nightly version only
	when it is known to work.
		```bash
		rustup toolchain install nightly-2020-02-17
		rustup update stable
		rustup target add wasm32-unknown-unknown --toolchain nightly
		```

* Question: Why do we install a specific version of Rust Nightly in the CI?
	* Answer: Since the latest version of Rust Nightly may break our build,
	and because developers may forget to update to the latest version of Rust
	Nightly locally. So the solution is to install a specific version of
	Rust Nightly in .github/workflows/rust.yml (i.e.
	`rustup toolchain install nightly-2020-02-17`), since for example
	the latest Rust Nightly version nightly-2020-02-20 may cause our CI tests
	to fail (i.e. https://github.com/DataHighway-DHX/node/issues/32)

* Question: Why does the `SessionKeys` struct of our chain only have [babe and grandpa](https://github.com/DataHighway-DHX/node/blob/master/runtime/src/lib.rs#L94), and not [im_online and authority_discovery](https://github.com/paritytech/substrate/blob/master/bin/node/runtime/src/lib.rs#L242).
	* Answer: Since we'll be a parachain im_online and authority_discovery are not required here.

* Question: How do I install specific dependencies
	* Answer:
		```bash
		cargo install cargo-edit
		cargo add ...
		```
* Question: How do I upgrade the runtime without stopping the blockchain
	* Answer: https://www.youtube.com/watch?v=0aTnxHrV_j4&list=PLOyWqupZ-WGt3mA_d9wu74vVe0bM37-39&index=9&t=0s
