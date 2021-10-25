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

pub fn westend_development_config(id: ParaId) -> ChainSpec {
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
            relay_chain: "westend-dev".into(),
            para_id: id.into(),
        },
    )
}

pub fn westend_local_testnet_config(id: ParaId) -> ChainSpec {
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
            relay_chain: "westend-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn polkadot_development_config(id: ParaId) -> ChainSpec {
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
            relay_chain: "polkadot-dev".into(),
            para_id: id.into(),
        },
    )
}

pub fn polkadot_local_testnet_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 18.into());
    ChainSpec::from_genesis(
        // Name
        "DataHighway Polkadot Local Testnet",
        // ID
        "datahighway-polkadot-local",
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
            relay_chain: "polkadot-local".into(),
            para_id: id.into(),
        },
    )
}

pub fn westend_parachain_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "DataHighway Baikal Westend Parachain Testnet",
        "datahighway_baikal",
        ChainType::Live,
        move || {
            baikal_testnet_genesis(
                vec![
                    // authority #1
                    (
                        //aura
                        hex!["2628f7a7bb067a23daa14b1aa9f10ff44545d37907f2d5cefee905236944060a"].unchecked_into()
                    ),
                    // authority #2
                    (
                        //aura
                        hex!["709f96ae975cd0cfafd98fb241810a2870d58fcfdbb1ee6892a8740525f4d871"].unchecked_into()
                    ),
                    // authority #3
                    (
                        //aura
                        hex!["ce7f04896b8d13da7a4f3f0a49bf6c1d77076043a1184a993ce75d96f6e0ee56"].unchecked_into()
                    ),
                    // authority #4
                    (
                        //aura
                        hex!["c27631914b41a8f58e24277158817d064a4144df430dd2cf7baeaa17414deb3e"].unchecked_into()
                    )

                ],
                hex!["4842a3314ad10a4e0053b59658f50b3fc5f1b6a9bee98608813a4b399aa3bf38"].into(),
                vec![
                    // Endow the Sudo account to cover transaction fees
                    hex!["4842a3314ad10a4e0053b59658f50b3fc5f1b6a9bee98608813a4b399aa3bf38"].into(),
                    // Endow this account with the DHX DAO Unlocked Reserves Balance
                    // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                    hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                    // Endow these accounts with a balance so they may bond as authorities
                    hex!["b41b286a78df1a87a07db8c8794923d8cc581c4b1a03d90be9ce46a03fbbaa2e"].into(),
                    hex!["bece77da74ab38eadde718ca30a0e46a0a3c5827f289c73d331755a7aaf19a11"].into(),
                    hex!["8cbd45146df7ce640231639dfd1a78dfd0dfb4d873b13226378c297110d50505"].into(),
                    hex!["2001d4a5b0e3c3ab39b88e7f85193a9a8340ca1b5803e9178f52dae126cd595b"].into(),
                    hex!["b20f2fab27d842763eb355ad978865e34f44da2fbf7a4182ab035d1bad34f021"].into(),
                    hex!["1aaaef87d9a3ec62ddcc959730b5d1b89d162fe8e432b0792540069bba518431"].into(),
                    hex!["62a173fb0a5bf0651559d560f44afa3de55d60cb0e0a06c9d0e1fef81f41b80a"].into(),
                    hex!["82e71bb9a9a8fc2aefbd17a41a4f7686cd95f46f3e3e0522caa6147289581562"].into(),
                ],
                id,
            )
        },
        boot_nodes,
        None,
        Some("dhx"),
        Some(properties),
        Extensions {
            relay_chain: "westend".into(),
            para_id: id.into(),
        },
    )
}

pub fn polkadot_parachain_config(id: ParaId) -> ChainSpec {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DHX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "DataHighway Tanganika Polkadot Parachain",
        "datahighway_tanganika",
        ChainType::Live,
        move || {
            tanganika_testnet_genesis(
                vec![
                    // authority #1
                    (
                        //aura
                        hex!["a8694c0c9e315e020844944ac76712c84f84a00007016e61c7e2f83fc56c5b3f"].unchecked_into()
                    ),
                    // authority #2
                    (
                        //aura
                        hex!["a8db9194388b3c038b126a5e2520515be2e989e3f380ce2cb5cf29d5a26c0522"].unchecked_into()
                    ),
                    // authority #3
                    (
                        //aura
                        hex!["b8212af17ba93d9175748469afa0a74357712ff4571a36d347df58cf3821cd3d"].unchecked_into()
                    ),
                    // authority #4
                    (
                        //aura
                        hex!["10a3d6854dc35e4b3fd77af4beda98f79dbe9edf5c29c14c8d57bec4bd733c0f"].unchecked_into()
                    )

                ],
                hex!["2402f0e0ce5856bb7224525aa9ab0408e4b75cf98d45bd0248a49d2bef01ee65"].into(),
                vec![
                    // Endow the Sudo account to cover transaction fees
                    hex!["2402f0e0ce5856bb7224525aa9ab0408e4b75cf98d45bd0248a49d2bef01ee65"].into(),
                    // Endow this account with the DHX DAO Unlocked Reserves Balance
                    // 5EWKojw2i3uoqfWx1dEgVjBsvK5xuTr5G3NjXYh47H6ycBWr
                    hex!["6d6f646c70792f74727372790000000000000000000000000000000000000000"].into(),
                    // Endow these accounts with a balance so they may bond as authorities
                    hex!["f8940eaa011b23f3469805062d1ae33c128caa6b10d71b04609f246cb947f92c"].into(),
                    hex!["e409a7faebf39ba76f46bfac84c8001c1243b980f5bac89fdd887eed1401bb35"].into(),
                    hex!["30a9048710bbc3791feb01e2c900f7290c09e124cd774b63950c52b8c6e5d644"].into(),
                    hex!["a0b3f77eec476b584fc24631c6a957254bc3e2d9e91c8abb8038e40ba045471f"].into(),
                    hex!["a2616fd57d21ed85a2deb41bb0628645db5ba24e9dc26c912cfa54608bf21d01"].into(),
                    hex!["46cfb03490de202950ea2433f0130730a3f84a4646acb6b10ff6510685457f40"].into(),
                    hex!["fa9089b3bcbad69451a162e1454a9e0aa9efc7bcdf9466f0a4bb762b4ed4755c"].into(),
                    hex!["123c907b49233a2ccb6a4d92a1266b3e2feccc10e880e8659368a6338842ba7f"].into(),
                ],
                id,
            )
        },
        boot_nodes,
        None,
        Some("dhx"),
        Some(properties),
        Extensions {
            relay_chain: "polkadot".into(),
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

fn baikal_testnet_genesis(
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

fn tanganika_testnet_genesis(
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
