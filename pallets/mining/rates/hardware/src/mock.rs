// Creating mock runtime here

use crate::{
    Module,
    Config,
};

use frame_support::{
    parameter_types, traits::Everything,
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
    pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
    type AccountData = pallet_balances::AccountData<u64>;
    type AccountId = u64;
    type BaseCallFilter = Everything;
    type BlockHashCount = BlockHashCount;
    type BlockNumber = u64;
    type BlockLength = ();
    type BlockWeights = ();
    type Call = Call;
    type DbWeight = ();
    type Event = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type AccountStore = System;
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}
parameter_types! {
    pub const OperationalFeeMultiplier: u8 = 5;
    pub const TransactionByteFee: u64 = 1;
}
impl pallet_transaction_payment::Config for Test {
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type FeeMultiplierUpdate = ();
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<u64>;
}

impl pallet_randomness_collective_flip::Config for Test {}

// FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
impl roaming_operators::Config for Test {
    type Currency = Balances;
    type Event = ();
    type Randomness = RandomnessCollectiveFlip;
    type RoamingOperatorIndex = u64;
}
impl Config for Test {
    type Event = ();
    type MiningRatesHardwareCategory1MaxTokenBonusPerGateway = u32;
    type MiningRatesHardwareCategory2MaxTokenBonusPerGateway = u32;
    type MiningRatesHardwareCategory3MaxTokenBonusPerGateway = u32;
    type MiningRatesHardwareIndex = u64;
    type MiningRatesHardwareInsecure = u32;
    type MiningRatesHardwareMaxHardware = u32;
    type MiningRatesHardwareSecure = u32;
}

pub type MiningRatesHardwareTestModule = Module<Test>;

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
