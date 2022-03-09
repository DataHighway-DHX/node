// Creating mock runtime here

use crate::{
    Pallet,
    Config,
};

use frame_support::{
    parameter_types,
    traits::{
        ConstU8,
        ConstU16,
        ConstU32,
        ConstU64,
        ConstU128,
    },
    weights::{
        IdentityFee,
        Weight,
    },
};

use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{
        BlakeTwo256,
        IdentityLookup,
    },
    Perbill,
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
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
    }
);

parameter_types! {
    pub const BlockHashCount: u32 = 250;
}
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u128; // u64 is not enough to hold bytes used to generate bounty account
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
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}
impl pallet_randomness_collective_flip::Config for Test {}
pub const ExistentialDepositAsConst: u64 = 1;
parameter_types! {
    pub const ExistentialDeposit: u64 = ExistentialDepositAsConst;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ConstU64<ExistentialDepositAsConst>;
    type AccountStore = System;
    type WeightInfo = ();
}
pub const OperationalFeeMultiplierAsConst: u8 = 5;
parameter_types! {
    pub const TransactionByteFee: u64 = 1;
    pub OperationalFeeMultiplier: u8 = OperationalFeeMultiplierAsConst;
}
impl pallet_transaction_payment::Config for Test {
    type FeeMultiplierUpdate = ();
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type OperationalFeeMultiplier = ConstU8<OperationalFeeMultiplierAsConst>;
    type WeightToFee = IdentityFee<u64>;
}
// FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
impl roaming_operators::Config for Test {
    type Currency = Balances;
    type Event = ();
    type Randomness = RandomnessCollectiveFlip;
    type RoamingOperatorIndex = u64;
}
impl mining_setting_token::Config for Test {
    type Event = ();
    // FIXME - restore when stop temporarily using roaming-operators
    // type Currency = Balances;
    // type Randomness = RandomnessCollectiveFlip;
    type MiningSettingTokenIndex = u64;
    type MiningSettingTokenLockAmount = u64;
    // Mining Speed Boost Token Mining Config
    // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
    type MiningSettingTokenType = Vec<u8>;
}
impl mining_eligibility_token::Config for Test {
    type Event = ();
    type MiningEligibilityTokenCalculatedEligibility = u64;
    type MiningEligibilityTokenIndex = u64;
    type MiningEligibilityTokenLockedPercentage = u32;
    // type MiningEligibilityTokenAuditorAccountID = u64;
}
impl mining_rates_token::Config for Test {
    type Event = ();
    type MiningRatesTokenIndex = u64;
    type MiningRatesTokenMaxLoyalty = u32;
    // Mining Speed Boost Max Rates
    type MiningRatesTokenMaxToken = u32;
    type MiningRatesTokenTokenDOT = u32;
    type MiningRatesTokenTokenIOTA = u32;
    // Mining Speed Boost Rate
    type MiningRatesTokenTokenMXC = u32;
}
impl mining_sampling_token::Config for Test {
    type Event = ();
    type MiningSamplingTokenIndex = u64;
    type MiningSamplingTokenSampleLockedAmount = u64;
}
impl Config for Test {
    type Event = ();
    type MiningClaimsTokenClaimAmount = u64;
    type MiningClaimsTokenIndex = u64;
}

pub type MiningClaimsTokenTestModule = Pallet<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
