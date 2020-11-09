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
    traits::Get,
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
#[macro_use]
extern crate alloc; // Required to use Vec

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + roaming_operators::Trait
    + roaming_network_servers::Trait
    + roaming_devices::Trait
    + roaming_sessions::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type RoamingPacketBundleIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingPacketBundleReceivedAtHome: Parameter + Member + Default;
    type RoamingPacketBundleReceivedPacketsCount: Parameter + Member + Default;
    type RoamingPacketBundleReceivedPacketsOkCount: Parameter + Member + Default;
    type RoamingPacketBundleReceivedStartedAt: Parameter + Member + Default;
    type RoamingPacketBundleReceivedEndedAt: Parameter + Member + Default;
    type RoamingPacketBundleExternalDataStorageHash: Parameter + Member + Default;
}

type BalanceOf<T> =
    <<T as roaming_operators::Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingPacketBundle(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
// Generic type parameters - Balance
pub struct RoamingPacketBundleReceiver<U, V, W, X, Y, Z> {
    packet_bundle_received_at_home: U,
    packet_bundle_received_packets_count: V,
    packet_bundle_received_packets_ok_count: W,
    packet_bundle_received_started_at: X,
    packet_bundle_received_ended_at: Y,
    packet_bundle_external_data_storage_hash: Z,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::RoamingPacketBundleIndex,
        <T as Trait>::RoamingPacketBundleReceivedAtHome,
        <T as Trait>::RoamingPacketBundleReceivedPacketsCount,
        <T as Trait>::RoamingPacketBundleReceivedPacketsOkCount,
        <T as Trait>::RoamingPacketBundleReceivedStartedAt,
        <T as Trait>::RoamingPacketBundleReceivedEndedAt,
        <T as Trait>::RoamingPacketBundleExternalDataStorageHash,
        // <T as roaming_devices::Trait>::RoamingDeviceIndex,
        <T as roaming_sessions::Trait>::RoamingSessionIndex,
        <T as roaming_network_servers::Trait>::RoamingNetworkServerIndex,
        // <T as roaming_operators::Trait>::RoamingOperatorIndex,
        Balance = BalanceOf<T>,
    {
        /// A roaming packet_bundle is created. (owner, roaming_packet_bundle_id)
        Created(AccountId, RoamingPacketBundleIndex),
        /// A roaming packet_bundle is transferred. (from, to, roaming_packet_bundle_id)
        Transferred(AccountId, AccountId, RoamingPacketBundleIndex),
        /// A roaming packet_bundle is available for sale. (owner, roaming_packet_bundle_id, price)
        PriceSet(AccountId, RoamingPacketBundleIndex, Option<Balance>),
        /// A roaming packet_bundle is sold. (from, to, roaming_packet_bundle_id, price)
        Sold(AccountId, AccountId, RoamingPacketBundleIndex, Balance),
        // /// A roaming packet_bundle receiveruration
        // RoamingPacketBundleReceiverSet(AccountId, RoamingPacketBundleIndex, RoamingPacketBundleNextBillingAt, RoamingPacketBundleFrequencyInDays),
        /// A roaming packet_bundle receiver was set
        RoamingPacketBundleReceiverSet(AccountId, RoamingPacketBundleIndex, RoamingNetworkServerIndex, RoamingPacketBundleReceivedAtHome,
            RoamingPacketBundleReceivedPacketsCount, RoamingPacketBundleReceivedPacketsOkCount, RoamingPacketBundleReceivedStartedAt,
            RoamingPacketBundleReceivedEndedAt, RoamingPacketBundleExternalDataStorageHash),
        // /// A roaming packet_bundle is assigned to a operator. (owner of session, roaming_packet_bundle_id, roaming_operator_id)
        // AssignedPacketBundleToOperator(AccountId, RoamingPacketBundleIndex, RoamingOperatorIndex),
        /// A roaming packet_bundle is assigned to a session. (owner of session, roaming_packet_bundle_id, roaming_session_id)
        AssignedPacketBundleToSession(AccountId, RoamingPacketBundleIndex, RoamingSessionIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingPacketBundles {
        /// Stores all the roaming packet_bundle, key is the roaming packet_bundle id / index
        pub RoamingPacketBundles get(fn roaming_packet_bundle): map hasher(opaque_blake2_256) T::RoamingPacketBundleIndex => Option<RoamingPacketBundle>;

        /// Stores the total number of roaming packet_bundles. i.e. the next roaming packet_bundle index
        pub RoamingPacketBundlesCount get(fn roaming_packet_bundles_count): T::RoamingPacketBundleIndex;

        /// Get roaming packet_bundle owner
        pub RoamingPacketBundleOwners get(fn roaming_packet_bundle_owner): map hasher(opaque_blake2_256) T::RoamingPacketBundleIndex => Option<T::AccountId>;

        /// Get roaming packet_bundle price. None means not for sale.
        pub RoamingPacketBundlePrices get(fn roaming_packet_bundle_price): map hasher(opaque_blake2_256) T::RoamingPacketBundleIndex => Option<BalanceOf<T>>;

        // /// Get roaming packet_bundle receiver
        // pub RoamingPacketBundleReceivers get(fn roaming_packet_bundle_receivers): map hasher(opaque_blake2_256) T::RoamingPacketBundleIndex => Option<RoamingPacketBundleReceiver<T::RoamingPacketBundleNextBillingAt, T::RoamingPacketBundleFrequencyInDays>>;

        /// Get roaming packet_bundle receiver
        pub RoamingPacketBundleReceivers get(fn roaming_packet_bundle_receivers): map hasher(opaque_blake2_256) (T::RoamingPacketBundleIndex, T::RoamingNetworkServerIndex) =>
            Option<RoamingPacketBundleReceiver<
                T::RoamingPacketBundleReceivedAtHome,
                T::RoamingPacketBundleReceivedPacketsCount,
                T::RoamingPacketBundleReceivedPacketsOkCount,
                T::RoamingPacketBundleReceivedStartedAt,
                T::RoamingPacketBundleReceivedEndedAt,
                T::RoamingPacketBundleExternalDataStorageHash
            >>;

        /// NetworkServer to PacketBundles mapping
        pub RoamingNetworkServerPacketBundles get(fn roaming_network_server_packet_bundles): map hasher(opaque_blake2_256) T::RoamingNetworkServerIndex => Option<Vec<T::RoamingPacketBundleIndex>>;

        // Device Session mapping
        pub RoamingPacketBundleDeviceSession get(fn roaming_packet_bundle_device_sessions): map hasher(opaque_blake2_256) T::RoamingPacketBundleIndex => Option<(T::RoamingDeviceIndex, T::RoamingSessionIndex)>;

        pub RoamingDeviceSessionPacketBundles get(fn roaming_device_session_packet_bundles): map hasher(opaque_blake2_256) (T::RoamingDeviceIndex, T::RoamingSessionIndex) => Option<Vec<T::RoamingPacketBundleIndex>>;

        // IPFS
        pub RoamingExternalDataStorageHashPacketBundle get(fn roaming_external_data_storage_hash_packet_bundle):  map hasher(opaque_blake2_256) T::RoamingPacketBundleExternalDataStorageHash => Option<Vec<T::RoamingPacketBundleIndex>>;

        /// Get roaming packet_bundle session
        pub RoamingPacketBundleSession get(fn roaming_packet_bundle_session): map hasher(opaque_blake2_256) T::RoamingPacketBundleIndex => Option<T::RoamingSessionIndex>;

        /// Get roaming session's packet bundles
        pub RoamingSessionPacketBundles get(fn roaming_session_packet_bundles): map hasher(opaque_blake2_256) T::RoamingSessionIndex => Option<Vec<T::RoamingPacketBundleIndex>>

        // /// Get roaming packet_bundle operator
        // pub RoamingPacketBundleOperator get(fn roaming_packet_bundle_operator): map hasher(opaque_blake2_256) T::RoamingPacketBundleIndex => Option<T::RoamingOperatorIndex>;

        // /// Get roaming operator's packet bundles
        // pub RoamingOperatorPacketBundles get(fn roaming_operator_packet_bundles): map hasher(opaque_blake2_256) T::RoamingOperatorIndex => Option<Vec<T::RoamingPacketBundleIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming packet_bundle
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_packet_bundle_id = Self::next_roaming_packet_bundle_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming packet_bundle
            let roaming_packet_bundle = RoamingPacketBundle(unique_id);
            Self::insert_roaming_packet_bundle(&sender, roaming_packet_bundle_id, roaming_packet_bundle);

            Self::deposit_event(RawEvent::Created(sender, roaming_packet_bundle_id));
        }

        /// Transfer a roaming packet_bundle to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_packet_bundle_id: T::RoamingPacketBundleIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_packet_bundle_owner(roaming_packet_bundle_id) == Some(sender.clone()), "Only owner can transfer roaming packet_bundle");

            Self::update_owner(&to, roaming_packet_bundle_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_packet_bundle_id));
        }

        /// Set a price for a roaming packet_bundle for sale
        /// None to delist the roaming packet_bundle
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_price(origin, roaming_packet_bundle_id: T::RoamingPacketBundleIndex, price: Option<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_packet_bundle_owner(roaming_packet_bundle_id) == Some(sender.clone()), "Only owner can set price for roaming packet_bundle");

            if let Some(ref price) = price {
                <RoamingPacketBundlePrices<T>>::insert(roaming_packet_bundle_id, price);
            } else {
                <RoamingPacketBundlePrices<T>>::remove(roaming_packet_bundle_id);
            }

            Self::deposit_event(RawEvent::PriceSet(sender, roaming_packet_bundle_id, price));
        }

        /// Set roaming packet_bundle receiver
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_receiver(
            origin,
            roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
            roaming_network_server_id: T::RoamingNetworkServerIndex,
            _packet_bundle_received_at_home: Option<T::RoamingPacketBundleReceivedAtHome>,
            _packet_bundle_received_packets_count: Option<T::RoamingPacketBundleReceivedPacketsCount>,
            _packet_bundle_received_packets_ok_count: Option<T::RoamingPacketBundleReceivedPacketsOkCount>,
            _packet_bundle_received_started_at: Option<T::RoamingPacketBundleReceivedStartedAt>,
            _packet_bundle_received_ended_at: Option<T::RoamingPacketBundleReceivedEndedAt>,
            _packet_bundle_external_data_storage_hash: Option<T::RoamingPacketBundleExternalDataStorageHash>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming packet bundle id whose receiver we want to change actually exists
            let is_roaming_packet_bundle = Self::exists_roaming_packet_bundle(roaming_packet_bundle_id).is_ok();
            ensure!(is_roaming_packet_bundle, "RoamingPacketBundle does not exist");

            // Ensure that the roaming network server id that we want to associate with the packet bundle id actually exists
            let is_roaming_network_server = Self::exists_roaming_network_server(roaming_network_server_id).is_ok();
            ensure!(is_roaming_network_server, "RoamingNetworkServer does not exist");

            // Ensure that the caller is owner of the given network server id
            let is_owned_by_network_server = Self::is_owned_by_network_server(roaming_network_server_id, sender.clone()).is_ok();
            ensure!(is_owned_by_network_server, "Ownership by given network server does not exist");

            // Ensure that the caller is owner of the packet bundle receiver they are trying to change
            ensure!(Self::roaming_packet_bundle_owner(roaming_packet_bundle_id) == Some(sender.clone()), "Only owner can set receiver for roaming packet_bundle");

            let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_packet_bundle_id, sender.clone()).is_ok();
            ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            let packet_bundle_received_at_home = match _packet_bundle_received_at_home {
                Some(value) => value,
                None => Default::default() // Default
            };
            let packet_bundle_received_packets_count = match _packet_bundle_received_packets_count {
                Some(value) => value,
                None => Default::default()
            };
            let packet_bundle_received_packets_ok_count = match _packet_bundle_received_packets_ok_count {
                Some(value) => value,
                None => Default::default()
            };
            let packet_bundle_received_started_at = match _packet_bundle_received_started_at {
                Some(value) => value,
                None => Default::default()
            };
            let packet_bundle_received_ended_at = match _packet_bundle_received_ended_at {
                Some(value) => value,
                None => Default::default()
            };
            let packet_bundle_external_data_storage_hash = match _packet_bundle_external_data_storage_hash {
                Some(value) => value,
                None => Default::default()
            };

            // Check if a roaming packet bundle receiver already exists with the given roaming packet bundle id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_packet_bundle_receiver_index(roaming_packet_bundle_id, roaming_network_server_id).is_ok() {
                debug::info!("Mutating values");
                <RoamingPacketBundleReceivers<T>>::mutate((roaming_packet_bundle_id, roaming_network_server_id), |packet_bundle_receiver| {
                    if let Some(_packet_bundle_receiver) = packet_bundle_receiver {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _packet_bundle_receiver.packet_bundle_received_at_home = packet_bundle_received_at_home.clone();
                        _packet_bundle_receiver.packet_bundle_received_packets_count = packet_bundle_received_packets_count.clone();
                        _packet_bundle_receiver.packet_bundle_received_packets_ok_count = packet_bundle_received_packets_ok_count.clone();
                        _packet_bundle_receiver.packet_bundle_received_started_at = packet_bundle_received_started_at.clone();
                        _packet_bundle_receiver.packet_bundle_received_ended_at = packet_bundle_received_ended_at.clone();
                        _packet_bundle_receiver.packet_bundle_external_data_storage_hash = packet_bundle_external_data_storage_hash.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_packet_bundle_receiver = <RoamingPacketBundleReceivers<T>>::get((roaming_packet_bundle_id, roaming_network_server_id));
                if let Some(_packet_bundle_receiver) = fetched_packet_bundle_receiver {
                    debug::info!("Latest field packet_bundle_received_at_home {:#?}", _packet_bundle_receiver.packet_bundle_received_at_home);
                    debug::info!("Latest field packet_bundle_received_packets_count {:#?}", _packet_bundle_receiver.packet_bundle_received_packets_count);
                    debug::info!("Latest field packet_bundle_received_packets_ok_count {:#?}", _packet_bundle_receiver.packet_bundle_received_packets_ok_count);
                    debug::info!("Latest field packet_bundle_received_started_at {:#?}", _packet_bundle_receiver.packet_bundle_received_started_at);
                    debug::info!("Latest field packet_bundle_received_ended_at {:#?}", _packet_bundle_receiver.packet_bundle_received_ended_at);
                    debug::info!("Latest field packet_bundle_external_data_storage_hash {:#?}", _packet_bundle_receiver.packet_bundle_external_data_storage_hash);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new roaming packet_bundle receiver instance with the input params
                let roaming_packet_bundle_receiver_instance = RoamingPacketBundleReceiver {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    packet_bundle_received_at_home: packet_bundle_received_at_home.clone(),
                    packet_bundle_received_packets_count: packet_bundle_received_packets_count.clone(),
                    packet_bundle_received_packets_ok_count: packet_bundle_received_packets_ok_count.clone(),
                    packet_bundle_received_started_at: packet_bundle_received_started_at.clone(),
                    packet_bundle_received_ended_at: packet_bundle_received_ended_at.clone(),
                    packet_bundle_external_data_storage_hash: packet_bundle_external_data_storage_hash.clone()
                };

                <RoamingPacketBundleReceivers<T>>::insert(
                    (roaming_packet_bundle_id, roaming_network_server_id),
                    &roaming_packet_bundle_receiver_instance
                );

                debug::info!("Checking inserted values");
                let fetched_packet_bundle_receiver = <RoamingPacketBundleReceivers<T>>::get((roaming_packet_bundle_id, roaming_network_server_id));
                if let Some(_packet_bundle_receiver) = fetched_packet_bundle_receiver {
                    debug::info!("Inserted field packet_bundle_received_at_home {:#?}", _packet_bundle_receiver.packet_bundle_received_at_home);
                    debug::info!("Inserted field packet_bundle_received_packets_count {:#?}", _packet_bundle_receiver.packet_bundle_received_packets_count);
                    debug::info!("Inserted field packet_bundle_received_packets_ok_count {:#?}", _packet_bundle_receiver.packet_bundle_received_packets_ok_count);
                    debug::info!("Inserted field packet_bundle_received_started_at {:#?}", _packet_bundle_receiver.packet_bundle_received_started_at);
                    debug::info!("Inserted field packet_bundle_received_ended_at {:#?}", _packet_bundle_receiver.packet_bundle_received_ended_at);
                    debug::info!("Inserted field packet_bundle_external_data_storage_hash {:#?}", _packet_bundle_receiver.packet_bundle_external_data_storage_hash);
                }
            }

            Self::deposit_event(RawEvent::RoamingPacketBundleReceiverSet(
                sender,
                roaming_packet_bundle_id,
                roaming_network_server_id,
                packet_bundle_received_at_home,
                packet_bundle_received_packets_count,
                packet_bundle_received_packets_ok_count,
                packet_bundle_received_started_at,
                packet_bundle_received_ended_at,
                packet_bundle_external_data_storage_hash
            ));
        }

        /// Buy a roaming packet_bundle with max price willing to pay
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn buy(origin, roaming_packet_bundle_id: T::RoamingPacketBundleIndex, price: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            let owner = Self::roaming_packet_bundle_owner(roaming_packet_bundle_id);
            ensure!(owner.is_some(), "RoamingPacketBundle owner does not exist");
            let owner = owner.unwrap();

            let roaming_packet_bundle_price = Self::roaming_packet_bundle_price(roaming_packet_bundle_id);
            ensure!(roaming_packet_bundle_price.is_some(), "RoamingPacketBundle not for sale");

            let roaming_packet_bundle_price = roaming_packet_bundle_price.unwrap();
            ensure!(price >= roaming_packet_bundle_price, "Price is too low");

            T::Currency::transfer(&sender, &owner, roaming_packet_bundle_price, ExistenceRequirement::AllowDeath)?;

            <RoamingPacketBundlePrices<T>>::remove(roaming_packet_bundle_id);

            Self::update_owner(&sender, roaming_packet_bundle_id);

            Self::deposit_event(RawEvent::Sold(owner, sender, roaming_packet_bundle_id, roaming_packet_bundle_price));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_packet_bundle_to_session(
            origin,
            roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
            roaming_session_id: T::RoamingSessionIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given session id already exists
            let is_roaming_session = <roaming_sessions::Module<T>>
                ::exists_roaming_session(roaming_session_id).is_ok();
            ensure!(is_roaming_session, "RoamingSession does not exist");

            // Ensure that caller of the function is the owner of the session id to assign the packet_bundle to
            ensure!(
                <roaming_sessions::Module<T>>::is_roaming_session_owner(roaming_session_id, sender.clone()).is_ok(),
                "Only the roaming session owner can assign itself a roaming packet bundle"
            );

            Self::associate_packet_bundle_with_session(roaming_packet_bundle_id, roaming_session_id)
                .expect("Unable to associate packet bundle with session");

            // Ensure that the given packet_bundle id already exists
            let roaming_packet_bundle = Self::roaming_packet_bundle(roaming_packet_bundle_id);
            ensure!(roaming_packet_bundle.is_some(), "Invalid roaming_packet_bundle_id");

            // Ensure that the packet_bundle is not already owned by a different session
            // Unassign the packet_bundle from any existing session since it may only be owned by one session
            <RoamingPacketBundleSession<T>>::remove(roaming_packet_bundle_id);

            // Assign the packet_bundle owner to the given session (even if already belongs to them)
            <RoamingPacketBundleSession<T>>::insert(roaming_packet_bundle_id, roaming_session_id);

            Self::deposit_event(RawEvent::AssignedPacketBundleToSession(sender, roaming_packet_bundle_id, roaming_session_id));
        }

        // pub fn assign_packet_bundle_to_operator(
        //     origin,
        //     roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
        //     roaming_operator_id: T::RoamingOperatorIndex
        // ) {
        //     let sender = ensure_signed(origin)?;

        //     // Ensure that the given session id already exists
        //     let is_roaming_operator = <roaming_operators::Module<T>>
        //         ::exists_roaming_operator(roaming_operator_id).is_ok();
        //     ensure!(is_roaming_operator, "RoamingOperator does not exist");

        //     // Ensure that caller of the function is the owner of the operator id to assign the packet_bundle to
        //     ensure!(
        //         <roaming_operators::Module<T>>::is_roaming_operator_owner(roaming_operator_id, sender.clone()).is_ok(),
        //         "Only the roaming operator owner can assign itself a roaming packet bundle"
        //     );

        //     Self::associate_packet_bundle_with_operator(roaming_packet_bundle_id, roaming_operator_id)
        //         .expect("Unable to associate packet bundle with operator");

        //     // Ensure that the given packet_bundle id already exists
        //     let roaming_packet_bundle = Self::roaming_packet_bundle(roaming_packet_bundle_id);
        //     ensure!(roaming_packet_bundle.is_some(), "Invalid roaming_packet_bundle_id");

        //     // Ensure that the packet_bundle is not already owned by a different operator
        //     // Unassign the packet_bundle from any existing operator since it may only be owned by one operator
        //     <RoamingPacketBundleOperator<T>>::remove(roaming_packet_bundle_id);

        //     // Assign the packet_bundle owner to the given operator (even if already belongs to them)
        //     <RoamingPacketBundleOperator<T>>::insert(roaming_packet_bundle_id, roaming_operator_id);

        //     Self::deposit_event(RawEvent::AssignedPacketBundleToOperator(sender, roaming_packet_bundle_id, roaming_operator_id));
        // }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_roaming_packet_bundle_owner(
        roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_packet_bundle_owner(&roaming_packet_bundle_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of RoamingPacketBundle"
        );
        Ok(())
    }

    pub fn is_owned_by_required_parent_relationship(
        roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        debug::info!("Get the packet bundle session id associated with the session of the given packet bundle id");
        let packet_bundle_session_id = Self::roaming_packet_bundle_session(roaming_packet_bundle_id);

        if let Some(_packet_bundle_session_id) = packet_bundle_session_id {
            // Ensure that the caller is owner of the session id associated with the packet bundle
            ensure!(
                (<roaming_sessions::Module<T>>::is_roaming_session_owner(
                    _packet_bundle_session_id.clone(),
                    sender.clone()
                ))
                .is_ok(),
                "Only owner of the session id associated with the given packet bundle can set an associated roaming \
                 packet bundle receiver"
            );
        } else {
            // There must be a packet bundle session id associated with the packet bundle
            return Err(DispatchError::Other("RoamingPacketBundleSession does not exist"));
        }
        Ok(())
    }

    pub fn exists_roaming_packet_bundle(
        roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
    ) -> Result<RoamingPacketBundle, DispatchError> {
        match Self::roaming_packet_bundle(roaming_packet_bundle_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("RoamingPacketBundle does not exist")),
        }
    }

    pub fn exists_roaming_packet_bundle_receiver(
        roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
        roaming_network_server_id: T::RoamingNetworkServerIndex,
    ) -> Result<(), DispatchError> {
        match Self::roaming_packet_bundle_receivers((roaming_packet_bundle_id, roaming_network_server_id)) {
            Some(_) => Ok(()),
            None => Err(DispatchError::Other("RoamingPacketBundleReceiver does not exist")),
        }
    }

    pub fn exists_roaming_network_server(
        roaming_network_server_id: T::RoamingNetworkServerIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Ensuring that the caller has provided a network server id that actually exists");
        match <roaming_network_servers::Module<T>>::exists_roaming_network_server(roaming_network_server_id) {
            Ok(_) => Ok(()),
            Err(_e) => Err(DispatchError::Other("RoamingNetworkServer does not exist")),
        }
    }

    pub fn is_owned_by_network_server(
        roaming_network_server_id: T::RoamingNetworkServerIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Ensuring that the caller is owner of the given network server id associated with the given packet bundle \
             id"
        );
        match <roaming_network_servers::Module<T>>::is_roaming_network_server_owner(roaming_network_server_id, sender) {
            Ok(_) => Ok(()),
            Err(_e) => {
                Err(DispatchError::Other(
                    "Only owner of the given network server id associated with the given packet bundle id can set it \
                     as an associated roaming packet bundle receiver",
                ))
            }
        }
    }

    pub fn has_value_for_packet_bundle_receiver_index(
        roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
        roaming_network_server_id: T::RoamingNetworkServerIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if packet bundle receiver has a value that is defined");
        let fetched_packet_bundle_receiver =
            <RoamingPacketBundleReceivers<T>>::get((roaming_packet_bundle_id, roaming_network_server_id));
        if let Some(_) = fetched_packet_bundle_receiver {
            debug::info!("Found value for packet bundle receiver");
            return Ok(());
        }
        debug::info!("No value for packet bundle receiver");
        Err(DispatchError::Other("No value for packet bundle receiver"))
    }

    /// Only push the packet bundle id onto the end of the vector if it does not already exist
    pub fn associate_packet_bundle_with_session(
        roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
        roaming_session_id: T::RoamingSessionIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given session id already exists as a key,
        // and where its corresponding value is a vector that already contains the given packet bundle id
        if let Some(session_packet_bundles) = Self::roaming_session_packet_bundles(roaming_session_id) {
            debug::info!("Session id key {:?} exists with value {:?}", roaming_session_id, session_packet_bundles);
            let not_session_contains_packet_bundle = !session_packet_bundles.contains(&roaming_packet_bundle_id);
            ensure!(not_session_contains_packet_bundle, "Session already contains the given packet bundle id");
            debug::info!("Session id key exists but its vector value does not contain the given packet bundle id");
            <RoamingSessionPacketBundles<T>>::mutate(roaming_session_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_packet_bundle_id);
                }
            });
            debug::info!(
                "Associated packet bundle {:?} with session {:?}",
                roaming_packet_bundle_id,
                roaming_session_id
            );
            Ok(())
        } else {
            debug::info!(
                "Session id key does not yet exist. Creating the session key {:?} and appending the packet bundle id \
                 {:?} to its vector value",
                roaming_session_id,
                roaming_packet_bundle_id
            );
            <RoamingSessionPacketBundles<T>>::insert(roaming_session_id, &vec![roaming_packet_bundle_id]);
            Ok(())
        }
    }

    // /// Only push the packet bundle id onto the end of the vector if it does not already exist
    // pub fn associate_packet_bundle_with_operator(
    //     roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
    //     roaming_operator_id: T::RoamingOperatorIndex
    // ) -> Result<(), DispatchError>
    // {
    //     // Early exit with error since do not want to append if the given operator id already exists as a key,
    //     // and where its corresponding value is a vector that already contains the given packet bundle id
    //     if let Some(operator_packet_bundles) = Self::roaming_operator_packet_bundles(roaming_operator_id) {
    //         debug::info!("Operator id key {:?} exists with value {:?}", roaming_operator_id,
    // operator_packet_bundles);         let not_operator_contains_packet_bundle =
    // !operator_packet_bundles.contains(&roaming_packet_bundle_id);         ensure!
    // (not_operator_contains_packet_bundle, "Operator already contains the given packet bundle id");         debug:
    // :info!("Operator id key exists but its vector value does not contain the given packet bundle id");
    //         <RoamingOperatorPacketBundles<T>>::mutate(roaming_operator_id, |v| {
    //             if let Some(value) = v {
    //                 value.push(roaming_packet_bundle_id);
    //             }
    //         });
    //         debug::info!("Associated packet bundle {:?} with operator {:?}", roaming_packet_bundle_id,
    // roaming_operator_id);         Ok(())
    //     } else {
    //         debug::info!("Operator id key does not yet exist. Creating the operator key {:?} and appending the packet
    // bundle id {:?} to its vector value", roaming_operator_id, roaming_packet_bundle_id);
    //         <RoamingOperatorPacketBundles<T>>::insert(roaming_operator_id, &vec![roaming_packet_bundle_id]);
    //         Ok(())
    //     }
    // }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <frame_system::Module<T>>::extrinsic_index(),
            <frame_system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_roaming_packet_bundle_id() -> Result<T::RoamingPacketBundleIndex, DispatchError> {
        let roaming_packet_bundle_id = Self::roaming_packet_bundles_count();
        if roaming_packet_bundle_id == <T::RoamingPacketBundleIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingPacketBundles count overflow"));
        }
        Ok(roaming_packet_bundle_id)
    }

    fn insert_roaming_packet_bundle(
        owner: &T::AccountId,
        roaming_packet_bundle_id: T::RoamingPacketBundleIndex,
        roaming_packet_bundle: RoamingPacketBundle,
    ) {
        // Create and store roaming packet_bundle
        <RoamingPacketBundles<T>>::insert(roaming_packet_bundle_id, roaming_packet_bundle);
        <RoamingPacketBundlesCount<T>>::put(roaming_packet_bundle_id + One::one());
        <RoamingPacketBundleOwners<T>>::insert(roaming_packet_bundle_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_packet_bundle_id: T::RoamingPacketBundleIndex) {
        <RoamingPacketBundleOwners<T>>::insert(roaming_packet_bundle_id, to);
    }
}
