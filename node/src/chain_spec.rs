use sp_finality_grandpa::AuthorityId as GrandpaId;
use hex_literal::hex;
use datahighway_runtime::{
    // opaque::{
    //     Block,
    //     SessionKeys,
    // },
    // AuthorityDiscoveryConfig,
    AuraConfig,
    BalancesConfig,
    Block,
    DemocracyConfig,
    TechnicalMembershipConfig,
    ElectionsConfig,
    GenesisConfig,
    GrandpaConfig,
    // ImOnlineConfig,
    IndicesConfig,
    SessionConfig,
    SessionKeys,
    StakerStatus,
    // StakingConfig,
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
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sc_telemetry::TelemetryEndpoints;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::map::Map;
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
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, AuraId) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<AuraId>(seed),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Wasm not available".to_string())?;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "DEV".into());
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
                get_authority_keys_from_seed("Alice"),
                get_authority_keys_from_seed("Bob"),
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
    properties.insert("tokenSymbol".into(), "DEV".into());
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

// WARNING: The purpose of this testnet is for initial experiementation with
// multiple validator nodes where chaos such as bricking the chain is permitted,
// to avoid potentially bricking the DataHighway Harbour Testnet and impacting users.
pub fn datahighway_testnet_brickable_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Wasm binary not available".to_string())?;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "BRI".into());
    properties.insert("tokenDecimals".into(), 18.into());

    Ok(ChainSpec::from_genesis(
        // Name
        "DataHighway Brickable Testnet",
        // ID
        "brickable",
        ChainType::Live,
        move || testnet_genesis(
            wasm_binary,
            // Initial NPoS authorities
            // Note: stash must be before controller
            vec![
                // authority #1
                (
                    // stash
                    hex!["20ee614cc59285dcbca2b4d50c2e20490a87370d4de15baeda649e3538005d4f"].into(),
                    // cont
                    hex!["ba75230fdee3ff9f069bcf8047a52f0655ea5053a04e7de509e3b1d019c2b511"].into(),
                    // gran
                    hex!["edb0bfee980d12609f9641e1720ee4b2d4bee53e052c71e13580ee8f144a361c"]
                        .unchecked_into(),
                    // aura
                    hex!["1a0d59f23987acb275148a20f432bbbe81fdc950f607c2f4aa9218581590d17c"]
                        .unchecked_into(),
                ),
            ],
            // Sudo account
            // 4MF7atBumtP8vGUGG1888e798TCYfXrHMNt52BxW8P3CQNpm
            hex!["9068e3ce9b1055605a3bc4120e697b576c2ac6a13ee6f6ab751ad82e79eb4957"].into(),
            // Pre-funded accounts
            vec![
                // Endow the Sudo account to cover transaction fees
                hex!["9068e3ce9b1055605a3bc4120e697b576c2ac6a13ee6f6ab751ad82e79eb4957"].into(),
                // Endow the Treasury account with the DHX DAO Unlocked Reserves Balance
                // 4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk
                hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                // Endow these accounts with a balance so they may bond as authorities.
                // IMPORTANT: All authorities must be included in the list below so they have
                // an account balance to avoid session error
                // `assertion failed: frame_system::Module::<T>::inc_consumers(&account).is_ok()`

                // authority #1
                hex!["20ee614cc59285dcbca2b4d50c2e20490a87370d4de15baeda649e3538005d4f"].into(),
                hex!["ba75230fdee3ff9f069bcf8047a52f0655ea5053a04e7de509e3b1d019c2b511"].into(),
                hex!["edb0bfee980d12609f9641e1720ee4b2d4bee53e052c71e13580ee8f144a361c"].into(),
                hex!["1a0d59f23987acb275148a20f432bbbe81fdc950f607c2f4aa9218581590d17c"].into(),

                // authority #2
                hex!["18c8fc8aac47703f11e022b304a78fcff4b06f4723e6a5e748e7ae15106a8c06"].into(),
                hex!["cc135d9509883c963bc58bf987c5f66867a3bf4a09c65b30bfb7654e88178c4d"].into(),
                hex!["4bbcc0bb7e3f10a8ad097f00c8cd87ab647904e87c9103a46a3db20e74507bea"].into(),
                hex!["ca44c0b08b439e43f439bfd8d95ea7030a1a2ff42ac1e7e54fa94080bebe0f3b"].into(),

                // authority #3
                hex!["26f3fc47ec49a2981a95352dba050435f135d6a76f25cc489004e2ae098c8c5e"].into(),
                hex!["6eccb2e1ee65d161e85b45b45a2f5d859dd214c075d5c2a0aa35174281af3e76"].into(),
                hex!["f8aaa52bc3a0b168fb1bdcfd3d4fe4220cfd893593371c4c2d21defd605ffa4e"].into(),
                hex!["de973ff7e8784a03a356cadcd7a3b6e49e0aab9d36db6c7365579dcb9debb97b"].into(),

                // authority #4
                hex!["3e41a44cee0ade04ae35e7c96bfb2e4070c605aa6d6dcab77130afa594eced68"].into(),
                hex!["c450c238e0bba0afe639c4c9e1b0f254b9acb193c9e7c390938e229446ac6161"].into(),
                hex!["5526a6397e13b10a1d2b27114ad8322b8339cdbff9e47867cc11f500783d26ef"].into(),
                hex!["de072d2080256e333ee336527b886272170a3ab64180d22ab379fb7c513e114d"].into(),

                // authority #5
                hex!["ce3b5b27aca4be7f957a91a1704ae2ceb8910241f88058c04ef2b6df9a8adb18"].into(),
                hex!["0213b3d5ea6e0d760fa7183defcef9ae139d8c5728faf6eba22c2cf5c119b838"].into(),
                hex!["fdb309f030d63d422cdbaf079ade8fbc580aa7debf6c8a18b7db4c097ae1e854"].into(),
                hex!["98ddd9048757bf6995e430f6e58623c1b236471bdcb0d488242d3f8666acf645"].into(),
            ],
            true,
        ),
        vec![
            "/ip4/3.67.117.245/tcp/30333/p2p/12D3KooWBjSUFeT4RrkaTNH5da5LEzZ7M8KNQc9q5s1biNvBD42c"
                .parse()
                .unwrap(),
        ],
        // Telemetry Endpoints
        Some(
            TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Polkadot telemetry url is valid; qed"),
        ),
        // Protocol ID
        Some("dhx-test-brickable"),
        // Properties
        Some(properties),
        // Extensions
        Default::default(),
    ))
}

pub fn datahighway_testnet_harbour_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Wasm binary not available".to_string())?;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "HBR".into());
    properties.insert("tokenDecimals".into(), 18.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"DataHighway Harbour Testnet",
		// ID
		"harbour",
        ChainType::Live,
		// TODO: regenerate alphanet according to aura-grandpa consensus
		// subkey inspect "$SECRET"
		// for i in 1 2 3 4; do for j in stash controller; do subkey inspect "$SECRET//$i//$j"; done;
		// done for i in 1 2 3 4; do for j in aura; do subkey inspect
		// --scheme=sr25519 "$SECRET//$i//$j"; done; done for i in 1 2 3 4; do
		// for j in grandpa; do subkey inspect --scheme=ed25519 "$SECRET//$i//$j"; done; done
		move || testnet_genesis(
			wasm_binary,
			// Initial NPoS authorities
            vec![
                // authority #1
                (
                    // stash
                    hex!["f64bae0f8fbe2eb59ff1c0ff760a085f55d69af5909aed280ebda09dc364d443"].into(),
                    // cont
                    hex!["ca907b74f921b74638eb40c289e9bf1142b0afcdb25e1a50383ab8f9d515da0d"].into(),
                    // gran
                    hex!["6a9da05f3e07d68bc29fb6cf9377a1537d59f082f49cb27a47881aef9fbaeaee"]
                        .unchecked_into(),
                    // aura
                    hex!["3aaedf2ef9e32f7e90bf8eb9bf49813188f111cc349807afa67e07cdba9d225d"]
                        .unchecked_into(),
                ),
            ],
			// Sudo account
            hex!["3c917f65753cd375582a6d7a1612c8f01df8805f5c8940a66e9bda3040f88f5d"].into(),
			// Pre-funded accounts
            vec![
                // Endow the Sudo account to cover transaction fees
                hex!["3c917f65753cd375582a6d7a1612c8f01df8805f5c8940a66e9bda3040f88f5d"].into(),
                // Endow the Treasury account with the DHX DAO Unlocked Reserves Balance
                // 4LTFqiD6H6g8a7ur9WH4RxhWx2givWfK7o5EDed3ai1nYTvk
                hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                // Endow these accounts with a balance so they may bond as authorities.
                // IMPORTANT: All authorities must be included in the list below so they have
                // an account balance to avoid session error
                // `assertion failed: frame_system::Module::<T>::inc_consumers(&account).is_ok()`

                // authority #1
                hex!["f64bae0f8fbe2eb59ff1c0ff760a085f55d69af5909aed280ebda09dc364d443"].into(),
                hex!["ca907b74f921b74638eb40c289e9bf1142b0afcdb25e1a50383ab8f9d515da0d"].into(),
                hex!["6a9da05f3e07d68bc29fb6cf9377a1537d59f082f49cb27a47881aef9fbaeaee"].into(),
                hex!["3aaedf2ef9e32f7e90bf8eb9bf49813188f111cc349807afa67e07cdba9d225d"].into(),

                // authority #2
                hex!["420a7b4a8c9f2388eded13c17841d2a0e08ea7c87eda84310da54f3ccecd3931"].into(),
                hex!["ae69db7838fb139cbf4f93bf877faf5bbef242f3f5aac6eb4f111398e9385e7d"].into(),
                hex!["9af1908ac74b042f4be713e10dcf6a2def3770cfce58951c839768e7d6bbcd8e"].into(),
                hex!["c46b84a4af0a79efa0de5194816b19650a24221e858f41d564d4843d2691ad32"].into(),

                // authority #3
                hex!["ceecb6cc08c20ff44052ff19952a810d08363aa26ea4fb0a64a62a4630d37f28"].into(),
                hex!["7652b25328d78d264aef01184202c9771b55f5b391359309a2559ef77fbbb33d"].into(),
                hex!["b8902681768fbda7a29666e1de8a18f5be3c778d92cf29139959a86e6bff13e7"].into(),
                hex!["367c0c647c9c417de992cfa762758e237bea5ff61653c69d29bd924ad6c4476e"].into(),

                // authority #4
                hex!["68bac5586028dd40db59a7becec349b42cd4229f9d3c31875c3eb7a57241cd42"].into(),
                hex!["eec96d02877a45fa524fcee1c6b7c849cbdc8cee01a95f5db168c427ae766849"].into(),
                hex!["f4807d86cca169a81d42fcf9c7abddeff107b0a73e9e7a809257ac7e4a164741"].into(),
                hex!["9a861c7c72bdf2105766e90376069a4f3237c6722fdcf27ca4bb37d0fc22be5d"].into(),

                // authority #5
                hex!["181319cac9b915a33586196cb1ed64b7f37f9d50ae9f5c04b0d92ecdb342cd0a"].into(),
                hex!["80c17c61a1f2f6252b23677510cbd0c3f3ad1ad8694dba9170e98ab430100a7f"].into(),
                hex!["a240fbd4575d03b4c62e2d2d546327e393db5fc508bc92203fba354a3232f006"].into(),
                hex!["b49084ae7915090d935213fb9468c123b3c86fafa978096324fe65e1c430f364"].into(),

                // authority #6
                hex!["04034be760fa29265c5df75891253c28add3d9dc6acc592ab287e4b4e2bdcb13"].into(),
                hex!["d0dca20158075b94a46aede114a3180d762a7deb9723baa5b3881bda19335716"].into(),
                hex!["16dcf1c6f2c9c37312d34f1f418e758eb9c97cbaf9ef10b06b6f3b4b4b33f724"].into(),
                hex!["4043544ca717ff30e6a626d981475fe2c4597efc61c121d39e7bcd0d5a46e51b"].into(),

                // authority #7
                hex!["1e7171d63c29cf75a022f9c7cf817774eb9976a481991fcd7a28c7286d5ec34e"].into(),
                hex!["4eba5183abe641a421249343037087f48cf3eba9c2628d566bd0e347f619db4a"].into(),
                hex!["3a592141f7aaef64444aa506a590fdf5769834b74b47f5087eda97ac6833b23f"].into(),
                hex!["34ae06aec9774add209bf9642ba1025a0d8903682e30f529ccb5e00a10d6814e"].into(),

                // authority #8
                hex!["3af49d16869a494380772e47fb2d92f114c3190e291ed655c3bf9b9b5df52e42"].into(),
                hex!["98758865268f1a6f7c1307b6ffe18cb764ce9d57561c97423ac145c8d84a4957"].into(),
                hex!["14d83dc11288148747df1f92d11ad4c5b42dbb12f4aa69f679b36f1f84d41ae0"].into(),
                hex!["46be5c8f44e80d096f11fdb9eb02606f838defbaa4df302b6c81ffdf7a41c615"].into(),

                // authority #9
                hex!["603af684fbf8a984af4cc7e147673b576f91da1b3f55e0835a5655a4470e7f22"].into(),
                hex!["b00da5291fc18c973700c7aeab0ca13bdb9f3e48127657c7c9273e75eb7eb27d"].into(),
                hex!["c95b718120a73e30ba70bb2a9d369eeb87ed1f5708f21e66b6cb5d7bcfb8c8f7"].into(),
                hex!["04b1679abdb4a8fe393a98a4763fbc08b6f381232c78d47b5b326ce8e7895620"].into(),

                // authority #10
                hex!["322b80c5529a20a1da5702acfb78879211eed2110d687ca001309c1c6d57030f"].into(),
                hex!["48357721e05e42c153e3f33e739b56c4f711762cb45c5463c0b0891bd49fa64b"].into(),
                hex!["f44fcaa91171530462d0d43225354d09c0e64fc9ac7e6bed017279947d6a4785"].into(),
                hex!["6cf5b174d8f18fc6e50e8f15aaeaa55d91da5461f7fd37eb82450cc2f9f7295b"].into(),
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
                // authority #1
                (
                    // stash
                    hex!["3c62839f3ce86df3c27c66016b092d15186b23e429db4ad8338501fc219bfa6c"].into(),
                    // cont
                    hex!["42b728d9c752fe87c3e3db40d9a7d02f22b81bc1f0e49c59a5e128b861f87b08"].into(),
                    // gran
                    hex!["dce69d42cf6c256e1ba1595300d72797429dc415f9803e54e822416b6748dfa2"]
                        .unchecked_into(),
                    // aura
                    hex!["f8ee54f09b5578ffa74921d48f2f930e2668b1e2484e2c1f50f14810ba967c70"]
                        .unchecked_into(),
                ),
            ],
			// Sudo account
            hex!["c201d4551d04a99772d8efe196490a96b4ee5e608ac8e495be9505a99e723069"].into(),
			// Pre-funded accounts
            vec![
                // Endow the Sudo account to cover transaction fees
                hex!["c201d4551d04a99772d8efe196490a96b4ee5e608ac8e495be9505a99e723069"].into(),
                // Endow the Treasury account with the DHX DAO Unlocked Reserves Balance
                hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                // Endow these accounts with a balance so they may bond as authorities.
                // IMPORTANT: All authorities must be included in the list below so they have
                // an account balance to avoid session error
                // `assertion failed: frame_system::Module::<T>::inc_consumers(&account).is_ok()`

                // authority #1
                // stash
                hex!["3c62839f3ce86df3c27c66016b092d15186b23e429db4ad8338501fc219bfa6c"].into(),
                // cont
                hex!["42b728d9c752fe87c3e3db40d9a7d02f22b81bc1f0e49c59a5e128b861f87b08"].into(),
                // gran
                hex!["dce69d42cf6c256e1ba1595300d72797429dc415f9803e54e822416b6748dfa2"].into(),
                // aura
                hex!["f8ee54f09b5578ffa74921d48f2f930e2668b1e2484e2c1f50f14810ba967c70"].into(),

                // authority #2
                // stash
                hex!["628580490f55f5340d8a5e5af85eb78fdc066f6ac0385ccb3d17dc9a8b3e651f"].into(),
                // cont
                hex!["e627945747e5a66afd5ff3f383819dfb6dd9633f6c9d52b24b10307cc841d577"].into(),
                // gran
                hex!["5e5101464eb9a9d2637a18627632e0817c8592472fc498da76661260337398d9"].into(),
                // aura
                hex!["7235c488d320c666fa55d7058765e378972dcac2bd1287edea378aa7cfc01f17"].into(),

                // authority #3
                // stash
                hex!["44d784f6d346d337a98e273f683590c661ea20b83aa6c9ac93754e02cee4372e"].into(),
                // cont
                hex!["32e28b6f06a728b13d782b007d57ab53d7fe3fa2d5def2c58585d27007d5ea05"].into(),
                // gran
                hex!["4b0d281fbe2d89bae3955146f10e79c1e63db6aa0d463dc574abafa79656e859"].into(),
                // aura
                hex!["8615111d8d7e915c54e592a2207423fa9752d46c40bb780c063893df9aa58e72"].into(),

                // authority #4
                // stash
                hex!["1e7c18d12311c46ca4fd9fba48f11ea81c7c87e992fc5b1abb4903219cc6b429"].into(),
                // cont
                hex!["3ee5ed5cdf314660b60c6acb87d51af5f10c5d8ea24dd762df4c5973a1683416"].into(),
                // gran
                hex!["d2e23c0445afc9e714a9ad9307255150e26e5661f1f2fa57cbb35666f1ff3bfd"].into(),
                // aura
                hex!["849a75365f627b2640e6d95b56be0a116f8b11223e412fe4c343a5c181460e61"].into(),

                // authority #5
                // stash
                hex!["b0e6b234c77cc3d9fde0f3a039779c8d6ae0a66cb95d764acc54667e74f4c800"].into(),
                // cont
                hex!["2643aaa4cf95aa0f014c015722e31a356eb8bd17888f74bca1ca56c404445c39"].into(),
                // gran
                hex!["683d89df05242920e0fdfac6f854b6c96f2fba7934b845bfb60869eeb21549b1"].into(),
                // aura
                hex!["d80d23d1a64c6adc0cbccb5b6a5fa36497dc0a63ee1ddd9c94cd6d124c7cb148"].into(),

                // authority #6
                // stash
                hex!["48e771e75097d353d06b5fd469edf8cb53d8c9b8eeeb1aa0f480ff7a42e27f28"].into(),
                // cont
                hex!["3e028f22ca42f5feee5344c3319ba97d23019087a124b43a825cb7d35ed7d522"].into(),
                // gran
                hex!["215a2ff9562ae7b2f71e07c5976637dfd9dae4c092e18af9828a078ba57c0da0"].into(),
                // aura
                hex!["a2db1487e879113b74902b0029cec7a8fec06d237e151e7e3a2d59dfd25ed109"].into(),

                // authority #7
                // stash
                hex!["1ae9a95688b6dcf6db83c61789e59352cdf363c5b13d39f37c27f7434439027e"].into(),
                // cont
                hex!["76ea25fc43fbdd113efabf6a12ea1f67c2916f9dd15d7c08a020269aa28cd521"].into(),
                // gran
                hex!["ef3f7c3d180b63988ab894d9461a8b989d215604e7c5dbb2ce07f51733c680c5"].into(),
                // aura
                hex!["d66319d543658bcd1d49cc32f60371c75f70e5c5c0ee584042e7f0e679675b4a"].into(),

                // authority #8
                // stash
                hex!["ec9ee2c38014483a454cfe108cc062e0ff641fef7c5c10f0a0f797cfdb860708"].into(),
                // cont
                hex!["ee323b965d01799f4af213347549ab5e8e533071df69c4d2ed122a354deb930c"].into(),
                // gran
                hex!["a50919f31950b902e110f0e455ca2307b021f062c24b087fd94a922be68c1618"].into(),
                // aura
                hex!["887d9e05cb095e54bce0e6bef236744309b0752da8b46d6a888333186c847f6d"].into(),

                // authority #9
                // stash
                hex!["746be7c22192aa11d46d3b2bdad13cc77b1bab69ab9eb10799b629dd7ddfbf7e"].into(),
                // cont
                hex!["cc517121c11d0135836a62816526bb40d9c9a0ae47f316c646c148a5cff0f200"].into(),
                // gran
                hex!["bcd9f49d8a3ce7f0cd71e7effbdfa1aa472f2bd7c66782a71d25be7d98a2c60f"].into(),
                // aura
                hex!["0ee9c2b0b13316df6f3cd6f6347acd5957eb2b25ac1e7299712ecfce561f140a"].into(),

                // authority #10
                // stash
                hex!["ecc71d63ef4b80d8feab189a764cd3e759f941062f9b07168bbfcb814da7bf33"].into(),
                // cont
                hex!["3c71cfbd77668301af5aefe0c81e3b4ffc1c0f0b07c0d42044922237d574455f"].into(),
                // gran
                hex!["55adaf3d56e97313351424ae62742678ead13c5835dc90b7d1a138ba300f8b93"].into(),
                // aura
                hex!["9e22874296ecae9b53a2c8cde67e65696f28c163e6e572b302c172495e99137c"].into(),
            ],
			true,
		),
        vec![
            // authority #1
            "/ip4/3.127.123.230/tcp/30333/p2p/12D3KooWPSVWEpuNPKE6EJBAMQQRCrKG4RTfyyabFRjT4xqMkuH5"
                .parse()
                .unwrap(),
            // authority #2
            "/ip4/3.65.196.4/tcp/30333/p2p/12D3KooWPZqAuWSez5uomot7GZvpuRQK198zqLYrLLZt5W7bvqPb"
                .parse()
                .unwrap(),
            // authority #3
            "/ip4/3.123.21.153/tcp/30333/p2p/12D3KooWAjdURBpSsRVWbvnGRbsqykvueM6Vuoe4x7MhV6cxTtje"
                .parse()
                .unwrap(),
            // authority #4
            "/ip4/18.184.76.132/tcp/30333/p2p/12D3KooWCWZc5L6ypCFcvDdGeGwsw9Mo4nniCwiVuU5MB6ApA4ZT"
                .parse()
                .unwrap(),
            // authority #5
            "/ip4/3.124.189.68/tcp/30333/p2p/12D3KooWJ1F4BsNgeaVkZVPw2kRhHxAtJuUqeEik2R7dv9ttgPcv"
                .parse()
                .unwrap(),
            // authority #6
            "/ip4/104.236.197.177/tcp/30333/p2p/12D3KooWGgVUU6V4MNqhw4Fcbb7u5abEdD2QgLgx3TKmVXovcUft"
                .parse()
                .unwrap(),
            // authority #7
            "/ip4/104.236.197.174/tcp/30333/p2p/12D3KooWFGzcJWw7a1q1Sgn8qiLgnQ8UBy3DZqP33ZhrEXabgpm5"
                .parse()
                .unwrap(),
            // authority #8
            "/ip4/104.236.197.180/tcp/30333/p2p/12D3KooWFz44eN1nhVEAeq4x7Z4Hdd6GA9dhpc9LSXDmJA4aqfd6"
                .parse()
                .unwrap(),
            // authority #9
            "/ip4/104.236.197.172/tcp/30333/p2p/12D3KooWJjuKnSjF3fgrsAPv1b3VZH2nd7qzcVmh9imPNJTbEtSV"
                .parse()
                .unwrap(),
            // authority #10
            "/ip4/104.236.197.182/tcp/30333/p2p/12D3KooWST5nKEAFNXnLLQvjnAX88D99yF8Y1XVebSybVgcLDJzz"
                .parse()
                .unwrap(),
        ],
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
    aura: AuraId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        aura,
    }
}

// Testnet

// in testnet total supply should be 100m, with 30m (30%) going to DHX DAO unlocked reserves, and the remaining
// 70m split between the initial accounts other than the reserves
const TESTNET_INITIAL_ENDOWMENT: u128 = 10_000_000_000_000_000_000_u128; // 10 DHX
const TESTNET_INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u128 = 30_000_000_000_000_000_000_000_000_u128; // 30M DHX
const TESTNET_INITIAL_STASH: u128 = MAINNET_INITIAL_ENDOWMENT / 10; // 1 DHX

// Mainnet
const MAINNET_INITIAL_ENDOWMENT: u128 = 10_000_000_000_000_000_000_u128; // 10 DHX
const MAINNET_INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u128 = 30_000_000_000_000_000_000_000_000_u128; // 30M DHX
const MAINNET_INITIAL_STASH: u128 = MAINNET_INITIAL_ENDOWMENT / 10; // 1 DHX

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, AuraId)>,
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
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone())))
                .collect::<Vec<_>>(),
        }),
        // pallet_staking: Some(StakingConfig {
        //     validator_count: initial_authorities.len() as u32 * 2,
        //     minimum_validator_count: initial_authorities.len() as u32,
        //     stakers: initial_authorities
        //         .iter()
        //         .map(|x| (x.0.clone(), x.1.clone(), TESTNET_INITIAL_STASH, StakerStatus::Validator))
        //         .collect(),
        //     invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
        //     slash_reward_fraction: Perbill::from_percent(10),
        //     ..Default::default()
        // }),
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
        pallet_aura: Some(AuraConfig {
            // empty otherwise `Thread 'main' panicked at 'Authorities are already initialized!`
            authorities: initial_authorities.iter().map(|x| (x.3.clone())).collect(),
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
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
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, AuraId)>,
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
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone())))
                .collect::<Vec<_>>(),
        }),
        // pallet_staking: Some(StakingConfig {
        //     validator_count: initial_authorities.len() as u32 * 2,
        //     minimum_validator_count: initial_authorities.len() as u32,
        //     stakers: initial_authorities
        //         .iter()
        //         .map(|x| (x.0.clone(), x.1.clone(), MAINNET_INITIAL_STASH, StakerStatus::Validator))
        //         .collect(),
        //     invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
        //     slash_reward_fraction: Perbill::from_percent(10),
        //     ..Default::default()
        // }),
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
        pallet_aura: Some(AuraConfig {
            // empty otherwise `Thread 'main' panicked at 'Authorities are already initialized!`
            authorities: initial_authorities.iter().map(|x| (x.3.clone())).collect(),
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
        }),
        pallet_membership_Instance1: Some(TechnicalMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        }),
        pallet_treasury: Some(TreasuryConfig::default()),
	}
}
