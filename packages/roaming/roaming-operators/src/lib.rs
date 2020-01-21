#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use runtime_io::hashing::{blake2_128};
use sr_primitives::traits::{Bounded, Member, One, SimpleArithmetic};
use support::traits::{Currency, ExistenceRequirement, Randomness};
/// A runtime module for managing non-fungible tokens
use support::{decl_event, decl_module, decl_storage, ensure, Parameter};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type RoamingOperatorIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    type Currency: Currency<Self::AccountId>;
    type Randomness: Randomness<Self::Hash>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingOperator(pub [u8; 16]);

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
        <T as Trait>::RoamingOperatorIndex,
		Balance = BalanceOf<T>,
	{
		/// A roaming operator is created. (owner, roaming_operator_id)
		Created(AccountId, RoamingOperatorIndex),
		/// A roaming operator is transferred. (from, to, roaming_operator_id)
		Transferred(AccountId, AccountId, RoamingOperatorIndex),
		/// A roaming operator is available for sale. (owner, roaming_operator_id, price)
		PriceSet(AccountId, RoamingOperatorIndex, Option<Balance>),
		/// A roaming operator is sold. (from, to, roaming_operator_id, price)
		Sold(AccountId, AccountId, RoamingOperatorIndex, Balance),
	}
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingOperators {
        /// Stores all the roaming operators, key is the roaming operator id / index
        pub RoamingOperators get(fn roaming_operator): map T::RoamingOperatorIndex => Option<RoamingOperator>;

        /// Stores the total number of roaming operators. i.e. the next roaming operator index
        pub RoamingOperatorsCount get(fn roaming_operators_count): T::RoamingOperatorIndex;

        /// Get roaming operator owner
        pub RoamingOperatorOwners get(fn roaming_operator_owner): map T::RoamingOperatorIndex => Option<T::AccountId>;

        /// Get roaming operator price. None means not for sale.
        pub RoamingOperatorPrices get(fn roaming_operator_price): map T::RoamingOperatorIndex => Option<BalanceOf<T>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming operator
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_operator_id = Self::next_roaming_operator_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming operator
            let roaming_operator = RoamingOperator(unique_id);
            Self::insert_roaming_operator(&sender, roaming_operator_id, roaming_operator);

            Self::deposit_event(RawEvent::Created(sender, roaming_operator_id));
        }

        /// Transfer a roaming operator to new owner
        pub fn transfer(origin, to: T::AccountId, roaming_operator_id: T::RoamingOperatorIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_operator_owner(roaming_operator_id) == Some(sender.clone()), "Only owner can transfer roaming operator");

            Self::update_owner(&to, roaming_operator_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_operator_id));
        }

        /// Set a price for a roaming operator for sale
        /// None to delist the roaming operator
        pub fn set_price(origin, roaming_operator_id: T::RoamingOperatorIndex, price: Option<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_operator_owner(roaming_operator_id) == Some(sender.clone()), "Only owner can set price for roaming operator");

            if let Some(ref price) = price {
                <RoamingOperatorPrices<T>>::insert(roaming_operator_id, price);
            } else {
                <RoamingOperatorPrices<T>>::remove(roaming_operator_id);
            }

            Self::deposit_event(RawEvent::PriceSet(sender, roaming_operator_id, price));
        }

        /// Buy a roaming operator with max price willing to pay
        pub fn buy(origin, roaming_operator_id: T::RoamingOperatorIndex, price: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            let owner = Self::roaming_operator_owner(roaming_operator_id);
            ensure!(owner.is_some(), "RoamingOperator owner does not exist");
            let owner = owner.unwrap();

            let roaming_operator_price = Self::roaming_operator_price(roaming_operator_id);
            ensure!(roaming_operator_price.is_some(), "RoamingOperator not for sale");

            let roaming_operator_price = roaming_operator_price.unwrap();
            ensure!(price >= roaming_operator_price, "Price is too low");

            T::Currency::transfer(&sender, &owner, roaming_operator_price, ExistenceRequirement::AllowDeath)?;

            <RoamingOperatorPrices<T>>::remove(roaming_operator_id);

            Self::update_owner(&sender, roaming_operator_id);

            Self::deposit_event(RawEvent::Sold(owner, sender, roaming_operator_id, roaming_operator_price));
        }
    }
}

impl<T: Trait> Module<T> {
	pub fn is_roaming_operator_owner(roaming_operator_id: T::RoamingOperatorIndex, sender: T::AccountId) -> Result<(), &'static str> {
        ensure!(
            Self::roaming_operator_owner(&roaming_operator_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of RoamingOperator"
        );
        Ok(())
    }

    pub fn exists_roaming_operator(roaming_operator_id: T::RoamingOperatorIndex) -> Result<RoamingOperator, &'static str> {
        match Self::roaming_operator(roaming_operator_id) {
            Some(roaming_operator) => Ok(roaming_operator),
            None => Err("RoamingOperator does not exist")
        }
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <system::Module<T>>::extrinsic_index(),
            <system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_roaming_operator_id() -> Result<T::RoamingOperatorIndex, &'static str> {
        let roaming_operator_id = Self::roaming_operators_count();
        if roaming_operator_id == <T::RoamingOperatorIndex as Bounded>::max_value() {
            return Err("RoamingOperators count overflow");
        }
        Ok(roaming_operator_id)
    }

    fn insert_roaming_operator(owner: &T::AccountId, roaming_operator_id: T::RoamingOperatorIndex, roaming_operator: RoamingOperator) {
        // Create and store roaming operator
        <RoamingOperators<T>>::insert(roaming_operator_id, roaming_operator);
        <RoamingOperatorsCount<T>>::put(roaming_operator_id + One::one());
        <RoamingOperatorOwners<T>>::insert(roaming_operator_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_operator_id: T::RoamingOperatorIndex) {
        <RoamingOperatorOwners<T>>::insert(roaming_operator_id, to);
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{H256};
    use sr_primitives::Perbill;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{assert_noop, assert_ok, impl_outer_origin, parameter_types, weights::Weight};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ();
        type TransferFee = ();
        type CreationFee = ();
    }
    impl transaction_payment::Trait for Test {
        type Currency = Balances;
        type OnTransactionPayment = ();
        type TransactionBaseFee = ();
        type TransactionByteFee = ();
        type WeightToFee = ();
        type FeeMultiplierUpdate = ();
    }
    impl Trait for Test {
        type Event = ();
        type Currency = Balances;
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    //type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type RoamingOperatorModule = Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        runtime_io::TestExternalities::new(t)
    }

    #[test]
    fn basic_setup_works() {
        new_test_ext().execute_with(|| {
            // Verify Initial Storage
            assert_eq!(RoamingOperatorModule::roaming_operators_count(), 0);
            assert!(RoamingOperatorModule::roaming_operator(0).is_none());
            assert_eq!(RoamingOperatorModule::roaming_operator_owner(0), None);
            assert_eq!(RoamingOperatorModule::roaming_operator_price(0), None);
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
        });
    }

    #[test]
    fn create_works() {
        new_test_ext().execute_with(|| {
            // Call Functions
            assert_ok!(RoamingOperatorModule::create(Origin::signed(1)));
            // Verify Storage
            assert_eq!(RoamingOperatorModule::roaming_operators_count(), 1);
            assert!(RoamingOperatorModule::roaming_operator(0).is_some());
            assert_eq!(RoamingOperatorModule::roaming_operator_owner(0), Some(1));
            assert_eq!(RoamingOperatorModule::roaming_operator_price(0), None);
        });
    }

    #[test]
    fn create_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            <RoamingOperatorsCount<Test>>::put(u64::max_value());
            // Call Functions
            assert_noop!(
                RoamingOperatorModule::create(Origin::signed(1)),
                "RoamingOperators count overflow"
            );
            // Verify Storage
            assert_eq!(RoamingOperatorModule::roaming_operators_count(), u64::max_value());
            assert!(RoamingOperatorModule::roaming_operator(0).is_none());
            assert_eq!(RoamingOperatorModule::roaming_operator_owner(0), None);
            assert_eq!(RoamingOperatorModule::roaming_operator_price(0), None);
        });
    }

    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingOperatorModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingOperatorModule::transfer(Origin::signed(1), 2, 0));
            // Verify Storage
            assert_eq!(RoamingOperatorModule::roaming_operators_count(), 1);
            assert!(RoamingOperatorModule::roaming_operator(0).is_some());
            assert_eq!(RoamingOperatorModule::roaming_operator_owner(0), Some(2));
            assert_eq!(RoamingOperatorModule::roaming_operator_price(0), None);
        });
    }

    #[test]
    fn transfer_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingOperatorModule::create(Origin::signed(1)));
            // Call Functions
            assert_noop!(
                RoamingOperatorModule::transfer(Origin::signed(2), 2, 0),
                "Only owner can transfer roaming operator"
            );
            assert_noop!(
                RoamingOperatorModule::transfer(Origin::signed(1), 2, 1),
                "Only owner can transfer roaming operator"
            );
            // Verify Storage
            assert_eq!(RoamingOperatorModule::roaming_operators_count(), 1);
            assert!(RoamingOperatorModule::roaming_operator(0).is_some());
            assert_eq!(RoamingOperatorModule::roaming_operator_owner(0), Some(1));
            assert_eq!(RoamingOperatorModule::roaming_operator_price(0), None);
        });
    }

    #[test]
    fn set_price_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingOperatorModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingOperatorModule::set_price(Origin::signed(1), 0, Some(10)));
            // Verify Storage
            assert_eq!(RoamingOperatorModule::roaming_operators_count(), 1);
            assert!(RoamingOperatorModule::roaming_operator(0).is_some());
            assert_eq!(RoamingOperatorModule::roaming_operator_owner(0), Some(1));
            assert_eq!(RoamingOperatorModule::roaming_operator_price(0), Some(10));
        });
    }

    #[test]
    fn buy_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingOperatorModule::create(Origin::signed(1)));
            assert_ok!(RoamingOperatorModule::set_price(Origin::signed(1), 0, Some(10)));
            // Call Functions
            assert_ok!(RoamingOperatorModule::buy(Origin::signed(2), 0, 10));
            // Verify Storage
            assert_eq!(RoamingOperatorModule::roaming_operators_count(), 1);
            assert!(RoamingOperatorModule::roaming_operator(0).is_some());
            assert_eq!(RoamingOperatorModule::roaming_operator_owner(0), Some(2));
            assert_eq!(RoamingOperatorModule::roaming_operator_price(0), None);
            assert_eq!(Balances::free_balance(1), 20);
            assert_eq!(Balances::free_balance(2), 10);
        });
    }
}
