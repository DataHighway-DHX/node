// use crate::*;
// use frame_support::{
//     assert_noop,
//     assert_ok,
//     impl_outer_event,
//     impl_outer_origin,
//     parameter_types,
// };
// use frame_system as system;
// use sp_core::H256;
// use sp_io::TestExternalities;
// use sp_runtime::{
//     testing::Header,
//     traits::{
//         BlakeTwo256,
//         IdentityLookup,
//     },
//     Perbill,
// };

// impl_outer_origin! {
//     pub enum Origin for TestRuntime {}
// }

// // Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct TestRuntime;
// parameter_types! {
//     pub const BlockHashCount: u64 = 250;
//     pub const SS58Prefix: u8 = 33;
// }
// impl frame_system::Config for TestRuntime {
//     type AccountData = pallet_balances::AccountData<u64>;
//     type AccountId = u64;
//     type BaseCallFilter = Everything;
//     type BlockHashCount = BlockHashCount;
//     type BlockLength = ();
//     type BlockNumber = u64;
//     type BlockWeights = ();
//     type Call = Call;
//     type DbWeight = ();
//     // type WeightMultiplierUpdate = ();
//     type Event = ();
//     type Hash = H256;
//     type Hashing = BlakeTwo256;
//     type Header = Header;
//     type Index = u64;
//     type Lookup = IdentityLookup<Self::AccountId>;
//     type OnKilledAccount = ();
//     type OnNewAccount = ();
//     type Origin = Origin;
//     type PalletInfo = PalletInfo;
//     type SS58Prefix = SS58Prefix;
//     type SystemWeightInfo = ();
//     type Version = ();
// }

// mod vec_set {
//     pub use crate::Event;
// }

// impl_outer_event! {
//     pub enum TestEvent for TestRuntime {
//         vec_set<T>,
//         system<T>,
//     }
// }

// impl Config for TestRuntime {
//     type Event = TestEvent;
// }

// pub type System = system::Module<TestRuntime>;
// pub type VecSet = Module<TestRuntime>;

// struct ExternalityBuilder;

// impl ExternalityBuilder {
//     pub fn build() -> TestExternalities {
//         let storage = system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();
//         let mut ext = TestExternalities::from(storage);
//         ext.execute_with(|| System::set_block_number(1));
//         ext
//     }
// }

// #[test]
// fn add_member_works() {
//     ExternalityBuilder::build().execute_with(|| {
//         assert_ok!(VecSet::add_member(Origin::root(), 2, 1));

//         let expected_event = TestEvent::vec_set(RawEvent::MemberAdded(2, 1));

//         assert_eq!(System::events()[0].event, expected_event,);

//         assert_eq!(VecSet::members(), vec![2]);
//         assert_eq!(VecSet::member_kinds(2), 1);
//     })
// }

// #[test]
// fn cant_add_duplicate_members() {
//     ExternalityBuilder::build().execute_with(|| {
//         assert_ok!(VecSet::add_member(Origin::root(), 2, 1));

//         assert_noop!(VecSet::add_member(Origin::root(), 2, 1), Error::<TestRuntime>::AlreadyMember);
//     })
// }

// #[test]
// fn cant_exceed_max_members() {
//     ExternalityBuilder::build().execute_with(|| {
//         // Add 16 members, reaching the max
//         for i in 0..16 {
//             assert_ok!(VecSet::add_member(Origin::root(), i, 2));
//         }

//         // Try to add the 17th member exceeding the max
//         assert_noop!(VecSet::add_member(Origin::root(), 16, 2), Error::<TestRuntime>::MembershipLimitReached);
//     })
// }

// #[test]
// fn remove_member_works() {
//     ExternalityBuilder::build().execute_with(|| {
//         assert_ok!(VecSet::add_member(Origin::root(), 2, 2));
//         assert_ok!(VecSet::remove_member(Origin::root(), 2, 2));

//         // check correct event emission
//         let expected_event = TestEvent::vec_set(RawEvent::MemberRemoved(2, 2));
//         assert!(System::events().iter().any(|a| a.event == expected_event));

//         // check storage changes
//         assert_eq!(VecSet::members(), Vec::<u64>::new());
//     })
// }

// #[test]
// fn remove_member_handles_errors() {
//     ExternalityBuilder::build().execute_with(|| {
//         // 2 is NOT previously added as a member
//         assert_noop!(VecSet::remove_member(Origin::root(), 2, 2), Error::<TestRuntime>::NotMember);
//     })
// }
