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

pub fn rococo_development_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            dev_genesis(
                vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    hex!["a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21"].into(),
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
            relay_chain: "rococo-dev".into(),
            para_id: id.into(),
        },
    )
}

pub fn rococo_local_testnet_config(id: ParaId) -> ChainSpec {
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
            testnet_genesis(
                vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    hex!["a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21"].into(),
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

pub fn chachacha_development_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            dev_genesis(
                vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    hex!["a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21"].into(),
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
            relay_chain: "chachacha-dev".into(),
            para_id: id.into(),
        },
    )
}

pub fn chachacha_local_testnet_config(id: ParaId) -> ChainSpec {
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
            testnet_genesis(
                vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    hex!["a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21"].into(),
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
            relay_chain: "chachacha-local".into(),
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
        "DataHighway Spreehafen Parachain Testnet",
        "datahighway_spreehafen",
        ChainType::Live,
        move || {
            spreehafen_testnet_genesis(
                vec![
                    // authority #1
                    (
                        //aura
                        hex!["106c208ac262aa3733629ad0860d0dc72d8b9152e1cdcab497949a3f9504517a"].unchecked_into()
                    ),
                    // authority #2
                    (
                        //aura
                        hex!["0234df0fce3e763e02b6644e589bd256bbd45121bdf6d98dd1cf1072b6228859"].unchecked_into()
                    ),
                    // authority #3
                    (
                        //aura
                        hex!["02fe175463b5c7c378416e06780f7c60520d4dbcf759a7634a311e562e13a765"].unchecked_into()
                    ),
                    // authority #4
                    (
                        //aura
                        hex!["ea239700d67f53d30e39bee0c056f1165a6fb59ad4d5dd495c06d001af366c02"].unchecked_into()
                    )

                ],
                hex!["c8c0ee501c4b115f08f677082b0f2beb59bd18f54f141588792e989bfb54e415"].into(),
                vec![
                    // Endow the Sudo account to cover transaction fees
                    hex!["c8c0ee501c4b115f08f677082b0f2beb59bd18f54f141588792e989bfb54e415"].into(),
                    // Endow this account with the DHX DAO Unlocked Reserves Balance
                    // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                    hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                    // Endow these accounts with a balance so they may bond as authorities
                    hex!["b2f1decb9c6a1e6df2cd7e7b73d6c7eada3683d958b2fed451fb045d2f7cdb55"].into(),
                    hex!["b2347d115c9300a433a59b0ef321430a6d418d0555a6a41dfebe99fb86765110"].into(),
                    hex!["f4062d6d4ac30ea04659b24994cc0ebf249fed1591e6cf1c25d5f4f78e78bb6b"].into(),
                    hex!["a0d56496c02c203312ebce4a2804c7e0c31e34f983b9bc037f7c95f34e416613"].into(),
                    hex!["467da0333f16ce430bfa18fb8c25cfbbc49f35946370989280aaf3142fff7344"].into(),
                    hex!["ac691d2b336f8347a22eb3831b381e4adac45ab6f0ad85abc1336633313f173d"].into(),
                    hex!["4cad3775c026114d4a6e965f72caf11c18eb03ea7a3b4c0516f4cb8856b2575f"].into(),
                    hex!["6cd4eeb38c45a073d3c8e3ddd24e2502707060f33a1d92e082e32c106512500f"].into(),
                ],
                id,
            )
        },
        boot_nodes,
        None,
        Some("dhx"),
        Some(properties),
        Extensions {
            relay_chain: "rococo".into(),
            para_id: id.into(),
        },
    )
}

pub fn chachacha_parachain_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "DataHighway Spreehafen Parachain Testnet",
        "datahighway_spreehafen",
        ChainType::Live,
        move || {
            spreehafen_testnet_genesis(
                vec![
                    // authority #1
                    (
                        //aura
                        hex!["106c208ac262aa3733629ad0860d0dc72d8b9152e1cdcab497949a3f9504517a"].unchecked_into()
                    ),
                    // authority #2
                    (
                        //aura
                        hex!["0234df0fce3e763e02b6644e589bd256bbd45121bdf6d98dd1cf1072b6228859"].unchecked_into()
                    ),
                    // authority #3
                    (
                        //aura
                        hex!["02fe175463b5c7c378416e06780f7c60520d4dbcf759a7634a311e562e13a765"].unchecked_into()
                    ),
                    // authority #4
                    (
                        //aura
                        hex!["ea239700d67f53d30e39bee0c056f1165a6fb59ad4d5dd495c06d001af366c02"].unchecked_into()
                    )

                ],
                hex!["c8c0ee501c4b115f08f677082b0f2beb59bd18f54f141588792e989bfb54e415"].into(),
                vec![
                    // Endow the Sudo account to cover transaction fees
                    hex!["c8c0ee501c4b115f08f677082b0f2beb59bd18f54f141588792e989bfb54e415"].into(),
                    // Endow this account with the DHX DAO Unlocked Reserves Balance
                    // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                    hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                    // Endow these accounts with a balance so they may bond as authorities
                    hex!["b2f1decb9c6a1e6df2cd7e7b73d6c7eada3683d958b2fed451fb045d2f7cdb55"].into(),
                    hex!["b2347d115c9300a433a59b0ef321430a6d418d0555a6a41dfebe99fb86765110"].into(),
                    hex!["f4062d6d4ac30ea04659b24994cc0ebf249fed1591e6cf1c25d5f4f78e78bb6b"].into(),
                    hex!["a0d56496c02c203312ebce4a2804c7e0c31e34f983b9bc037f7c95f34e416613"].into(),
                    hex!["467da0333f16ce430bfa18fb8c25cfbbc49f35946370989280aaf3142fff7344"].into(),
                    hex!["ac691d2b336f8347a22eb3831b381e4adac45ab6f0ad85abc1336633313f173d"].into(),
                    hex!["4cad3775c026114d4a6e965f72caf11c18eb03ea7a3b4c0516f4cb8856b2575f"].into(),
                    hex!["6cd4eeb38c45a073d3c8e3ddd24e2502707060f33a1d92e082e32c106512500f"].into(),
                ],
                id,
            )
        },
        boot_nodes,
        None,
        Some("dhx"),
        Some(properties),
        Extensions {
            relay_chain: "chachacha".into(),
            para_id: id.into(),
        },
    )
}

// total supply should be 100m, with 30m (30%) going to DHX DAO unlocked reserves, and the remaining
// 70m split between the initial 8x accounts other than the reserves such that each should receive 8750
const INITIAL_BALANCE: u128 = 8_750_000_000_000_000_000_000_u128; // $70M 70_000_000_000_000_000_000_000_u128
const INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u128 = 30_000_000_000_000_000_000_000_u128; // $30M
// const INITIAL_STAKING: u128 = 1_000_000_000_000_000_000_u128;

fn spreehafen_testnet_genesis(
    initial_authorities: Vec<(AuraId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
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
                .map(|x| {
                    // Insert Public key (hex) of the account without the 0x prefix below
                    if x == UncheckedFrom::unchecked_from(
                        hex!("6d6f646c70792f74727372790000000000000000000000000000000000000000").into(),
                    ) {
                        // If we use println, then the top of the chain specification file that gets
                        // generated contains the println, and then we have to remove the println from
                        // the top of that file to generate the "raw" chain definition
                        // println!("endowed_account treasury {:?}", x.clone());
                        return (x, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
                    } else {
                        // println!("endowed_account {:?}", x.clone());
                        return (x, INITIAL_BALANCE);
                    }
                })
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
    initial_authorities: Vec<AuraId>,
    root_key: AccountId,
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
                .map(|x| {
                    // Insert Public key (hex) of the account without the 0x prefix below
                    if x == UncheckedFrom::unchecked_from(
                        hex!("a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21").into(),
                    ) {
                        // If we use println, then the top of the chain specification file that gets
                        // generated contains the println, and then we have to remove the println from
                        // the top of that file to generate the "raw" chain definition
                        // println!("endowed_account treasury {:?}", x.clone());
                        return (x, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
                    } else {
                        // println!("endowed_account {:?}", x.clone());
                        return (x, INITIAL_BALANCE);
                    }
                })
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
    initial_authorities: Vec<AuraId>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> datahighway_parachain_runtime::GenesisConfig {
    datahighway_parachain_runtime::GenesisConfig {
        system: datahighway_parachain_runtime::SystemConfig {
            code: datahighway_parachain_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| {
                    // Insert Public key (hex) of the account without the 0x prefix below
                    if x == UncheckedFrom::unchecked_from(
                        hex!("a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21").into(),
                    ) {
                        // If we use println, then the top of the chain specification file that gets
                        // generated contains the println, and then we have to remove the println from
                        // the top of that file to generate the "raw" chain definition
                        // println!("endowed_account treasury {:?}", x.clone());
                        return (x, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
                    } else {
                        // println!("endowed_account {:?}", x.clone());
                        return (x, INITIAL_BALANCE);
                    }
                })
                .collect(),
        },
        sudo: SudoConfig {
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
        aura: AuraConfig { authorities: initial_authorities },
        aura_ext: Default::default(),
        parachain_system: Default::default(),
    }
}
