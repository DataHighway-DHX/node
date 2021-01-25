#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
mod constants;
mod types;

use sp_api::impl_runtime_apis;
use sp_core::{
    crypto::KeyTypeId,
    u32_trait::{
        _2,
        _3,
        _4,
    },
    OpaqueMetadata,
};
use sp_runtime::{
    create_runtime_str,
    generic,
    impl_opaque_keys,
    traits::{
        BlakeTwo256,
        Block as BlockT,
        IdentityLookup,
    },
    transaction_validity::{
        TransactionSource,
        TransactionValidity,
    },
    ApplyExtrinsicResult,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use frame_system::limits::{BlockLength, BlockWeights};

use polkadot_parachain::primitives::Sibling;
use xcm::v0::{Junction, MultiLocation, NetworkId};
use xcm_builder::{
	AccountId32Aliases, CurrencyAdapter, LocationInverter, ParentIsDefault, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SovereignSignedViaLocation,
};
use xcm_executor::{
	traits::{IsConcrete, NativeAsset},
	Config, XcmExecutor,
};

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    construct_runtime,
    parameter_types,
    traits::{
        Contains,
        ContainsLengthBound,
        KeyOwnerProofSystem,
        Randomness,
    },
    weights::{
        constants::{
            BlockExecutionWeight,
            ExtrinsicBaseWeight,
            RocksDbWeight,
            WEIGHT_PER_SECOND,
        },
        IdentityFee,
        Weight,
        DispatchClass,
    },
    StorageValue,
};
pub use pallet_balances::Call as BalancesCall;
//pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{
    ModuleId,
    Perbill,
    Percent,
    Permill,
};

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
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
        pub struct SessionKeys {}
    }
}

pub use constants::time::*;
pub use types::*;

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("datahighway"),
    impl_name: create_runtime_str!("datahighway"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
    // pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    /// Assume 10% of weight for average on_initialize calls.
    // pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
    //     .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
        .base_block(BlockExecutionWeight::get())
        .for_class(DispatchClass::all(), |weights| {
            weights.base_extrinsic = ExtrinsicBaseWeight::get();
        })
        .for_class(DispatchClass::Normal, |weights| {
            weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
        })
        .for_class(DispatchClass::Operational, |weights| {
            weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
            // Operational transactions have some extra reserved space, so that they
            // are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
            weights.reserved = Some(
                MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
            );
        })
        .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
        .build_or_panic();
    pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// Portion of the block weight that is available to all normal transactions.
    // type AvailableBlockRatio = AvailableBlockRatio;
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
    /// The weight of the overhead invoked on the block import process, independent of the
    /// extrinsics included in that block.
    // type BlockExecutionWeight = BlockExecutionWeight;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// The ubiquitous event type.
    type Event = Event;
    /// The base weight of any extrinsic processed by the runtime, independent of the
    /// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
    // type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = IdentityLookup<AccountId>;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    // type MaximumBlockLength = MaximumBlockLength;
    /// Maximum weight of each block.
    // type MaximumBlockWeight = MaximumBlockWeight;
    /// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
    /// idependent of the logic of that extrinsics. (Roughly max block weight - average on
    /// initialize cost).
    // type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// Version of the runtime.
    type Version = Version;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

// impl pallet_indices::Config for Runtime {
//     /// The type for recording indexing into the account enumeration. If this ever overflows, there
//     /// will be problems!
//     type AccountIndex = AccountIndex;
//     /// The currency type.
//     type Currency = Balances;
//     /// How much an index costs.
//     type Deposit = IndexDeposit;
//     /// The ubiquitous event type.
//     type Event = Event;
//     type WeightInfo = ();
// }

parameter_types! {
    /// How much an index costs.
    pub const IndexDeposit: u128 = 100;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = Moment;
    type OnTimestampSet = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    /// The type for recording an account's balance.
    type Balance = Balance;
    type DustRemoval = ();
    /// The ubiquitous event type.
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type WeightInfo = ();
}

parameter_types! {
    pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Config for Runtime {
    type FeeMultiplierUpdate = ();
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
}

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type GeneralCouncilInstance = pallet_collective::Instance1;
impl pallet_collective::Config<GeneralCouncilInstance> for Runtime {
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type Event = Event;
    type MaxMembers = CouncilMaxMembers;
    type MaxProposals = CouncilMaxProposals;
    type MotionDuration = CouncilMotionDuration;
    type Origin = Origin;
    type Proposal = Call;
    type WeightInfo = ();
}

type GeneralCouncilMembershipInstance = pallet_membership::Instance1;
impl pallet_membership::Config<GeneralCouncilMembershipInstance> for Runtime {
    type AddOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
    type Event = Event;
    type MembershipChanged = GeneralCouncil;
    type MembershipInitialized = GeneralCouncil;
    type PrimeOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
    type RemoveOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
    type ResetOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
    type SwapOrigin = pallet_collective::EnsureProportionMoreThan<_3, _4, AccountId, GeneralCouncilInstance>;
}

pub struct GeneralCouncilProvider;
impl Contains<AccountId> for GeneralCouncilProvider {
    fn contains(who: &AccountId) -> bool {
        GeneralCouncil::is_member(who)
    }

    fn sorted_members() -> Vec<AccountId> {
        GeneralCouncil::members()
    }
}
impl ContainsLengthBound for GeneralCouncilProvider {
    fn min_len() -> usize {
        0
    }

    fn max_len() -> usize {
        100000
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
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyValueMinimum: u64 = 1;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyDepositBase: u64 = 80;
    pub const BountyDepositPayoutDelay: u32 = 3;
    pub const BountyUpdatePeriod: u32 = 20;
    pub const DataDepositPerByte: u64 = 1;
    pub const TreasuryModuleId: ModuleId = ModuleId(*b"py/trsry");
}

impl pallet_treasury::Config for Runtime {
    type ApproveOrigin = pallet_collective::EnsureMembers<_4, AccountId, GeneralCouncilInstance>;
    // type BountyCuratorDeposit = BountyCuratorDeposit;
    // type BountyDepositBase = BountyDepositBase;
    // type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    // type BountyUpdatePeriod = BountyUpdatePeriod;
    // type BountyValueMinimum = BountyValueMinimum;
    type Burn = Burn;
    type BurnDestination = ();
    type Currency = Balances;
    // type DataDepositPerByte = DataDepositPerByte;
    type Event = Event;
    // type MaximumReasonLength = MaximumReasonLength;
    type ModuleId = TreasuryModuleId;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type RejectOrigin = pallet_collective::EnsureMembers<_2, AccountId, GeneralCouncilInstance>;
    type SpendPeriod = SpendPeriod;
    type SpendFunds = ();
    // type TipCountdown = TipCountdown;
    // type TipFindersFee = TipFindersFee;
    // type TipReportDepositBase = TipReportDepositBase;
    // type Tippers = GeneralCouncilProvider;
    // Just gets burned.
    type WeightInfo = ();
}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

// pallet_staking_reward_curve::build! {
//     const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
//         min_inflation: 0_025_000,
//         max_inflation: 0_100_000,
//         ideal_stake: 0_500_000,
//         falloff: 0_050_000,
//         max_piece_count: 40,
//         test_precision: 0_005_000,
//     );
// }

parameter_types! {
    //pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    //pub const BondingDuration: pallet_staking::EraIndex = 24 * 28;
    //pub const SlashDeferDuration: pallet_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
    //pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    //pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
    pub const MaxIterations: u32 = 10;
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
	/// A limit for off-chain phragmen unsigned solution submission.
	///
	/// We want to keep it as high as possible, but can't risk having it reject,
	/// so we always subtract the base block execution weight.
	pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
        .get(DispatchClass::Normal)
        .max_extrinsic
        .expect("Normal extrinsics have weight limit configured by default; qed")
        .saturating_sub(BlockExecutionWeight::get());
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

// impl pallet_staking::Config for Runtime {
//     type BondingDuration = BondingDuration;
//     type Call = Call;
//     type Currency = Balances;
//     type CurrencyToVote = frame_support::traits::U128CurrencyToVote;
//     type ElectionLookahead = ElectionLookahead;
//     type Event = Event;
//     type MaxIterations = MaxIterations;
//     type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
//     type MinSolutionScoreBump = MinSolutionScoreBump;
//     type NextNewSession = Session;
//     // send the slashed funds to the pallet treasury.
//     type Reward = ();
//     type RewardCurve = RewardCurve;
//     type RewardRemainder = PalletTreasury;
//     type SessionInterface = Self;
//     type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
//     // rewards are minted from the void
//     type SessionsPerEra = SessionsPerEra;
//     type Slash = PalletTreasury;
//     /// A super-majority of the council can cancel the slash.
//     type SlashCancelOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, GeneralCouncilInstance>;
//     type SlashDeferDuration = SlashDeferDuration;
//     type UnixTime = Timestamp;
//     type UnsignedPriority = StakingUnsignedPriority;
//     type WeightInfo = ();
// }

impl roaming_operators::Config for Runtime {
    type Currency = Balances;
    type Event = Event;
    type Randomness = RandomnessCollectiveFlip;
    type RoamingOperatorIndex = u64;
}

impl roaming_networks::Config for Runtime {
    type Event = Event;
    type RoamingNetworkIndex = u64;
}

impl roaming_organizations::Config for Runtime {
    type Event = Event;
    type RoamingOrganizationIndex = u64;
}

impl roaming_network_servers::Config for Runtime {
    type Event = Event;
    type RoamingNetworkServerIndex = u64;
}

impl roaming_devices::Config for Runtime {
    type Event = Event;
    type RoamingDeviceIndex = u64;
}

impl roaming_routing_profiles::Config for Runtime {
    type Event = Event;
    // https://polkadot.js.org/api/types/#primitive-types
    type RoamingRoutingProfileAppServer = Vec<u8>;
    type RoamingRoutingProfileIndex = u64;
}

impl roaming_service_profiles::Config for Runtime {
    type Event = Event;
    type RoamingServiceProfileDownlinkRate = u32;
    type RoamingServiceProfileIndex = u64;
    type RoamingServiceProfileUplinkRate = u32;
}

impl roaming_accounting_policies::Config for Runtime {
    type Event = Event;
    type RoamingAccountingPolicyDownlinkFeeFactor = u32;
    type RoamingAccountingPolicyIndex = u64;
    type RoamingAccountingPolicyType = Vec<u8>;
    type RoamingAccountingPolicyUplinkFeeFactor = u32;
}

impl roaming_agreement_policies::Config for Runtime {
    type Event = Event;
    type RoamingAgreementPolicyActivationType = Vec<u8>;
    type RoamingAgreementPolicyExpiry = u64;
    type RoamingAgreementPolicyIndex = u64; // <pallet_timestamp::Module<Runtime> as Config>::Moment` timestamp::Module<Runtime>::Moment;
}

impl roaming_network_profiles::Config for Runtime {
    type Event = Event;
    type RoamingNetworkProfileIndex = u64;
}

impl roaming_device_profiles::Config for Runtime {
    type Event = Event;
    type RoamingDeviceProfileDevAddr = Vec<u8>;
    type RoamingDeviceProfileDevEUI = Vec<u8>;
    type RoamingDeviceProfileIndex = u64;
    type RoamingDeviceProfileJoinEUI = Vec<u8>;
    type RoamingDeviceProfileVendorID = Vec<u8>;
}

impl roaming_sessions::Config for Runtime {
    type Event = Event;
    type RoamingSessionIndex = u64;
    type RoamingSessionJoinRequestAcceptAcceptedAt = u64;
    type RoamingSessionJoinRequestAcceptExpiry = u64;
    type RoamingSessionJoinRequestRequestedAt = u64;
}

impl roaming_billing_policies::Config for Runtime {
    type Event = Event;
    type RoamingBillingPolicyFrequencyInDays = u64;
    type RoamingBillingPolicyIndex = u64;
    type RoamingBillingPolicyNextBillingAt = u64;
}

impl roaming_charging_policies::Config for Runtime {
    type Event = Event;
    type RoamingChargingPolicyDelayAfterBillingInDays = u64;
    type RoamingChargingPolicyIndex = u64;
    type RoamingChargingPolicyNextChargingAt = u64;
}

impl roaming_packet_bundles::Config for Runtime {
    type Event = Event;
    type RoamingPacketBundleExternalDataStorageHash = Hash;
    type RoamingPacketBundleIndex = u64;
    type RoamingPacketBundleReceivedAtHome = bool;
    type RoamingPacketBundleReceivedEndedAt = u64;
    type RoamingPacketBundleReceivedPacketsCount = u64;
    type RoamingPacketBundleReceivedPacketsOkCount = u64;
    type RoamingPacketBundleReceivedStartedAt = u64;
}

impl mining_speed_boosts_configuration_token_mining::Config for Runtime {
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

impl mining_speed_boosts_configuration_hardware_mining::Config for Runtime {
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

impl mining_speed_boosts_rates_token_mining::Config for Runtime {
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

impl mining_speed_boosts_rates_hardware_mining::Config for Runtime {
    type Event = Event;
    type MiningSpeedBoostRatesHardwareMiningHardwareInsecure = u32;
    // Mining Speed Boost Rate
    type MiningSpeedBoostRatesHardwareMiningHardwareSecure = u32;
    type MiningSpeedBoostRatesHardwareMiningIndex = u64;
    // Mining Speed Boost Max Rates
    type MiningSpeedBoostRatesHardwareMiningMaxHardware = u32;
}

impl mining_speed_boosts_sampling_token_mining::Config for Runtime {
    type Event = Event;
    type MiningSpeedBoostSamplingTokenMiningIndex = u64;
    type MiningSpeedBoostSamplingTokenMiningSampleDate = u64;
    type MiningSpeedBoostSamplingTokenMiningSampleTokensLocked = u64;
}

impl mining_speed_boosts_sampling_hardware_mining::Config for Runtime {
    type Event = Event;
    type MiningSpeedBoostSamplingHardwareMiningIndex = u64;
    type MiningSpeedBoostSamplingHardwareMiningSampleDate = u64;
    type MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline = u64;
}

impl mining_speed_boosts_eligibility_token_mining::Config for Runtime {
    type Event = Event;
    type MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility = u64;
    type MiningSpeedBoostEligibilityTokenMiningIndex = u64;
    type MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage = u32;
    // type MiningSpeedBoostEligibilityTokenMiningDateAudited = u64;
    // type MiningSpeedBoostEligibilityTokenMiningAuditorAccountID = u64;
}

impl mining_speed_boosts_eligibility_hardware_mining::Config for Runtime {
    type Event = Event;
    type MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility = u64;
    type MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage = u32;
    type MiningSpeedBoostEligibilityHardwareMiningIndex = u64;
    // type MiningSpeedBoostEligibilityHardwareMiningDateAudited = u64;
    // type MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID = u64;
}

impl mining_speed_boosts_lodgements_token_mining::Config for Runtime {
    type Event = Event;
    type MiningSpeedBoostLodgementsTokenMiningIndex = u64;
    type MiningSpeedBoostLodgementsTokenMiningLodgementAmount = u64;
    type MiningSpeedBoostLodgementsTokenMiningLodgementDateRedeemed = u64;
}

impl mining_speed_boosts_lodgements_hardware_mining::Config for Runtime {
    type Event = Event;
    type MiningSpeedBoostLodgementsHardwareMiningIndex = u64;
    type MiningSpeedBoostLodgementsHardwareMiningLodgementAmount = u64;
    type MiningSpeedBoostLodgementsHardwareMiningLodgementDateRedeemed = u64;
}

parameter_types! {
	pub const RococoLocation: MultiLocation = MultiLocation::X1(Junction::Parent);
	pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayChainOrigin: Origin = xcm_handler::Origin::Relay.into();
	pub Ancestry: MultiLocation = Junction::Parachain {
		id: ParachainInfo::parachain_id().into()
	}.into();
}

type LocationConverter = (
	ParentIsDefault<AccountId>,
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<RococoNetwork, AccountId>,
);

type LocalAssetTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RococoLocation>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationConverter,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
>;

type LocalOriginConverter = (
	SovereignSignedViaLocation<LocationConverter, Origin>,
	RelayChainAsNative<RelayChainOrigin, Origin>,
	SiblingParachainAsNative<xcm_handler::Origin, Origin>,
	SignedAccountId32AsNative<RococoNetwork, Origin>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmHandler;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = LocalOriginConverter;
	type IsReserve = NativeAsset;
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
}

impl xcm_handler::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type UpwardMessageSender = MessageBroker;
	type HrmpMessageSender = MessageBroker;
}

impl cumulus_parachain_upgrade::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = parachain_info::Module<Runtime>;
}

impl cumulus_message_broker::Config for Runtime {
	type DownwardMessageHandlers = ();
	type HrmpMessageHandlers = ();
}

impl parachain_info::Config for Runtime {}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        //Indices: pallet_indices::{Module, Call, Storage, Event<T>, Config<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Module, Storage},
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
        GeneralCouncil: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        GeneralCouncilMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
        PalletTreasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>},
        //Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>},
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
        DataHighwayMiningSpeedBoostLodgementsTokenMining: mining_speed_boosts_lodgements_token_mining::{Module, Call, Storage, Event<T>},
        DataHighwayMiningSpeedBoostLodgementsHardwareMining: mining_speed_boosts_lodgements_hardware_mining::{Module, Call, Storage, Event<T>},
        // PARACHAIN
		ParachainUpgrade: cumulus_parachain_upgrade::{Module, Call, Storage, Inherent, Event},
		MessageBroker: cumulus_message_broker::{Module, Storage, Call, Inherent},
		ParachainInfo: parachain_info::{Module, Storage, Config},
		XcmHandler: xcm_handler::{Module, Event<T>, Origin},
    }
);

/// The address format for describing accounts.
pub type Address = AccountId;
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
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
    frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllModules>;

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
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
    }
}

cumulus_runtime::register_validate_block!(Block, Executive);
