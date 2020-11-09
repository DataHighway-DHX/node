// Creating mock runtime here

use crate::{
    Module,
    Trait,
};

use frame_support::{
    impl_outer_origin,
    parameter_types,
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

impl_outer_origin! {
    pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl frame_system::Trait for Test {
    type AccountData = pallet_balances::AccountData<u64>;
    type AccountId = u64;
    type AvailableBlockRatio = AvailableBlockRatio;
    type BaseCallFilter = ();
    type BlockExecutionWeight = ();
    type BlockHashCount = BlockHashCount;
    type BlockNumber = u64;
    type Call = ();
    type DbWeight = ();
    // type WeightMultiplierUpdate = ();
    type Event = ();
    type ExtrinsicBaseWeight = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaximumBlockLength = MaximumBlockLength;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type Origin = Origin;
    type PalletInfo = ();
    type SystemWeightInfo = ();
    type Version = ();
}
parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Trait for Test {
    type AccountStore = System;
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = ();
    type WeightInfo = ();
}
impl pallet_transaction_payment::Trait for Test {
    type Currency = Balances;
    type FeeMultiplierUpdate = ();
    type OnTransactionPayment = ();
    type TransactionByteFee = ();
    type WeightToFee = IdentityFee<u64>;
}
impl roaming_operators::Trait for Test {
    type Currency = Balances;
    type Event = ();
    type Randomness = Randomness;
    type RoamingOperatorIndex = u64;
}
impl roaming_networks::Trait for Test {
    type Event = ();
    type RoamingNetworkIndex = u64;
}
impl roaming_network_servers::Trait for Test {
    type Event = ();
    type RoamingNetworkServerIndex = u64;
}
impl roaming_organizations::Trait for Test {
    type Event = ();
    type RoamingOrganizationIndex = u64;
}
impl roaming_devices::Trait for Test {
    type Event = ();
    type RoamingDeviceIndex = u64;
}
impl roaming_sessions::Trait for Test {
    type Event = ();
    type RoamingSessionIndex = u64;
    type RoamingSessionJoinRequestAcceptAcceptedAt = u64;
    type RoamingSessionJoinRequestAcceptExpiry = u64;
    type RoamingSessionJoinRequestRequestedAt = u64;
}
impl Trait for Test {
    type Event = ();
    type RoamingPacketBundleExternalDataStorageHash = H256;
    type RoamingPacketBundleIndex = u64;
    type RoamingPacketBundleReceivedAtHome = bool;
    type RoamingPacketBundleReceivedEndedAt = u64;
    type RoamingPacketBundleReceivedPacketsCount = u64;
    type RoamingPacketBundleReceivedPacketsOkCount = u64;
    type RoamingPacketBundleReceivedStartedAt = u64;
}
type System = frame_system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
pub type RoamingPacketBundleModule = Module<Test>;
type Randomness = pallet_randomness_collective_flip::Module<Test>;

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
