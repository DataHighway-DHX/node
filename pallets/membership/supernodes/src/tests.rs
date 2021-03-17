use crate::*;
use frame_support::{
    assert_noop,
    assert_ok,
    impl_outer_event,
    impl_outer_origin,
    parameter_types,
};
use frame_system as system;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{
        BlakeTwo256,
        IdentityLookup,
    },
    Perbill,
};

impl_outer_origin! {
    pub enum Origin for TestRuntime {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRuntime;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl system::Trait for TestRuntime {
    type AccountData = ();
    type AccountId = u64;
    type AvailableBlockRatio = AvailableBlockRatio;
    type BaseCallFilter = ();
    type BlockExecutionWeight = ();
    type BlockHashCount = BlockHashCount;
    type BlockNumber = u64;
    type Call = ();
    type DbWeight = ();
    type Event = TestEvent;
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

mod vec_set {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum TestEvent for TestRuntime {
        vec_set<T>,
        system<T>,
    }
}

impl Trait for TestRuntime {
    type Event = TestEvent;
}

pub type System = system::Module<TestRuntime>;
pub type VecSet = Module<TestRuntime>;

struct ExternalityBuilder;

impl ExternalityBuilder {
    pub fn build() -> TestExternalities {
        let storage = system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();
        let mut ext = TestExternalities::from(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

#[test]
fn add_member_works() {
    ExternalityBuilder::build().execute_with(|| {
        assert_ok!(VecSet::add_member(Origin::signed(1), 2));

        let expected_event = TestEvent::vec_set(RawEvent::MemberAdded(2));

        assert_eq!(System::events()[0].event, expected_event,);

        assert_eq!(VecSet::members(), vec![2]);
    })
}

#[test]
fn cant_add_duplicate_members() {
    ExternalityBuilder::build().execute_with(|| {
        assert_ok!(VecSet::add_member(Origin::signed(1), 2));

        assert_noop!(VecSet::add_member(Origin::signed(1), 2), Error::<TestRuntime>::AlreadyMember);
    })
}

#[test]
fn cant_exceed_max_members() {
    ExternalityBuilder::build().execute_with(|| {
        // Add 16 members, reaching the max
        for i in 0..16 {
            assert_ok!(VecSet::add_member(Origin::signed(i), i));
        }

        // Try to add the 17th member exceeding the max
        assert_noop!(VecSet::add_member(Origin::signed(16), 16), Error::<TestRuntime>::MembershipLimitReached);
    })
}

#[test]
fn remove_member_works() {
    ExternalityBuilder::build().execute_with(|| {
        assert_ok!(VecSet::add_member(Origin::signed(1), 2));
        assert_ok!(VecSet::remove_member(Origin::signed(1), 2));

        // check correct event emission
        let expected_event = TestEvent::vec_set(RawEvent::MemberRemoved(2));
        assert!(System::events().iter().any(|a| a.event == expected_event));

        // check storage changes
        assert_eq!(VecSet::members(), Vec::<u64>::new());
    })
}

#[test]
fn remove_member_handles_errors() {
    ExternalityBuilder::build().execute_with(|| {
        // 2 is NOT previously added as a member
        assert_noop!(VecSet::remove_member(Origin::signed(2), 2), Error::<TestRuntime>::NotMember);
    })
}
