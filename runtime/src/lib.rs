//! The DataHighway runtime. This can be compiled with `#[no_std]`, ready for Wasm.

// Ignore clippy error error: this public function dereferences a raw pointer but is not marked `unsafe`
#![cfg_attr(feature = "cargo-clippy", allow(clippy::not_unsafe_ptr_arg_deref))]

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod constants;
mod types;

use pallet_grandpa::{
    fg_primitives,
    AuthorityList as GrandpaAuthorityList,
};
use sp_api::impl_runtime_apis;
use sp_core::{
    u32_trait::{
        _1,
        _2,
        _3,
        _4,
    },
    OpaqueMetadata,
};
use sp_runtime::{
    create_runtime_str,
    curve::PiecewiseLinear,
    generic,
    impl_opaque_keys,
    traits::{
        BlakeTwo256,
        Block as BlockT,
        Convert,
        ConvertInto,
        OpaqueKeys,
        StaticLookup,
    },
    transaction_validity::TransactionValidity,
    ApplyExtrinsicResult,
};
use sp_std::prelude::*; // Imports Vec
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
// use std::str::FromStr;

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime,
    parameter_types,
    traits::{
        Contains,
        Randomness,
    },
    weights::Weight,
    StorageValue,
};
pub use module_primitives::{Balance, CurrencyId};
pub use pallet_balances::Call as BalancesCall; // TODO: remove?
pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{
    Perbill,
    Percent,
    Permill,
}; // TODO: remove?

use orml_currencies::BasicCurrencyAdapter;
// use orml_oracle::OperatorProvider;
use module_primitives::{Price}; // Price from orml_prices or module_primitives?

// TODO: Balance type of u128 was replaced with Amount type of i128

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
            pub babe: Babe,
            pub grandpa: Grandpa,
        }
    }
}

pub use constants::time::*;
pub use types::*;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    // TODO: rename to datahighway-chain, and elsewhere?
    spec_name: create_runtime_str!("datahighway"),
    impl_name: create_runtime_str!("datahighway"),
    authoring_version: 1,
    spec_version: 3,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
};

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
// 			_ => Err(format!("Invalid mining_speed_boosts_configuration_token_mining_token_type: {}",
// mining_speed_boosts_configuration_token_mining_token_type)), 		}
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
// 			_ => Err(format!("Invalid mining_speed_boosts_configuration_hardware_mining_hardware_type: {}",
// mining_speed_boosts_configuration_hardware_mining_hardware_type)), 		}
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
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// Portion of the block weight that is available to all normal transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The ubiquitous event type.
    type Event = Event;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = Indices;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    /// Maximum weight of each block.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type ModuleToIndex = ModuleToIndex;
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Version of the runtime.
    type Version = Version;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl pallet_babe::Trait for Runtime {
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
}

impl pallet_grandpa::Trait for Runtime {
    type Event = Event;
}

parameter_types! {
    /// How much an index costs.
    pub const IndexDeposit: u128 = 100;
}

impl pallet_indices::Trait for Runtime {
    /// The type for recording indexing into the account enumeration. If this ever overflows, there
    /// will be problems!
    type AccountIndex = AccountIndex;
    /// The currency type.
    type Currency = Balances;
    /// How much an index costs.
    type Deposit = IndexDeposit;
    /// The ubiquitous event type.
    type Event = Event;
}
parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Runtime {
    type MinimumPeriod = MinimumPeriod;
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = Moment;
    type OnTimestampSet = Babe;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
}

impl pallet_balances::Trait for Runtime {
    type AccountStore = System;
    /// The type for recording an account's balance.
    type Balance = Balance;
    type DustRemoval = ();
    /// The ubiquitous event type.
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
}

parameter_types! {
    pub const TransactionBaseFee: Balance = 0;
    pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Trait for Runtime {
    type Currency = pallet_balances::Module<Runtime>;
    type FeeMultiplierUpdate = ();
    type OnTransactionPayment = ();
    type TransactionBaseFee = TransactionBaseFee;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = ConvertInto;
}

impl pallet_sudo::Trait for Runtime {
    type Call = Call;
    type Event = Event;
}

type GeneralCouncilInstance = pallet_collective::Instance1;
impl pallet_collective::Trait<GeneralCouncilInstance> for Runtime {
    type Event = Event;
    type Origin = Origin;
    type Proposal = Call;
}

type GeneralCouncilMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Trait<GeneralCouncilMembershipInstance> for Runtime {
    type AddOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
    type Event = Event;
    type MembershipChanged = GeneralCouncil;
    type MembershipInitialized = GeneralCouncil;
    type RemoveOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
    type ResetOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
    type SwapOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
}

// type FinancialCouncilInstance = pallet_collective::Instance2;
// impl pallet_collective::Trait<FinancialCouncilInstance> for Runtime {
// 	type Origin = Origin;
// 	type Proposal = Call;
// 	type Event = Event;
// }

// type FinancialCouncilMembershipInstance = pallet_membership::Instance2;
// impl pallet_membership::Trait<FinancialCouncilMembershipInstance> for Runtime {
// 	type Event = Event;
// 	type AddOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>;
// 	type RemoveOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>;
// 	type SwapOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>;
// 	type ResetOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, GeneralCouncilInstance>;
// 	type MembershipInitialized = FinancialCouncil;
// 	type MembershipChanged = FinancialCouncil;
// }

// type OperatorCollectiveInstance = pallet_collective::Instance3;
// impl pallet_collective::Trait<OperatorCollectiveInstance> for Runtime {
// 	type Origin = Origin;
// 	type Proposal = Call;
// 	type Event = Event;
// }

// type OperatorMembershipInstance = pallet_membership::Instance3;
// impl pallet_membership::Trait<OperatorMembershipInstance> for Runtime {
// 	type Event = Event;
// 	type AddOrigin = pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, GeneralCouncilInstance>;
// 	type RemoveOrigin = pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, GeneralCouncilInstance>;
// 	type SwapOrigin = pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, GeneralCouncilInstance>;
// 	type ResetOrigin = pallet_collective::EnsureProportionMoreThan<_1, _3, AccountId, GeneralCouncilInstance>;
// 	type MembershipInitialized = OperatorCollective;
// 	type MembershipChanged = OperatorCollective;
// }

pub struct GeneralCouncilProvider;
impl Contains<AccountId> for GeneralCouncilProvider {
    fn contains(who: &AccountId) -> bool {
        GeneralCouncil::is_member(who)
    }

    fn sorted_members() -> Vec<AccountId> {
        GeneralCouncil::members()
    }
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 1_000_000_000_000_000_000;
    pub const SpendPeriod: BlockNumber = 1 * DAYS;
    pub const Burn: Permill = Permill::from_percent(0);
    pub const TipCountdown: BlockNumber = 1 * DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = 1_000_000_000_000_000_000;
    pub const TipReportDepositPerByte: Balance = 1_000_000_000_000_000;
}

impl pallet_treasury::Trait for Runtime {
    type ApproveOrigin = pallet_collective::EnsureMembers<_4, AccountId, GeneralCouncilInstance>;
    type Burn = Burn;
    type Currency = Balances;
    type Event = Event;
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type ProposalRejection = ();
    type RejectOrigin = pallet_collective::EnsureMembers<_2, AccountId, GeneralCouncilInstance>;
    type SpendPeriod = SpendPeriod;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type TipReportDepositPerByte = TipReportDepositPerByte;
    type Tippers = GeneralCouncilProvider;
}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Trait for Runtime {
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type Event = Event;
    type Keys = opaque::SessionKeys;
    type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type SessionManager = Staking;
    type ShouldEndSession = Babe;
    type ValidatorId = <Self as system::Trait>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
}

impl pallet_session::historical::Trait for Runtime {
    type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
    type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
    const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
        min_inflation: 0_025_000,
        max_inflation: 0_100_000,
        ideal_stake: 0_500_000,
        falloff: 0_050_000,
        max_piece_count: 40,
        test_precision: 0_005_000,
    );
}

/// Struct that handles the conversion of Balance -> `u64`. This is used for staking's election
/// calculation.
pub struct CurrencyToVoteHandler;

impl CurrencyToVoteHandler {
    fn factor() -> Balance {
        (Balances::total_issuance() / u64::max_value() as Balance).max(1)
    }
}

impl Convert<Balance, u64> for CurrencyToVoteHandler {
    fn convert(x: Balance) -> u64 {
        (x / Self::factor()) as u64
    }
}

impl Convert<u128, Balance> for CurrencyToVoteHandler {
    fn convert(x: u128) -> Balance {
        x * Self::factor()
    }
}

parameter_types! {
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    pub const BondingDuration: pallet_staking::EraIndex = 24 * 28;
    pub const SlashDeferDuration: pallet_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
}

impl pallet_staking::Trait for Runtime {
    type BondingDuration = BondingDuration;
    type Currency = Balances;
    type CurrencyToVote = CurrencyToVoteHandler;
    type Event = Event;
    // send the slashed funds to the pallet treasury.
    type Reward = ();
    type RewardCurve = RewardCurve;
    type RewardRemainder = PalletTreasury;
    type SessionInterface = Self;
    // rewards are minted from the void
    type SessionsPerEra = SessionsPerEra;
    type Slash = PalletTreasury;
    /// A super-majority of the council can cancel the slash.
    type SlashCancelOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, GeneralCouncilInstance>;
    type SlashDeferDuration = SlashDeferDuration;
    type Time = Timestamp;
}

// pub struct OperatorCollectiveProvider;
// impl OperatorProvider<AccountId> for OperatorCollectiveProvider {
// 	fn can_feed_data(who: &AccountId) -> bool {
// 		OperatorCollective::is_member(who)
// 	}

// 	fn operators() -> Vec<AccountId> {
// 		OperatorCollective::members()
// 	}
// }

// parameter_types! {
// 	pub const MinimumCount: u32 = 1; // TODO: change this
// 	pub const ExpiresIn: u64 = 1000 * 60 * 60 * 24 * 100; // 100days for now TODO: change this
// }

// impl orml_oracle::Trait for Runtime {
// 	type Event = Event;
// 	type Call = Call;
// 	type OnNewData = (); // TODO: update this
// 	type OnRedundantCall = (); // TODO: update this
// 	type OperatorProvider = OperatorCollectiveProvider;
// 	type CombineData = orml_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn>;
// 	type Time = Timestamp;
// 	type OracleKey = CurrencyId;
// 	type OracleValue = Price;
// }

// pub type TimeStampedPrice = orml_oracle::TimestampedValueOf<Runtime>;

// impl orml_tokens::Trait for Runtime {
// 	type Event = Event;
// 	type Balance = Balance;
// 	type Amount = Amount;
// 	type CurrencyId = CurrencyId;
// 	type ExistentialDeposit = ExistentialDeposit;
// 	type DustRemoval = ();
// }

// parameter_types! {
// 	pub const GetDataHighwayTokenId: CurrencyId = CurrencyId::DHX;
// }

// pub type DataHighwayToken = BasicCurrencyAdapter<Runtime, pallet_balances::Module<Runtime>, Balance>;

// impl orml_currencies::Trait for Runtime {
// 	type Event = Event;
// 	type MultiCurrency = orml_tokens::Module<Runtime>;
// 	type NativeCurrency = DataHighwayToken;
// 	type GetNativeCurrencyId = GetDataHighwayTokenId;
// }

// impl orml_prices::Trait for Runtime {
// 	type CurrencyId = CurrencyId;
// 	type Source = orml_oracle::Module<Runtime>;
// }

impl roaming_operators::Trait for Runtime {
    type Currency = Balances;
    type Event = Event;
    type Randomness = RandomnessCollectiveFlip;
    type RoamingOperatorIndex = u64;
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
    // https://polkadot.js.org/api/types/#primitive-types
    type RoamingRoutingProfileAppServer = Vec<u8>;
    type RoamingRoutingProfileIndex = u64;
}

impl roaming_service_profiles::Trait for Runtime {
    type Event = Event;
    type RoamingServiceProfileDownlinkRate = u32;
    type RoamingServiceProfileIndex = u64;
    type RoamingServiceProfileUplinkRate = u32;
}

impl roaming_accounting_policies::Trait for Runtime {
    type Event = Event;
    type RoamingAccountingPolicyDownlinkFeeFactor = u32;
    type RoamingAccountingPolicyIndex = u64;
    type RoamingAccountingPolicyType = Vec<u8>;
    type RoamingAccountingPolicyUplinkFeeFactor = u32;
}

impl roaming_agreement_policies::Trait for Runtime {
    type Event = Event;
    type RoamingAgreementPolicyActivationType = Vec<u8>;
    type RoamingAgreementPolicyExpiry = u64;
    type RoamingAgreementPolicyIndex = u64; // <pallet_timestamp::Module<Runtime> as Trait>::Moment` timestamp::Module<Runtime>::Moment;
}

impl roaming_network_profiles::Trait for Runtime {
    type Event = Event;
    type RoamingNetworkProfileIndex = u64;
}

impl roaming_device_profiles::Trait for Runtime {
    type Event = Event;
    type RoamingDeviceProfileDevAddr = Vec<u8>;
    type RoamingDeviceProfileDevEUI = Vec<u8>;
    type RoamingDeviceProfileIndex = u64;
    type RoamingDeviceProfileJoinEUI = Vec<u8>;
    type RoamingDeviceProfileVendorID = Vec<u8>;
}

impl roaming_sessions::Trait for Runtime {
    type Event = Event;
    type RoamingSessionIndex = u64;
    type RoamingSessionJoinRequestAcceptAcceptedAt = u64;
    type RoamingSessionJoinRequestAcceptExpiry = u64;
    type RoamingSessionJoinRequestRequestedAt = u64;
}

impl roaming_billing_policies::Trait for Runtime {
    type Event = Event;
    type RoamingBillingPolicyFrequencyInDays = u64;
    type RoamingBillingPolicyIndex = u64;
    type RoamingBillingPolicyNextBillingAt = u64;
}

impl roaming_charging_policies::Trait for Runtime {
    type Event = Event;
    type RoamingChargingPolicyDelayAfterBillingInDays = u64;
    type RoamingChargingPolicyIndex = u64;
    type RoamingChargingPolicyNextChargingAt = u64;
}

impl roaming_packet_bundles::Trait for Runtime {
    type Event = Event;
    type RoamingPacketBundleExternalDataStorageHash = Hash;
    type RoamingPacketBundleIndex = u64;
    type RoamingPacketBundleReceivedAtHome = bool;
    type RoamingPacketBundleReceivedEndedAt = u64;
    type RoamingPacketBundleReceivedPacketsCount = u64;
    type RoamingPacketBundleReceivedPacketsOkCount = u64;
    type RoamingPacketBundleReceivedStartedAt = u64;
}

impl mining_speed_boosts_configuration_token_mining::Trait for Runtime {
    type Event = Event;
    // FIXME - restore when stop temporarily using roaming-operators
    // type Currency = Balances;
    // type Randomness = RandomnessCollectiveFlip;
    type MiningSpeedBoostConfigurationTokenMiningIndex = u64;
    type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod = u32;
    type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate = u64;
    type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate = u64;
    // type MiningSpeedBoostConfigurationTokenMiningTokenType = MiningSpeedBoostConfigurationTokenMiningTokenTypes;
    type MiningSpeedBoostConfigurationTokenMiningTokenLockedAmount = u64;
    // Mining Speed Boost Token Mining Config
    // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
    type MiningSpeedBoostConfigurationTokenMiningTokenType = Vec<u8>;
}

impl mining_speed_boosts_configuration_hardware_mining::Trait for Runtime {
    type Event = Event;
    type MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI = u64;
    // type MiningSpeedBoostConfigurationHardwareMiningHardwareType =
    // MiningSpeedBoostConfigurationHardwareMiningHardwareTypes;
    type MiningSpeedBoostConfigurationHardwareMiningHardwareID = u64;
    type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate = u64;
    type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate = u64;
    // Mining Speed Boost Hardware Mining Config
    type MiningSpeedBoostConfigurationHardwareMiningHardwareSecure = bool;
    // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
    type MiningSpeedBoostConfigurationHardwareMiningHardwareType = Vec<u8>;
    // FIXME - restore when stop temporarily using roaming-operators
    // type Currency = Balances;
    // type Randomness = RandomnessCollectiveFlip;
    type MiningSpeedBoostConfigurationHardwareMiningIndex = u64;
}

impl mining_speed_boosts_rates_token_mining::Trait for Runtime {
    type Event = Event;
    type MiningSpeedBoostRatesTokenMiningIndex = u64;
    type MiningSpeedBoostRatesTokenMiningMaxLoyalty = u32;
    // Mining Speed Boost Max Rates
    type MiningSpeedBoostRatesTokenMiningMaxToken = u32;
    type MiningSpeedBoostRatesTokenMiningTokenDOT = u32;
    type MiningSpeedBoostRatesTokenMiningTokenIOTA = u32;
    // Mining Speed Boost Rate
    type MiningSpeedBoostRatesTokenMiningTokenMXC = u32;
}

impl mining_speed_boosts_rates_hardware_mining::Trait for Runtime {
    type Event = Event;
    type MiningSpeedBoostRatesHardwareMiningHardwareInsecure = u32;
    // Mining Speed Boost Rate
    type MiningSpeedBoostRatesHardwareMiningHardwareSecure = u32;
    type MiningSpeedBoostRatesHardwareMiningIndex = u64;
    // Mining Speed Boost Max Rates
    type MiningSpeedBoostRatesHardwareMiningMaxHardware = u32;
}

impl mining_speed_boosts_sampling_token_mining::Trait for Runtime {
    type Event = Event;
    type MiningSpeedBoostSamplingTokenMiningIndex = u64;
    type MiningSpeedBoostSamplingTokenMiningSampleDate = u64;
    type MiningSpeedBoostSamplingTokenMiningSampleTokensLocked = u64;
}

impl mining_speed_boosts_sampling_hardware_mining::Trait for Runtime {
    type Event = Event;
    type MiningSpeedBoostSamplingHardwareMiningIndex = u64;
    type MiningSpeedBoostSamplingHardwareMiningSampleDate = u64;
    type MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline = u64;
}

impl mining_speed_boosts_eligibility_token_mining::Trait for Runtime {
    type Event = Event;
    type MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility = u64;
    type MiningSpeedBoostEligibilityTokenMiningIndex = u64;
    type MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage = u32;
    // type MiningSpeedBoostEligibilityTokenMiningDateAudited = u64;
    // type MiningSpeedBoostEligibilityTokenMiningAuditorAccountID = u64;
}

impl mining_speed_boosts_eligibility_hardware_mining::Trait for Runtime {
    type Event = Event;
    type MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility = u64;
    type MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage = u32;
    type MiningSpeedBoostEligibilityHardwareMiningIndex = u64;
    // type MiningSpeedBoostEligibilityHardwareMiningDateAudited = u64;
    // type MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID = u64;
}

impl mining_speed_boosts_claims_token_mining::Trait for Runtime {
    type Event = Event;
    type MiningSpeedBoostClaimsTokenMiningClaimAmount = u64;
    type MiningSpeedBoostClaimsTokenMiningClaimDateRedeemed = u64;
    type MiningSpeedBoostClaimsTokenMiningIndex = u64;
}

impl mining_speed_boosts_claims_hardware_mining::Trait for Runtime {
    type Event = Event;
    type MiningSpeedBoostClaimsHardwareMiningClaimAmount = u64;
    type MiningSpeedBoostClaimsHardwareMiningClaimDateRedeemed = u64;
    type MiningSpeedBoostClaimsHardwareMiningIndex = u64;
}

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: system::{Module, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Babe: pallet_babe::{Module, Call, Storage, Config, Inherent(Timestamp)},
        Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
        Indices: pallet_indices::{Module, Call, Storage, Event<T>, Config<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Module, Storage},
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
        GeneralCouncil: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        GeneralCouncilMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
		// FinancialCouncil: pallet_collective::<Instance2>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
		// FinancialCouncilMembership: pallet_membership::<Instance2>::{Module, Call, Storage, Event<T>, Config<T>},
        // OperatorCollective: pallet_collective::<Instance3>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
		// OperatorMembership: pallet_membership::<Instance3>::{Module, Call, Storage, Event<T>, Config<T>},
        // Oracle: orml_oracle::{Module, Call, Storage, Event<T>},
        PalletTreasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>},
        Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
		// Tokens: orml_tokens::{Module, Storage, Call, Event<T>, Config<T>},
		// Currencies: orml_currencies::{Module, Call, Event<T>},
		// Prices: orml_prices::{Module, Storage},
        DataHighwayRoamingOperators: roaming_operators::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingNetworks: roaming_networks::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingOrganizations: roaming_organizations::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingNetworkServers: roaming_network_servers::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingDevices: roaming_devices::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingRoutingProfiles: roaming_routing_profiles::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingServiceProfiles: roaming_service_profiles::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingAccountingPolicies: roaming_accounting_policies::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingAgreementPolicies: roaming_agreement_policies::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingNetworkProfiles: roaming_network_profiles::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingDeviceProfiles: roaming_device_profiles::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingSessions: roaming_sessions::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingBillingPolicies: roaming_billing_policies::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingChargingPolicies: roaming_charging_policies::{Module, Call, Storage, Event<T>},
        DataHighwayRoamingPacketBundles: roaming_packet_bundles::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostConfigurationTokenMining: mining_speed_boosts_configuration_token_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostConfigurationHardwareMining: mining_speed_boosts_configuration_hardware_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostRatesTokenMining: mining_speed_boosts_rates_token_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostRatesHardwareMining: mining_speed_boosts_rates_hardware_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostSamplingTokenMining: mining_speed_boosts_sampling_token_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostSamplingHardwareMining: mining_speed_boosts_sampling_hardware_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostEligibilityTokenMining: mining_speed_boosts_eligibility_token_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostEligibilityHardwareMining: mining_speed_boosts_eligibility_hardware_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostClaimsTokenMining: mining_speed_boosts_claims_token_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostClaimsHardwareMining: mining_speed_boosts_claims_hardware_mining::{Module, Call, Storage, Event<T>},
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
	// orml_oracle::CheckOperator<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
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

        fn apply_trusted_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_trusted_extrinsic(extrinsic)
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

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                secondary_slots: true,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::SlotNumber {
            Babe::current_epoch_start()
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

    impl system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
        UncheckedExtrinsic,
    > for Runtime {
        fn query_info(uxt: UncheckedExtrinsic, len: u32) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
    }

    // impl orml_oracle_rpc_runtime_api::OracleApi<
    //     Block,
    //     CurrencyId,
    //     TimeStampedPrice,
    // > for Runtime {
    //     fn get_value(key: CurrencyId) -> Option<TimeStampedPrice> {
    //         Oracle::get_no_op(&key)
    //     }
    // }
}
