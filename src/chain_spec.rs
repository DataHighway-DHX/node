use datahighway_runtime::{
    opaque::{
        Block,
        SessionKeys,
    },
    AccountId,
    AuraConfig,
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
    WASM_BINARY,
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
use sp_consensus_aura::sr25519::{AuthorityId as AuraId};
use sp_finality_grandpa::AuthorityId as GrandpaId;

use sp_core::{
    crypto::UncheckedInto,
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
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client::BadBlocks<Block>,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::ChainSpec<GenesisConfig, Extensions>;

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
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, AuraId, GrandpaId) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<AuraId>(seed),
        get_from_seed::<GrandpaId>(seed),
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
                    || {
                        dev_genesis(
                            vec![
                                get_authority_keys_from_seed("Alice")
                            ],
                            get_account_id_from_seed::<sr25519::Public>("Alice"),
                            vec![
                                get_account_id_from_seed::<sr25519::Public>("Alice"),
                                get_account_id_from_seed::<sr25519::Public>("Bob"),
                                get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                                get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                            ],
                            true,
                        )
                    },
                    // bootnodes
                    vec![
                        // Alice
                        "/ip4/127.0.0.1/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ".to_string(),
                    ],
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
                            true,
                        )
                    },
                    // bootnodes
                    vec![
                        // Alice
                        "/ip4/127.0.0.1/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ".to_string(),
                    ],
                    None,
                    None,
                    Some(properties),
                    Default::default(),
                )
            }
            // Alternative::DataHighwayTestnet => {
            //     ChainSpec::from_json_bytes(
            //         &include_bytes!("./chain-definition-custom/chain_def_testnet_v0.1.0.json")[..],
            //     )?
            // }
            // FIXME: Not working for some reason. Only 'local' works (error insufficient balance to bond)
            Alternative::DataHighwayTestnetLatest => {
                ChainSpec::from_genesis(
                    "DataHighway Testnet",
                    "testnet-latest",
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
                        )
                    },
                    // bootnodes
                    vec![
                        // Alice
                        "/ip4/127.0.0.1/tcp/30333/p2p/QmWYmZrHFPkgX8PgMgUpHJsK6Q6vWbeVXrKhciunJdRvKZ".to_string(),
                    ],
                    // telemetry endpoints
                    Some(TelemetryEndpoints::new(vec![("wss://telemetry.polkadot.io/submit/".into(), 0)])),
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
            "" | "testnet-latest" => Some(Alternative::DataHighwayTestnetLatest),
            _ => None,
        }
    }
}

fn session_keys(grandpa: GrandpaId, aura: AuraId) -> SessionKeys {
    SessionKeys {
        grandpa,
        aura,
    }
}

const INITIAL_BALANCE: u128 = 1_000_000_000_000_000_000_000_u128; // $1M
const INITIAL_STAKING: u128 = 1_000_000_000_000_000_000_u128;

fn dev_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, AuraId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_indices: Some(IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts.iter().cloned().map(|k| (k, INITIAL_BALANCE)).collect(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone())))
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            current_era: 0,
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
        pallet_aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.3.clone())).collect(),
		}),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
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
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, AuraId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    // No println
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_indices: Some(IndicesConfig {
            indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts.iter().cloned().map(|k| (k, INITIAL_BALANCE)).collect(),
        }),
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone())))
                .collect::<Vec<_>>(),
        }),
        pallet_staking: Some(StakingConfig {
            current_era: 0,
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
        pallet_aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.3.clone())).collect(),
		}),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
        }),
        pallet_collective_Instance1: Some(Default::default()),
        pallet_membership_Instance1: Some(GeneralCouncilMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        }),
        pallet_treasury: Some(Default::default()),
    }
}

pub fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
    Ok(match Alternative::from(id) {
        Some(spec) => Box::new(spec.load()?),
        None => Box::new(ChainSpec::from_json_file(std::path::PathBuf::from(id))?),
    })
}
