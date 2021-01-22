use cumulus_primitives::ParaId;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{
    crypto::{
        UncheckedFrom,
        UncheckedInto,
    },
    sr25519,
    Pair,
    Public,
};
use sp_runtime::traits::{IdentifyAccount, Verify};
use hex_literal::hex;
use sc_telemetry::TelemetryEndpoints;
use serde_json::map::Map;
use parachain_runtime::{
    opaque::{
        Block,
        SessionKeys,
    },
	AccountId,  
    BalancesConfig,
    GeneralCouncilMembershipConfig,
    GenesisConfig,
    Signature,
    SudoConfig,
    SystemConfig,};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
pub use sp_runtime::{
    Perbill,
    Permill,
};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<parachain_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
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

pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, BabeId) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
    )
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config(id: ParaId) -> ChainSpec {
	let mut properties = Map::new();
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
				vec![get_authority_keys_from_seed("Alice")],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
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
			relay_chain: "rococo-dev".into(),
			para_id: id.into(),
		},
	)
}

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
	let mut properties = Map::new();
	properties.insert("tokenSymbol".into(), "DHX".into());
	properties.insert("tokenDecimals".into(), 18.into());
	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
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
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
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

pub fn harbor_testnet_config(id: ParaId) -> ChainSpec {
	let mut properties = Map::new();
	properties.insert("tokenSymbol".into(), "DHX".into());
	properties.insert("tokenDecimals".into(), 18.into());
	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
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
				id,
			)
		},
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(POLKADOT_STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Polkadot telemetry url is valid; qed"),
		),
		Some("dhx-test"),
		Some(properties),
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: id.into(),
		},
	)
}

// fn session_keys(grandpa: GrandpaId, babe: BabeId) -> SessionKeys {
//     SessionKeys {
//         grandpa,
//         babe,
//     }
// }

// total supply should be 100m, with 30m (30%) going to DHX DAO unlocked reserves, and the remaining
// 70m split between the initial 8x accounts other than the reserves such that each should receive 8750
const INITIAL_BALANCE: u128 = 8_750_000_000_000_000_000_000_u128; // $70M 70_000_000_000_000_000_000_000_u128
const INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u128 = 30_000_000_000_000_000_000_000_u128; // $30M
const INITIAL_STAKING: u128 = 1_000_000_000_000_000_000_u128;


fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> GenesisConfig {
    GenesisConfig {
		frame_system: Some(parachain_runtime::SystemConfig {
			code: parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		}),
        // pallet_indices: Some(IndicesConfig {
        //     indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        // }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| (x, INITIAL_BALANCE))
                .into_iter()
                .map(|k| (k.0, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE))
                .collect(),
        }),
        // pallet_session: Some(SessionConfig {
        //     keys: initial_authorities
        //         .iter()
        //         .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone())))
        //         .collect::<Vec<_>>(),
        // }),
        // pallet_staking: Some(StakingConfig {
        //     validator_count: initial_authorities.len() as u32 * 2,
        //     minimum_validator_count: initial_authorities.len() as u32,
        //     stakers: initial_authorities
        //         .iter()
        //         .map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
        //         .collect(),
        //     invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
        //     slash_reward_fraction: Perbill::from_percent(10),
        //     ..Default::default()
        // }),
        pallet_sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        // pallet_babe: Some(BabeConfig {
        //     authorities: vec![],
        // }),
        // pallet_grandpa: Some(GrandpaConfig {
        //     authorities: vec![],
        // }),
        pallet_collective_Instance1: Some(Default::default()),
        pallet_membership_Instance1: Some(GeneralCouncilMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        }),
        pallet_treasury: Some(Default::default()),
		parachain_info: Some(parachain_runtime::ParachainInfoConfig { parachain_id: id }),
    }
}

fn dev_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> parachain_runtime::GenesisConfig {
	parachain_runtime::GenesisConfig {
		frame_system: Some(parachain_runtime::SystemConfig {
			code: parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		}),
        // pallet_indices: Some(IndicesConfig {
        //     indices: endowed_accounts.iter().enumerate().map(|(index, x)| (index as u32, (*x).clone())).collect(),
        // }),
		pallet_balances: Some(parachain_runtime::BalancesConfig {
            balances: endowed_accounts.iter().cloned().map(|x|
                // Insert Public key (hex) of the account without the 0x prefix below
                if x == UncheckedFrom::unchecked_from(hex!("a42b7518d62a942344fec55d414f1654bf3fd325dbfa32a3c30534d5976acb21").into()) {
                    (x, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE)
                } else {
                    (x, INITIAL_BALANCE)
                }
            )
            .collect(),
		}),
        // pallet_session: Some(SessionConfig {
        //     keys: initial_authorities
        //         .iter()
        //         .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.2.clone(), x.3.clone())))
        //         .collect::<Vec<_>>(),
        // }),
        // pallet_staking: Some(StakingConfig {
        //     validator_count: initial_authorities.len() as u32 * 2,
        //     minimum_validator_count: initial_authorities.len() as u32,
        //     stakers: initial_authorities
        //         .iter()
        //         .map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
        //         .collect(),
        //     invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
        //     slash_reward_fraction: Perbill::from_percent(10),
        //     ..Default::default()
        // }),
		pallet_sudo: Some(parachain_runtime::SudoConfig { key: root_key.clone() }),
        // pallet_babe: Some(BabeConfig {
        //     authorities: vec![],
        // }),
        // pallet_grandpa: Some(GrandpaConfig {
        //     authorities: vec![],
        // }),
        pallet_collective_Instance1: Some(Default::default()),
        pallet_membership_Instance1: Some(GeneralCouncilMembershipConfig {
            members: vec![root_key.clone()],
            phantom: Default::default(),
        }),
        pallet_treasury: Some(Default::default()),
		parachain_info: Some(parachain_runtime::ParachainInfoConfig { parachain_id: id }),
	}
}
