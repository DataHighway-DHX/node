use cumulus_primitives_core::ParaId;
use datahighway_parachain_runtime::{
    AccountId,
    AuraId,
    AuraConfig,
    BalancesConfig,
    GeneralCouncilMembershipConfig,
    GenesisConfig,
    Signature,
    SudoConfig,
};
use hex_literal::hex;
use sc_chain_spec::{
    ChainSpecExtension,
    ChainSpecGroup,
};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{
    Deserialize,
    Serialize,
};
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

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<datahighway_parachain_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None).expect("static values are valid; qed").public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

// Note this is the URL for the telemetry server
const POLKADOT_STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Local,
        move || {
            dev_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                id,
            )
        },
        vec![],
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "rococo-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            dev_genesis(
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                id,
            )
        },
        Vec::new(),
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "rococo-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn rococo_parachain_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "DataHighway",
        "datahighway",
        ChainType::Live,
        move || {
            spreehafen_testnet_genesis(
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
                hex!["3c917f65753cd375582a6d7a1612c8f01df8805f5c8940a66e9bda3040f88f5d"].into(),
                vec![],
                id,
            )
        },
        boot_nodes,
        None,
        Some("dhx"),
        Some(properties),
        Extensions {
            relay_chain: "rococo-chachacha".into(),
            para_id: 2_u32.into(),
        },
    )
}

// total supply should be 100m, with 30m (30%) going to DHX DAO unlocked reserves, and the remaining
// 70m split between the initial 8x accounts other than the reserves such that each should receive 8750
const INITIAL_BALANCE: u128 = 8_750_000_000_000_000_000_000_u128; // $70M 70_000_000_000_000_000_000_000_u128
const INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u128 = 30_000_000_000_000_000_000_000_u128; // $30M
// const INITIAL_STAKING: u128 = 1_000_000_000_000_000_000_u128;

fn spreehafen_testnet_genesis(
    endowed_accounts: Vec<AccountId>,
    root_key: AccountId,
    initial_authorities: Vec<AuraId>,
    id: ParaId
) -> GenesisConfig {
    GenesisConfig {
        system: datahighway_parachain_runtime::SystemConfig {
            code: datahighway_parachain_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| (x, INITIAL_BALANCE))
                .into_iter()
                .map(|k| (k.0, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE))
                .collect(),
        },
        general_council: Default::default(),
        general_council_membership: GeneralCouncilMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        },
        pallet_treasury: Default::default(),
        sudo: SudoConfig {
            key: root_key.clone(),
        },
        parachain_info: datahighway_parachain_runtime::ParachainInfoConfig {
            parachain_id: id,
        },
        aura: AuraConfig { authorities: initial_authorities },
        aura_ext: Default::default(),
        parachain_system: Default::default(),
    }
}

fn testnet_genesis(
    root_key: AccountId,
    initial_authorities: Vec<AuraId>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> GenesisConfig {
    GenesisConfig {
        system: datahighway_parachain_runtime::SystemConfig {
            code: datahighway_parachain_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| (x, INITIAL_BALANCE))
                .into_iter()
                .map(|k| (k.0, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE))
                .collect(),
        },
        sudo: SudoConfig {
            key: root_key.clone(),
        },
        general_council: Default::default(),
        general_council_membership: GeneralCouncilMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        },
        pallet_treasury: Default::default(),
        parachain_info: datahighway_parachain_runtime::ParachainInfoConfig {
            parachain_id: id,
        },
        aura: AuraConfig { authorities: initial_authorities },
        aura_ext: Default::default(),
        parachain_system: Default::default(),
    }
}

fn dev_genesis(
    root_key: AccountId,
    initial_authorities: Vec<AuraId>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> datahighway_parachain_runtime::GenesisConfig {
    datahighway_parachain_runtime::GenesisConfig {
        system: datahighway_parachain_runtime::SystemConfig {
            code: datahighway_parachain_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: datahighway_parachain_runtime::BalancesConfig {
            balances: endowed_accounts.iter().cloned().map(|x|
                // Insert Public key (hex) of the account without the 0x prefix below
                if x == UncheckedFrom::unchecked_from(hex!("a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21").into()) {
                    (x, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE)
                } else {
                    (x, INITIAL_BALANCE)
                }
            )
            .collect(),
        },
        sudo: datahighway_parachain_runtime::SudoConfig {
            key: root_key.clone(),
        },
        general_council: Default::default(),
        general_council_membership: GeneralCouncilMembershipConfig {
            members: vec![root_key],
            phantom: Default::default(),
        },
        pallet_treasury: Default::default(),
        parachain_info: datahighway_parachain_runtime::ParachainInfoConfig {
            parachain_id: id,
        },
        aura: datahighway_parachain_runtime::AuraConfig { authorities: initial_authorities },
        aura_ext: Default::default(),
        parachain_system: Default::default(),
    }
}
