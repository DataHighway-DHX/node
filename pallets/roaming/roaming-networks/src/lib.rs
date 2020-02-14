#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_io::hashing::{blake2_128};
use sp_runtime::traits::{Bounded, Member, One, SimpleArithmetic};
use frame_support::traits::{Currency, ExistenceRequirement, Randomness};
/// A runtime module for managing non-fungible tokens
use frame_support::{decl_event, decl_module, decl_storage, ensure, Parameter, debug};
use system::ensure_signed;
use sp_std::prelude::*; // Imports Vec

use roaming_operators;

/// The module's configuration trait.
pub trait Trait: system::Trait + roaming_operators::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type RoamingNetworkIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
}

type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingNetwork(pub [u8; 16]);

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
		<T as Trait>::RoamingNetworkIndex,
		<T as roaming_operators::Trait>::RoamingOperatorIndex,
		Balance = BalanceOf<T>,
	{
		/// A roaming network is created. (owner, roaming_network_id)
		Created(AccountId, RoamingNetworkIndex),
		/// A roaming network is transferred. (from, to, roaming_network_id)
		Transferred(AccountId, AccountId, RoamingNetworkIndex),
		/// A roaming network is available for sale. (owner, roaming_network_id, price)
		PriceSet(AccountId, RoamingNetworkIndex, Option<Balance>),
		/// A roaming network is sold. (from, to, roaming_network_id, price)
		Sold(AccountId, AccountId, RoamingNetworkIndex, Balance),
		/// A roaming network is assigned to an operator. (owner of operator, roaming_network_id, roaming_operator_id)
		AssignedNetworkToOperator(AccountId, RoamingNetworkIndex, RoamingOperatorIndex),
	}
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingNetworks {
        /// Stores all the roaming networks, key is the roaming network id / index
        pub RoamingNetworks get(fn roaming_network): map hasher(blake2_256) T::RoamingNetworkIndex => Option<RoamingNetwork>;

		/// Stores the total number of roaming networks. i.e. the next roaming network index
        pub RoamingNetworksCount get(fn roaming_networks_count): T::RoamingNetworkIndex;

        /// Get roaming network owner
        pub RoamingNetworkOwners get(fn roaming_network_owner): map hasher(blake2_256) T::RoamingNetworkIndex => Option<T::AccountId>;

        /// Get roaming network price. None means not for sale.
        pub RoamingNetworkPrices get(fn roaming_network_price): map hasher(blake2_256) T::RoamingNetworkIndex => Option<BalanceOf<T>>;

        /// Get roaming operator belonging to a network
        pub RoamingNetworkOperator get(fn roaming_network_operator): map hasher(blake2_256) T::RoamingNetworkIndex => Option<T::RoamingOperatorIndex>;

        /// Get roaming operator networks
        pub RoamingOperatorNetworks get(fn roaming_operator_networks): map hasher(blake2_256) T::RoamingOperatorIndex => Option<Vec<T::RoamingNetworkIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming network
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_network_id = Self::next_roaming_network_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming network
            let roaming_network = RoamingNetwork(unique_id);
            Self::insert_roaming_network(&sender, roaming_network_id, roaming_network);

            Self::deposit_event(RawEvent::Created(sender, roaming_network_id));
        }

        /// Transfer a roaming network to new owner
        pub fn transfer(origin, to: T::AccountId, roaming_network_id: T::RoamingNetworkIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_network_owner(roaming_network_id) == Some(sender.clone()), "Only owner can transfer roaming network");

            Self::update_owner(&to, roaming_network_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_network_id));
        }

        /// Set a price for a roaming network for sale
        /// None to delist the roaming network
        pub fn set_price(origin, roaming_network_id: T::RoamingNetworkIndex, price: Option<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_network_owner(roaming_network_id) == Some(sender.clone()), "Only owner can set price for roaming network");

            if let Some(ref price) = price {
                <RoamingNetworkPrices<T>>::insert(roaming_network_id, price);
            } else {
                <RoamingNetworkPrices<T>>::remove(roaming_network_id);
            }

            Self::deposit_event(RawEvent::PriceSet(sender, roaming_network_id, price));
        }

        /// Buy a roaming network with max price willing to pay
        pub fn buy(origin, roaming_network_id: T::RoamingNetworkIndex, price: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            let owner = Self::roaming_network_owner(roaming_network_id);
            ensure!(owner.is_some(), "RoamingNetwork does not exist");
            let owner = owner.unwrap();

            let roaming_network_price = Self::roaming_network_price(roaming_network_id);
            ensure!(roaming_network_price.is_some(), "RoamingNetwork not for sale");

            let roaming_network_price = roaming_network_price.unwrap();
            ensure!(price >= roaming_network_price, "Price is too low");

            T::Currency::transfer(&sender, &owner, roaming_network_price, ExistenceRequirement::AllowDeath)?;

            <RoamingNetworkPrices<T>>::remove(roaming_network_id);

            Self::update_owner(&sender, roaming_network_id);

            Self::deposit_event(RawEvent::Sold(owner, sender, roaming_network_id, roaming_network_price));
        }

        pub fn assign_network_to_operator(
          origin,
          roaming_network_id: T::RoamingNetworkIndex,
          roaming_operator_id: T::RoamingOperatorIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given operator id already exists
            let is_roaming_operator = <roaming_operators::Module<T>>
                ::exists_roaming_operator(roaming_operator_id).is_ok();
            ensure!(is_roaming_operator, "RoamingOperator does not exist");

            // Ensure that caller of the function is the owner of the operator id to assign the network to
            ensure!(
                <roaming_operators::Module<T>>::is_roaming_operator_owner(roaming_operator_id, sender.clone()).is_ok(),
                "Only the roaming operator owner can assign itself a roaming network"
            );

            Self::associate_network_with_operator(roaming_network_id, roaming_operator_id)
                .expect("Unable to associate network with operator");

            // Ensure that the given network id already exists
            let roaming_network = Self::roaming_network(roaming_network_id);
            ensure!(roaming_network.is_some(), "Invalid roaming_network_id");

            // Ensure that the network is not already owned by a different operator
            // Unassign the network from any existing operator since it may only be owned by one operator
            <RoamingNetworkOperator<T>>::remove(roaming_network_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <RoamingNetworkOperator<T>>::insert(roaming_network_id, roaming_operator_id);

            Self::deposit_event(RawEvent::AssignedNetworkToOperator(sender, roaming_network_id, roaming_operator_id));
		}
    }
}

impl<T: Trait> Module<T> {
	pub fn is_roaming_network_owner(roaming_network_id: T::RoamingNetworkIndex, sender: T::AccountId) -> Result<(), &'static str> {
        ensure!(
            Self::roaming_network_owner(&roaming_network_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of RoamingNetwork"
        );
        Ok(())
    }

	pub fn exists_roaming_network(roaming_network_id: T::RoamingNetworkIndex) -> Result<RoamingNetwork, &'static str> {
		match Self::roaming_network(roaming_network_id) {
			Some(roaming_network) => Ok(roaming_network),
			None => Err("RoamingNetwork does not exist")
		}
    }

    /// Only push the network id onto the end of the vector if it does not already exist
    pub fn associate_network_with_operator(
        roaming_network_id: T::RoamingNetworkIndex,
        roaming_operator_id: T::RoamingOperatorIndex
    ) -> Result<(), &'static str>
    {
        // Early exit with error since do not want to append if the given operator id already exists as a key,
        // and where its corresponding value is a vector that already contains the given network id
        if let Some(operator_networks) = Self::roaming_operator_networks(roaming_operator_id) {
            debug::info!("Operator id key {:?} exists with value {:?}", roaming_operator_id, operator_networks);
            let not_operator_contains_network = !operator_networks.contains(&roaming_network_id);
            ensure!(not_operator_contains_network, "Operator already contains the given network id");
            debug::info!("Operator id key exists but its vector value does not contain the given network id");
            <RoamingOperatorNetworks<T>>::mutate(roaming_operator_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_network_id);
                }
            });
            debug::info!("Associated network {:?} with operator {:?}", roaming_network_id, roaming_operator_id);
            Ok(())
        } else {
            debug::info!("Operator id key does not yet exist. Creating the operator key {:?} and appending the network id {:?} to its vector value", roaming_operator_id, roaming_network_id);
            <RoamingOperatorNetworks<T>>::insert(roaming_operator_id, &vec![roaming_network_id]);
            Ok(())
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

    fn next_roaming_network_id() -> Result<T::RoamingNetworkIndex, &'static str> {
        let roaming_network_id = Self::roaming_networks_count();
        if roaming_network_id == <T::RoamingNetworkIndex as Bounded>::max_value() {
            return Err("RoamingNetworks count overflow");
        }
        Ok(roaming_network_id)
    }

    fn insert_roaming_network(owner: &T::AccountId, roaming_network_id: T::RoamingNetworkIndex, roaming_network: RoamingNetwork) {
        // Create and store roaming network
        <RoamingNetworks<T>>::insert(roaming_network_id, roaming_network);
        <RoamingNetworksCount<T>>::put(roaming_network_id + One::one());
        <RoamingNetworkOwners<T>>::insert(roaming_network_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_network_id: T::RoamingNetworkIndex) {
        <RoamingNetworkOwners<T>>::insert(roaming_network_id, to);
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

	use sp_core::H256;
	use frame_support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
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
        type ModuleToIndex = ();
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ();
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
    impl roaming_operators::Trait for Test {
        type Event = ();
        type Currency = Balances;
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl Trait for Test {
        type Event = ();
        type RoamingNetworkIndex = u64;
    }
    //type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type RoamingNetworkModule = Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }

    #[test]
    fn basic_setup_works() {
        new_test_ext().execute_with(|| {
            // Verify Initial Storage
            assert_eq!(RoamingNetworkModule::roaming_networks_count(), 0);
            assert!(RoamingNetworkModule::roaming_network(0).is_none());
            assert_eq!(RoamingNetworkModule::roaming_network_owner(0), None);
            assert_eq!(RoamingNetworkModule::roaming_network_price(0), None);
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
        });
    }

    #[test]
    fn create_works() {
        new_test_ext().execute_with(|| {
            // Call Functions
            assert_ok!(RoamingNetworkModule::create(Origin::signed(1)));
            // Verify Storage
            assert_eq!(RoamingNetworkModule::roaming_networks_count(), 1);
            assert!(RoamingNetworkModule::roaming_network(0).is_some());
            assert_eq!(RoamingNetworkModule::roaming_network_owner(0), Some(1));
            assert_eq!(RoamingNetworkModule::roaming_network_price(0), None);
        });
    }

    #[test]
    fn create_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            <RoamingNetworksCount<Test>>::put(u64::max_value());
            // Call Functions
            assert_noop!(
                RoamingNetworkModule::create(Origin::signed(1)),
                "RoamingNetworks count overflow"
            );
            // Verify Storage
            assert_eq!(RoamingNetworkModule::roaming_networks_count(), u64::max_value());
            assert!(RoamingNetworkModule::roaming_network(0).is_none());
            assert_eq!(RoamingNetworkModule::roaming_network_owner(0), None);
            assert_eq!(RoamingNetworkModule::roaming_network_price(0), None);
        });
    }

    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingNetworkModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingNetworkModule::transfer(Origin::signed(1), 2, 0));
            // Verify Storage
            assert_eq!(RoamingNetworkModule::roaming_networks_count(), 1);
            assert!(RoamingNetworkModule::roaming_network(0).is_some());
            assert_eq!(RoamingNetworkModule::roaming_network_owner(0), Some(2));
            assert_eq!(RoamingNetworkModule::roaming_network_price(0), None);
        });
    }

    #[test]
    fn transfer_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingNetworkModule::create(Origin::signed(1)));
            // Call Functions
            assert_noop!(
                RoamingNetworkModule::transfer(Origin::signed(2), 2, 0),
                "Only owner can transfer roaming network"
            );
            assert_noop!(
                RoamingNetworkModule::transfer(Origin::signed(1), 2, 1),
                "Only owner can transfer roaming network"
            );
            // Verify Storage
            assert_eq!(RoamingNetworkModule::roaming_networks_count(), 1);
            assert!(RoamingNetworkModule::roaming_network(0).is_some());
            assert_eq!(RoamingNetworkModule::roaming_network_owner(0), Some(1));
            assert_eq!(RoamingNetworkModule::roaming_network_price(0), None);
        });
    }

    #[test]
    fn set_price_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingNetworkModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingNetworkModule::set_price(Origin::signed(1), 0, Some(10)));
            // Verify Storage
            assert_eq!(RoamingNetworkModule::roaming_networks_count(), 1);
            assert!(RoamingNetworkModule::roaming_network(0).is_some());
            assert_eq!(RoamingNetworkModule::roaming_network_owner(0), Some(1));
            assert_eq!(RoamingNetworkModule::roaming_network_price(0), Some(10));
        });
    }

    #[test]
    fn buy_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingNetworkModule::create(Origin::signed(1)));
            assert_ok!(RoamingNetworkModule::set_price(Origin::signed(1), 0, Some(10)));
            // Call Functions
            assert_ok!(RoamingNetworkModule::buy(Origin::signed(2), 0, 10));
            // Verify Storage
            assert_eq!(RoamingNetworkModule::roaming_networks_count(), 1);
            assert!(RoamingNetworkModule::roaming_network(0).is_some());
            assert_eq!(RoamingNetworkModule::roaming_network_owner(0), Some(2));
            assert_eq!(RoamingNetworkModule::roaming_network_price(0), None);
            assert_eq!(Balances::free_balance(1), 20);
            assert_eq!(Balances::free_balance(2), 10);
        });
    }
}
