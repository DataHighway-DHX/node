#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

use codec::{
    Decode,
    Encode,
};
use pallet_grandpa::{
    fg_primitives,
    AuthorityId as GrandpaId,
    AuthorityList as GrandpaAuthorityList,
};
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{
    crypto::KeyTypeId,
    u32_trait::{
        _1,
        _2,
        _3,
        _4,
        _5,
    },
    OpaqueMetadata,
};
use sp_inherents::{
    CheckInherentsResult,
    InherentData,
};
use sp_runtime::{
    create_runtime_str,
    curve::PiecewiseLinear,
    generic,
    impl_opaque_keys,
    traits::{
        AccountIdLookup,
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
    ModuleId,
    Perbill,
    Percent,
    Permill,
    Perquintill,
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
        Currency,
        Contains,
        ContainsLengthBound,
        OnUnbalanced,
        KeyOwnerProofSystem,
        Randomness,
        U128CurrencyToVote,
    },
    weights::{
        constants::{
            BlockExecutionWeight,
            ExtrinsicBaseWeight,
            RocksDbWeight,
            WEIGHT_PER_SECOND,
        },
        DispatchClass,
        IdentityFee,
        Weight,
    },
    StorageValue,
};
use frame_system::{
    limits::{
        BlockLength,
        BlockWeights,
    },
    EnsureOneOf,
    EnsureRoot,
};
use pallet_session::historical as pallet_session_historical;
pub use pallet_transaction_payment::{
    CurrencyAdapter,
    Multiplier,
    TargetedFeeAdjustment,
};
use pallet_transaction_payment::{
    FeeDetails,
    RuntimeDispatchInfo,
};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    pub use super::{
        BlockNumber,
        Hash,
    };

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub babe: Babe,
        pub grandpa: Grandpa,
        // pub im_online: ImOnline,
        pub authority_discovery: AuthorityDiscovery,
    }
}

/// Constant values used within the runtime.
pub mod constants;
pub use constants::{
    currency::*,
    time::*,
};
pub mod types;
pub use types::*;



// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(feature = "std")]
pub fn wasm_binary_unwrap() -> &'static [u8] {
    WASM_BINARY.expect(
        "Development wasm binary is not available. This means the client is built with `SKIP_WASM_BUILD` flag and it \
         is only usable for production chains. Please rebuild with the flag disabled.",
    )
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("datahighway"),
    impl_name: create_runtime_str!("datahighway"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

/// Native version.
#[cfg(any(feature = "std", test))]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub const BlockHashCount: BlockNumber = 2400;
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
    pub const SS58Prefix: u8 = 33;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = ();
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
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
    type Lookup = AccountIdLookup<AccountId, ()>;
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
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// Version of the runtime.
    type Version = Version;
}

parameter_types! {
    pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
    pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
    pub const ReportLongevity: u64 =
        BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
    type EpochChangeTrigger = pallet_babe::ExternalTrigger;
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    type HandleEquivocation = ();
    // type HandleEquivocation = pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
    type KeyOwnerIdentification =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::IdentificationTuple;
    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
    type KeyOwnerProofSystem = Historical;
    type WeightInfo = ();
}

// parameter_types! {
//     pub OffencesWeightSoftLimit: Weight = Perbill::from_percent(60) * RuntimeBlockWeights::get().max_block;
// }

// impl pallet_offences::Config for Runtime {
//     type Event = Event;
//     type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
//     type OnOffenceHandler = Staking;
//     type WeightSoftLimit = OffencesWeightSoftLimit;
// }

impl pallet_authority_discovery::Config for Runtime {}

impl pallet_grandpa::Config for Runtime {
    // type Call = Call;
    // type Event = Event;
    // type HandleEquivocation = ();
    // type KeyOwnerIdentification =
    //     <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;
    // type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    // type KeyOwnerProofSystem = ();
    // type WeightInfo = ();
    type Call = Call;
    type Event = Event;
    type HandleEquivocation = ();
    // type HandleEquivocation =
    //     pallet_grandpa::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;
    type KeyOwnerIdentification =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;
    type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type KeyOwnerProofSystem = Historical;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: Moment = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Babe;
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1 * DOLLARS;
    // For weight estimation, we assume that the most locks on an individual account will be 50.
    // This number may need to be adjusted in the future if this assumption no longer holds true.
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type AccountStore = frame_system::Module<Runtime>;
    /// The type for recording an account's balance.
    type Balance = Balance;
    type DustRemoval = ();
    /// The ubiquitous event type.
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Config for Runtime {
    type FeeMultiplierUpdate = ();
    type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
}

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

parameter_types! {
    pub const IndexDeposit: Balance = 1 * DOLLARS;
}

impl pallet_indices::Config for Runtime {
    /// The type for recording indexing into the account enumeration. If this ever overflows, there
    /// will be problems!
    type AccountIndex = AccountIndex;
    /// The currency type.
    type Currency = Balances;
    /// How much an index costs.
    type Deposit = IndexDeposit;
    /// The ubiquitous event type.
    type Event = Event;
    type WeightInfo = pallet_indices::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type Event = Event;
    type MaxMembers = CouncilMaxMembers;
    type MaxProposals = CouncilMaxProposals;
    type MotionDuration = CouncilMotionDuration;
    type Origin = Origin;
    type Proposal = Call;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
    pub const TechnicalMaxProposals: u32 = 100;
    pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type Event = Event;
    type MaxMembers = TechnicalMaxMembers;
    type MaxProposals = TechnicalMaxProposals;
    type MotionDuration = TechnicalMotionDuration;
    type Origin = Origin;
    type Proposal = Call;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

type EnsureRootOrHalfCouncil = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;
impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
    type AddOrigin = EnsureRootOrHalfCouncil;
    type Event = Event;
    type MembershipChanged = TechnicalCommittee;
    type MembershipInitialized = TechnicalCommittee;
    type PrimeOrigin = EnsureRootOrHalfCouncil;
    type RemoveOrigin = EnsureRootOrHalfCouncil;
    type ResetOrigin = EnsureRootOrHalfCouncil;
    type SwapOrigin = EnsureRootOrHalfCouncil;
}

// pub struct GeneralCouncilProvider;
// impl Contains<AccountId> for GeneralCouncilProvider {
//     fn contains(who: &AccountId) -> bool {
//         Council::is_member(who)
//     }

//     fn sorted_members() -> Vec<AccountId> {
//         Council::members()
//     }
// }
// impl ContainsLengthBound for GeneralCouncilProvider {
//     fn min_len() -> usize {
//         0
//     }

//     fn max_len() -> usize {
//         100000
//     }
// }

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
    pub const SpendPeriod: BlockNumber = 1 * DAYS;
    pub const Burn: Permill = Permill::from_percent(50);
    pub const TipCountdown: BlockNumber = 1 * DAYS;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: Balance = 1 * DOLLARS;
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyValueMinimum: Balance = 5 * DOLLARS;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyDepositBase: Balance = 1 * DOLLARS;
    pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
    pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
    pub const DataDepositPerByte: Balance = 1 * CENTS;
    pub const TreasuryModuleId: ModuleId = ModuleId(*b"py/trsry");
}

impl pallet_treasury::Config for Runtime {
    type ApproveOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
    >;
    type Burn = Burn;
    type BurnDestination = ();
    type Currency = Balances;
    type Event = Event;
    type ModuleId = TreasuryModuleId;
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type RejectOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
    >;
    type SpendFunds = ();
    type SpendPeriod = SpendPeriod;
    // Just gets burned.
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
}

// impl pallet_bounties::Config for Runtime {
//     type BountyCuratorDeposit = BountyCuratorDeposit;
//     type BountyDepositBase = BountyDepositBase;
//     type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
//     type BountyUpdatePeriod = BountyUpdatePeriod;
//     type BountyValueMinimum = BountyValueMinimum;
//     type DataDepositPerByte = DataDepositPerByte;
//     type Event = Event;
//     type MaximumReasonLength = MaximumReasonLength;
//     type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
// }

// impl pallet_tips::Config for Runtime {
//     type DataDepositPerByte = DataDepositPerByte;
//     type Event = Event;
//     type MaximumReasonLength = MaximumReasonLength;
//     type TipCountdown = TipCountdown;
//     type TipFindersFee = TipFindersFee;
//     type TipReportDepositBase = TipReportDepositBase;
//     // TODO - change value to `Elections`. See Substrate 3 migration guide
//     type Tippers = GeneralCouncilProvider;
//     type WeightInfo = pallet_tips::weights::SubstrateWeight<Runtime>;
// }

parameter_types! {
    pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Config for Runtime {
    type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    type Event = Event;
    type Keys = SessionKeys;
    type NextSessionRotation = Babe;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
    type ShouldEndSession = Babe;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = pallet_staking::StashOf<Self>;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
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

// /// Struct that handles the conversion of Balance -> `u64`. This is used for staking's election
// /// calculation.
// pub struct CurrencyToVoteHandler;

// impl CurrencyToVoteHandler {
//     fn factor() -> Balance {
//         (Balances::total_issuance() / u64::max_value() as Balance).max(1)
//     }
// }

// impl Convert<Balance, u64> for CurrencyToVoteHandler {
//     fn convert(x: Balance) -> u64 {
//         (x / Self::factor()) as u64
//     }
// }

// impl Convert<u128, Balance> for CurrencyToVoteHandler {
//     fn convert(x: u128) -> Balance {
//         x * Self::factor()
//     }
// }

// parameter_types! {
//     // phase durations. 1/4 of the last session for each.
//     pub const SignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;
//     pub const UnsignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;

//     // fallback: no need to do on-chain phragmen initially.
//     pub const Fallback: pallet_election_provider_multi_phase::FallbackStrategy =
//         pallet_election_provider_multi_phase::FallbackStrategy::Nothing;

//     pub SolutionImprovementThreshold: Perbill = Perbill::from_rational_approximation(1u32, 10_000);

//     // miner configs
//     pub const MultiPhaseUnsignedPriority: TransactionPriority = StakingUnsignedPriority::get() - 1u64;
//     pub const MinerMaxIterations: u32 = 10;
//     pub MinerMaxWeight: Weight = RuntimeBlockWeights::get()
//         .get(DispatchClass::Normal)
//         .max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
//         .saturating_sub(BlockExecutionWeight::get());
// }

// impl pallet_election_provider_multi_phase::Config for Runtime {
//     type BenchmarkingConfig = ();
//     type CompactSolution = pallet_staking::CompactAssignments;
//     type Currency = Balances;
//     type DataProvider = Staking;
//     type Event = Event;
//     type Fallback = Fallback;
//     type MinerMaxIterations = MinerMaxIterations;
//     type MinerMaxWeight = MinerMaxWeight;
//     type MinerTxPriority = MultiPhaseUnsignedPriority;
//     type OnChainAccuracy = Perbill;
//     type SignedPhase = SignedPhase;
//     type SolutionImprovementThreshold = MinSolutionScoreBump;
//     type UnsignedPhase = UnsignedPhase;
//     type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Runtime>;
// }

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
    pub OffchainSolutionWeightLimit: Weight = RuntimeBlockWeights::get()
        .get(DispatchClass::Normal)
        .max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
        .saturating_sub(BlockExecutionWeight::get());
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

impl pallet_staking::Config for Runtime {
    type BondingDuration = BondingDuration;
    type Call = Call;
    type Currency = Balances;
    type CurrencyToVote = U128CurrencyToVote;
    type ElectionLookahead = ElectionLookahead;
    // FIXME
    // type ElectionProvider = ElectionProviderMultiPhase;
    // type ElectionProvider = GeneralCouncilProvider;
    type Event = Event;
    type MaxIterations = MaxIterations;
    type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
    type MinSolutionScoreBump = MinSolutionScoreBump;
    type NextNewSession = Session;
    // The unsigned solution weight targeted by the OCW. We set it to the maximum possible value of
    // a single extrinsic.
    type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
    // send the slashed funds to the treasury.
    type Reward = ();
    type RewardCurve = RewardCurve;
    type RewardRemainder = Treasury;
    type SessionInterface = Self;
    // rewards are minted from the void
    type SessionsPerEra = SessionsPerEra;
    type Slash = Treasury;
    /// A super-majority of the council can cancel the slash.
    type SlashCancelOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>,
    >;
    type SlashDeferDuration = SlashDeferDuration;
    type UnixTime = Timestamp;
    type UnsignedPriority = StakingUnsignedPriority;
    type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
}

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
}

impl roaming_billing_policies::Config for Runtime {
    type Event = Event;
    type RoamingBillingPolicyIndex = u64;
}

impl roaming_charging_policies::Config for Runtime {
    type Event = Event;
    type RoamingChargingPolicyIndex = u64;
}

impl roaming_packet_bundles::Config for Runtime {
    type Event = Event;
    type RoamingPacketBundleExternalDataStorageHash = Hash;
    type RoamingPacketBundleIndex = u64;
    type RoamingPacketBundleReceivedAtHome = bool;
    type RoamingPacketBundleReceivedPacketsCount = u64;
    type RoamingPacketBundleReceivedPacketsOkCount = u64;
}

impl mining_config_token::Config for Runtime {
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

impl mining_config_hardware::Config for Runtime {
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

impl mining_rates_token::Config for Runtime {
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

impl mining_rates_hardware::Config for Runtime {
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

impl mining_sampling_token::Config for Runtime {
    type Event = Event;
    type MiningSamplingTokenIndex = u64;
    type MiningSamplingTokenSampleLockedAmount = u64;
}

impl mining_sampling_hardware::Config for Runtime {
    type Event = Event;
    type MiningSamplingHardwareIndex = u64;
    type MiningSamplingHardwareSampleHardwareOnline = u64;
}

impl mining_eligibility_token::Config for Runtime {
    type Event = Event;
    type MiningEligibilityTokenCalculatedEligibility = u64;
    type MiningEligibilityTokenIndex = u64;
    type MiningEligibilityTokenLockedPercentage = u32;
    // type MiningEligibilityTokenAuditorAccountID = u64;
}

impl mining_eligibility_hardware::Config for Runtime {
    type Event = Event;
    type MiningEligibilityHardwareCalculatedEligibility = u64;
    type MiningEligibilityHardwareIndex = u64;
    type MiningEligibilityHardwareUptimePercentage = u32;
    // type MiningEligibilityHardwareAuditorAccountID = u64;
}

impl mining_claims_token::Config for Runtime {
    type Event = Event;
    type MiningClaimsTokenClaimAmount = u64;
    type MiningClaimsTokenIndex = u64;
}

impl mining_claims_hardware::Config for Runtime {
    type Event = Event;
    type MiningClaimsHardwareClaimAmount = u64;
    type MiningClaimsHardwareIndex = u64;
}

impl mining_execution_token::Config for Runtime {
    type Event = Event;
    type MiningExecutionTokenIndex = u64;
}

impl exchange_rate::Config for Runtime {
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
        AuthorityDiscovery: pallet_authority_discovery::{Module, Call, Config},
        // Offences: pallet_offences::{Module, Call, Storage, Event},
        Historical: pallet_session_historical::{Module},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Babe: pallet_babe::{Module, Call, Storage, Config, ValidateUnsigned},
        Grandpa: pallet_grandpa::{Module, Call, Storage, Config, Event, ValidateUnsigned},
        Indices: pallet_indices::{Module, Call, Storage, Config<T>, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Module, Storage},
        // ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Module, Call, Storage, Event<T>, ValidateUnsigned},
        Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
        Council: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        TechnicalMembership: pallet_membership::<Instance1>::{Module, Call, Storage, Event<T>, Config<T>},
        TechnicalCommittee: pallet_collective::<Instance2>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
        // Bounties: pallet_bounties::{Module, Call, Storage, Config, Event<T>},
        // Tips: pallet_tips::{Module, Call, Storage, Config, Event<T>},
        Treasury: pallet_treasury::{Module, Call, Storage, Config, Event<T>},
        Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
        Staking: pallet_staking::{Module, Call, Config<T>, Storage, Event<T>, ValidateUnsigned},
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
        MiningClaimsToken: mining_claims_token::{Module, Call, Storage, Event<T>},
        MiningClaimsHardware: mining_claims_hardware::{Module, Call, Storage, Event<T>},
        MiningExecutionToken: mining_execution_token::{Module, Call, Storage, Event<T>},
        ExchangeRate: exchange_rate::{Module, Call, Storage, Event<T>},
    }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
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
            let key_owner_proof = _key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                _equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            use codec::Encode;

            Historical::prove((fg_primitives::KEY_TYPE, _authority_id))
                .map(|p| p.encode())
                .map(fg_primitives::OpaqueKeyOwnershipProof::new)
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

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Babe::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
        }
    }

    impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
        fn authorities() -> Vec<AuthorityDiscoveryId> {
            AuthorityDiscovery::authorities()
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
        Block,
        Balance,
    > for Runtime {
        fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade() -> Result<(Weight, Weight), sp_runtime::RuntimeString> {
            frame_support::debug::RuntimeLogger::init();
            let weight = Executive::try_runtime_upgrade()?;
            Ok((weight, RuntimeBlockWeights::get().max_block))
        }
    }
}
