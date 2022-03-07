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
        Get,
        Randomness,
    },
    Parameter,
};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
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
pub trait Config:
    frame_system::Config + roaming_operators::Config + roaming_devices::Config + roaming_network_servers::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RoamingSessionIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingSession(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
// Generic type parameters - Balance
pub struct RoamingSessionJoinRequest<U, V> {
    session_network_server_id: U,
    session_join_requested_at_block: V,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
// Generic type parameters - Balance
pub struct RoamingSessionJoinAccept<U, V> {
    session_join_request_accept_expiry: U,
    session_join_request_accept_accepted_at_block: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::RoamingSessionIndex,
        <T as roaming_devices::Config>::RoamingDeviceIndex,
        <T as roaming_network_servers::Config>::RoamingNetworkServerIndex,
        <T as frame_system::Config>::BlockNumber,
    {
        /// A roaming session is created. (owner, roaming_session_id)
        Created(AccountId, RoamingSessionIndex),
        /// A roaming session is transferred. (from, to, roaming_session_id)
        Transferred(AccountId, AccountId, RoamingSessionIndex),
        /// A roaming session join request requested
        RoamingSessionJoinRequestRequested(AccountId, RoamingSessionIndex, RoamingNetworkServerIndex, BlockNumber),
        /// A roaming session join request accepted
        RoamingSessionJoinRequestAccepted(AccountId, RoamingSessionIndex, BlockNumber, BlockNumber),
        /// A roaming session is assigned to a device. (owner of device, roaming_session_id, roaming_device_id)
        AssignedSessionToDevice(AccountId, RoamingSessionIndex, RoamingDeviceIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as RoamingSessions {
        /// Stores all the roaming sessions, key is the roaming session id / index
        pub RoamingSessions get(fn roaming_session): map hasher(opaque_blake2_256) T::RoamingSessionIndex => Option<RoamingSession>;

        /// Stores the total number of roaming sessions. i.e. the next roaming session index
        pub RoamingSessionsCount get(fn roaming_sessions_count): T::RoamingSessionIndex;

        /// Get roaming session owner
        pub RoamingSessionOwners get(fn roaming_session_owner): map hasher(opaque_blake2_256) T::RoamingSessionIndex => Option<T::AccountId>;

        /// Get roaming session join requests
        pub RoamingSessionJoinRequests get(fn roaming_session_join_requests): map hasher(opaque_blake2_256) T::RoamingSessionIndex => Option<RoamingSessionJoinRequest<T::RoamingNetworkServerIndex, T::BlockNumber>>;

        /// Get roaming session join accepts
        pub RoamingSessionJoinAccepts get(fn roaming_session_join_accepts): map hasher(opaque_blake2_256) T::RoamingSessionIndex => Option<RoamingSessionJoinAccept<T::BlockNumber, T::BlockNumber>>;

        /// Get roaming session device
        pub RoamingSessionDevices get(fn roaming_session_device): map hasher(opaque_blake2_256) T::RoamingSessionIndex => Option<T::RoamingDeviceIndex>;

        /// Get roaming device sessions
        pub RoamingDeviceSessions get(fn roaming_device_sessions): map hasher(opaque_blake2_256) T::RoamingDeviceIndex => Option<Vec<T::RoamingSessionIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming session
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_session_id = Self::next_roaming_session_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming session
            let roaming_session = RoamingSession(unique_id);
            Self::insert_roaming_session(&sender, roaming_session_id, roaming_session);

            Self::deposit_event(RawEvent::Created(sender, roaming_session_id));
        }

        /// Transfer a roaming session to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_session_id: T::RoamingSessionIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_session_owner(roaming_session_id) == Some(sender.clone()), "Only owner can transfer roaming session");

            Self::update_owner(&to, roaming_session_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_session_id));
        }

        /// Set roaming session join request
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_join_request(
            origin,
            roaming_session_id: T::RoamingSessionIndex,
            _session_network_server_id: Option<T::RoamingNetworkServerIndex>,
            // FIXME - we shouldn't be passing the requested_at timestamp as an argument, it should be generated from the current time within this function
            _session_join_requested_at_block: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming session id whose join request we want to change actually exists
            let is_roaming_session = Self::exists_roaming_session(roaming_session_id).is_ok();
            ensure!(is_roaming_session, "RoamingSession does not exist");

            // Ensure that the caller is owner of the session join request they are trying to change
            ensure!(Self::roaming_session_owner(roaming_session_id) == Some(sender.clone()), "Only owner can set join request for roaming session");

            let session_network_server_id = match _session_network_server_id {
                Some(value) => value,
                None => Default::default() // Default
            };
            let session_join_requested_at_block = match _session_join_requested_at_block {
                Some(value) => value,
                None => Default::default()
            };

            info!("Checking that only the owner of the given network server id that the device is trying to connect to can set an associated roaming session join request");
            // Ensure that the caller is owner of the network server id that the device is trying to connect to for the session join request
            ensure!((<roaming_network_servers::Module<T>>::is_roaming_network_server_owner(
                        session_network_server_id.clone(),
                        sender.clone()
                    )).is_ok(), "Only owner of the given network server id that the device is trying to connect to can set an associated roaming session join request"
            );

            // Check if a roaming session join request already exists with the given roaming session id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_session_join_request_index(roaming_session_id).is_ok() {
                info!("Mutating values");
                <RoamingSessionJoinRequests<T>>::mutate(roaming_session_id, |session_join_request| {
                    if let Some(_session_join_request) = session_join_request {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _session_join_request.session_network_server_id = session_network_server_id.clone();
                        _session_join_request.session_join_requested_at_block = session_join_requested_at_block.clone();
                    }
                });
                info!("Checking mutated values");
                let fetched_session_join_request = <RoamingSessionJoinRequests<T>>::get(roaming_session_id);
                if let Some(_session_join_request) = fetched_session_join_request {
                    info!("Latest field session_network_server_id {:#?}", _session_join_request.session_network_server_id);
                    info!("Latest field session_join_requested_at_block {:#?}", _session_join_request.session_join_requested_at_block);
                }
            } else {
                info!("Inserting values");

                // Create a new roaming session join request instance with the input params
                let roaming_session_join_request_instance = RoamingSessionJoinRequest {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    session_network_server_id: session_network_server_id.clone(),
                    session_join_requested_at_block: session_join_requested_at_block.clone()
                };

                <RoamingSessionJoinRequests<T>>::insert(
                    roaming_session_id,
                    &roaming_session_join_request_instance
                );

                info!("Checking inserted values");
                let fetched_session_join_request = <RoamingSessionJoinRequests<T>>::get(roaming_session_id);
                if let Some(_session_join_request) = fetched_session_join_request {
                    info!("Inserted field session_network_server_id {:#?}", _session_join_request.session_network_server_id);
                    info!("Inserted field session_join_requested_at_block {:#?}", _session_join_request.session_join_requested_at_block);
                }
            }

            Self::deposit_event(RawEvent::RoamingSessionJoinRequestRequested(
                sender,
                roaming_session_id,
                session_network_server_id,
                session_join_requested_at_block
            ));
        }

        /// Set roaming session join accept
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_join_accept(
            origin,
            roaming_session_id: T::RoamingSessionIndex,
            // FIXME - this may stay optional, but if it's not provided we need to set a default value for how long until a join accept expires
            _session_join_request_accept_expiry: Option<T::BlockNumber>,
            // FIXME - we shouldn't be passing the accepted_at timestamp as an argument, it should be generated from the current time within this function
            _session_join_request_accept_accepted_at_block: Option<T::BlockNumber>,
        ) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming session id whose join accept we want to change actually exists
            let is_roaming_session = Self::exists_roaming_session(roaming_session_id).is_ok();
            ensure!(is_roaming_session, "RoamingSession does not exist");

            // Ensure that the caller is owner of the session join accept they are trying to change
            ensure!(Self::roaming_session_owner(roaming_session_id) == Some(sender.clone()), "Only owner can set join accept for roaming session");

            info!("Get the network server id associated with the join request of the given session id");
            let session_join_request = Self::roaming_session_join_requests(roaming_session_id);

            if let Some(_session_join_request) = session_join_request {
                // Ensure that the caller is owner of the network server id that the device is trying to connect to for the session join request
                ensure!((<roaming_network_servers::Module<T>>::is_roaming_network_server_owner(
                        _session_join_request.session_network_server_id.clone(),
                        sender.clone()
                    )).is_ok(), "Only owner of the given network server id that the device is trying to connect to can set an associated roaming session join accept"
                );
            } else {
                // There must be a session join request associated with the session join accept
                return Err(DispatchError::Other("RoamingSessionJoinRequest does not exist"));
            }

            let session_join_request_accept_expiry = match _session_join_request_accept_expiry {
                Some(value) => value,
                None => Default::default() // Default
            };
            let session_join_request_accept_accepted_at_block = match _session_join_request_accept_accepted_at_block {
                Some(value) => value,
                None => Default::default()
            };

            // Check if a roaming session join accept already exists with the given roaming session id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_session_join_accept_index(roaming_session_id).is_ok() {
                info!("Mutating values");
                <RoamingSessionJoinAccepts<T>>::mutate(roaming_session_id, |session_join_accept| {
                    if let Some(_session_join_accept) = session_join_accept {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _session_join_accept.session_join_request_accept_expiry = session_join_request_accept_expiry.clone();
                        _session_join_accept.session_join_request_accept_accepted_at_block = session_join_request_accept_accepted_at_block.clone();
                    }
                });
                info!("Checking mutated values");
                let fetched_session_join_accept = <RoamingSessionJoinAccepts<T>>::get(roaming_session_id);
                if let Some(_session_join_accept) = fetched_session_join_accept {
                    info!("Latest field session_join_request_accept_expiry {:#?}", _session_join_accept.session_join_request_accept_expiry);
                    info!("Latest field session_join_request_accept_accepted_at_block {:#?}", _session_join_accept.session_join_request_accept_accepted_at_block);
                }
            } else {
                info!("Inserting values");

                // Create a new roaming session join accept instance with the input params
                let roaming_session_join_accept_instance = RoamingSessionJoinAccept {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    session_join_request_accept_expiry: session_join_request_accept_expiry.clone(),
                    session_join_request_accept_accepted_at_block: session_join_request_accept_accepted_at_block.clone()
                };

                <RoamingSessionJoinAccepts<T>>::insert(
                    roaming_session_id,
                    &roaming_session_join_accept_instance
                );

                info!("Checking inserted values");
                let fetched_session_join_accept = <RoamingSessionJoinAccepts<T>>::get(roaming_session_id);
                if let Some(_session_join_accept) = fetched_session_join_accept {
                    info!("Inserted field session_join_request_accept_expiry {:#?}", _session_join_accept.session_join_request_accept_expiry);
                    info!("Inserted field session_join_request_accept_accepted_at_block {:#?}", _session_join_accept.session_join_request_accept_accepted_at_block);
                }
            }

            Self::deposit_event(RawEvent::RoamingSessionJoinRequestAccepted(
                sender,
                roaming_session_id,
                session_join_request_accept_expiry,
                session_join_request_accept_accepted_at_block
            ));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_session_to_device(
            origin,
            roaming_session_id: T::RoamingSessionIndex,
            roaming_device_id: T::RoamingDeviceIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given device id already exists
            let is_roaming_device = <roaming_devices::Module<T>>
                ::exists_roaming_device(roaming_device_id).is_ok();
            ensure!(is_roaming_device, "RoamingDevice does not exist");

            // Ensure that caller of the function is the owner of the device id to assign the session to
            ensure!(
                <roaming_devices::Module<T>>::is_roaming_device_owner(roaming_device_id, sender.clone()).is_ok(),
                "Only the roaming device owner can assign itself a roaming session"
            );

            Self::associate_session_with_device(roaming_session_id, roaming_device_id)
                .expect("Unable to associate session with device");

            // Ensure that the given session id already exists
            let roaming_session = Self::roaming_session(roaming_session_id);
            ensure!(roaming_session.is_some(), "Invalid roaming_session_id");

            // Ensure that the session is not already owned by a different device
            // Unassign the session from any existing device since it may only be owned by one device
            <RoamingSessionDevices<T>>::remove(roaming_session_id);

            // Assign the session owner to the given device (even if already belongs to them)
            <RoamingSessionDevices<T>>::insert(roaming_session_id, roaming_device_id);

            Self::deposit_event(RawEvent::AssignedSessionToDevice(sender, roaming_session_id, roaming_device_id));
        }
    }
}

impl<T: Config> Module<T> {
    pub fn is_roaming_session_owner(
        roaming_session_id: T::RoamingSessionIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_session_owner(&roaming_session_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of RoamingSession"
        );
        Ok(())
    }

    pub fn exists_roaming_session(roaming_session_id: T::RoamingSessionIndex) -> Result<RoamingSession, DispatchError> {
        match Self::roaming_session(roaming_session_id) {
            Some(roaming_session) => Ok(roaming_session),
            None => Err(DispatchError::Other("RoamingSession does not exist")),
        }
    }

    pub fn exists_roaming_session_join_request(
        roaming_session_id: T::RoamingSessionIndex,
    ) -> Result<(), DispatchError> {
        match Self::roaming_session_join_requests(roaming_session_id) {
            Some(_) => Ok(()),
            None => Err(DispatchError::Other("RoamingSessionJoinRequest does not exist")),
        }
    }

    pub fn exists_roaming_session_join_accept(roaming_session_id: T::RoamingSessionIndex) -> Result<(), DispatchError> {
        match Self::roaming_session_join_accepts(roaming_session_id) {
            Some(_) => Ok(()),
            None => Err(DispatchError::Other("RoamingSessionJoinAccept does not exist")),
        }
    }

    pub fn has_value_for_session_join_request_index(
        roaming_session_id: T::RoamingSessionIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if session join request has a value that is defined");
        let fetched_session_join_request = <RoamingSessionJoinRequests<T>>::get(roaming_session_id);
        if let Some(_) = fetched_session_join_request {
            info!("Found value for session join request");
            return Ok(());
        }
        warn!("No value for session join request");
        Err(DispatchError::Other("No value for session join request"))
    }

    pub fn has_value_for_session_join_accept_index(
        roaming_session_id: T::RoamingSessionIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if session join accept has a value that is defined");
        let fetched_session_join_accept = <RoamingSessionJoinAccepts<T>>::get(roaming_session_id);
        if let Some(_) = fetched_session_join_accept {
            info!("Found value for session join accept");
            return Ok(());
        }
        warn!("No value for session join accept");
        Err(DispatchError::Other("No value for session join accept"))
    }

    /// Only push the session id onto the end of the vector if it does not already exist
    pub fn associate_session_with_device(
        roaming_session_id: T::RoamingSessionIndex,
        roaming_device_id: T::RoamingDeviceIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given device id already exists as a key,
        // and where its corresponding value is a vector that already contains the given session id
        if let Some(device_sessions) = Self::roaming_device_sessions(roaming_device_id) {
            info!("Device id key {:?} exists with value {:?}", roaming_device_id, device_sessions);
            let not_device_contains_session = !device_sessions.contains(&roaming_session_id);
            ensure!(not_device_contains_session, "Device already contains the given session id");
            info!("Device id key exists but its vector value does not contain the given session id");
            <RoamingDeviceSessions<T>>::mutate(roaming_device_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_session_id);
                }
            });
            info!("Associated session {:?} with device {:?}", roaming_session_id, roaming_device_id);
            Ok(())
        } else {
            info!(
                "Device id key does not yet exist. Creating the device key {:?} and appending the session id {:?} to \
                 its vector value",
                roaming_device_id,
                roaming_session_id
            );
            <RoamingDeviceSessions<T>>::insert(roaming_device_id, &vec![roaming_session_id]);
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

    fn next_roaming_session_id() -> Result<T::RoamingSessionIndex, DispatchError> {
        let roaming_session_id = Self::roaming_sessions_count();
        if roaming_session_id == <T::RoamingSessionIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingSessions count overflow"));
        }
        Ok(roaming_session_id)
    }

    fn insert_roaming_session(
        owner: &T::AccountId,
        roaming_session_id: T::RoamingSessionIndex,
        roaming_session: RoamingSession,
    ) {
        // Create and store roaming session
        <RoamingSessions<T>>::insert(roaming_session_id, roaming_session);
        <RoamingSessionsCount<T>>::put(roaming_session_id + One::one());
        <RoamingSessionOwners<T>>::insert(roaming_session_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_session_id: T::RoamingSessionIndex) {
        <RoamingSessionOwners<T>>::insert(roaming_session_id, to);
    }
}
