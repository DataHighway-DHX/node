# Table of contents

* [Example "dev" development PoS testnet with single nodes](#chapter-ca9336)
* [Example "local" PoS testnet with multiple nodes](#chapter-f21efd)
* [Live (Alpha) "testnet_latest" PoS testnet (with multiple nodes)](#chapter-f0264f)
* [Live (Alpha) "harbour" PoS testnet (with multiple nodes)](#chapter-f023e2)
* [Interact with blockchain using Polkadot.js Apps UI](#chapter-6d9058)

## Example "dev" development PoS testnet (with single node) <a id="chapter-f21efd"></a>

### Intro

The development testnet only requires a single node to produce and finalize blocks.

### Run on Local Machine

* Fork and clone the repository

* Install or update Rust and dependencies. Build the WebAssembly binary from all code.
* Note that since we have two separate runtimes for testnet and mainnet, they will both be built at the same time.

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast && \
./scripts/init.sh && \
cargo build --release
```

* Purge the chain (remove relevant existing blockchain testnet database blocks and keys)

```bash
./target/release/datahighway purge-chain --dev --base-path /tmp/polkadot-chains/alice
./target/release/datahighway purge-chain --dev --base-path /tmp/polkadot-chains/bob
./target/release/datahighway purge-chain --dev --base-path /tmp/polkadot-chains/charlie
./target/release/datahighway purge-chain --dev --base-path /tmp/polkadot-chains/node-1
./target/release/datahighway purge-chain --dev --base-path /tmp/polkadot-chains/node-2
./target/release/datahighway purge-chain --dev --base-path /tmp/polkadot-chains/node-3
```

Or just:
```
rm -rf /tmp/polkadot-chains/alice /tmp/polkadot-chains/bob /tmp/polkadot-chains/charlie /tmp/polkadot-chains/node-1 /tmp/polkadot-chains/node-2 /tmp/polkadot-chains/node-3
```

* Connect to development testnet (`--chain development` is abbreviated `--dev`)

```bash
./target/release/datahighway \
  --base-path /tmp/polkadot-chains/alice \
  --name "Data Highway Development Chain" \
  --dev \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0"
```

## Example "local" PoS testnet (with multiple nodes) <a id="chapter-f21efd"></a>

### Intro

Run a multiple node PoS testnet on your local machine with built-in keys (Alice, Bob, Charlie) using a custom Substrate-based blockchain configuration using multiple terminals windows.
* Configure and import custom raw chain definition
* Use default accounts Alice, Bob, and Charlie as the three initial authorities of the genesis configuration that have been endowed with testnet units that will run validator nodes
* **Important**: Since we are using GRANDPA where you have authority set of size 4, it means you need 3 nodes running in order to **finalize** the blocks that are authored. (Credit: @bkchr Bastian Köcher)

### Run on Local Machine (without Docker)

#### Fetch repository and dependencies

* Fork and clone the repository

* Install or update Rust and dependencies. Build the WebAssembly binary from all code

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast && \
./scripts/init.sh
```

#### Build runtime code

```bash
cargo build --release
```

#### Create custom blockchain configuration

* Create latest chain specification code changes of <CHAIN_ID> "local"

> Other chains are specified in src/chain_spec.rs (i.e. dev, local, or testnet_latest).

* Generate the chain specification JSON file from src/chain_spec.rs

```bash
rm ./node/src/chain-spec-templates/chain_spec_local.json
touch ./node/src/chain-spec-templates/chain_spec_local.json
mkdir -p ./node/src/chain-spec-templates
./target/release/datahighway build-spec \
  --chain=local > ./node/src/chain-spec-templates/chain_spec_local.json
```

* Build "raw" chain definition for the new chain from it

```bash
rm ./node/src/chain-definition-custom/chain_def_local.json
touch ./node/src/chain-definition-custom/chain_def_local.json
mkdir -p ./node/src/chain-definition-custom
./target/release/datahighway build-spec \
  --chain ./node/src/chain-spec-templates/chain_spec_local.json \
  --raw > ./node/src/chain-definition-custom/chain_def_local.json
```

> Remember to purge the chain state if you change anything (database and keys)

```bash
./target/release/datahighway purge-chain --chain "local" --base-path /tmp/polkadot-chains/alice
./target/release/datahighway purge-chain --chain "local" --base-path /tmp/polkadot-chains/bob
./target/release/datahighway purge-chain --chain "local" --base-path /tmp/polkadot-chains/charlie
```

Or just:
```
rm -rf /tmp/polkadot-chains/alice /tmp/polkadot-chains/bob /tmp/polkadot-chains/charlie /tmp/polkadot-chains/node-1 /tmp/polkadot-chains/node-2 /tmp/polkadot-chains/node-3
```

#### Terminal 1

Run Alice's bootnode using the raw chain definition file that was generated

```bash
./target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/alice \
  --keystore-path "/tmp/polkadot-chains/alice/keys" \
  --chain ./node/src/chain-definition-custom/chain_def_local.json \
  --node-key 88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee \
  --alice \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
```

When the node has started, copy the libp2p local node identity of the node, and paste in the `bootNodes` of chain_def_local.json if necessary.

* Notes:
  * Alice's Substrate-based node on default TCP port 30333
  * Her chain database stored locally at `/tmp/polkadot-chains/alice`
  * Bootnode ID of her node is `Local node identity is: QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ` (peer id), which is generated from the `--node-key` value specified below and shown when the node is running. Note that `--alice` provides Alice's session key that is shown when you run `subkey -e inspect //Alice`, alternatively you could provide the private key that is necessary to produce blocks with `--key "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice"`. In production the session keys are provided to the node using RPC calls `author_insertKey` and `author_rotateKeys`. If you explicitly specify a `--node-key` (i.e. `--node-key 88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee`) when you start your validator node, the logs will still display your peer id with `Local node identity is: Qxxxxxx`, and you could then include it in the chain_spec_local.json file under "bootNodes". Also the peer id is listed when you go to view the list of full nodes and authority nodes at Polkadot.js Apps https://polkadot.js.org/apps/#/explorer/datahighway

#### Terminal 2

Run Bob's Substrate-based node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/polkadot-chains/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
./target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/bob \
  --keystore-path "/tmp/polkadot-chains/bob/keys" \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ \
  --chain ./node/src/chain-definition-custom/chain_def_local.json \
  --bob \
  --port 30334 \
  --ws-port 9945 \
  --rpc-port 9934 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
```

> Important: Since in GRANDPA you have authority set of size 4, it means you need 3 nodes running in order to **finalize** the blocks that are authored. (Credit: @bkchr Bastian Köcher)

#### Terminal 3

Run Charlie's Substrate-based node on a different TCP port of 30335, and with his chain database stored locally at `/tmp/polkadot-chains/charlie`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
./target/release/datahighway --validator \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/charlie \
  --keystore-path "/tmp/polkadot-chains/charlie/keys" \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ \
  --chain ./node/src/chain-definition-custom/chain_def_local.json \
  --charlie \
  --port 30335 \
  --ws-port 9946 \
  --rpc-port 9935 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
```

* Check that the chain is finalizing blocks (i.e. finalized is non-zero `main-tokio- INFO substrate  Idle (2 peers), best: #3 (0xaede…b8d9), finalized #1 (0x4c69…f605), ⬇ 3.3kiB/s ⬆ 3.7kiB/s`)

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

#### Terminal 4 (Optional)

* Add session keys for account(s) to be configured as authorities (validators). Run cURL to insert session key for each key type (i.e. "aura"), by providing the associated secret key, and associated Public key (hex)
```bash
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["aura", "", "0x408f99b525d90cce76288245cb975771282c2cefa89d693b9da2cdbed6cd9152"],"id":1 }' 127.0.0.1:9933
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["babe", "//Alice//babe", "0x46ffa3a808850b2ad55732e958e781146ed1e6436ffb83290e0cb810aacf5070"],"id":1 }' 127.0.0.1:9933
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["imon", "//Alice//imonline", "0xee725cf87fa2d6f264f26d7d8b84b1054d2182cdcce51fdea95ec868be9d1e17"],"id":1 }' 127.0.0.1:9933
curl -vH 'Content-Type: application/json' --data '{ "jsonrpc":"2.0", "method":"author_insertKey", "params":["gran", "//Alice//grandpa", "0x6e2de2e5087b56ed2370359574f479d7e5da1973e17ca1b55882c4773f154d2f"],"id":1 }' 127.0.0.1:9933
```

* Check that the output from each cURL request is `{"jsonrpc":"2.0","result":null,"id":1}`, since with a successful output `null` is returned https://github.com/paritytech/substrate/blob/db1ab7d18fbe7876cdea43bbf30f147ddd263f94/client/rpc-api/src/author/mod.rs#L47. Also check that the following folder is not empty /tmp/polkadot-chains/alice/keys (it should now contain four keys).

* Reference: https://substrate.dev/docs/en/next/tutorials/start-a-private-network/alicebob

#### Additional Steps (Optional)

* Follow the steps to [interact with blockchain using Polkadot.js Apps UI](#chapter-6d9058)

* View on [Polkadot Telemetry](https://telemetry.polkadot.io/#list/DataHighway%20Local%20PoA%20Testnet%20v0.1.0)

* Distribute the custom chain definition (i.e. chain_def_local.json) to allow others to synchronise and validate if they are an authority

#### Optional: Run without generating chain definition

```
SKIP_WASM_BUILD=1 RUST_LOG=runtime=debug \
./target/release/datahighway \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/alice \
  --chain local \
  --alice \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --validator \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe

SKIP_WASM_BUILD=1 RUST_LOG=runtime=debug \
./target/release/datahighway \
  --unsafe-ws-external \
  --unsafe-rpc-external \
  --rpc-cors=all \
  --base-path /tmp/polkadot-chains/bob \
  --chain local \
  --bob \
  --port 30334 \
  --ws-port 9945 \
  --rpc-port 9934 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --validator \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
  --execution=native \
  -lruntime=debug \
  --rpc-methods=Unsafe
```

### Run on Local Machine (WITH Docker) (Alice, Bob, Charlie)

#### Fetch repository and dependencies

* Fork and clone the repository

#### Edit the Docker Compose

* Update docker-compose-dev.yml. Rename each of the Docker Images that will be created to be:
```
image: "dhxdocker/datahighway:<YOUR_BRANCH_NAME>"
```
* Note: By default they are `image: "dhxdocker/datahighway:latest"`

#### Build a Docker Image

* Install or update Rust and dependencies. Build the WebAssembly binary from all code. Create blockchain configuration from chain specification and "raw" chain definition.

```
docker-compose --env-file=./.env --file docker-compose-dev.yml --verbose build --no-cache --build-arg CHAIN_VERSION="local"
```

Note: If you get error `error: failed to parse manifest at /dhx/runtime/Cargo.toml Caused by: no targets specified in the manifest either src/lib.rs, src/main.rs, a [lib] section, or [[bin]] section must be present` then it's because the necessary folders haven't been copied using Docker's `COPY` (i.e. `COPY ./abc/* /root/abc` doesn't work, it shouldn't have the `*`)

### Run Docker Containers for each Node (Alice, Bob, and Charlie)

```
docker-compose -f docker-compose-dev.yml up --detach alice && \
docker-compose -f docker-compose-dev.yml up --detach bob && \
docker-compose -f docker-compose-dev.yml up --detach charlie
```

### View Logs of each Node

```
docker-compose logs --follow
```

### Interact using UI

Follow the steps to [interact with blockchain using Polkadot.js Apps UI](#chapter-6d9058)

View the balances endowed in the Genesis block by going to https://polkadot.js.org/apps/#/js and pasting the following, click the Play icon, and view the output on the right
```
const DHX_DAO = '5FmxcuFwGK7kPmQCB3zhk3HtxxJUyb3WjxosF8jvnkrVRLUG';

const { data: balance } = await api.query.system.account(DHX_DAO);
const totalIssuance = await api.query.balances.totalIssuance();
console.log(`DHX DAO Unlocked Reserves has a balance of ${balance.free} DHX`);
console.log(`DataHighway has a total supply of ${totalIssuance} DHX`);
```

### Stop or Restart Docker Container

```
docker-compose -f docker-compose-dev.yml stop alice && \
docker-compose -f docker-compose-dev.yml stop bob && \
docker-compose -f docker-compose-dev.yml stop charlie
```

```
docker-compose -f docker-compose-dev.yml start alice && \
docker-compose -f docker-compose-dev.yml start bob && \
docker-compose -f docker-compose-dev.yml start charlie
```

Note: Where `<SERVICE>` is `alice`, `bob`, or `charlie`, as defined in docker-compose-dev.yml file

### Other

#### Access the Docker Container

```
docker-compose -f docker-compose-dev.yml exec alice bash
```

## Testnet (Alpha) "testnet_latest" PoS testnet (with multiple nodes) <a id="chapter-f0264f"></a>

### Intro

Join the multiple node PoS testnet (alpha), where you will be using the latest custom chain definition for the testnet (i.e. chain_def_testnet_latest.json).

### Run (with Docker containers)

1. Fork and clone the repository
2. Install and run Docker
3. Replace [docker-compose-custom.yml](./docker-compose-custom.yml) file with your custom node (e.g. rename node from `node-1` to something else or add additional nodes). By default it will run two validator nodes (i.e. node-1 and node-2).
4. Update the relevant ./scripts/docker-entrypoint-<NODE_NAME>.sh (i.e. [docker-entrypoint-node-1.sh](./scripts/docker-entrypoint-node-1.sh) and [docker-entrypoint-node-2.sh](./scripts/docker-entrypoint-node-2.sh) with your node specific information (e.g. change the value provided to `--name` and rename `node-1` or `node-2` to your custom node name) and run `chmod 755 ./scripts/docker-entrypoint-<NODE_NAME>.sh` if you create an new scripts so they are executable, where `<NODE_NAME>` would be your chosen custom node name.
5. If you modify the code, rebuild the chain configuration file and purge the chain (see section "Create custom blockchain configuration")
6. Remove old containers and images:
```
docker rm node_1 node_2 node_3 node_bob_1 node_alice_1 node_charlie_1
docker rmi dhxdocker/datahighway
```
6. Run the Docker container in the background as a daemon and view the logs on-demand (the image will be built on first run based on the Dockerfile). It will install dependencies and build chain runtime code. See the notes below for an alternative approach.
  ```bash
  docker-compose --file docker-compose-custom.yml --verbose up --detach
  docker-compose logs --follow
  ```
Alternatively just run `docker-compose --file docker-compose-custom.yml --verbose up`.
If you change the code, then be rebuild the code, rebuild the chain configuration file and purge the chain, then destroy and rebuild containers from the image by running `docker-compose --file docker-compose-custom.yml --verbose down && docker-compose --file docker-compose-custom.yml --verbose up`
To just restart existing containers of the node, run `docker-compose --file docker-compose-custom.yml --verbose restart`.
  * Screenshot:
  ![](./assets/images/logs.png)
Note: If you get error building after modifying docker-compose: `FileNotFoundError: [Errno 2] No such file or directory`, then try running `rm -rf target/rls/debug/`
Note: If you get error building like: `Cannot start service alice: OCI runtime create failed`, then try running `./scripts/docker-clean.sh` (beware this deletes all Docker containers, images, and cache for all your projects, not just Datahighway), and then `rm -rf ./target/rls/debug` a few times until it no longer says `Directory not empty`
Note: If you get error building like: `Compiling parity-multiaddr v0.7.2 error[E0308]: mismatched types --> /root/.cargo/registry/src/github.com-1ecc6299db9ec823/parity-multiaddr-0.7.2/src/onion_addr.rs:23:9` then you need to use an older version of Rust Nightly, and to set Nightly as the default.
Note: If you get error running docker-compose: `Creating node_alice_1 ... error compose.parallel.feed_queue: Pending: set() ERROR: for node_alice_1  Cannot start service alice: OCI runtime create failed: container_linux.go:349: starting container process caused "exec: \"./docker-entrypoint-alice.sh\": stat ./docker-entrypoint-alice.sh: no such file or directory": unknown`, then it's likely the last time you tried to build the Docker container it failed, so you need to delete the container and possibly the image and cache too and try again.
Note: If you get error: `FileNotFoundError: [Errno 2] No such file or directory: '/Users/ls/code/src/DataHighway-com/node/target/rls/debug/deps/save-analysis/libsc_executor_common-f236f3ddcd6862b3.json'` then run `rm -rf ./target/rls/debug` a few times until it no longer says `Directory not empty`
7. Follow the steps to [interact with blockchain using Polkadot.js Apps UI](#chapter-6d9058)

Note:
* Only DataHighway admins that need to additionally update the ["testnet_latest" chain spec](./src/chain_spec.rs), to generate and share the raw chain definition with other nodes.
* Only DataHighway admins should use the docker-compose-admin.yml file to start the initial bootnodes, whereas other community nodes that connect to the DataHighway should use docker-compose-custom.yml instead.
* Refer to the FAQ or contact Technical Support provided in [CONTRIBUTING.md](./CONTRIBUTING.md) if you encounter any issues.
* If all services defined in docker-compose-custom.yml will be running in Docker containers on the same host machine, then each service must expose different ports (on the left side of the colon), however the ports that are used within each Docker container may be the same.

## Testnet (Alpha) "harbour" PoS testnet (with multiple nodes) <a id="chapter-f023e2"></a>

* Refer to the documentation to setup a validator node and to obtain bootnode to connect to https://dev.datahighway.com/docs/en/tutorials/tutorials-nodes-validator-setup

## Interact with blockchain using Polkadot.js Apps UI <a id="chapter-6d9058"></a>

* Setup connection between the UI and the node:
  * Go to Polkadot.js Apps at https://polkadot.js.org/apps
	* Click "Settings" from the sidebar menu, and click its "Developer" tab to be taken to https://polkadot.js.org/apps/#/settings/developer to add Custom Types. Copy the contents of [custom_types.json](./custom_types.json), and pasting it into the input field, then click "Save".
  * Click "Settings" from the sidebar menu again, and click its "General" tab to be taken to https://polkadot.js.org/apps/#/settings. Click the "remote node/endpoint to connect to" selection box, and choose "Local Node (127.0.0.1:9944)" option from the list, then click "Save".
  * Wait for the UI to refresh (i.e. additional sidebar menu items will appear including "Explorer", "Accounts", "Address book", "Staking", "Chain state", etc).
  * Click "Explore" from the sidebar menu to be taken to https://polkadot.js.org/apps/#/explorer/node and shown the "Node info", including connected peers.

Once you've established a connection between the UI and the DataHighway testnet, you may try the following:

* Create accounts and transfer funds:
  * Click "Accounts" from the sidebar menu, then click tab "My accounts", and click button "Add Account"
  * Import Bob's built-in stash account (with 1,000 DHX balance) from the [test keyring](https://github.com/polkadot-js/apps/issues/1117#issuecomment-491020187) by entering:
    * name: "Bob"
    * mnemonic seed: "bottom drive obey lake curtain smoke basket hold race lonely fit walk"
    * password: "bob"
    * password (repeat): "bob"
    * secret derivation path: "//Bob//stash"
* Transfer funds between accounts:
  * Click "Transfer" from the sidebar menu
* Stake on the testnet (using testnet DHX that has been endowed to accounts)
  * Click "Stake" from the sidebar menu. Refer to the [Polkadot wiki's collator, validator, and nominator guides](https://wiki.polkadot.network/docs/en/maintain-guides-how-to-validate-kusama)
* Chain state interaction (i.e. roaming, mining, etc):
  * Click "Chain state" from the sidebar menu, then click tab "Storage"
  * Click "selected state query" selection box, and then as an example, choose "dataHighwayMiningClaim", and see if it works yet (WIP).
* Extrinsics interaction (i.e. roaming, mining, etc):
  * Click "Extrinsics" from the sidebar menu.

* **Important**:
  * Input parameter quirk: Sometimes it is necessary to modify the value of one of the input parameters to allow you to click "Submit Transaction" (i.e. if the first arguments input value is already 0 and appears valid, but the "Submit Transaction" button appears disabled, just delete the 0 value and re-enter 0 again)
  * Prior to being able to submit extrinics at https://polkadot.js.org/apps/#/extrinsics (i.e. roaming > createNetwork()) or to view StorageMap values, it is necessary to Add Custom Types (see earlier step), otherwise the "Submit Transaction" button will not work.
