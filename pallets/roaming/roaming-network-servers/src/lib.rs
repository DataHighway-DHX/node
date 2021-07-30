#![cfg_attr(not(feature = "std"), no_std)]

use log::{warn, info};
use codec::{
    Decode,
    Encode,
};
use frame_support::{
    decl_event,
    decl_module,
    decl_storage,
    ensure,
    traits::{
        Currency,
        ExistenceRequirement,
        Get,
        Randomness,
    },
    Parameter,
};
use frame_system::ensure_signed;
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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config: frame_system::Config + roaming_operators::Config + roaming_networks::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RoamingNetworkServerIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> =
    <<T as roaming_operators::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingNetworkServer(pub [u8; 16]);

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::RoamingNetworkServerIndex,
        <T as roaming_networks::Config>::RoamingNetworkIndex,
        <T as roaming_operators::Config>::RoamingOperatorIndex,
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
    trait Store for Module<T: Config> as RoamingNetworkServers {
        /// Stores all the roaming network_servers, key is the roaming network_server id / index
        pub RoamingNetworkServers get(fn roaming_network_server): map hasher(opaque_blake2_256) T::RoamingNetworkServerIndex => Option<RoamingNetworkServer>;

        /// Stores the total number of roaming network_servers. i.e. the next roaming network_server index
        pub RoamingNetworkServersCount get(fn roaming_network_servers_count): T::RoamingNetworkServerIndex;

        /// Get roaming network_server owner
        pub RoamingNetworkServerOwners get(fn roaming_network_server_owner): map hasher(opaque_blake2_256) T::RoamingNetworkServerIndex => Option<T::AccountId>;

        /// Get roaming network_server price. None means not for sale.
        pub RoamingNetworkServerPrices get(fn roaming_network_server_price): map hasher(opaque_blake2_256) T::RoamingNetworkServerIndex => Option<BalanceOf<T>>;

        /// Get roaming network_server network
        pub RoamingNetworkServerNetwork get(fn roaming_network_server_network): map hasher(opaque_blake2_256) T::RoamingNetworkServerIndex => Option<T::RoamingNetworkIndex>;

        /// Get roaming network_server operators
        pub RoamingNetworkServerOperator get(fn roaming_network_server_operators): map hasher(opaque_blake2_256) T::RoamingNetworkServerIndex => Option<T::RoamingOperatorIndex>;

        /// Get roaming network's network servers
        pub RoamingNetworkNetworkServers get(fn roaming_network_network_servers): map hasher(opaque_blake2_256) T::RoamingNetworkIndex => Option<Vec<T::RoamingNetworkServerIndex>>;

        /// Get roaming operator's network servers
        pub RoamingOperatorNetworkServers get(fn roaming_operator_network_servers): map hasher(opaque_blake2_256) T::RoamingOperatorIndex => Option<Vec<T::RoamingNetworkServerIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming network_server
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
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
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_network_server_id: T::RoamingNetworkServerIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_network_server_owner(roaming_network_server_id) == Some(sender.clone()), "Only owner can transfer roaming network_server");

            Self::update_owner(&to, roaming_network_server_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_network_server_id));
        }

        /// Set a price for a roaming network_server for sale
        /// None to delist the roaming network_server
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
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
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
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

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
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

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
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

impl<T: Config> Module<T> {
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
            info!("Network id key {:?} exists with value {:?}", roaming_network_id, network_network_servers);
            let not_network_contains_network_server = !network_network_servers.contains(&roaming_network_server_id);
            ensure!(not_network_contains_network_server, "Network already contains the given network server id");
            info!("Network id key exists but its vector value does not contain the given network server id");
            <RoamingNetworkNetworkServers<T>>::mutate(roaming_network_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_network_server_id);
                }
            });
            info!(
                "Associated network server {:?} with network {:?}",
                roaming_network_server_id,
                roaming_network_id
            );
            Ok(())
        } else {
            info!(
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
            info!("Operator id key {:?} exists with value {:?}", roaming_operator_id, operator_network_servers);
            let not_operator_contains_network_server = !operator_network_servers.contains(&roaming_network_server_id);
            ensure!(not_operator_contains_network_server, "Operator already contains the given network server id");
            info!("Operator id key exists but its vector value does not contain the given network server id");
            <RoamingOperatorNetworkServers<T>>::mutate(roaming_operator_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_network_server_id);
                }
            });
            info!(
                "Associated network server {:?} with operator {:?}",
                roaming_network_server_id,
                roaming_operator_id
            );
            Ok(())
        } else {
            info!(
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
            <frame_system::Pallet<T>>::extrinsic_index(),
            <frame_system::Pallet<T>>::block_number(),
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
