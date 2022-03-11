# Table of contents

* [Pull Requests](#chapter-4a9b69)
* [Continuous Integration](#chapter-7a8301)
* [Linting](#chapter-c345d7)
* [Debugging](#chapter-93c645)
* [Testing](#chapter-e146ec)
* [Code Editor Configuration](#chapter-d5a9de)
* [Create new runtime modules](#chapter-18873f)
* [FAQ](#chapter-f078a2)
* [Technical Support](#chapter-c00ab7)

Note: Generate a new chapter with `openssl rand -hex 3`

## Pull Requests <a id="chapter-4a9b69"></a>

All Pull Requests should first be made into the 'develop' branch, since the Github Actions CI badge build status that is shown in the README depends on the outcome of building Pull Requests from the 'develop' branch.

### Skipping CI

To skip running the CI unnecessarily for simple changes such as updating the documentation, include `[ci skip]` or `[skip ci]` in your Git commit message.

### Linting

Check with Rust Format. Note: If you need a specific version of it replace `+nightly` with say `+nightly-2021-12-15`
```
cargo +nightly fmt --all -- --check
```

If you wish to apply Rust Format on your changes prior to creating a PR. See [Linting](#chapter-c345d7).

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
use log::{error, warn, info, debug, trace};
...
log::debug!("hello {:?}", world); // Only shows in terminal in debug mode
log::info!("hello {:?}", world); // Shows in terminal in release mode
```

Note: The use of `debug::native::info!("hello {:?}", world);` does not appear to work anymore since Substrate updates in Feb 2021.

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
cargo test -p mining-setting-token &&
cargo test -p mining-setting-hardware &&
cargo test -p mining-rates-token &&
cargo test -p mining-rates-hardware &&
cargo test -p mining-sampling-token &&
cargo test -p mining-sampling-hardware &&
cargo test -p mining-eligibility-token &&
cargo test -p mining-eligibility-hardware &&
cargo test -p mining-claims-token &&
cargo test -p mining-claims-hardware
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
View the latest [CI Build Status](https://github.com/DataHighway-DHX/node/actions?query=branch%3Adevelop) of the 'develop' branch, from which all Pull Requests are made into the 'master' branch.

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
rustup component add clippy --toolchain nightly-2021-12-15-x86_64-unknown-linux-gnu
rustup component add clippy-preview --toolchain nightly-2021-12-15-x86_64-unknown-linux-gnu
cargo +nightly-2021-12-15 clippy-preview -Zunstable-options
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
rustup component add rustfmt --toolchain nightly-2021-12-15-x86_64-unknown-linux-gnu
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
		rustup toolchain install nightly-2021-12-15
		rustup update stable
		rustup target add wasm32-unknown-unknown --toolchain nightly
		```

* Question: Why do we install a specific version of Rust Nightly in the CI?
	* Answer: Since the latest version of Rust Nightly may break our build,
	and because developers may forget to update to the latest version of Rust
	Nightly locally. So the solution is to install a specific version of
	Rust Nightly in .github/workflows/rust.yml (i.e.
	`rustup toolchain install nightly-2021-12-15`), since for example
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
    * Additionally read both of these as comments are scattered:
        * Knowledgebase https://substrate.io/docs/en/knowledgebase/runtime/upgrades#runtime-versioning * Substrate API Docs code comments https://substrate.io/rustdocs/v3.0.0/substrate_test_runtime_client/sc_executor/struct.RuntimeVersion.html#structfield.spec_version
        * Other comments in Substrate codebase say:
            * bug fixes
                * should result in an increment of spec_version and possibly authoring_version,
                * absolutely not impl_version since they change the semantics of the runtime.
            * runtime behavior changes
                * increment spec_version and set impl_version to 0
            * only runtime implementation changes and behavior does not change
                * then leave spec_version as is and increment impl_version.

* Question: How may I debug and contribute to fixing UI errors or any errors in the browser console that I encounter when using Polkadot.js Apps https://polkadot.js.org/apps?
	* Answer: If you run Polkadot.js Apps locally from your machine then the errors are easier to debug. Follow the instructions at https://github.com/polkadot-js/apps, including cloning it, and running it. Try to identify and fix the error, and raise an issue in that repository if necessary.

* Question: How do I stop and remove all the Docker containers and images?
	* Answer: Run `./scripts/docker-clean.sh`
	* **WARNING**: This stops and removes **all** your Docker containers and images, not just DataHighway relates ones.

* Question: How to access the Docker container of a running node and run shell commands?
	* Answer: `docker exec -it node_alice_1 /bin/bash`, where `node_alice_1` is the Container Name that is shown when you run `docker ps -a`.

* Question: How do I restart the testnet Docker containers (including each chain databases)?
	* Answer: Run the following, where `node_alice_1` is a Container Name that is shown when you run `docker ps -a`.
		```bash
		docker stop node_alice_1 node_bob_1 node_charlie_1
		docker rm node_alice_1 node_bob_1 node_charlie_1
		docker-compose --verbose up -d
		docker-compose logs -f
		```

* Question: Why can't I syncronize my node?
	* Answer: Run `./scripts/docker-clean.sh` before starting them again with either `docker-compose up` or `docker-compose --verbose up -d; docker-compose logs -f`, incase a cached image is still being used locally
	* **WARNING**: This stops and removes **all** your Docker containers and images, not just DataHighway relates ones.

* Question: How do I run two nodes on the same host machine?
	* Answer:
		* Refer to "Example "local" PoS testnet (with multiple nodes)" in [EXAMPLES](./EXAMPLES.md).

* Question: Why I try to connect to my Substrate node usig Polkadot.js, by going to https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944, why do I get error `WebSocket connection to 'ws://127.0.0.1:9944/' failed: Unknown reason, API-WS: disconnected from ws://127.0.0.1:9944: 1006:: Abnormal Closure`
	* Answer:
		* Try using a different web browser. Brave may not work, however Chrome might. Try running Polkadot.js app locally instead. See https://stackoverflow.com/questions/45572440/how-to-access-an-insecure-websocket-from-a-secure-website

* Question: If I update to the latest version of Substrate 'master' branch instead of just stable. How resolve an error like:
```
error[E0034]: multiple applicable items in scope
   --> /Users/me/.cargo/registry/src/github.com-1ecc6299db9ec823/bitvec-0.20.1/src/order.rs:476:7
    |
476 |             R::BITS,
    |                ^^^^ multiple `BITS` found
```
    * Answer: Try downgrading one of the dependencies that are mentioned with say `cargo update -p funty --precise 1.1.0`. Do a search of Substrate Technical on Element to see where other's have asked the same question.

* Question: How resolve error like `None => 1.into() // Default ^^^^ the trait `From<i32>` is not implemented for <T as Config>::MiningClaimsTokenClaimAmount`
    * Answer: It is not necessary to add `From<i32>` to your trait to get it to work, simply replace `None => 1.into()` with `None => 1u32.into()` since types are `i32` by default
```
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningClaimsTokenClaimAmount: Parameter + ... + From<i32>;
```

* Question: Why am I getting an unknown error like `Chain does not have enough staking candidates to operate. Era Some(0)`.
    * Answer: You may have the wrong 'order' of pallets in `construct_runtime` (i.e. System first). They need to also match that used in the Substrate codebase

* Question: Why are there multiple runtimes (i.e. testnet_runtime, and mainnet_runtime)?
    * Answer: Because we have a different SS58 Address Prefix for each of those chains and it is no longer configurable in the chain_spec in Substrate 3 like it was in Substrate 2. Testnet is 42 (Substrate default), and Mainnet is 33.

* Question: Why won't the blocks finalize?
    * Answer: When we updated from Substrate 2 to Substrate 3, we added ImOnline and AuthorityDiscover. So now it is necessary to be running at least 4x nodes (i.e. Alice, Bob, Charlie, Dave, Eve) before it will start **finalizing* blocks.

* Question: When using Docker you get error building after modifying docker-compose: `FileNotFoundError: [Errno 2] No such file or directory`?
    * Answer: Try running `rm -rf target/rls/debug/`

* Question: When using Docker you get error building like: `Cannot start service alice: OCI runtime create failed`
    * Answer: Try running `./scripts/docker-clean.sh` (beware this deletes all Docker containers, images, and cache for all your projects, not just Datahighway), and then `rm -rf ./target/rls/debug` a few times until it no longer says `Directory not empty`

* Question: When using Docker you get error building like: `Compiling parity-multiaddr v0.7.2 error[E0308]: mismatched types --> /root/.cargo/registry/src/github.com-1ecc6299db9ec823/parity-multiaddr-0.7.2/src/onion_addr.rs:23:9`
    * Answer: Try use an older version of Rust Nightly, and to set Nightly as the default

* Question: When using Docker you get error running docker-compose: `Creating node_alice_1 ... error compose.parallel.feed_queue: Pending: set() ERROR: for node_alice_1  Cannot start service alice: OCI runtime create failed: container_linux.go:349: starting container process caused "exec: \"./docker-entrypoint-alice.sh\": stat ./docker-entrypoint-alice.sh: no such file or directory": unknown`
    * Answer: It's likely the last time you tried to build the Docker container it failed, so you need to delete the container and possibly the image and cache too and try again.

* Question: When using Docker you get error: `FileNotFoundError: [Errno 2] No such file or directory: '/Users/ls/code/src/DataHighway-com/node/target/rls/debug/deps/save-analysis/libsc_executor_common-f236f3ddcd6862b3.json'`
    * Answer: Try run `rm -rf ./target/rls/debug` a few times until it no longer says `Directory not empty`

* Quesion: If I am using an Apple ARM (M1) processor instead of an Apple Intel processor, it gives warnings like `warning: toolchain 'nightly-2021-12-15-x86_64-unknown-linux-gnu' may not be able to run on this system.` and you are unable to install it with `x86_64-unknown-linux-gnu`, what may I need to do?
    * Answer: Try using `aarch64-apple-darwin` instead, e.g.
```
softwareupdate --install-rosetta
xcode-select --install
mkdir homebrew && curl -L https://github.com/Homebrew/brew/tarball/master | tar xz --strip 1 -C homebrew
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> /Users/ls2/.profile
eval "$(/opt/homebrew/bin/brew shellenv)"
brew update

rustup toolchain install nightly-2021-12-15-aarch64-apple-darwin
rustup component add rustfmt --toolchain nightly-2021-12-15-aarch64-apple-darwin
cargo +nightly-2021-12-15-aarch64-apple-darwin fmt --all -- --check
```

* Question: Why do I get an error `1002: Verification Error: Execution: Could not convert parameter 'tx' between node and runtime: Error decoding field Call:: MiningEligibilityProxy.0 RuntimeApi` when in polkadot.js.apps when I try to 'Add Item' with data to a Vec?
    * Ans: Because there is a type mismatch, perhaps the first variable requires a `Balance` instead of `u32` in custom_types.json or similar.

* Question: How to debug when running tests?
    * Ans: Run tests with this `SKIP_WASM_BUILD=1 RUST_LOG=runtime=debug` in front, i.e. `SKIP_WASM_BUILD=1 RUST_LOG=runtime=debug cargo +nightly-2021-12-15 test -p datahighway-runtime`, and use `println!` where you want to log the output (i.e. `println!("claim duration {:#?}", claim_duration);`), as `debug::info!` not work in tests. Try using `info!` (have not tried yet)

* Question: Why can't I connect my node to telemetry?
    * Ans: Try use these flags when running your node `--telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' --unsafe-ws-external --unsafe-rpc-external --rpc-cors=all --rpc-methods=Unsafe`

* Question: What is the Peer ID and the `--node-key`?
    * Ans: See the documentation here https://substrate.io/docs/en/knowledgebase/integrate/subkey#generating-node-keys. Run the command `subkey generate-node-key --file node-key` to generate and output to the screen a Peer ID for that you may share publicly to the list of bootnodes that validators may connect to. It also generates a file 'node-key' that contains the node's libp2p key that you provide as the value of `--node-key` when starting that validator bootnode, but you should keep the `--node-key` private because if someone else starts their node with the same `--node-key` that you're using then you might get slashed.

* Question: Why do I get this error when trying to run a node on chain "local" `Error: Service(Network(The same bootnode (`/ip4/127.0.0.1/tcp/30333`) is registered with two different peer ids: 12D3KooWKS7jU8ti7S5PDqCNWEj692eUSK3DLssHNwTQsto9ynVo and 12D3KooWC92KaQrzxLa3xk7yVJwCCs9vMGndt23dZAtMoR1aQc3V))`?
    * Ans: It is likely that you have run the first node with the following Node Key, where chain_def_local.json was build on your local machine, and the 2nd node was run as shown below and tries to connect to `QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ`, which is actually the Peer ID for DataHighway Harbour Testnet, when you should instead be using the Peer ID of `12D3KooWKS7jU8ti7S5PDqCNWEj692eUSK3DLssHNwTQsto9ynVo` for DataHighway Local Testnet that has already been included in chain_spec.rs genesis configuration such that the `--bootnodes` flag does not need to be specified.

First node
```
./target/release/datahighway --validator \
  ...
  --node-key 88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee
  --chain ./res/chain_def_local.json \
  --name "Local Validator Node 1" \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  ...
```

Second node

```
./target/release/datahighway --validator \
  ...
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWKS7jU8ti7S5PDqCNWEj692eUSK3DLssHNwTQsto9ynVo \
  --chain ./res/chain_def_local.json \
  --name "Local Validator Node 2" \
  --port 30334 \
  --ws-port 9945 \
  --rpc-port 9934 \
  ...
```

* Question: Why do I get this error when trying to run a node on chain "local"
```
ERROR tokio-runtime-worker sync: Bootnode with peer id `12D3KooWKS7jU8ti7S5PDqCNWEj692eUSK3DLssHNwTQsto9ynVo` is on a different chain (our genesis: 0x0aa8…cdf9 theirs: 0x7787…87ed)
```
    * Ans: Check that you have added all necessary flags in your command, and if you've used multiple lines make sure there are no missing newline characters `\` so that it recognises the `--chain ...` that you've provided

* Question: When I run a local network using chain "local" by connecting the peer nodes using `--chain local`, why doesn't it generate blocks like it does when using `--chain ./res/chain_def_local.json` and finalise blocks after running at least five nodes, where the chain_def_local.json file that is used was built on the same machine we are running the command from?
    * Ans: Check that you have added all necessary flags in your command, and if you've used multiple lines make sure there are no missing newline characters `\` so that it recognises the `--chain ...` that you've provided

* Question: How to overcome this error when updating my Substrate-based chain to a more recent commit of Substrate?

<<<<<<< HEAD
After updating the DataHighway-DHX/node repository from [Substrate commit 027368fe34e9a57ead752d4f900db6b5f85352e6](https://github.com/paritytech/substrate/commits/master?before=9b5e654074655737ac041d059d88bfd68593b069+245&branch=master) to Substrate commit dfe52f7de12a7e308c0bc3a25e211b9cc4d61871 in this DataHighway commit https://github.com/DataHighway-DHX/node/pull/215/commits/0a56ac0b6bb73b6faac8dcb0a533768dfb644f39 by doing a search and replace for `027368fe34e9a57ead752d4f900db6b5f85352e6` and replacing it with contents like the following, as well as applying any other changes as necessary:
=======
After updating the DataHighway-DHX/node repository from [Substrate commit 027368fe34e9a57ead752d4f900db6b5f85352e6](https://github.com/paritytech/substrate/commits/master?before=9b5e654074655737ac041d059d88bfd68593b069+245&branch=master) to Substrate commit 7e643ceabf11858cd3c059579ab45794793f2a78 in this DataHighway commit https://github.com/DataHighway-DHX/node/pull/215/commits/0a56ac0b6bb73b6faac8dcb0a533768dfb644f39 by doing a search and replace for `027368fe34e9a57ead752d4f900db6b5f85352e6` and replacing it with contents like the following, as well as applying any other changes as necessary:
>>>>>>> parent of a8ac31a5 (update to substrate commit 0ac1a00c54a65416ed44e4c779f1483e32355130)
```
sp-runtime = { git = 'https://github.com/paritytech/substrate', rev = '027368fe34e9a57ead752d4f900db6b5f85352e6', default-features = false }
```
and replacing it with
```
<<<<<<< HEAD
sp-runtime = { git = 'https://github.com/paritytech/substrate', rev = 'dfe52f7de12a7e308c0bc3a25e211b9cc4d61871', default-features = false }
=======
sp-runtime = { git = 'https://github.com/paritytech/substrate', rev = '7e643ceabf11858cd3c059579ab45794793f2a78', default-features = false }
>>>>>>> parent of a8ac31a5 (update to substrate commit 0ac1a00c54a65416ed44e4c779f1483e32355130)
```
when trying to compile it with `cargo build --release` it gave error:
```
error: failed to select a version for the requirement `sp-keystore = "*"`
candidate versions found which didn't match: 0.10.0
<<<<<<< HEAD
location searched: Git repository https://github.com/paritytech/substrate?rev=dfe52f7de12a7e308c0bc3a25e211b9cc4d61871#50ab759f
=======
location searched: Git repository https://github.com/paritytech/substrate?rev=7e643ceabf11858cd3c059579ab45794793f2a78#50ab759f
>>>>>>> parent of a8ac31a5 (update to substrate commit 0ac1a00c54a65416ed44e4c779f1483e32355130)
required by package `datahighway v3.0.6 (/Users/ls2/code/DataHighway-DHX/node/node)`
```
each error like this was caused because the Substrate repository was using a version like `0.10.0-dev`, whereas only `0.10.0` was available at https://docs.rs/sp-runtime.
so to fix each error it was necessary to modify the Cargo.lock file, by searching for
```
[[package]]
name = "sp-keystore"
version = "0.10.0-dev"
...
```
and removing the `-dev` so it became:
```
[[package]]
name = "sp-keystore"
version = "0.10.0"
...
```
and then running `cargo build --release` again to check the next error if any.

## Technical Support <a id="chapter-c00ab7"></a>

* [Discord Chat](https://discord.gg/UuZN2tE)

* [Twitter](https://twitter.com/DataHighway_DHX)
