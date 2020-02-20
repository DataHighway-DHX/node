#![cfg_attr(not(feature = "std"), no_std)]

use codec::{
    Decode,
    Encode,
};
use frame_support::traits::{
    Currency,
    ExistenceRequirement,
    Randomness,
};
/// A runtime module for managing non-fungible tokens
use frame_support::{
    debug,
    decl_event,
    decl_module,
    decl_storage,
    ensure,
    Parameter,
};
use sp_io::hashing::blake2_128;
use sp_runtime::{
    traits::{
        AtLeast32Bit,
        Bounded,
        Member,
        One,
    },
    DispatchError,
};
use sp_std::prelude::*; // Imports Vec
use system::ensure_signed;

use roaming_networks;
use roaming_operators;

/// The module's configuration trait.
pub trait Trait: system::Trait + roaming_operators::Trait + roaming_networks::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type RoamingNetworkServerIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingNetworkServer(pub [u8; 16]);

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::RoamingNetworkServerIndex,
        <T as roaming_networks::Trait>::RoamingNetworkIndex,
        <T as roaming_operators::Trait>::RoamingOperatorIndex,
        Balance = BalanceOf<T>,
    {
        /// A roaming network_server is created. (owner, roaming_network_server_id)
        Created(AccountId, RoamingNetworkServerIndex),
        /// A roaming network_server is transferred. (from, to, roaming_network_server_id)
        Transferred(AccountId, AccountId, RoamingNetworkServerIndex),
        /// A roaming network_server is available for sale. (owner, roaming_network_server_id, price)
        PriceSet(AccountId, RoamingNetworkServerIndex, Option<Balance>),
        /// A roaming network_server is sold. (from, to, roaming_network_server_id, price)
        Sold(AccountId, AccountId, RoamingNetworkServerIndex, Balance),
        /// A roaming network_server is assigned to a network. (owner of network, roaming_network_server_id, roaming_network_id)
        AssignedNetworkServerToNetwork(AccountId, RoamingNetworkServerIndex, RoamingNetworkIndex),
        /// A roaming network_server is assigned to an operator. (owner of network, roaming_network_server_id, roaming_operator_id)
        AssignedNetworkServerToOperator(AccountId, RoamingNetworkServerIndex, RoamingOperatorIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingNetworkServers {
        /// Stores all the roaming network_servers, key is the roaming network_server id / index
        pub RoamingNetworkServers get(fn roaming_network_server): map hasher(blake2_256) T::RoamingNetworkServerIndex => Option<RoamingNetworkServer>;

        /// Stores the total number of roaming network_servers. i.e. the next roaming network_server index
        pub RoamingNetworkServersCount get(fn roaming_network_servers_count): T::RoamingNetworkServerIndex;

        /// Get roaming network_server owner
        pub RoamingNetworkServerOwners get(fn roaming_network_server_owner): map hasher(blake2_256) T::RoamingNetworkServerIndex => Option<T::AccountId>;

        /// Get roaming network_server price. None means not for sale.
        pub RoamingNetworkServerPrices get(fn roaming_network_server_price): map hasher(blake2_256) T::RoamingNetworkServerIndex => Option<BalanceOf<T>>;

        /// Get roaming network_server network
        pub RoamingNetworkServerNetwork get(fn roaming_network_server_network): map hasher(blake2_256) T::RoamingNetworkServerIndex => Option<T::RoamingNetworkIndex>;

        /// Get roaming network_server operators
        pub RoamingNetworkServerOperator get(fn roaming_network_server_operators): map hasher(blake2_256) T::RoamingNetworkServerIndex => Option<T::RoamingOperatorIndex>;

        /// Get roaming network's network servers
        pub RoamingNetworkNetworkServers get(fn roaming_network_network_servers): map hasher(blake2_256) T::RoamingNetworkIndex => Option<Vec<T::RoamingNetworkServerIndex>>;

        /// Get roaming operator's network servers
        pub RoamingOperatorNetworkServers get(fn roaming_operator_network_servers): map hasher(blake2_256) T::RoamingOperatorIndex => Option<Vec<T::RoamingNetworkServerIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming network_server
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_network_server_id = Self::next_roaming_network_server_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming network_server
            let roaming_network_server = RoamingNetworkServer(unique_id);
            Self::insert_roaming_network_server(&sender, roaming_network_server_id, roaming_network_server);

            Self::deposit_event(RawEvent::Created(sender, roaming_network_server_id));
        }

        /// Transfer a roaming network_server to new owner
        pub fn transfer(origin, to: T::AccountId, roaming_network_server_id: T::RoamingNetworkServerIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_network_server_owner(roaming_network_server_id) == Some(sender.clone()), "Only owner can transfer roaming network_server");

            Self::update_owner(&to, roaming_network_server_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_network_server_id));
        }

        /// Set a price for a roaming network_server for sale
        /// None to delist the roaming network_server
        pub fn set_price(origin, roaming_network_server_id: T::RoamingNetworkServerIndex, price: Option<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_network_server_owner(roaming_network_server_id) == Some(sender.clone()), "Only owner can set price for roaming network_server");

            if let Some(ref price) = price {
                <RoamingNetworkServerPrices<T>>::insert(roaming_network_server_id, price);
            } else {
                <RoamingNetworkServerPrices<T>>::remove(roaming_network_server_id);
            }

            Self::deposit_event(RawEvent::PriceSet(sender, roaming_network_server_id, price));
        }

        /// Buy a roaming network_server with max price willing to pay
        pub fn buy(origin, roaming_network_server_id: T::RoamingNetworkServerIndex, price: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            let owner = Self::roaming_network_server_owner(roaming_network_server_id);
            ensure!(owner.is_some(), "RoamingNetworkServer owner does not exist");
            let owner = owner.unwrap();

            let roaming_network_server_price = Self::roaming_network_server_price(roaming_network_server_id);
            ensure!(roaming_network_server_price.is_some(), "RoamingNetworkServer not for sale");

            let roaming_network_server_price = roaming_network_server_price.unwrap();
            ensure!(price >= roaming_network_server_price, "Price is too low");

            T::Currency::transfer(&sender, &owner, roaming_network_server_price, ExistenceRequirement::AllowDeath)?;

            <RoamingNetworkServerPrices<T>>::remove(roaming_network_server_id);

            Self::update_owner(&sender, roaming_network_server_id);

            Self::deposit_event(RawEvent::Sold(owner, sender, roaming_network_server_id, roaming_network_server_price));
        }

        pub fn assign_network_server_to_network(
            origin,
            roaming_network_server_id: T::RoamingNetworkServerIndex,
            roaming_network_id: T::RoamingNetworkIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_network = <roaming_networks::Module<T>>
                ::exists_roaming_network(roaming_network_id).is_ok();
            ensure!(is_roaming_network, "RoamingNetwork does not exist");

            // Ensure that caller of the function is the owner of the network id to assign the network_server to
            ensure!(
                <roaming_networks::Module<T>>::is_roaming_network_owner(roaming_network_id, sender.clone()).is_ok(),
                "Only the roaming network owner can assign itself a roaming network server"
            );

            Self::associate_network_server_with_network(roaming_network_server_id, roaming_network_id)
                .expect("Unable to associate network server with network");

            // Ensure that the given network_server id already exists
            let roaming_network_server = Self::roaming_network_server(roaming_network_server_id);
            ensure!(roaming_network_server.is_some(), "Invalid roaming_network_server_id");

            // Ensure that the network_server is not already owned by a different network
            // Unassign the network_server from any existing network since it may only be owned by one network
            <RoamingNetworkServerNetwork<T>>::remove(roaming_network_server_id);

            // Assign the network_server owner to the given network (even if already belongs to them)
            <RoamingNetworkServerNetwork<T>>::insert(roaming_network_server_id, roaming_network_id);

            Self::deposit_event(RawEvent::AssignedNetworkServerToNetwork(sender, roaming_network_server_id, roaming_network_id));
        }

        pub fn assign_network_server_to_operator(
            origin,
            roaming_network_server_id: T::RoamingNetworkServerIndex,
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
                "Only the roaming operator owner can assign itself a roaming network server"
            );

            Self::associate_network_server_with_operator(roaming_network_server_id, roaming_operator_id)
                .expect("Unable to associate network server with operator");

            // Ensure that the given network_server id already exists
            let roaming_network_server = Self::roaming_network_server(roaming_network_server_id);
            ensure!(roaming_network_server.is_some(), "Invalid roaming_network_server_id");

            // Ensure that the network_server is not already owned by a different operator
            // Unassign the network_server from any existing operator since it may only be owned by one operator
            <RoamingNetworkServerOperator<T>>::remove(roaming_network_server_id);

            // Assign the network_server owner to the given operator (even if already belongs to them)
            <RoamingNetworkServerOperator<T>>::insert(roaming_network_server_id, roaming_operator_id);

            Self::deposit_event(RawEvent::AssignedNetworkServerToOperator(sender, roaming_network_server_id, roaming_operator_id));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_roaming_network_server_owner(
        roaming_network_server_id: T::RoamingNetworkServerIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_network_server_owner(&roaming_network_server_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of RoamingNetworkServer"
        );
        Ok(())
    }

    pub fn exists_roaming_network_server(
        roaming_network_server_id: T::RoamingNetworkServerIndex,
    ) -> Result<RoamingNetworkServer, DispatchError> {
        match Self::roaming_network_server(roaming_network_server_id) {
            Some(roaming_network_server) => Ok(roaming_network_server),
            None => Err(DispatchError::Other("RoamingNetworkServer does not exist")),
        }
    }

    /// Only push the network server id onto the end of the vector if it does not already exist
    pub fn associate_network_server_with_network(
        roaming_network_server_id: T::RoamingNetworkServerIndex,
        roaming_network_id: T::RoamingNetworkIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given network id already exists as a key,
        // and where its corresponding value is a vector that already contains the given network server id
        if let Some(network_network_servers) = Self::roaming_network_network_servers(roaming_network_id) {
            debug::info!("Network id key {:?} exists with value {:?}", roaming_network_id, network_network_servers);
            let not_network_contains_network_server = !network_network_servers.contains(&roaming_network_server_id);
            ensure!(not_network_contains_network_server, "Network already contains the given network server id");
            debug::info!("Network id key exists but its vector value does not contain the given network server id");
            <RoamingNetworkNetworkServers<T>>::mutate(roaming_network_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_network_server_id);
                }
            });
            debug::info!(
                "Associated network server {:?} with network {:?}",
                roaming_network_server_id,
                roaming_network_id
            );
            Ok(())
        } else {
            debug::info!(
                "Network id key does not yet exist. Creating the network key {:?} and appending the network server id \
                 {:?} to its vector value",
                roaming_network_id,
                roaming_network_server_id
            );
            <RoamingNetworkNetworkServers<T>>::insert(roaming_network_id, &vec![roaming_network_server_id]);
            Ok(())
        }
    }

    /// Only push the network server id onto the end of the vector if it does not already exist
    pub fn associate_network_server_with_operator(
        roaming_network_server_id: T::RoamingNetworkServerIndex,
        roaming_operator_id: T::RoamingOperatorIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given operator id already exists as a key,
        // and where its corresponding value is a vector that already contains the given network server id
        if let Some(operator_network_servers) = Self::roaming_operator_network_servers(roaming_operator_id) {
            debug::info!("Operator id key {:?} exists with value {:?}", roaming_operator_id, operator_network_servers);
            let not_operator_contains_network_server = !operator_network_servers.contains(&roaming_network_server_id);
            ensure!(not_operator_contains_network_server, "Operator already contains the given network server id");
            debug::info!("Operator id key exists but its vector value does not contain the given network server id");
            <RoamingOperatorNetworkServers<T>>::mutate(roaming_operator_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_network_server_id);
                }
            });
            debug::info!(
                "Associated network server {:?} with operator {:?}",
                roaming_network_server_id,
                roaming_operator_id
            );
            Ok(())
        } else {
            debug::info!(
                "Operator id key does not yet exist. Creating the operator key {:?} and appending the network server \
                 id {:?} to its vector value",
                roaming_operator_id,
                roaming_network_server_id
            );
            <RoamingOperatorNetworkServers<T>>::insert(roaming_operator_id, &vec![roaming_network_server_id]);
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

    fn next_roaming_network_server_id() -> Result<T::RoamingNetworkServerIndex, DispatchError> {
        let roaming_network_server_id = Self::roaming_network_servers_count();
        if roaming_network_server_id == <T::RoamingNetworkServerIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingNetworkServers count overflow"));
        }
        Ok(roaming_network_server_id)
    }

    fn insert_roaming_network_server(
        owner: &T::AccountId,
        roaming_network_server_id: T::RoamingNetworkServerIndex,
        roaming_network_server: RoamingNetworkServer,
    ) {
        // Create and store roaming network_server
        <RoamingNetworkServers<T>>::insert(roaming_network_server_id, roaming_network_server);
        <RoamingNetworkServersCount<T>>::put(roaming_network_server_id + One::one());
        <RoamingNetworkServerOwners<T>>::insert(roaming_network_server_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_network_server_id: T::RoamingNetworkServerIndex) {
        <RoamingNetworkServerOwners<T>>::insert(roaming_network_server_id, to);
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_ok,
        impl_outer_origin,
        parameter_types,
        weights::Weight,
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
    impl system::Trait for Test {
        type AccountId = u64;
        type AvailableBlockRatio = AvailableBlockRatio;
        type BlockHashCount = BlockHashCount;
        type BlockNumber = u64;
        type Call = ();
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Header = Header;
        type Index = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type MaximumBlockLength = MaximumBlockLength;
        type MaximumBlockWeight = MaximumBlockWeight;
        type ModuleToIndex = ();
        type Origin = Origin;
        type Version = ();
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ();
    }
    impl transaction_payment::Trait for Test {
        type Currency = Balances;
        type FeeMultiplierUpdate = ();
        type OnTransactionPayment = ();
        type TransactionBaseFee = ();
        type TransactionByteFee = ();
        type WeightToFee = ();
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
    impl Trait for Test {
        type Event = ();
        type RoamingNetworkServerIndex = u64;
    }
    // type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type RoamingNetworkServerModule = Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
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
            assert_eq!(RoamingNetworkServerModule::roaming_network_servers_count(), 0);
            assert!(RoamingNetworkServerModule::roaming_network_server(0).is_none());
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_owner(0), None);
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_price(0), None);
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
        });
    }

    #[test]
    fn create_works() {
        new_test_ext().execute_with(|| {
            // Call Functions
            assert_ok!(RoamingNetworkServerModule::create(Origin::signed(1)));
            // Verify Storage
            assert_eq!(RoamingNetworkServerModule::roaming_network_servers_count(), 1);
            assert!(RoamingNetworkServerModule::roaming_network_server(0).is_some());
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_owner(0), Some(1));
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_price(0), None);
        });
    }

    #[test]
    fn create_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            <RoamingNetworkServersCount<Test>>::put(u64::max_value());
            // Call Functions
            assert_noop!(RoamingNetworkServerModule::create(Origin::signed(1)), "RoamingNetworkServers count overflow");
            // Verify Storage
            assert_eq!(RoamingNetworkServerModule::roaming_network_servers_count(), u64::max_value());
            assert!(RoamingNetworkServerModule::roaming_network_server(0).is_none());
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_owner(0), None);
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_price(0), None);
        });
    }

    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingNetworkServerModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingNetworkServerModule::transfer(Origin::signed(1), 2, 0));
            // Verify Storage
            assert_eq!(RoamingNetworkServerModule::roaming_network_servers_count(), 1);
            assert!(RoamingNetworkServerModule::roaming_network_server(0).is_some());
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_owner(0), Some(2));
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_price(0), None);
        });
    }

    #[test]
    fn transfer_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingNetworkServerModule::create(Origin::signed(1)));
            // Call Functions
            assert_noop!(
                RoamingNetworkServerModule::transfer(Origin::signed(2), 2, 0),
                "Only owner can transfer roaming network_server"
            );
            assert_noop!(
                RoamingNetworkServerModule::transfer(Origin::signed(1), 2, 1),
                "Only owner can transfer roaming network_server"
            );
            // Verify Storage
            assert_eq!(RoamingNetworkServerModule::roaming_network_servers_count(), 1);
            assert!(RoamingNetworkServerModule::roaming_network_server(0).is_some());
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_owner(0), Some(1));
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_price(0), None);
        });
    }

    #[test]
    fn set_price_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingNetworkServerModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingNetworkServerModule::set_price(Origin::signed(1), 0, Some(10)));
            // Verify Storage
            assert_eq!(RoamingNetworkServerModule::roaming_network_servers_count(), 1);
            assert!(RoamingNetworkServerModule::roaming_network_server(0).is_some());
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_owner(0), Some(1));
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_price(0), Some(10));
        });
    }

    #[test]
    fn buy_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingNetworkServerModule::create(Origin::signed(1)));
            assert_ok!(RoamingNetworkServerModule::set_price(Origin::signed(1), 0, Some(10)));
            // Call Functions
            assert_ok!(RoamingNetworkServerModule::buy(Origin::signed(2), 0, 10));
            // Verify Storage
            assert_eq!(RoamingNetworkServerModule::roaming_network_servers_count(), 1);
            assert!(RoamingNetworkServerModule::roaming_network_server(0).is_some());
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_owner(0), Some(2));
            assert_eq!(RoamingNetworkServerModule::roaming_network_server_price(0), None);
            assert_eq!(Balances::free_balance(1), 20);
            assert_eq!(Balances::free_balance(2), 10);
        });
    }
}
