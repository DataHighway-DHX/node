// Creating mock runtime here
use crate as mining_rewards_allowance;
use crate::{
    Config as MiningRewardsAllowanceConfig,
};
use frame_support::{
    parameter_types,
    traits::{
        ContainsLengthBound,
        LockIdentifier,
        SortedMembers,
    },
    weights::{
        IdentityFee,
        Weight,
    },
    PalletId,
};
use frame_system::{
    EnsureOneOf,
    EnsureRoot,
};
use sp_core::{
    H256,
    u32_trait::{
        _1,
        _2,
        _3,
        _4,
    },
};
use codec::{
    Decode,
    Encode,
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::{
    testing::Header,
    traits::{
        BlakeTwo256,
        IdentityLookup,
    },
    Perbill,
    Percent,
    Permill,
};
use std::cell::RefCell;
use static_assertions::const_assert;
pub use module_primitives::{
	constants::currency::{
        // CENTS, // Use override below
        deposit,
        // DOLLARS, // Use override below
        // MILLICENTS, // Use override below
    },
    constants::time::{
        // DAYS, // Use override below
        SLOT_DURATION,
        // MINUTES, // Use override below
    },
	types::{
        // AccountId, // Use override below
        // Balance, // Use override below
        // BlockNumber, // Use override below
        Moment,
    },
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Aura: pallet_aura::{Pallet, Config<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
        Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
        Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
        Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>},
        TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
        Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
        Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>},
        Tips: pallet_tips::{Pallet, Call, Storage, Event<T>},
        MiningRewardsAllowanceTestModule: mining_rewards_allowance::{Pallet, Call, Storage, Config<T>, Event<T>},
    }
);

// Override primitives
pub type AccountId = u128;
pub type Balance = u64;
pub type BlockNumber = u64;
// pub type SysEvent = frame_system::Event<Test>;

pub const MILLISECS_PER_BLOCK: Moment = 4320;
pub const MILLICENTS: Balance = 1_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
			frame_system::limits::BlockWeights::simple_max(2_000_000_000_000);
}
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId; // u64 is not enough to hold bytes used to generate bounty account
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}
impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Test {
    type Event = ();
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<u128>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = ();
}

impl pallet_aura::Config for Test {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}
parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}
impl pallet_timestamp::Config for Test {
    type MinimumPeriod = MinimumPeriod;
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = Moment;
    type OnTimestampSet = Aura;
    type WeightInfo = ();
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
	type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}
parameter_types! {
    pub const TransactionByteFee: u64 = 1;
}
impl pallet_transaction_payment::Config for Test {
    type FeeMultiplierUpdate = ();
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<u64>;
}

parameter_types! {
    pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
    type Origin = Origin;
    type Proposal = Call;
    type Event = ();
    type MotionDuration = CouncilMotionDuration;
    type MaxProposals = CouncilMaxProposals;
    type MaxMembers = CouncilMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Test>;
}

parameter_types! {
    pub const CandidacyBond: Balance = 10 * DOLLARS;
    // 1 storage item created, key size is 32 bytes, value size is 16+16.
    pub const VotingBondBase: Balance = 1;
    // additional data per vote is 32 bytes (account id).
    pub const VotingBondFactor: Balance = 1;
    pub const TermDuration: BlockNumber = 7 * DAYS;
    // Check chain_spec. This value should be greater than or equal to the amount of
    // endowed accounts that are added to election_phragmen
    pub const DesiredMembers: u32 = 62; // validators 1-10 + sudo + treasury
    pub const DesiredRunnersUp: u32 = 7;
    pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Test {
    type Event = ();
    type PalletId = ElectionsPhragmenPalletId;
    type Currency = Balances;
    type ChangeMembers = Council;
    // NOTE: this implies that council's genesis members cannot be set directly and must come from
    // this module.
    type InitializeMembers = Council;
    type CurrencyToVote = frame_support::traits::SaturatingCurrencyToVote;
    type CandidacyBond = CandidacyBond;
    type VotingBondBase = VotingBondBase;
    type VotingBondFactor = VotingBondFactor;
    type LoserCandidate = ();
    type KickedMember = ();
    type DesiredMembers = DesiredMembers;
    type DesiredRunnersUp = DesiredRunnersUp;
    type TermDuration = TermDuration;
    type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Test>;
}

parameter_types! {
    pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
    pub const TechnicalMaxProposals: u32 = 100;
    pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Test {
    type Origin = Origin;
    type Proposal = Call;
    type Event = ();
    type MotionDuration = TechnicalMotionDuration;
    type MaxProposals = TechnicalMaxProposals;
    type MaxMembers = TechnicalMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Test>;
}

type EnsureRootOrHalfCouncil = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;
impl pallet_membership::Config<pallet_membership::Instance1> for Test {
    type Event = ();
    type AddOrigin = EnsureRootOrHalfCouncil;
    type RemoveOrigin = EnsureRootOrHalfCouncil;
    type SwapOrigin = EnsureRootOrHalfCouncil;
    type ResetOrigin = EnsureRootOrHalfCouncil;
    type PrimeOrigin = EnsureRootOrHalfCouncil;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Test>;
}

thread_local! {
    static TEN_TO_FOURTEEN: RefCell<Vec<u128>> = RefCell::new(vec![10,11,12,13,14]);
}
pub struct TenToFourteen;
impl SortedMembers<u128> for TenToFourteen {
    fn sorted_members() -> Vec<u128> {
        TEN_TO_FOURTEEN.with(|v| v.borrow().clone())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn add(new: &u128) {
        TEN_TO_FOURTEEN.with(|v| {
            let mut members = v.borrow_mut();
            members.push(*new);
            members.sort();
        })
    }
}
impl ContainsLengthBound for TenToFourteen {
    fn max_len() -> usize {
        TEN_TO_FOURTEEN.with(|v| v.borrow().len())
    }

    fn min_len() -> usize {
        0
    }
}

parameter_types! {
    pub const ProposalBond: Permill = Permill::from_percent(5);
    pub const ProposalBondMinimum: u64 = 1_000_000_000_000_000_000;
    pub const SpendPeriod: BlockNumber = 1 * DAYS;
    pub const Burn: Permill = Permill::from_percent(0);
    pub const TipCountdown: BlockNumber = 1;
    pub const TipFindersFee: Percent = Percent::from_percent(20);
    pub const TipReportDepositBase: u64 = 1_000_000_000_000_000_000;
    pub const DataDepositPerByte: u64 = 1;
    pub const BountyDepositBase: u64 = 80;
    pub const BountyDepositPayoutDelay: u32 = 3;
    pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
    pub const BountyUpdatePeriod: u32 = 20;
    pub const MaximumReasonLength: u32 = 16384;
    pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
    pub const BountyValueMinimum: u64 = 1;
    pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Test {
    type PalletId = TreasuryPalletId;
    type Currency = Balances;
    type ApproveOrigin = EnsureRoot<u128>;
    type RejectOrigin = EnsureRoot<u128>;
    type Event = ();
    type OnSlash = ();
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type SpendPeriod = SpendPeriod;
    type Burn = Burn;
    type BurnDestination = ();
    type SpendFunds = Bounties;
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Test>;
    type MaxApprovals = MaxApprovals;
}

impl pallet_bounties::Config for Test {
    type Event = ();
    type BountyDepositBase = BountyDepositBase;
    type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
    type BountyUpdatePeriod = BountyUpdatePeriod;
    type BountyCuratorDeposit = BountyCuratorDeposit;
    type BountyValueMinimum = BountyValueMinimum;
    type DataDepositPerByte = DataDepositPerByte;
    type MaximumReasonLength = MaximumReasonLength;
    type WeightInfo = pallet_bounties::weights::SubstrateWeight<Test>;
}

impl pallet_tips::Config for Test {
    type Event = ();
    type DataDepositPerByte = DataDepositPerByte;
    type MaximumReasonLength = MaximumReasonLength;
    type Tippers = TenToFourteen;
    type TipCountdown = TipCountdown;
    type TipFindersFee = TipFindersFee;
    type TipReportDepositBase = TipReportDepositBase;
    type WeightInfo = pallet_tips::weights::SubstrateWeight<Test>;
}

parameter_types! {
    pub const LaunchPeriod: BlockNumber = 1 * MINUTES;
    pub const VotingPeriod: BlockNumber = 1 * MINUTES;
    pub const FastTrackVotingPeriod: BlockNumber = 1 * MINUTES;
    pub const InstantAllowed: bool = true;
    pub const MinimumDeposit: Balance = 1 * DOLLARS;
    pub const EnactmentPeriod: BlockNumber = 1 * MINUTES;
    pub const CooloffPeriod: BlockNumber = 1 * MINUTES;
    // One cent: $10,000 / MB
    pub const PreimageByteDeposit: Balance = 1 * CENTS;
    pub const MaxVotes: u32 = 100;
    pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Test {
    type Proposal = Call;
    type Event = ();
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type MinimumDeposit = MinimumDeposit;
    /// A straight majority of the council can decide what their next motion is.
    type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
    /// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
    /// A unanimous council can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
    /// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
    type InstantOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
    type InstantAllowed = InstantAllowed;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    // To cancel a proposal which has been passed, 2/3 of the council must agree to it.
    type CancellationOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
    // To cancel a proposal before it has been passed, the technical committee must be unanimous or
    // Root must agree.
    type CancelProposalOrigin = EnsureOneOf<
        AccountId,
        EnsureRoot<AccountId>,
        pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
    >;
    type BlacklistOrigin = EnsureRoot<AccountId>;
    // Any single technical committee member may veto a coming council proposal, however they can
    // only do it once and it lasts only for the cooloff period.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
    type CooloffPeriod = CooloffPeriod;
    type PreimageByteDeposit = PreimageByteDeposit;
    type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
    type Slash = Treasury;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = MaxVotes;
    type WeightInfo = ();
    type MaxProposals = MaxProposals;
}

impl MiningRewardsAllowanceConfig for Test {
    type Event = ();
    type Currency = Balances;
}

pub type SysEvent = frame_system::Event<Test>;
pub type DemocracyEvent = pallet_democracy::Event<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(0, 10), (1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
