use sp_finality_grandpa::AuthorityId as GrandpaId;
use hex_literal::hex;
use datahighway_runtime::{
    // opaque::{
    //     Block,
    //     SessionKeys,
    // },
    AuthorityDiscoveryConfig,
    BabeConfig,
    BalancesConfig,
    Block,
    DemocracyConfig,
    TechnicalMembershipConfig,
    ElectionsConfig,
    GenesisConfig,
    GrandpaConfig,
    ImOnlineConfig,
    IndicesConfig,
    SessionConfig,
    SessionKeys,
    StakerStatus,
    StakingConfig,
    SudoConfig,
    SystemConfig,
    TreasuryConfig,
    WASM_BINARY,
};
use module_primitives::{
    constants::currency::{
        DOLLARS,
    },
	types::{
        AccountId,
        Balance,
        Signature,
    },
};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::map::Map;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{
    crypto::{
        UncheckedFrom,
        UncheckedInto,
    },
    sr25519,
    Pair,
    Public,
};
use sp_runtime::traits::{
    IdentifyAccount,
    Verify,
};
pub use sp_runtime::{
    Perbill,
    Permill,
};

// Note this is the URL for the telemetry server
const POLKADOT_STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate an authority key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Wasm not available".to_string())?;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
        ChainType::Development,
		move || testnet_genesis(
			wasm_binary,
			// Initial NPoS authorities
            vec![
                get_authority_keys_from_seed("Alice")
            ],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
            vec![
                // DHX DAO Unlocked Reserves Balance
                // Given a Treasury ModuleId in runtime parameter_types of
                // `py/trsry`, we convert that to its associated address
                // using Module ID" to Address" at https://www.shawntabrizi.com/substrate-js-utilities/,
                // which generates 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z,
                // and find its corresponding hex value by pasting the address into
                // "AccountId to Hex" at that same link to return
                // 6d6f646c70792f74727372790000000000000000000000000000000000000000.
                // But since DataHighway is using an SS58 address prefix of 33 instead of
                // Substrate's default of 42, the address corresponds to
                // 4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk.
                // This is pallet_treasury's account_id.
                //
                // Substrate 2 does not have instantiable support for treasury
                // is only supported in Substrate 3 and was fixed here
                // https://github.com/paritytech/substrate/pull/7058
                //
                // Since we have now updated to Substrate 3, we may transfer funds
                // directly to the Treasury, which will hold the
                // DHX DAO Unlocked Reserves Balance.
                //
                // Note: The original DataHighway Testnet Genesis has used:
                //   5FmxcuFwGK7kPmQCB3zhk3HtxxJUyb3WjxosF8jvnkrVRLUG
                //   hex: a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21
                //
                // However, the DataHighway Westlake Mainnet will transfer the funds to:
                //   4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk
                //   6d6f646c70792f74727372790000000000000000000000000000000000000000
                //
                // To transfer funds from the Treasury, either the Sudo user needs to
                // call the `forceTransfer` extrinsic to transfer funds from the Treasury,
                // or a proposal is required.
                hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                get_account_id_from_seed::<sr25519::Public>("Dave"),
                // Required otherwise get error when compiling
                // `Stash does not have enough balance to bond`
                get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
            ],
			true,
		),
		// Bootnodes
        vec![],
        // Telemetry Endpoints
        None,
        // Protocol ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Default::default(),
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Wasm not available".to_string())?;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local",
        ChainType::Local,
		move || testnet_genesis(
			wasm_binary,
			// Initial NPoS authorities
            vec![
                get_authority_keys_from_seed("Alice"),
                get_authority_keys_from_seed("Bob"),
                get_authority_keys_from_seed("Charlie"),
                get_authority_keys_from_seed("Dave"),
                get_authority_keys_from_seed("Eve"),
                get_authority_keys_from_seed("Ferdie"),
            ],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
            vec![
                // Endow this account with the DHX DAO Unlocked Reserves Balance
                // 4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk
                hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                // Endow these accounts with a balance so they may bond as authorities
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                get_account_id_from_seed::<sr25519::Public>("Dave"),
                get_account_id_from_seed::<sr25519::Public>("Eve"),
                get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                // Required otherwise get error when compiling
                // `Stash does not have enough balance to bond`
                get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
            ],
			true,
		),
        // Bootnodes
        vec![
            // Note: The local node identity that is shown when you start the bootnode
            // with the following flags and options is
            // `12D3KooWKS7jU8ti7S5PDqCNWEj692eUSK3DLssHNwTQsto9ynVo`:
            // ./target/release/datahighway \
            //   ...
            //   --alice \
            //   --node-key 88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee \
            //   ...
            // Since it is an IP address we use `/ip4/`, whereas if it were a domain we'd use `/dns/`
            "/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWKS7jU8ti7S5PDqCNWEj692eUSK3DLssHNwTQsto9ynVo"
                .parse()
                .unwrap(),
        ],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some("dhx-test"),
        // Properties
        Some(properties),
        // Extensions
        Default::default(),
	))
}

pub fn datahighway_testnet_harbour_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Wasm binary not available".to_string())?;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"DataHighway Harbour Testnet",
		// ID
		"harbour",
        ChainType::Live,
        // TODO: regenerate alphanet according to babe-grandpa consensus
        // subkey inspect "$SECRET"
        // for i in 1 2 3 4; do for j in stash controller; do subkey inspect "$SECRET//$i//$j"; done;
        // done for i in 1 2 3 4; do for j in babe; do subkey inspect
        // --scheme=sr25519 "$SECRET//$i//$j"; done; done for i in 1 2 3 4; do
        // for j in grandpa; do subkey inspect --scheme=ed25519 "$SECRET//$i//$j"; done; done
		move || testnet_genesis(
			wasm_binary,
			// Initial NPoS authorities
            vec![
                // authority #0
                (
                    hex!["f64bae0f8fbe2eb59ff1c0ff760a085f55d69af5909aed280ebda09dc364d443"].into(),
                    hex!["ca907b74f921b74638eb40c289e9bf1142b0afcdb25e1a50383ab8f9d515da0d"].into(),
                    hex!["6a9da05f3e07d68bc29fb6cf9377a1537d59f082f49cb27a47881aef9fbaeaee"]
                        .unchecked_into(),
                    hex!["f2bf53bfe43164d88fcb2e83891137e7cf597857810a870b4c24fb481291b43a"]
                        .unchecked_into(),
                    // im_online
                    hex!["504cf9eb92f9e992a15370d2536063df83b69744ad4502e05dc91e3dae3b2649"]
                        .unchecked_into(),
                    // authority_discovery
                    hex!["7941e1c7ff93c9541a779b40780715fe6211407c970906f3b554b294a6ba7ec7"]
                        .unchecked_into(),
                ),
                // authority #1
                (
                    hex!["420a7b4a8c9f2388eded13c17841d2a0e08ea7c87eda84310da54f3ccecd3931"].into(),
                    hex!["ae69db7838fb139cbf4f93bf877faf5bbef242f3f5aac6eb4f111398e9385e7d"].into(),
                    hex!["9af1908ac74b042f4be713e10dcf6a2def3770cfce58951c839768e7d6bbcd8e"]
                        .unchecked_into(),
                    hex!["1e91a7902c89289f97756c4e20c0e9536f34de61c7c21af7773d670b0e644030"]
                        .unchecked_into(),
                    // im_online
                    hex!["65be4fe3c1efba2b10f05b50bdb25cce394ed9c30ab5ebf2d604c0628d8500e7"]
                        .unchecked_into(),
                    // authority_discovery
                    hex!["48a3726bef363c990585cb550da2b1d5e309ae6a0ea1d4d06bf389bce303dae8"]
                        .unchecked_into(),
                ),
                // authority #2
                (
                    hex!["ceecb6cc08c20ff44052ff19952a810d08363aa26ea4fb0a64a62a4630d37f28"].into(),
                    hex!["7652b25328d78d264aef01184202c9771b55f5b391359309a2559ef77fbbb33d"].into(),
                    hex!["b8902681768fbda7a29666e1de8a18f5be3c778d92cf29139959a86e6bff13e7"]
                        .unchecked_into(),
                    hex!["aaabcb653ce5dfd63035430dba10ce9aed5d064883b9e2b19ec5d9b26a457f57"]
                        .unchecked_into(),
                    // im_online
                    hex!["76a21eefe9824ae269d4fc6609e310f323053acd2ef47fe36e6d0be75a5cd75b"]
                        .unchecked_into(),
                    // authority_discovery
                    hex!["138b33e1bd40d51e95455c19a9558ba4b5db4969943183592b10227537366a8e"]
                        .unchecked_into(),
                ),
                // authority #3
                (
                    hex!["68bac5586028dd40db59a7becec349b42cd4229f9d3c31875c3eb7a57241cd42"].into(),
                    hex!["eec96d02877a45fa524fcee1c6b7c849cbdc8cee01a95f5db168c427ae766849"].into(),
                    hex!["f4807d86cca169a81d42fcf9c7abddeff107b0a73e9e7a809257ac7e4a164741"]
                        .unchecked_into(),
                    hex!["a49ac1053a40a2c7c33ffa41cb285cef7c3bc9db7e03a16d174cc8b5b5ac0247"]
                        .unchecked_into(),
                    // im_online
                    hex!["2bb62b29a6c80c32a9284026044483caf85b699744bc155e8c1a0c5819578f4b"]
                        .unchecked_into(),
                    // authority_discovery
                    hex!["a3ca5592ca7de669469a94c7ff6f64d8e2fcc5ab226833420748d25fab56bbe0"]
                        .unchecked_into(),
                ),
                // authority #4
                (
                    hex!["ca181fc1f02a0aa144885d3b6f95d333a3a84ecc448b4d9f3541b26d21729168"].into(),
                    hex!["f406b4141e7cab5b09e670c617ab65e911da684e4deb76d0d29e94f77a535b39"].into(),
                    hex!["9edf290adfc576f4de8b90a09b3b378263f34748f201a1966153f26a879e5a39"]
                        .unchecked_into(),
                    hex!["03ead710287b634d6cdf2db7be3815a48a612fd2bec3e812c6cbe3721d01e756"]
                        .unchecked_into(),
                    // im_online
                    hex!["d47678b3604e1d61301790e505606ba05d601180b541d0495adac79ff43a5e41"]
                        .unchecked_into(),
                    // authority_discovery
                    hex!["f96cb52e84e128ba23a7627b154f015c15f54acd31881522e505e28076b8dc66"]
                        .unchecked_into(),
                ),
            ],
			// Sudo account
            hex!["3c917f65753cd375582a6d7a1612c8f01df8805f5c8940a66e9bda3040f88f5d"].into(),
			// Pre-funded accounts
            vec![
                // Endow the Treasury account with the DHX DAO Unlocked Reserves Balance
                // 4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk
                hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                // Endow these accounts with a balance so they may bond as authorities.
                // IMPORTANT: All authorities must be included in the list below so they have
                // an account balance to avoid session error
                // `assertion failed: frame_system::Module::<T>::inc_consumers(&account).is_ok()`

                // authority #0
                hex!["f64bae0f8fbe2eb59ff1c0ff760a085f55d69af5909aed280ebda09dc364d443"].into(),
                hex!["ca907b74f921b74638eb40c289e9bf1142b0afcdb25e1a50383ab8f9d515da0d"].into(),
                hex!["6a9da05f3e07d68bc29fb6cf9377a1537d59f082f49cb27a47881aef9fbaeaee"].into(),
                hex!["f2bf53bfe43164d88fcb2e83891137e7cf597857810a870b4c24fb481291b43a"].into(),
                hex!["504cf9eb92f9e992a15370d2536063df83b69744ad4502e05dc91e3dae3b2649"].into(),
                hex!["7941e1c7ff93c9541a779b40780715fe6211407c970906f3b554b294a6ba7ec7"].into(),

                // authority #1
                hex!["420a7b4a8c9f2388eded13c17841d2a0e08ea7c87eda84310da54f3ccecd3931"].into(),
                hex!["ae69db7838fb139cbf4f93bf877faf5bbef242f3f5aac6eb4f111398e9385e7d"].into(),
                hex!["9af1908ac74b042f4be713e10dcf6a2def3770cfce58951c839768e7d6bbcd8e"].into(),
                hex!["1e91a7902c89289f97756c4e20c0e9536f34de61c7c21af7773d670b0e644030"].into(),
                hex!["65be4fe3c1efba2b10f05b50bdb25cce394ed9c30ab5ebf2d604c0628d8500e7"].into(),
                hex!["48a3726bef363c990585cb550da2b1d5e309ae6a0ea1d4d06bf389bce303dae8"].into(),

                // authority #2
                hex!["ceecb6cc08c20ff44052ff19952a810d08363aa26ea4fb0a64a62a4630d37f28"].into(),
                hex!["7652b25328d78d264aef01184202c9771b55f5b391359309a2559ef77fbbb33d"].into(),
                hex!["b8902681768fbda7a29666e1de8a18f5be3c778d92cf29139959a86e6bff13e7"].into(),
                hex!["aaabcb653ce5dfd63035430dba10ce9aed5d064883b9e2b19ec5d9b26a457f57"].into(),
                hex!["76a21eefe9824ae269d4fc6609e310f323053acd2ef47fe36e6d0be75a5cd75b"].into(),
                hex!["138b33e1bd40d51e95455c19a9558ba4b5db4969943183592b10227537366a8e"].into(),

                // authority #3
                hex!["68bac5586028dd40db59a7becec349b42cd4229f9d3c31875c3eb7a57241cd42"].into(),
                hex!["eec96d02877a45fa524fcee1c6b7c849cbdc8cee01a95f5db168c427ae766849"].into(),
                hex!["f4807d86cca169a81d42fcf9c7abddeff107b0a73e9e7a809257ac7e4a164741"].into(),
                hex!["a49ac1053a40a2c7c33ffa41cb285cef7c3bc9db7e03a16d174cc8b5b5ac0247"].into(),
                hex!["2bb62b29a6c80c32a9284026044483caf85b699744bc155e8c1a0c5819578f4b"].into(),
                hex!["a3ca5592ca7de669469a94c7ff6f64d8e2fcc5ab226833420748d25fab56bbe0"].into(),

                // authority #4
                hex!["ca181fc1f02a0aa144885d3b6f95d333a3a84ecc448b4d9f3541b26d21729168"].into(),
                hex!["f406b4141e7cab5b09e670c617ab65e911da684e4deb76d0d29e94f77a535b39"].into(),
                hex!["9edf290adfc576f4de8b90a09b3b378263f34748f201a1966153f26a879e5a39"].into(),
                hex!["03ead710287b634d6cdf2db7be3815a48a612fd2bec3e812c6cbe3721d01e756"].into(),
                hex!["d47678b3604e1d61301790e505606ba05d601180b541d0495adac79ff43a5e41"].into(),
                hex!["f96cb52e84e128ba23a7627b154f015c15f54acd31881522e505e28076b8dc66"].into(),
            ],
			true,
		),
        vec![
            "/ip4/18.185.37.254/tcp/30333/p2p/12D3KooWFmR35FFHiXcQv8hsFWDq6ofttqBPeMkd4Jt6qRgq3HnT"
                .parse()
                .unwrap(),
        ],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some("dhx-test"),
        // Properties
        Some(properties),
        // Extensions
        Default::default(),
	))
}

pub fn datahighway_mainnet_westlake_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Wasm binary not available".to_string())?;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"DataHighway Westlake Mainnet",
		// ID
		"westlake",
        ChainType::Live,
		move || mainnet_genesis(
			wasm_binary,
			// Initial NPoS authorities
            vec![
                // authority #0
                (
                    // cont
                    hex!["76d0d3add586849d643444ff9fc55413222dea22d5fa297ba1b39386c79a6618"].into(),
                    // stash
                    hex!["a0dd28a829885c10b413095454a9f774b6cf3bc07f797c33afda2c2f87fb8f35"].into(),
                    // gran
                    hex!["3a1de065f4e475584f4943559fbd3b57563117150c98bda1a6c9ebba0ecc8368"]
                        .unchecked_into(),
                    // babe
                    hex!["a8e804e4a03e5afa89d3d4cacc041562fd5145b4b2866eb191265aa9262acc06"]
                        .unchecked_into(),
                    // imon
                    hex!["78ae9f082530ca77f3e0b608df13bd7dc087049fd36d5c999ccc9a0fcb3b6517"]
                        .unchecked_into(),
                    // audi
                    hex!["029632937ad7171486a0c452f8bdb182a2b5a3311e003d7525fd360e059bb668"]
                        .unchecked_into(),
                ),
                // authority #1
                (
                    // cont
                    hex!["a2c5ecadc845295b1c92585007aabd88826111a090908ec03eb6696b69211636"].into(),
                    // stash
                    hex!["5c680bbcad7e145f5ec27c27012cbabd32fe203208e96a3c080f8ef32164de51"].into(),
                    // gran
                    hex!["cab049448517b38e3fb22df1565d25365dd09e6f05af36d2ddd5f77b13a95921"]
                        .unchecked_into(),
                    // babe
                    hex!["9ca738e4af3a18b2b84c32e5e74a323ccb24aacc7f9337da23c0edaa589fe828"]
                        .unchecked_into(),
                    // imon
                    hex!["d2107868645df1c62ae2e794376ccd2708bfe88052d3298e5cf5ef76312bad6f"]
                        .unchecked_into(),
                    // audi
                    hex!["40ec24198297c21198c8cf3c2172e7096c3085de96bcef497561a8d81b67230d"]
                        .unchecked_into(),
                ),
                // authority #2
                (
                    // cont
                    hex!["805f3114e41a99126fbc1a56955dbb29df9f267c68721fb6b3c88c4fdd9f0318"].into(),
                    // stash
                    hex!["62a92866e30a34c72bcf829ec86f7b6883f335151cbe27a99978d5b1ce2b5c31"].into(),
                    // gran
                    hex!["e260ef20e7d3fb5a7c4384b625cf548914fe03959dded41a1dc78206b8e1fa31"]
                        .unchecked_into(),
                    // babe
                    hex!["e2f238e6f968fd44f834967cf39c0c7896380401467cb76745db62ffbf942e4e"]
                        .unchecked_into(),
                    // imon
                    hex!["2063cc1a936f466fd58954d6b28e1b8c84298f0b3b61e64bbed10c74519a2c66"]
                        .unchecked_into(),
                    // audi
                    hex!["706a1ba3800edaf2f84bc80f7ad88b4c9ac74cc78aa99825a50517997a8feb1e"]
                        .unchecked_into(),
                ),
                // authority #3
                (
                    // cont
                    hex!["283c513c1af26294775258f28cb3ab728e02481c16edf463d260a7d95ecfd66c"].into(),
                    // stash
                    hex!["0ac6bc8c098d6fcf819de3bf30b410e3fcd06d3b8b94f02d7ad0bdec808b686c"].into(),
                    // gran
                    hex!["464cfc0c22b9ab67a7e5d526d482dcbdf763e86994034bbf46ea2b007ae3583d"]
                        .unchecked_into(),
                    // babe
                    hex!["2aa395a47ed11d9fc30295350e7eb56e59681b49a71d10d9ecc3078b42017b32"]
                        .unchecked_into(),
                    // imon
                    hex!["76626a3b013571687f201106a3b789c907a5b6e34e918f1ace454f75b7464112"]
                        .unchecked_into(),
                    // audi
                    hex!["40b8f682f2778bd2107a379c8c9f0c01851e819c1a665ef0719944a7ed28b53c"]
                        .unchecked_into(),
                ),
                // authority #4
                (
                    // cont
                    hex!["981114473ce826f7a000e97d33ad716dfc055f0e279ab2ea5972a8a2d2dfa706"].into(),
                    // stash
                    hex!["e4f734b12b3cd7977e1edf756255930bc5390844c098ccf699bfcdbe18fff536"].into(),
                    // gran
                    hex!["f4bc2127d317dbae6556f4884ba7e9ae2257f0079b66bd975157b599dc198401"]
                        .unchecked_into(),
                    // babe
                    hex!["229cecefd4737a64b87bbde740af607a0000bcd8770ee8470a33427b0944586d"]
                        .unchecked_into(),
                    // imon
                    hex!["f690950a9f553f42255cf3a71bd00e73ab55df31a0f16968829ad9291bf14971"]
                        .unchecked_into(),
                    // audi
                    hex!["3c1107461309ba8598b586b29887eebdafcd0b7c3208d251368b286d83ecad5f"]
                        .unchecked_into(),
                ),
            ],
			// Sudo account
            hex!["86361c4e8b7ca00a64ecd1f655689bbe06339ae9cf366a8decbec5c07a71360f"].into(),
			// Pre-funded accounts
            vec![
                // Endow the Sudo account to cover transaction fees
                // 4M1k2CkBgitVm5tSyj9JWiWZDK8puzPzpQZZh8HdHHeRK16e
                hex!["86361c4e8b7ca00a64ecd1f655689bbe06339ae9cf366a8decbec5c07a71360f"].into(),
                // Endow the Treasury account with the DHX DAO Unlocked Reserves Balance
                // 4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk
                hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                // Endow these accounts with a balance so they may bond as authorities.
                // IMPORTANT: All authorities must be included in the list below so they have
                // an account balance to avoid session error
                // `assertion failed: frame_system::Module::<T>::inc_consumers(&account).is_ok()`

                // authority #0
                // cont
                hex!["76d0d3add586849d643444ff9fc55413222dea22d5fa297ba1b39386c79a6618"].into(),
                // stash
                hex!["a0dd28a829885c10b413095454a9f774b6cf3bc07f797c33afda2c2f87fb8f35"].into(),
                hex!["3a1de065f4e475584f4943559fbd3b57563117150c98bda1a6c9ebba0ecc8368"]
                    .unchecked_into(),
                // babe
                hex!["a8e804e4a03e5afa89d3d4cacc041562fd5145b4b2866eb191265aa9262acc06"]
                    .unchecked_into(),
                // imon
                hex!["78ae9f082530ca77f3e0b608df13bd7dc087049fd36d5c999ccc9a0fcb3b6517"]
                    .unchecked_into(),
                // audi
                hex!["029632937ad7171486a0c452f8bdb182a2b5a3311e003d7525fd360e059bb668"]
                    .unchecked_into(),

                // authority #1
                // cont
                hex!["a2c5ecadc845295b1c92585007aabd88826111a090908ec03eb6696b69211636"].into(),
                // stash
                hex!["5c680bbcad7e145f5ec27c27012cbabd32fe203208e96a3c080f8ef32164de51"].into(),
                // gran
                hex!["cab049448517b38e3fb22df1565d25365dd09e6f05af36d2ddd5f77b13a95921"]
                    .unchecked_into(),
                // babe
                hex!["9ca738e4af3a18b2b84c32e5e74a323ccb24aacc7f9337da23c0edaa589fe828"]
                    .unchecked_into(),
                // imon
                hex!["d2107868645df1c62ae2e794376ccd2708bfe88052d3298e5cf5ef76312bad6f"]
                    .unchecked_into(),
                // audi
                hex!["40ec24198297c21198c8cf3c2172e7096c3085de96bcef497561a8d81b67230d"]
                    .unchecked_into(),

                // authority #2
                // cont
                hex!["805f3114e41a99126fbc1a56955dbb29df9f267c68721fb6b3c88c4fdd9f0318"].into(),
                // stash
                hex!["62a92866e30a34c72bcf829ec86f7b6883f335151cbe27a99978d5b1ce2b5c31"].into(),
                // gran
                hex!["e260ef20e7d3fb5a7c4384b625cf548914fe03959dded41a1dc78206b8e1fa31"]
                    .unchecked_into(),
                // babe
                hex!["e2f238e6f968fd44f834967cf39c0c7896380401467cb76745db62ffbf942e4e"]
                    .unchecked_into(),
                // imon
                hex!["2063cc1a936f466fd58954d6b28e1b8c84298f0b3b61e64bbed10c74519a2c66"]
                    .unchecked_into(),
                // audi
                hex!["706a1ba3800edaf2f84bc80f7ad88b4c9ac74cc78aa99825a50517997a8feb1e"]
                    .unchecked_into(),

                // authority #3
                // cont
                hex!["283c513c1af26294775258f28cb3ab728e02481c16edf463d260a7d95ecfd66c"].into(),
                // stash
                hex!["0ac6bc8c098d6fcf819de3bf30b410e3fcd06d3b8b94f02d7ad0bdec808b686c"].into(),
                // gran
                hex!["464cfc0c22b9ab67a7e5d526d482dcbdf763e86994034bbf46ea2b007ae3583d"]
                    .unchecked_into(),
                // babe
                hex!["2aa395a47ed11d9fc30295350e7eb56e59681b49a71d10d9ecc3078b42017b32"]
                    .unchecked_into(),
                // imon
                hex!["76626a3b013571687f201106a3b789c907a5b6e34e918f1ace454f75b7464112"]
                    .unchecked_into(),
                // audi
                hex!["40b8f682f2778bd2107a379c8c9f0c01851e819c1a665ef0719944a7ed28b53c"]
                    .unchecked_into(),

                // authority #4
                // cont
                hex!["981114473ce826f7a000e97d33ad716dfc055f0e279ab2ea5972a8a2d2dfa706"].into(),
                // stash
                hex!["e4f734b12b3cd7977e1edf756255930bc5390844c098ccf699bfcdbe18fff536"].into(),
                // gran
                hex!["f4bc2127d317dbae6556f4884ba7e9ae2257f0079b66bd975157b599dc198401"]
                    .unchecked_into(),
                // babe
                hex!["229cecefd4737a64b87bbde740af607a0000bcd8770ee8470a33427b0944586d"]
                    .unchecked_into(),
                // imon
                hex!["f690950a9f553f42255cf3a71bd00e73ab55df31a0f16968829ad9291bf14971"]
                    .unchecked_into(),
                // audi
                hex!["3c1107461309ba8598b586b29887eebdafcd0b7c3208d251368b286d83ecad5f"]
                    .unchecked_into(),
            ],
			true,
		),
        vec![],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some("dhx-mainnet"),
        // Properties
        Some(properties),
        // Extensions
        Default::default(),
	))
}

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        babe,
        im_online,
        authority_discovery,
    }
}

// Testnet

// in testnet total supply should be 100m, with 30m (30%) going to DHX DAO unlocked reserves, and the remaining
// 70m split between the initial 8x accounts other than the reserves such that each should receive 8750
const TESTNET_INITIAL_ENDOWMENT: u128 = 8_750_000_000_000_000_000_000_000_u128; // 70M DHX
const TESTNET_INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u128 = 30_000_000_000_000_000_000_000_000_u128; // 30M DHX
const TESTNET_INITIAL_STASH: u128 = TESTNET_INITIAL_ENDOWMENT / 1000;

// Mainnet
const MAINNET_INITIAL_ENDOWMENT: u128 = 10_000_000_000_000_000_000_u128; // 10 DHX
const MAINNET_INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u128 = 30_000_000_000_000_000_000_000_000_u128; // 30M DHX
const MAINNET_INITIAL_STASH: u128 = MAINNET_INITIAL_ENDOWMENT / 10; // 1 DHX

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool, // No println
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();

	GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| {
                    // Insert Public key (hex) of the account without the 0x prefix below
                    if x == UncheckedFrom::unchecked_from(
                        hex!("6d6f646c70792f74727372790000000000000000000000000000000000000000").into(),
                    ) {
                        // If we use println, then the top of the chain specification file that gets
                        // generated contains the println, and then we have to remove the println from
                        // the top of that file to generate the "raw" chain definition
                        // println!("endowed_account treasury {:?}", x.clone());
                        return (x, TESTNET_INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
                    } else {
                        // println!("endowed_account {:?}", x.clone());
                        return (x, TESTNET_INITIAL_ENDOWMENT);
                    }
                })
            .collect(),
        }),
        pallet_indices: Some(IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone())))
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), TESTNET_INITIAL_STASH, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_democracy: Some(DemocracyConfig::default()),
        pallet_elections_phragmen: Some(ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, TESTNET_INITIAL_STASH))
                .collect(),
        }),
        pallet_collective_Instance1: Some(Default::default()),
        pallet_collective_Instance2: Some(Default::default()),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key.clone(),
        }),
        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(ImOnlineConfig {
            keys: vec![],
        }),
        pallet_authority_discovery: Some(AuthorityDiscoveryConfig {
            keys: vec![],
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_membership_Instance1: Some(TechnicalMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        }),
        pallet_treasury: Some(TreasuryConfig::default()),
	}
}

/// Configure initial storage state for FRAME modules.
fn mainnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool, // No println
) -> GenesisConfig {
    let num_endowed_accounts = endowed_accounts.len();

	GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| {
                    // Insert Public key (hex) of the account without the 0x prefix below
                    if x == UncheckedFrom::unchecked_from(
                        hex!("6d6f646c70792f74727372790000000000000000000000000000000000000000").into(),
                    ) {
                        // println!("endowed_account treasury {:?}", x.clone());
                        return (x, MAINNET_INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
                    } else {
                        // println!("endowed_account {:?}", x.clone());
                        return (x, MAINNET_INITIAL_ENDOWMENT);
                    }
                })
            .collect(),
        }),
        pallet_indices: Some(IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone())))
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), MAINNET_INITIAL_STASH, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_democracy: Some(DemocracyConfig::default()),
        pallet_elections_phragmen: Some(ElectionsConfig {
            members: endowed_accounts
                .iter()
                .take((num_endowed_accounts + 1) / 2)
                .cloned()
                .map(|member| (member, MAINNET_INITIAL_STASH))
                .collect(),
        }),
        pallet_collective_Instance1: Some(Default::default()),
        pallet_collective_Instance2: Some(Default::default()),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key.clone(),
        }),
        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(ImOnlineConfig {
            keys: vec![],
        }),
        pallet_authority_discovery: Some(AuthorityDiscoveryConfig {
            keys: vec![],
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_membership_Instance1: Some(TechnicalMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        }),
        pallet_treasury: Some(TreasuryConfig::default()),
	}
}
