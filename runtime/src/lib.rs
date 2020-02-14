//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_std::prelude::*;
use sp_core::OpaqueMetadata;
use sp_runtime::{
	ApplyExtrinsicResult, transaction_validity::TransactionValidity, generic, create_runtime_str,
	impl_opaque_keys, MultiSignature,
};
use sp_runtime::traits::{
	BlakeTwo256, Block as BlockT, StaticLookup, Verify, ConvertInto, IdentifyAccount
};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use grandpa::AuthorityList as GrandpaAuthorityList;
use grandpa::fg_primitives;
use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
// use std::str::FromStr;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use timestamp::Call as TimestampCall;
pub use balances::Call as BalancesCall;
pub use sp_runtime::{Permill, Perbill};
pub use frame_support::{
	StorageValue, construct_runtime, parameter_types,
	traits::Randomness,
	weights::Weight,
};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature; // previously AnySignature

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("node"),
	impl_name: create_runtime_str!("node"),
	authoring_version: 3,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

pub const EPOCH_DURATION_IN_BLOCKS: u32 = 10 * MINUTES;

// These time units are defined in number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

// FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)

// /// Mining Speed Boost Token Types
// #[derive(Debug, Clone, PartialEq)]
// pub enum MiningSpeedBoostConfigurationTokenMiningTokenTypes {
// 	MXC,
// 	IOTA,
// 	DOT
// }

// impl FromStr for MiningSpeedBoostConfigurationTokenMiningTokenTypes {
// 	type Err = String;
// 	fn from_str(mining_speed_boosts_configuration_token_mining_token_type: &str) -> Result<Self, Self::Err> {
// 		match mining_speed_boosts_configuration_hardware_mining_hardware_type {
// 			"MXC" => Ok(MiningSpeedBoostConfigurationTokenMiningTokenTypes::MXC),
// 			"IOTA" => Ok(MiningSpeedBoostConfigurationTokenMiningTokenTypes::IOTA),
// 			"DOT" => Ok(MiningSpeedBoostConfigurationTokenMiningTokenTypes::DOT),
// 			_ => Err(format!("Invalid mining_speed_boosts_configuration_token_mining_token_type: {}", mining_speed_boosts_configuration_token_mining_token_type)),
// 		}
// 	}
// }

// /// Mining Speed Boost Hardware Types
// #[derive(Debug, Clone, PartialEq)]
// pub enum MiningSpeedBoostConfigurationHardwareMiningHardwareTypes {
// 	EndDevice,
// 	Gateway,
// 	Supernode,
// 	Collator
// }

// impl FromStr for MiningSpeedBoostConfigurationHardwareMiningHardwareTypes {
// 	type Err = String;
// 	fn from_str(mining_speed_boosts_configuration_hardware_mining_hardware_type: &str) -> Result<Self, Self::Err> {
// 		match mining_speed_boosts_configuration_hardware_mining_hardware_type {
// 			"EndDevice" => Ok(MiningSpeedBoostConfigurationHardwareMiningHardwareTypes::EndDevice),
// 			"Gateway" => Ok(MiningSpeedBoostConfigurationHardwareMiningHardwareTypes::Gateway),
// 			"Supernode" => Ok(MiningSpeedBoostConfigurationHardwareMiningHardwareTypes::Supernode),
// 			"Collator" => Ok(MiningSpeedBoostConfigurationHardwareMiningHardwareTypes::Collator),
// 			_ => Err(format!("Invalid mining_speed_boosts_configuration_hardware_mining_hardware_type: {}", mining_speed_boosts_configuration_hardware_mining_hardware_type)),
// 		}
// 	}
// }

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const MaximumBlockWeight: Weight = 1_000_000;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
	pub const Version: RuntimeVersion = VERSION;
}

impl system::Trait for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = Indices;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Maximum weight of each block.
	type MaximumBlockWeight = MaximumBlockWeight;
	/// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
	type MaximumBlockLength = MaximumBlockLength;
	/// Portion of the block weight that is available to all normal transactions.
	type AvailableBlockRatio = AvailableBlockRatio;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type ModuleToIndex = ModuleToIndex;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnReapAccount = Balances;
	/// The data to be stored in an account.
	type AccountData = balances::AccountData<Balance>;
}

impl aura::Trait for Runtime {
	type AuthorityId = AuraId;
}

impl grandpa::Trait for Runtime {
	type Event = Event;
}

parameter_types! {
	/// How much an index costs.
	pub const IndexDeposit: u128 = 100;
}

impl indices::Trait for Runtime {
	/// The type for recording indexing into the account enumeration. If this ever overflows, there
	/// will be problems!
	type AccountIndex = AccountIndex;
	/// The ubiquitous event type.
	type Event = Event;
	/// The currency type.
	type Currency = Balances;
	/// How much an index costs.
	type Deposit = IndexDeposit;
}
parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
}

impl balances::Trait for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
}

parameter_types! {
	pub const TransactionBaseFee: Balance = 0;
	pub const TransactionByteFee: Balance = 1;
}

impl transaction_payment::Trait for Runtime {
	type Currency = balances::Module<Runtime>;
	type OnTransactionPayment = ();
	type TransactionBaseFee = TransactionBaseFee;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = ConvertInto;
	type FeeMultiplierUpdate = ();
}

impl sudo::Trait for Runtime {
	type Event = Event;
	type Proposal = Call;
}

impl roaming_operators::Trait for Runtime {
	type Event = Event;
	type RoamingOperatorIndex = u64;
	type Currency = Balances;
	type Randomness = RandomnessCollectiveFlip;
}

impl roaming_networks::Trait for Runtime {
	type Event = Event;
	type RoamingNetworkIndex = u64;
}

impl roaming_organizations::Trait for Runtime {
	type Event = Event;
	type RoamingOrganizationIndex = u64;
}

impl roaming_network_servers::Trait for Runtime {
	type Event = Event;
	type RoamingNetworkServerIndex = u64;
}

impl roaming_devices::Trait for Runtime {
	type Event = Event;
	type RoamingDeviceIndex = u64;
}

impl roaming_routing_profiles::Trait for Runtime {
	type Event = Event;
	type RoamingRoutingProfileIndex = u64;
	// https://polkadot.js.org/api/types/#primitive-types
	type RoamingRoutingProfileAppServer = Vec<u8>;
}

// impl roaming_service_profiles::Trait for Runtime {
// 	type Event = Event;
// 	type RoamingServiceProfileIndex = u64;
// 	type RoamingServiceProfileUplinkRate = u32;
// 	type RoamingServiceProfileDownlinkRate = u32;
// }

// impl roaming_accounting_policies::Trait for Runtime {
// 	type Event = Event;
// 	type RoamingAccountingPolicyIndex = u64;
// 	type RoamingAccountingPolicyType = Vec<u8>;
// 	type RoamingAccountingPolicyUplinkFeeFactor = u32;
// 	type RoamingAccountingPolicyDownlinkFeeFactor = u32;
// }

// impl roaming_agreement_policies::Trait for Runtime {
// 	type Event = Event;
// 	type RoamingAgreementPolicyIndex = u64;
// 	type RoamingAgreementPolicyActivationType = Vec<u8>;
// 	type RoamingAgreementPolicyExpiry = u64; // <pallet_timestamp::Module<Runtime> as Trait>::Moment` timestamp::Module<Runtime>::Moment;
// }

// impl roaming_network_profiles::Trait for Runtime {
// 	type Event = Event;
// 	type RoamingNetworkProfileIndex = u64;
// }

// impl roaming_device_profiles::Trait for Runtime {
// 	type Event = Event;
// 	type RoamingDeviceProfileIndex = u64;
// 	type RoamingDeviceProfileDevAddr = Vec<u8>;
// 	type RoamingDeviceProfileDevEUI = Vec<u8>;
// 	type RoamingDeviceProfileJoinEUI = Vec<u8>;
// 	type RoamingDeviceProfileVendorID = Vec<u8>;
// }

// impl roaming_sessions::Trait for Runtime {
// 	type Event = Event;
// 	type RoamingSessionIndex = u64;
// 	type RoamingSessionJoinRequestRequestedAt = u64;
// 	type RoamingSessionJoinRequestAcceptExpiry = u64;
// 	type RoamingSessionJoinRequestAcceptAcceptedAt = u64;
// }

// impl roaming_billing_policies::Trait for Runtime {
// 	type Event = Event;
// 	type RoamingBillingPolicyIndex = u64;
// 	type RoamingBillingPolicyNextBillingAt = u64;
// 	type RoamingBillingPolicyFrequencyInDays = u64;
// }

// impl roaming_charging_policies::Trait for Runtime {
// 	type Event = Event;
// 	type RoamingChargingPolicyIndex = u64;
// 	type RoamingChargingPolicyNextChargingAt = u64;
// 	type RoamingChargingPolicyDelayAfterBillingInDays = u64;
// }

// impl roaming_packet_bundles::Trait for Runtime {
// 	type Event = Event;
// 	type RoamingPacketBundleIndex = u64;
// 	type RoamingPacketBundleReceivedAtHome = bool;
// 	type RoamingPacketBundleReceivedPacketsCount = u64;
// 	type RoamingPacketBundleReceivedPacketsOkCount = u64;
// 	type RoamingPacketBundleReceivedStartedAt = u64;
// 	type RoamingPacketBundleReceivedEndedAt = u64;
// 	type RoamingPacketBundleExternalDataStorageHash = Hash;
// }

// impl mining_speed_boosts_configuration_token_mining::Trait for Runtime {
// 	type Event = Event;
// 	// FIXME - restore when stop temporarily using roaming-operators
// 	// type Currency = Balances;
// 	// type Randomness = RandomnessCollectiveFlip;
// 	type MiningSpeedBoostConfigurationTokenMiningIndex = u64;
// 	// Mining Speed Boost Token Mining Config
// 	// FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
// 	type MiningSpeedBoostConfigurationTokenMiningTokenType = Vec<u8>;
// 	// type MiningSpeedBoostConfigurationTokenMiningTokenType = MiningSpeedBoostConfigurationTokenMiningTokenTypes;
// 	type MiningSpeedBoostConfigurationTokenMiningTokenLockedAmount = u64;
// 	type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod = u32;
// 	type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate = u64;
// 	type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate = u64;
// }

// impl mining_speed_boosts_configuration_hardware_mining::Trait for Runtime {
// 	type Event = Event;
// 	// FIXME - restore when stop temporarily using roaming-operators
// 	// type Currency = Balances;
// 	// type Randomness = RandomnessCollectiveFlip;
// 	type MiningSpeedBoostConfigurationHardwareMiningIndex = u64;
// 	// Mining Speed Boost Hardware Mining Config
// 	type MiningSpeedBoostConfigurationHardwareMiningHardwareSecure = bool;
// 	// FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
// 	type MiningSpeedBoostConfigurationHardwareMiningHardwareType = Vec<u8>;
// 	// type MiningSpeedBoostConfigurationHardwareMiningHardwareType = MiningSpeedBoostConfigurationHardwareMiningHardwareTypes;
// 	type MiningSpeedBoostConfigurationHardwareMiningHardwareID = u64;
// 	type MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI = u64;
// 	type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate = u64;
// 	type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate = u64;
// }

// impl mining_speed_boosts_rates_token_mining::Trait for Runtime {
// 	type Event = Event;
// 	type MiningSpeedBoostRatesTokenMiningIndex = u64;
// 	// Mining Speed Boost Rate
// 	type MiningSpeedBoostRatesTokenMiningTokenMXC = u32;
// 	type MiningSpeedBoostRatesTokenMiningTokenIOTA = u32;
// 	type MiningSpeedBoostRatesTokenMiningTokenDOT = u32;
// 	// Mining Speed Boost Max Rates
// 	type MiningSpeedBoostRatesTokenMiningMaxToken = u32;
// 	type MiningSpeedBoostRatesTokenMiningMaxLoyalty = u32;
// }

// impl mining_speed_boosts_rates_hardware_mining::Trait for Runtime {
// 	type Event = Event;
// 	type MiningSpeedBoostRatesHardwareMiningIndex = u64;
// 	// Mining Speed Boost Rate
// 	type MiningSpeedBoostRatesHardwareMiningHardwareSecure = u32;
// 	type MiningSpeedBoostRatesHardwareMiningHardwareInsecure = u32;
// 	// Mining Speed Boost Max Rates
// 	type MiningSpeedBoostRatesHardwareMiningMaxHardware = u32;
// }

// impl mining_speed_boosts_sampling_token_mining::Trait for Runtime {
// 	type Event = Event;
// 	type MiningSpeedBoostSamplingTokenMiningIndex = u64;
// 	type MiningSpeedBoostSamplingTokenMiningSampleDate = u64;
// 	type MiningSpeedBoostSamplingTokenMiningSampleTokensLocked = u64;
// }

// impl mining_speed_boosts_sampling_hardware_mining::Trait for Runtime {
// 	type Event = Event;
// 	type MiningSpeedBoostSamplingHardwareMiningIndex = u64;
// 	type MiningSpeedBoostSamplingHardwareMiningSampleDate = u64;
// 	type MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline = u64;
// }

// impl mining_speed_boosts_eligibility_token_mining::Trait for Runtime {
// 	type Event = Event;
// 	type MiningSpeedBoostEligibilityTokenMiningIndex = u64;
// 	type MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility = u64;
// 	type MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage = u32;
// 	// type MiningSpeedBoostEligibilityTokenMiningDateAudited = u64;
// 	// type MiningSpeedBoostEligibilityTokenMiningAuditorAccountID = u64;
// }

// impl mining_speed_boosts_eligibility_hardware_mining::Trait for Runtime {
// 	type Event = Event;
// 	type MiningSpeedBoostEligibilityHardwareMiningIndex = u64;
// 	type MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility = u64;
// 	type MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage = u32;
// 	// type MiningSpeedBoostEligibilityHardwareMiningDateAudited = u64;
// 	// type MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID = u64;
// }

// impl mining_speed_boosts_claims_token_mining::Trait for Runtime {
// 	type Event = Event;
// 	type MiningSpeedBoostClaimsTokenMiningIndex = u64;
// 	type MiningSpeedBoostClaimsTokenMiningClaimAmount = u64;
// 	type MiningSpeedBoostClaimsTokenMiningClaimDateRedeemed = u64;
// }

// impl mining_speed_boosts_claims_hardware_mining::Trait for Runtime {
// 	type Event = Event;
// 	type MiningSpeedBoostClaimsHardwareMiningIndex = u64;
// 	type MiningSpeedBoostClaimsHardwareMiningClaimAmount = u64;
// 	type MiningSpeedBoostClaimsHardwareMiningClaimDateRedeemed = u64;
// }

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: system::{Module, Call, Config, Storage, Event<T>},
		Timestamp: timestamp::{Module, Call, Storage, Inherent},
		Aura: aura::{Module, Config<T>, Inherent(Timestamp)},
		Grandpa: grandpa::{Module, Call, Storage, Config, Event},
		Indices: indices::{Module, Call, Storage, Event<T>, Config<T>},
		Balances: balances::{Module, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: transaction_payment::{Module, Storage},
		Sudo: sudo,
		// Used for the module template in `./template.rs`
		DataHighwayRoamingOperators: roaming_operators::{Module, Call, Storage, Event<T>},
		DataHighwayRoamingNetworks: roaming_networks::{Module, Call, Storage, Event<T>},
		DataHighwayRoamingOrganizations: roaming_organizations::{Module, Call, Storage, Event<T>},
		DataHighwayRoamingNetworkServers: roaming_network_servers::{Module, Call, Storage, Event<T>},
		DataHighwayRoamingDevices: roaming_devices::{Module, Call, Storage, Event<T>},
		DataHighwayRoamingRoutingProfiles: roaming_routing_profiles::{Module, Call, Storage, Event<T>},
		// DataHighwayRoamingServiceProfiles: roaming_service_profiles::{Module, Call, Storage, Event<T>},
		// DataHighwayRoamingAccountingPolicies: roaming_accounting_policies::{Module, Call, Storage, Event<T>},
		// DataHighwayRoamingAgreementPolicies: roaming_agreement_policies::{Module, Call, Storage, Event<T>},
		// DataHighwayRoamingNetworkProfiles: roaming_network_profiles::{Module, Call, Storage, Event<T>},
		// DataHighwayRoamingDeviceProfiles: roaming_device_profiles::{Module, Call, Storage, Event<T>},
		// DataHighwayRoamingSessions: roaming_sessions::{Module, Call, Storage, Event<T>},
		// DataHighwayRoamingBillingPolicies: roaming_billing_policies::{Module, Call, Storage, Event<T>},
		// DataHighwayRoamingChargingPolicies: roaming_charging_policies::{Module, Call, Storage, Event<T>},
		// DataHighwayRoamingPacketBundles: roaming_packet_bundles::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostConfigurationTokenMining: mining_speed_boosts_configuration_token_mining::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostConfigurationHardwareMining: mining_speed_boosts_configuration_hardware_mining::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostRatesTokenMining: mining_speed_boosts_rates_token_mining::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostRatesHardwareMining: mining_speed_boosts_rates_hardware_mining::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostSamplingTokenMining: mining_speed_boosts_sampling_token_mining::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostSamplingHardwareMining: mining_speed_boosts_sampling_hardware_mining::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostEligibilityTokenMining: mining_speed_boosts_eligibility_token_mining::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostEligibilityHardwareMining: mining_speed_boosts_eligibility_hardware_mining::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostClaimsTokenMining: mining_speed_boosts_claims_token_mining::{Module, Call, Storage, Event<T>},
		// DataHighwayMiningSpeedBoostClaimsHardwareMining: mining_speed_boosts_claims_hardware_mining::{Module, Call, Storage, Event<T>},
		RandomnessCollectiveFlip: randomness_collective_flip::{Module, Call, Storage},
	}
);

/// The address format for describing accounts.
pub type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	system::CheckVersion<Runtime>,
	system::CheckGenesis<Runtime>,
	system::CheckEra<Runtime>,
	system::CheckNonce<Runtime>,
	system::CheckWeight<Runtime>,
	transaction_payment::ChargeTransactionPayment<Runtime>
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			Executive::validate_transaction(tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> u64 {
			Aura::slot_duration()
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}
	}
}
