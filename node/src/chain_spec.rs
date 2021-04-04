use datahighway_runtime::{
    opaque::{
        Block,
        SessionKeys,
    },
    wasm_binary_unwrap,
    AccountId,
    BabeConfig,
    BalancesConfig,
    GeneralCouncilMembershipConfig,
    GenesisConfig,
    GrandpaConfig,
    IndicesConfig,
    SessionConfig,
    Signature,
    StakerStatus,
    StakingConfig,
    SudoConfig,
    SystemConfig,
};
use hex_literal::hex;
use sc_chain_spec::ChainSpecExtension;
use sc_service;
use sc_telemetry::TelemetryEndpoints;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::map::Map;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;

use sc_service::ChainType;
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
    pub fork_blocks: sc_client_api::client::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::client::BadBlocks<Block>,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    // DataHighwayTestnet,
    DataHighwayTestnetLatest,
    DataHighwayTestnetHarbour,
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None).expect("static values are valid; qed").public()
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
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, BabeId) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
    )
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        let mut properties = Map::new();
        properties.insert("tokenSymbol".into(), "DHX".into());
        properties.insert("tokenDecimals".into(), 18.into());

        Ok(match self {
            Alternative::Development => {
                ChainSpec::from_genesis(
                    "Development",
                    "dev",
                    ChainType::Development,
                    || {
                        dev_genesis(
                            vec![get_authority_keys_from_seed("Alice")],
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            vec![
                                // DHX DAO Unlocked Reserves Balance
                                // Given a Treasury ModuleId in runtime parameter_types of
                                // `py/trsry`, we convert that to its associated address
                                // using Module ID" to Address" at https://www.shawntabrizi.com/substrate-js-utilities/,
                                // which generates 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z,
                                // and find its corresponding hex value by pasting the address into
                                // "AccountId to Hex" at that same link to return
                                // 6d6f646c70792f74727372790000000000000000000000000000000000000000.
                                // This is pallet_treasury's account_id.
                                //
                                // Substrate 2 does not have instantiable support for treasury
                                // is only supported in Substrate 3 and was fixed here
                                // https://github.com/paritytech/substrate/pull/7058
                                // so instead we will transfer funds to
                                //
                                // DHX DAO Unlocked Reserves Balance
                                // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                                hex!["6c029e6fc41ec44d420030071f04995bac19e59a0f0a1a610f9f0f6d689e2262"].into(),
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
                        )
                    },
                    vec![],
                    None,
                    None,
                    Some(properties),
                    Default::default(),
                )
            }
            Alternative::LocalTestnet => {
                ChainSpec::from_genesis(
                    "Local Testnet",
                    "local",
                    ChainType::Local,
                    || {
                        dev_genesis(
                            vec![
                                get_authority_keys_from_seed("Alice"),
                                get_authority_keys_from_seed("Bob"),
                                get_authority_keys_from_seed("Charlie"),
                                get_authority_keys_from_seed("Dave"),
                            ],
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            vec![
                                // DHX DAO Unlocked Reserves Balance
                                // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                                hex!["6c029e6fc41ec44d420030071f04995bac19e59a0f0a1a610f9f0f6d689e2262"].into(),
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
                        )
                    },
                    vec![],
                    Some(
                        TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                            .expect("Polkadot telemetry url is valid; qed"),
                    ),
                    None,
                    Some(properties),
                    Default::default(),
                )
            }
            // Alternative::DataHighwayTestnet => {
            //     ChainSpec::from_json_bytes(
            //         &include_bytes!("./chain-definition-custom/chain_def_testnet_latest.json")[..],
            //     )?
            // }
            // FIXME: Not working for some reason. Only 'local' works (error insufficient balance to bond)
            Alternative::DataHighwayTestnetLatest => {
                ChainSpec::from_genesis(
                    "DataHighway Testnet",
                    "testnet_latest",
                    ChainType::Live,
                    || {
                        // TODO: regenerate alphanet according to babe-grandpa consensus
                        // export SECRET=test && echo $SECRET
                        // ./target/release/subkey --sr25519 inspect "$SECRET//datahighway//aura"
                        // ./target/release/subkey --sr25519 inspect "$SECRET//datahighway//babe"
                        // ./target/release/subkey --sr25519 inspect "$SECRET//datahighway//imonline"
                        // ./target/release/subkey --ed25519 inspect "$SECRET//datahighway//grandpa"
                        // ./target/release/subkey inspect "$SECRET//datahighway//root"
                        testnet_genesis(
                            vec![
                                get_authority_keys_from_seed("Alice"),
                                get_authority_keys_from_seed("Bob"),
                                get_authority_keys_from_seed("Charlie"),
                                get_authority_keys_from_seed("Dave"),
                            ],
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            vec![
                                // Endow this account with the DHX DAO Unlocked Reserves Balance
                                // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                                hex!["6c029e6fc41ec44d420030071f04995bac19e59a0f0a1a610f9f0f6d689e2262"].into(),
                                // Endow these accounts with a balance so they may bond as authorities
                                get_account_id_from_seed::<sr25519::Public>("Alice"),
                                get_account_id_from_seed::<sr25519::Public>("Bob"),
                                get_account_id_from_seed::<sr25519::Public>("Charlie"),
                                get_account_id_from_seed::<sr25519::Public>("Dave"),
                                get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                            ],
                        )
                    },
                    vec![],
                    // telemetry endpoints
                    Some(
                        TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                            .expect("Polkadot telemetry url is valid; qed"),
                    ),
                    // protocol id
                    Some("dhx-test"),
                    // properties
                    Some(properties),
                    // extensions
                    Default::default(),
                )
            }
            Alternative::DataHighwayTestnetHarbour => {
                ChainSpec::from_genesis(
                    "DataHighway Harbour Testnet",
                    "harbour",
                    ChainType::Live,
                    || {
                        // TODO: regenerate alphanet according to babe-grandpa consensus
                        // subkey inspect "$SECRET"
                        // for i in 1 2 3 4; do for j in stash controller; do subkey inspect "$SECRET//$i//$j"; done;
                        // done for i in 1 2 3 4; do for j in babe; do subkey inspect
                        // --scheme=sr25519 "$SECRET//$i//$j"; done; done for i in 1 2 3 4; do
                        // for j in grandpa; do subkey inspect --scheme=ed25519 "$SECRET//$i//$j"; done; done
                        testnet_genesis(
                            vec![
                                (
                                    hex!["f64bae0f8fbe2eb59ff1c0ff760a085f55d69af5909aed280ebda09dc364d443"].into(),
                                    hex!["ca907b74f921b74638eb40c289e9bf1142b0afcdb25e1a50383ab8f9d515da0d"].into(),
                                    hex!["6a9da05f3e07d68bc29fb6cf9377a1537d59f082f49cb27a47881aef9fbaeaee"]
                                        .unchecked_into(),
                                    hex!["f2bf53bfe43164d88fcb2e83891137e7cf597857810a870b4c24fb481291b43a"]
                                        .unchecked_into(),
                                ),
                                (
                                    hex!["420a7b4a8c9f2388eded13c17841d2a0e08ea7c87eda84310da54f3ccecd3931"].into(),
                                    hex!["ae69db7838fb139cbf4f93bf877faf5bbef242f3f5aac6eb4f111398e9385e7d"].into(),
                                    hex!["9af1908ac74b042f4be713e10dcf6a2def3770cfce58951c839768e7d6bbcd8e"]
                                        .unchecked_into(),
                                    hex!["1e91a7902c89289f97756c4e20c0e9536f34de61c7c21af7773d670b0e644030"]
                                        .unchecked_into(),
                                ),
                                (
                                    hex!["ceecb6cc08c20ff44052ff19952a810d08363aa26ea4fb0a64a62a4630d37f28"].into(),
                                    hex!["7652b25328d78d264aef01184202c9771b55f5b391359309a2559ef77fbbb33d"].into(),
                                    hex!["b8902681768fbda7a29666e1de8a18f5be3c778d92cf29139959a86e6bff13e7"]
                                        .unchecked_into(),
                                    hex!["aaabcb653ce5dfd63035430dba10ce9aed5d064883b9e2b19ec5d9b26a457f57"]
                                        .unchecked_into(),
                                ),
                                (
                                    hex!["68bac5586028dd40db59a7becec349b42cd4229f9d3c31875c3eb7a57241cd42"].into(),
                                    hex!["eec96d02877a45fa524fcee1c6b7c849cbdc8cee01a95f5db168c427ae766849"].into(),
                                    hex!["f4807d86cca169a81d42fcf9c7abddeff107b0a73e9e7a809257ac7e4a164741"]
                                        .unchecked_into(),
                                    hex!["a49ac1053a40a2c7c33ffa41cb285cef7c3bc9db7e03a16d174cc8b5b5ac0247"]
                                        .unchecked_into(),
                                ),
                            ],
                            hex!["3c917f65753cd375582a6d7a1612c8f01df8805f5c8940a66e9bda3040f88f5d"].into(),
                            vec![
                                // Endow this account with the DHX DAO Unlocked Reserves Balance
                                // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                                hex!["6c029e6fc41ec44d420030071f04995bac19e59a0f0a1a610f9f0f6d689e2262"].into(),
                                // Endow these accounts with a balance so they may bond as authorities
                                hex!["ca907b74f921b74638eb40c289e9bf1142b0afcdb25e1a50383ab8f9d515da0d"].into(),
                                hex!["ae69db7838fb139cbf4f93bf877faf5bbef242f3f5aac6eb4f111398e9385e7d"].into(),
                                hex!["7652b25328d78d264aef01184202c9771b55f5b391359309a2559ef77fbbb33d"].into(),
                                hex!["eec96d02877a45fa524fcee1c6b7c849cbdc8cee01a95f5db168c427ae766849"].into(),
                                hex!["f64bae0f8fbe2eb59ff1c0ff760a085f55d69af5909aed280ebda09dc364d443"].into(),
                                hex!["420a7b4a8c9f2388eded13c17841d2a0e08ea7c87eda84310da54f3ccecd3931"].into(),
                                hex!["ceecb6cc08c20ff44052ff19952a810d08363aa26ea4fb0a64a62a4630d37f28"].into(),
                                hex!["68bac5586028dd40db59a7becec349b42cd4229f9d3c31875c3eb7a57241cd42"].into(),
                            ],
                        )
                    },
                    vec![],
                    // telemetry endpoints
                    Some(
                        TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
                            .expect("Polkadot telemetry url is valid; qed"),
                    ),
                    // protocol id
                    Some("dhx-test"),
                    // properties
                    Some(properties),
                    // extensions
                    Default::default(),
                )
            }
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "local" => Some(Alternative::LocalTestnet),
            // "" | "testnet" => Some(Alternative::DataHighwayTestnet),
            "testnet_latest" => Some(Alternative::DataHighwayTestnetLatest),
            "harbour" => Some(Alternative::DataHighwayTestnetHarbour),
            _ => None,
        }
    }
}

fn session_keys(grandpa: GrandpaId, babe: BabeId) -> SessionKeys {
    SessionKeys {
        grandpa,
        babe,
    }
}

// total supply should be 100m, with 30m (30%) going to DHX DAO unlocked reserves, and the remaining
// 70m split between the initial 8x accounts other than the reserves such that each should receive 8750
const INITIAL_BALANCE: u128 = 8_750_000_000_000_000_000_000_u128; // $70M 70_000_000_000_000_000_000_000_u128
const INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u128 = 30_000_000_000_000_000_000_000_u128; // $30M
const INITIAL_STAKING: u128 = 1_000_000_000_000_000_000_u128;

fn dev_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_indices: Some(IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| {
                    // Insert Public key (hex) of the account without the 0x prefix below
                    if x == UncheckedFrom::unchecked_from(
                        hex!("6c029e6fc41ec44d420030071f04995bac19e59a0f0a1a610f9f0f6d689e2262").into(),
                    ) {
                        println!("endowed_account treasury {:?}", x.clone());
                        return (x, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
                    } else {
                        println!("endowed_account {:?}", x.clone());
                        return (x, INITIAL_BALANCE);
                    }
                })
                .collect(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone())))
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_collective_Instance1: Some(Default::default()),
        pallet_membership_Instance1: Some(GeneralCouncilMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        }),
        pallet_treasury: Some(Default::default()),
    }
}

fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    // No println
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_indices: Some(IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| (x, INITIAL_BALANCE))
                .into_iter()
                .map(|k| (k.0, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE))
                .collect(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone())))
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_collective_Instance1: Some(Default::default()),
        pallet_membership_Instance1: Some(GeneralCouncilMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        }),
        pallet_treasury: Some(Default::default()),
    }
}
// Result<Box<ChainSpec>, String>
pub fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
    let option = match Alternative::from(id) {
        Some(spec) => Some(spec.load()?),
        _path => None,
    };

    let spec = Box::new(match option {
        Some(v) => v,
        None => ChainSpec::from_json_file(std::path::PathBuf::from(id))?,
    }) as Box<dyn sc_service::ChainSpec>;

    return Ok(spec);
}
