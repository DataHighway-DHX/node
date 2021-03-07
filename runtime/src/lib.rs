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
    AuthorityId as GrandpaId,
    AuthorityList as GrandpaAuthorityList,
};
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
    curve::PiecewiseLinear,
    generic,
    impl_opaque_keys,
    traits::{
        BlakeTwo256,
        Block as BlockT,
        Convert,
        IdentifyAccount,
        IdentityLookup,
        NumberFor,
        OpaqueKeys,
        Saturating,
        Verify,
    },
    transaction_validity::{
        TransactionPriority,
        TransactionSource,
        TransactionValidity,
    },
    ApplyExtrinsicResult,
    MultiSignature,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

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
    },
    StorageValue,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_staking::StakerStatus;
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
        pub struct SessionKeys {
            pub babe: Babe,
            pub grandpa: Grandpa,
        }
    }
}

pub use constants::time::{
    DAYS,
    EPOCH_DURATION_IN_BLOCKS,
    EPOCH_DURATION_IN_SLOTS,
    HOURS,
    MILLISECS_PER_BLOCK,
    MINUTES,
    PRIMARY_PROBABILITY,
    SLOT_DURATION,
};
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

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
    WASM_BINARY.expect(
        "Development wasm binary is not available. This means the client is built with `BUILD_DUMMY_WASM_BINARY` flag \
         and it is only usable for production chains. Please rebuild with the flag disabled.",
    )
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    /// We allow for 2 seconds of compute with a 6 second average block time.
    pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    /// Assume 10% of weight for average on_initialize calls.
    pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
        .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Trait for Runtime {
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// Portion of the block weight that is available to all normal transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// The weight of the overhead invoked on the block import process, independent of the
    /// extrinsics included in that block.
    type BlockExecutionWeight = BlockExecutionWeight;
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
    type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
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
    type MaximumBlockLength = MaximumBlockLength;
    /// Maximum weight of each block.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
    /// idependent of the logic of that extrinsics. (Roughly max block weight - average on
    /// initialize cost).
    type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
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
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
}

impl pallet_babe::Trait for Runtime {
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type HandleEquivocation = ();
    type KeyOwnerIdentification =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;
    type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type KeyOwnerProofSystem = ();
    type WeightInfo = ();
}

impl pallet_grandpa::Trait for Runtime {
    type Call = Call;
    type Event = Event;
    type HandleEquivocation = ();
    type KeyOwnerIdentification =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;
    type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type KeyOwnerProofSystem = ();
    type WeightInfo = ();
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
    type WeightInfo = ();
}

parameter_types! {
    /// How much an index costs.
    pub const IndexDeposit: u128 = 100;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Runtime {
    type MinimumPeriod = MinimumPeriod;
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = Moment;
    type OnTimestampSet = Babe;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Trait for Runtime {
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

impl pallet_transaction_payment::Trait for Runtime {
    type Currency = Balances;
    type FeeMultiplierUpdate = ();
    type OnTransactionPayment = ();
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
}

impl pallet_sudo::Trait for Runtime {
    type Call = Call;
    type Event = Event;
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type GeneralCouncilInstance = pallet_collective::Instance1;
impl pallet_collective::Trait<GeneralCouncilInstance> for Runtime {
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
impl pallet_membership::Trait<GeneralCouncilMembershipInstance> for Runtime {
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

impl pallet_treasury::Trait for Runtime {
    type ApproveOrigin = pallet_collective::EnsureMembers<_4, AccountId, GeneralCouncilInstance>;
    type BountyCuratorDeposit = BountyCuratorDeposit;
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type BountyValueMinimum = BountyValueMinimum;
    type Burn = Burn;
    type BurnDestination = ();
    type Currency = Balances;
    type DataDepositPerByte = DataDepositPerByte;
    type Event = Event;
    type MaximumReasonLength = MaximumReasonLength;
    type ModuleId = TreasuryModuleId;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type RejectOrigin = pallet_collective::EnsureMembers<_2, AccountId, GeneralCouncilInstance>;
    type SpendPeriod = SpendPeriod;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type Tippers = GeneralCouncilProvider;
    // Just gets burned.
    type WeightInfo = ();
}

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Trait for Runtime {
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type Event = Event;
    type Keys = opaque::SessionKeys;
    type NextSessionRotation = Babe;
    type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type SessionManager = Staking;
    type ShouldEndSession = Babe;
    type ValidatorId = <Self as frame_system::Trait>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type WeightInfo = ();
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
    // 1 hour session, 6 hour era
    pub const SessionsPerEra: sp_staking::SessionIndex = 6;
    // 28 eras * 6 hours/era = 7 day bonding duration
    pub const BondingDuration: pallet_staking::EraIndex = 28;
    // 27 eras * 6 hours/era = 6.75 day slash duration in which slashes can be cancelled
    pub const SlashDeferDuration: pallet_staking::EraIndex = 27;
    pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
    pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
    pub const MaxNominatorRewardedPerValidator: u32 = 64;
    pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
    pub const MaxIterations: u32 = 10;
    pub const ElectionLookahead: BlockNumber = EPOCH_DURATION_IN_BLOCKS / 4;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

impl pallet_staking::Trait for Runtime {
    type BondingDuration = BondingDuration;
    type Call = Call;
    type Currency = Balances;
    type CurrencyToVote = CurrencyToVoteHandler;
    type ElectionLookahead = ElectionLookahead;
    type Event = Event;
    type MaxIterations = MaxIterations;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type NextNewSession = Session;
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
    type UnixTime = Timestamp;
    type UnsignedPriority = StakingUnsignedPriority;
    type WeightInfo = ();
}

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
}

impl roaming_billing_policies::Trait for Runtime {
    type Event = Event;
    type RoamingBillingPolicyIndex = u64;
}

impl roaming_charging_policies::Trait for Runtime {
    type Event = Event;
    type RoamingChargingPolicyIndex = u64;
}

impl roaming_packet_bundles::Trait for Runtime {
    type Event = Event;
    type RoamingPacketBundleExternalDataStorageHash = Hash;
    type RoamingPacketBundleIndex = u64;
    type RoamingPacketBundleReceivedAtHome = bool;
    type RoamingPacketBundleReceivedPacketsCount = u64;
    type RoamingPacketBundleReceivedPacketsOkCount = u64;
}

impl mining_config_token::Trait for Runtime {
    type Event = Event;
    // FIXME - restore when stop temporarily using roaming-operators
    // type Currency = Balances;
    // type Randomness = RandomnessCollectiveFlip;
    type MiningConfigTokenIndex = u64;
    type MiningConfigTokenLockAmount = u64;
    // Mining Speed Boost Token Mining Config
    // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
    type MiningConfigTokenType = Vec<u8>;
}

impl mining_config_hardware::Trait for Runtime {
    type Event = Event;
    type MiningConfigHardwareDevEUI = u64;
    // type MiningConfigHardwareType =
    // MiningConfigHardwareTypes;
    type MiningConfigHardwareID = u64;
    // FIXME - restore when stop temporarily using roaming-operators
    // type Currency = Balances;
    // type Randomness = RandomnessCollectiveFlip;
    type MiningConfigHardwareIndex = u64;
    // Mining Speed Boost Hardware Mining Config
    type MiningConfigHardwareSecure = bool;
    // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
    type MiningConfigHardwareType = Vec<u8>;
}

impl mining_rates_token::Trait for Runtime {
    type Event = Event;
    type MiningRatesTokenIndex = u64;
    type MiningRatesTokenMaxLoyalty = u32;
    // Mining Speed Boost Max Rates
    type MiningRatesTokenMaxToken = u32;
    type MiningRatesTokenTokenDOT = u32;
    type MiningRatesTokenTokenIOTA = u32;
    // Mining Speed Boost Rate
    type MiningRatesTokenTokenMXC = u32;
}

impl mining_rates_hardware::Trait for Runtime {
    type Event = Event;
    type MiningRatesHardwareCategory1MaxTokenBonusPerGateway = u32;
    type MiningRatesHardwareCategory2MaxTokenBonusPerGateway = u32;
    type MiningRatesHardwareCategory3MaxTokenBonusPerGateway = u32;
    type MiningRatesHardwareIndex = u64;
    type MiningRatesHardwareInsecure = u32;
    // Mining Speed Boost Max Rates
    type MiningRatesHardwareMaxHardware = u32;
    // Mining Speed Boost Rate
    type MiningRatesHardwareSecure = u32;
}

impl mining_sampling_token::Trait for Runtime {
    type Event = Event;
    type MiningSamplingTokenIndex = u64;
    type MiningSamplingTokenSampleLockedAmount = u64;
}

impl mining_sampling_hardware::Trait for Runtime {
    type Event = Event;
    type MiningSamplingHardwareIndex = u64;
    type MiningSamplingHardwareSampleHardwareOnline = u64;
}

impl mining_eligibility_token::Trait for Runtime {
    type Event = Event;
    type MiningEligibilityTokenCalculatedEligibility = u64;
    type MiningEligibilityTokenIndex = u64;
    type MiningEligibilityTokenLockedPercentage = u32;
    // type MiningEligibilityTokenAuditorAccountID = u64;
}

impl mining_eligibility_hardware::Trait for Runtime {
    type Event = Event;
    type MiningEligibilityHardwareCalculatedEligibility = u64;
    type MiningEligibilityHardwareIndex = u64;
    type MiningEligibilityHardwareUptimePercentage = u32;
    // type MiningEligibilityHardwareAuditorAccountID = u64;
}

impl mining_eligibility_proxy::Trait for Runtime {
    type Currency = Balances;
    type Event = Event;
    type MiningEligibilityProxyClaimBlockRedeemed = u64;
    type MiningEligibilityProxyClaimTotalRewardAmount = u32;
    type MiningEligibilityProxyIndex = u64;
}

impl mining_claims_token::Trait for Runtime {
    type Event = Event;
    type MiningClaimsTokenClaimAmount = u64;
    type MiningClaimsTokenIndex = u64;
}

impl mining_claims_hardware::Trait for Runtime {
    type Event = Event;
    type MiningClaimsHardwareClaimAmount = u64;
    type MiningClaimsHardwareIndex = u64;
}

impl mining_execution_token::Trait for Runtime {
    type Event = Event;
    type MiningExecutionTokenIndex = u64;
}

impl exchange_rate::Trait for Runtime {
    type DOTRate = u64;
    type DecimalsAfterPoint = u32;
    type Event = Event;
    type ExchangeRateIndex = u64;
    type FILRate = u64;
    type HBTCRate = u64;
    type IOTARate = u64;
}

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
        Babe: pallet_babe::{Module, Config, Inherent},
        Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event},
        Indices: pallet_indices::{Module, Call, Storage, Event<T>, Config<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Module, Storage},
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
        GeneralCouncil: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        GeneralCouncilMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
        PalletTreasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>},
        RoamingOperators: roaming_operators::{Module, Call, Storage, Event<T>},
        RoamingNetworks: roaming_networks::{Module, Call, Storage, Event<T>},
        RoamingOrganizations: roaming_organizations::{Module, Call, Storage, Event<T>},
        RoamingNetworkServers: roaming_network_servers::{Module, Call, Storage, Event<T>},
        RoamingDevices: roaming_devices::{Module, Call, Storage, Event<T>},
        RoamingRoutingProfiles: roaming_routing_profiles::{Module, Call, Storage, Event<T>},
        RoamingServiceProfiles: roaming_service_profiles::{Module, Call, Storage, Event<T>},
        RoamingAccountingPolicies: roaming_accounting_policies::{Module, Call, Storage, Event<T>},
        RoamingAgreementPolicies: roaming_agreement_policies::{Module, Call, Storage, Event<T>},
        RoamingNetworkProfiles: roaming_network_profiles::{Module, Call, Storage, Event<T>},
        RoamingDeviceProfiles: roaming_device_profiles::{Module, Call, Storage, Event<T>},
        RoamingSessions: roaming_sessions::{Module, Call, Storage, Event<T>},
        RoamingBillingPolicies: roaming_billing_policies::{Module, Call, Storage, Event<T>},
        RoamingChargingPolicies: roaming_charging_policies::{Module, Call, Storage, Event<T>},
        RoamingPacketBundles: roaming_packet_bundles::{Module, Call, Storage, Event<T>},
        MiningConfigToken: mining_config_token::{Module, Call, Storage, Event<T>},
        MiningConfigHardware: mining_config_hardware::{Module, Call, Storage, Event<T>},
        MiningRatesToken: mining_rates_token::{Module, Call, Storage, Event<T>},
        MiningRatesHardware: mining_rates_hardware::{Module, Call, Storage, Event<T>},
        MiningSamplingToken: mining_sampling_token::{Module, Call, Storage, Event<T>},
        MiningSamplingHardware: mining_sampling_hardware::{Module, Call, Storage, Event<T>},
        MiningEligibilityToken: mining_eligibility_token::{Module, Call, Storage, Event<T>},
        MiningEligibilityHardware: mining_eligibility_hardware::{Module, Call, Storage, Event<T>},
        MiningEligibilityProxy: mining_eligibility_proxy::{Module, Call, Storage, Event<T>},
        MiningClaimsToken: mining_claims_token::{Module, Call, Storage, Event<T>},
        MiningClaimsHardware: mining_claims_hardware::{Module, Call, Storage, Event<T>},
        MiningExecutionToken: mining_execution_token::{Module, Call, Storage, Event<T>},
        ExchangeRate: exchange_rate::{Module, Call, Storage, Event<T>},
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

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
            // The choice of `c` parameter (where `1 - c` represents the
            // probability of a slot being empty), is done in accordance to the
            // slot duration and expected target block time, for safely
            // resisting network delays of maximum two seconds.
            // <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
            sp_consensus_babe::BabeGenesisConfiguration {
                slot_duration: Babe::slot_duration(),
                epoch_length: EpochDuration::get(),
                c: PRIMARY_PROBABILITY,
                genesis_authorities: Babe::authorities(),
                randomness: Babe::randomness(),
                allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
            }
        }

        fn current_epoch_start() -> sp_consensus_babe::SlotNumber {
            Babe::current_epoch_start()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            _key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        )  -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _slot_number: sp_consensus_babe::SlotNumber,
            _authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
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

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
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
    }
}
