#![cfg_attr(not(feature = "std"), no_std)]

use codec::{
    Decode,
    Encode,
};
use frame_support::{
    debug,
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
pub trait Trait:
    frame_system::Trait + roaming_operators::Trait + roaming_networks::Trait + roaming_devices::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type RoamingNetworkProfileIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingNetworkProfile(pub [u8; 16]);

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::RoamingNetworkProfileIndex,
        <T as roaming_networks::Trait>::RoamingNetworkIndex,
        <T as roaming_operators::Trait>::RoamingOperatorIndex,
        <T as roaming_devices::Trait>::RoamingDeviceIndex,
    {
        /// A roaming network_profile is created. (owner, roaming_network_profile_id)
        Created(AccountId, RoamingNetworkProfileIndex),
        /// A roaming network_profile is transferred. (from, to, roaming_network_profile_id)
        Transferred(AccountId, AccountId, RoamingNetworkProfileIndex),
        /// A roaming network_profile restricted access to any devices
        RoamingNetworkProfileDeviceAccessAllowedSet(AccountId, RoamingNetworkProfileIndex, bool),
        /// A roaming network_profile whitelisted network for visiting devices was added
        AddedRoamingNetworkProfileWhitelistedNetwork(AccountId, RoamingNetworkProfileIndex, RoamingNetworkIndex),
        /// A roaming network_profile whitelisted network for visiting devices was removed
        RemovedRoamingNetworkProfileWhitelistedNetwork(AccountId, RoamingNetworkProfileIndex, RoamingNetworkIndex),
        /// A roaming network_profile blacklisted device for visiting devices was added
        AddedRoamingNetworkProfileBlacklistedDevice(AccountId, RoamingNetworkProfileIndex, RoamingDeviceIndex),
        /// A roaming network_profile blacklisted device for visiting devices was removed
        RemovedRoamingNetworkProfileBlacklistedDevice(AccountId, RoamingNetworkProfileIndex, RoamingDeviceIndex),
        /// A roaming network_profile is assigned to a network. (owner of network, roaming_network_profile_id, roaming_network_id)
        AssignedNetworkProfileToNetwork(AccountId, RoamingNetworkProfileIndex, RoamingNetworkIndex),
        /// A roaming network_profile is assigned to an operator. (owner of network, roaming_network_profile_id, roaming_operator_id)
        AssignedNetworkProfileToOperator(AccountId, RoamingNetworkProfileIndex, RoamingOperatorIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingNetworkProfiles {
        /// Stores all the roaming network_profiles, key is the roaming network_profile id / index
        pub RoamingNetworkProfiles get(fn roaming_network_profile): map hasher(opaque_blake2_256) T::RoamingNetworkProfileIndex => Option<RoamingNetworkProfile>;

        /// Stores the total number of roaming network_profiles. i.e. the next roaming network_profile index
        pub RoamingNetworkProfilesCount get(fn roaming_network_profiles_count): T::RoamingNetworkProfileIndex;

        /// Get roaming network_profile owner
        pub RoamingNetworkProfileOwners get(fn roaming_network_profile_owner): map hasher(opaque_blake2_256) T::RoamingNetworkProfileIndex => Option<T::AccountId>;

        /// Get roaming network_policy status of whether any device visitors are allowed to roam at all
        pub RoamingNetworkProfileDeviceAccessAllowed get(fn roaming_network_profile_restricted_access): map hasher(opaque_blake2_256) T::RoamingNetworkProfileIndex => Option<bool>;

        /// Get roaming network_policy whitelisted networks of visiting devices
        pub RoamingNetworkProfileWhitelistedNetworks get(fn roaming_network_profile_whitelisted_networks): map hasher(opaque_blake2_256) T::RoamingNetworkProfileIndex => Option<Vec<T::RoamingNetworkIndex>>;

        /// Get roaming network_policy blacklisted devices of that are visiting
        pub RoamingNetworkProfileBlacklistedDevices get(fn roaming_network_profile_blacklisted_devices): map hasher(opaque_blake2_256) T::RoamingNetworkProfileIndex => Option<Vec<T::RoamingDeviceIndex>>;

        /// Get roaming network_profile network
        pub RoamingNetworkProfileNetwork get(fn roaming_network_profile_network): map hasher(opaque_blake2_256) T::RoamingNetworkProfileIndex => Option<T::RoamingNetworkIndex>;

        /// Get roaming network_profile operators
        pub RoamingNetworkProfileOperator get(fn roaming_network_profile_operators): map hasher(opaque_blake2_256) T::RoamingNetworkProfileIndex => Option<T::RoamingOperatorIndex>;

        /// Get roaming network's network profiles
        pub RoamingNetworkNetworkProfiles get(fn roaming_network_network_profiles): map hasher(opaque_blake2_256) T::RoamingNetworkIndex => Option<Vec<T::RoamingNetworkProfileIndex>>;

        /// Get roaming operator's network profiles
        pub RoamingOperatorNetworkProfiles get(fn roaming_operator_network_profiles): map hasher(opaque_blake2_256) T::RoamingOperatorIndex => Option<Vec<T::RoamingNetworkProfileIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming network_profile
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_network_profile_id = Self::next_roaming_network_profile_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming network_profile
            let roaming_network_profile = RoamingNetworkProfile(unique_id);
            Self::insert_roaming_network_profile(&sender, roaming_network_profile_id, roaming_network_profile);

            Self::deposit_event(RawEvent::Created(sender, roaming_network_profile_id));
        }

        /// Transfer a roaming network_profile to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_network_profile_id: T::RoamingNetworkProfileIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_network_profile_owner(roaming_network_profile_id) == Some(sender.clone()), "Only owner can transfer roaming network_profile");

            Self::update_owner(&to, roaming_network_profile_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_network_profile_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_device_access_allowed(origin, roaming_network_profile_id: T::RoamingNetworkProfileIndex, device_access_allowed: bool) {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming network_profile id actually exists
            let is_roaming_network_profile = Self::exists_roaming_network_profile(roaming_network_profile_id).is_ok();
            ensure!(is_roaming_network_profile, "RoamingNetworkProfile does not exist");

            // Ensure that the caller is owner of the network_profile whitelisted network they are trying to change
            ensure!(Self::roaming_network_profile_owner(roaming_network_profile_id) == Some(sender.clone()), "Only owner can set whitelisted network for roaming network_profile");

            let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_network_profile_id, sender.clone()).is_ok();
            ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            <RoamingNetworkProfileDeviceAccessAllowed<T>>::insert(
                roaming_network_profile_id,
                &device_access_allowed
            );

            Self::deposit_event(RawEvent::RoamingNetworkProfileDeviceAccessAllowedSet(sender, roaming_network_profile_id, device_access_allowed));
        }

        /// Add roaming network_profile whitelisted network
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn add_whitelisted_network(
            origin,
            roaming_network_profile_id: T::RoamingNetworkProfileIndex,
            roaming_network_id: T::RoamingNetworkIndex
        ) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming network_profile id whose config we want to change actually exists
            let is_roaming_network_profile = Self::exists_roaming_network_profile(roaming_network_profile_id).is_ok();
            ensure!(is_roaming_network_profile, "RoamingNetworkProfile does not exist");

            // Ensure that the caller is owner of the network_profile whitelisted network they are trying to change
            ensure!(Self::roaming_network_profile_owner(roaming_network_profile_id) == Some(sender.clone()), "Only owner can set whitelisted network for roaming network_profile");

            // Ensure that the given network id already exists
            let is_roaming_network = <roaming_networks::Module<T>>
                ::exists_roaming_network(roaming_network_id).is_ok();
            ensure!(is_roaming_network, "RoamingNetwork does not exist");

            let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_network_profile_id, sender.clone()).is_ok();
            ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            let mut fetched_whitelisted_networks;

            // Check roaming network_profile whitelisted network vector already exists with the given roaming network_profile id
            // to determine whether to insert new or mutate existing.
            debug::info!("Checking if vector of whitelisted networks is defined");
            if Self::has_value_for_network_profile_whitelisted_networks(roaming_network_profile_id).is_ok() {
                debug::info!("Checking if whitelisted network id already exists to mutate its value in the vector");

                debug::info!("Getting vector of whitelisted networks");
                fetched_whitelisted_networks = <RoamingNetworkProfileWhitelistedNetworks<T>>::get(roaming_network_profile_id);

                if let Some(whitelisted_networks) = fetched_whitelisted_networks {
                    debug::info!("Search for element in vector of whitelisted networks that matches the network_id provided");
                    if whitelisted_networks.contains(&roaming_network_id) {
                        debug::info!("Provided network_id is already a whitelisted network");
                        return Err(DispatchError::Other("Provided network_id is already a whitelisted network"));
                    }

                    // If it doesn't exist, but we still already have a vector with other whitelisted networks
                    // then we'll append the new whitelisted network to the end of the vector
                    let next_index = whitelisted_networks.len() - 1;
                    debug::info!("Updating whitelisted networks by appending a new whitelisted network at next_index {:?}: ", next_index);

                    <RoamingNetworkProfileWhitelistedNetworks<T>>::mutate(roaming_network_profile_id, |v| {
                        if let Some(value) = v {
                            value.push(roaming_network_id);
                        }
                    });

                    debug::info!("Appended whitelisted network");

                    debug::info!("Checking inserted values");
                    fetched_whitelisted_networks = <RoamingNetworkProfileWhitelistedNetworks<T>>::get(roaming_network_profile_id);

                    if let Some(_whitelisted_networks) = fetched_whitelisted_networks {
                        debug::info!("Inserted field roaming_network_id {:#?}", _whitelisted_networks);
                    }
                }
            } else {
                debug::info!("Inserting new vector with the whitelisted network provided since no vector value is defined");

                let mut new_whitelisted_networks = Vec::new();
                new_whitelisted_networks.push(roaming_network_id);

                <RoamingNetworkProfileWhitelistedNetworks<T>>::insert(
                    roaming_network_profile_id,
                    &new_whitelisted_networks
                );

                debug::info!("Checking inserted values");
                fetched_whitelisted_networks = <RoamingNetworkProfileWhitelistedNetworks<T>>::get(roaming_network_profile_id);

                if let Some(_whitelisted_networks) = fetched_whitelisted_networks {
                    // Get the whitelisted network at the 0 index that was inserted
                    if let Some (_whitelisted_network) = _whitelisted_networks.get(0) {
                        debug::info!("Inserted field roaming_network_id {:#?}", _whitelisted_network);
                    }
                }
            }

            Self::deposit_event(RawEvent::AddedRoamingNetworkProfileWhitelistedNetwork(
                sender,
                roaming_network_profile_id,
                roaming_network_id
            ));

            Ok(())
        }

        /// Add roaming network_profile whitelisted network
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn remove_whitelisted_network(
            origin,
            roaming_network_profile_id: T::RoamingNetworkProfileIndex,
            roaming_network_id: T::RoamingNetworkIndex
        ) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming network_profile id whose config we want to change actually exists
            let is_roaming_network_profile = Self::exists_roaming_network_profile(roaming_network_profile_id).is_ok();
            ensure!(is_roaming_network_profile, "RoamingNetworkProfile does not exist");

            // Ensure that the caller is owner of the network_profile whitelisted network they are trying to change
            ensure!(Self::roaming_network_profile_owner(roaming_network_profile_id) == Some(sender.clone()), "Only owner can set whitelisted network for roaming network_profile");

            let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_network_profile_id, sender.clone()).is_ok();
            ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            let mut fetched_whitelisted_networks;

            // Check roaming network_profile whitelisted network vector already exists with the given roaming network_profile id
            // to determine whether to insert new or mutate existing.
            debug::info!("Checking if vector of whitelisted networks is defined");
            if Self::has_value_for_network_profile_whitelisted_networks(roaming_network_profile_id).is_ok() {
                debug::info!("Checking if whitelisted network id already exists to mutate its value in the vector");

                debug::info!("Getting vector of whitelisted networks");
                fetched_whitelisted_networks = <RoamingNetworkProfileWhitelistedNetworks<T>>::get(roaming_network_profile_id);

                if let Some(whitelisted_networks) = fetched_whitelisted_networks {
                    let mut found_index;

                    debug::info!("Search for element in vector of whitelisted networks that matches the network_id provided");
                    for (index, whitelisted_network) in whitelisted_networks.iter().enumerate() {
                        if whitelisted_network == &roaming_network_id {
                            found_index = index;

                            debug::info!("Provided network_id is already a whitelisted network at index {:?}", found_index);
                            debug::info!("Removing whitelisted network at index {:?}: ", found_index);

                            <RoamingNetworkProfileWhitelistedNetworks<T>>::mutate(roaming_network_profile_id, |v| {
                                if let Some(value) = v {
                                    // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.remove
                                    value.remove(found_index);
                                }
                            });

                            debug::info!("Removed whitelisted network");

                            debug::info!("Checking inserted values");
                            fetched_whitelisted_networks = <RoamingNetworkProfileWhitelistedNetworks<T>>::get(roaming_network_profile_id);

                            if let Some(_whitelisted_networks) = fetched_whitelisted_networks {
                                debug::info!("Removed field roaming_network_id {:#?}", _whitelisted_networks);
                            }
                        }
                    }
                }
            }

            Self::deposit_event(RawEvent::RemovedRoamingNetworkProfileWhitelistedNetwork(
                sender,
                roaming_network_profile_id,
                roaming_network_id
            ));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn add_blacklisted_device(
            origin,
            roaming_network_profile_id: T::RoamingNetworkProfileIndex,
            roaming_device_id: T::RoamingDeviceIndex
        ) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming network_profile id whose config we want to change actually exists
            let is_roaming_network_profile = Self::exists_roaming_network_profile(roaming_network_profile_id).is_ok();
            ensure!(is_roaming_network_profile, "RoamingNetworkProfile does not exist");

            // Ensure that the caller is owner of the network_profile blacklisted device they are trying to change
            ensure!(Self::roaming_network_profile_owner(roaming_network_profile_id) == Some(sender.clone()), "Only owner can set blacklisted device for roaming network_profile");

            // Ensure that the given network id already exists
            let is_roaming_device = <roaming_devices::Module<T>>
                ::exists_roaming_device(roaming_device_id).is_ok();
            ensure!(is_roaming_device, "RoamingDevice does not exist");

            let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_network_profile_id, sender.clone()).is_ok();
            ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            let mut fetched_blacklisted_devices;

            // Check roaming network_profile blacklisted device vector already exists with the given roaming network_profile id
            // to determine whether to insert new or mutate existing.
            debug::info!("Checking if vector of blacklisted devices is defined");
            if Self::has_value_for_network_profile_blacklisted_devices(roaming_network_profile_id).is_ok() {
                debug::info!("Checking if blacklisted device id already exists to mutate its value in the vector");

                debug::info!("Getting vector of blacklisted devices");
                fetched_blacklisted_devices = <RoamingNetworkProfileBlacklistedDevices<T>>::get(roaming_network_profile_id);

                if let Some(blacklisted_devices) = fetched_blacklisted_devices {
                    debug::info!("Search for element in vector of blacklisted devices that matches the network_id provided");
                    if blacklisted_devices.contains(&roaming_device_id) {
                        debug::info!("Provided network_id is already a blacklisted device");
                        return Err(DispatchError::Other("Provided network_id is already a blacklisted device"));
                    }

                    // If it doesn't exist, but we still already have a vector with other blacklisted devices
                    // then we'll append the new blacklisted device to the end of the vector
                    let next_index = blacklisted_devices.len() - 1;
                    debug::info!("Updating blacklisted devices by appending a new blacklisted device at next_index {:?}: ", next_index);

                    <RoamingNetworkProfileBlacklistedDevices<T>>::mutate(roaming_network_profile_id, |v| {
                        if let Some(value) = v {
                            value.push(roaming_device_id);
                        }
                    });

                    debug::info!("Appended blacklisted device");

                    debug::info!("Checking inserted values");
                    fetched_blacklisted_devices = <RoamingNetworkProfileBlacklistedDevices<T>>::get(roaming_network_profile_id);

                    if let Some(_blacklisted_devices) = fetched_blacklisted_devices {
                        debug::info!("Inserted field roaming_device_id {:#?}", _blacklisted_devices);
                    }
                }
            } else {
                debug::info!("Inserting new vector with the blacklisted device provided since no vector value is defined");

                let mut new_blacklisted_devices = Vec::new();
                new_blacklisted_devices.push(roaming_device_id);

                <RoamingNetworkProfileBlacklistedDevices<T>>::insert(
                    roaming_network_profile_id,
                    &new_blacklisted_devices
                );

                debug::info!("Checking inserted values");
                fetched_blacklisted_devices = <RoamingNetworkProfileBlacklistedDevices<T>>::get(roaming_network_profile_id);

                if let Some(_blacklisted_devices) = fetched_blacklisted_devices {
                    // Get the blacklisted device at the 0 index that was inserted
                    if let Some (_blacklisted_device) = _blacklisted_devices.get(0) {
                        debug::info!("Inserted field roaming_device_id {:#?}", _blacklisted_device);
                    }
                }
            }

            Self::deposit_event(RawEvent::AddedRoamingNetworkProfileBlacklistedDevice(
                sender,
                roaming_network_profile_id,
                roaming_device_id
            ));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn remove_blacklisted_device(
            origin,
            roaming_network_profile_id: T::RoamingNetworkProfileIndex,
            roaming_device_id: T::RoamingDeviceIndex
        ) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming network_profile id whose config we want to change actually exists
            let is_roaming_network_profile = Self::exists_roaming_network_profile(roaming_network_profile_id).is_ok();
            ensure!(is_roaming_network_profile, "RoamingNetworkProfile does not exist");

            // Ensure that the caller is owner of the network_profile blacklisted device they are trying to change
            ensure!(Self::roaming_network_profile_owner(roaming_network_profile_id) == Some(sender.clone()), "Only owner can set blacklisted device for roaming network_profile");

            let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_network_profile_id, sender.clone()).is_ok();
            ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            let mut fetched_blacklisted_devices;

            // Check roaming network_profile blacklisted device vector already exists with the given roaming network_profile id
            // to determine whether to insert new or mutate existing.
            debug::info!("Checking if vector of blacklisted devices is defined");
            if Self::has_value_for_network_profile_blacklisted_devices(roaming_network_profile_id).is_ok() {
                debug::info!("Checking if blacklisted device id already exists to mutate its value in the vector");

                debug::info!("Getting vector of blacklisted devices");
                fetched_blacklisted_devices = <RoamingNetworkProfileBlacklistedDevices<T>>::get(roaming_network_profile_id);

                if let Some(blacklisted_devices) = fetched_blacklisted_devices {
                    let mut found_index;

                    debug::info!("Search for element in vector of blacklisted devices that matches the network_id provided");
                    for (index, blacklisted_device) in blacklisted_devices.iter().enumerate() {
                        if blacklisted_device == &roaming_device_id {
                            found_index = index;

                            debug::info!("Provided network_id is already a blacklisted device at index {:?}", found_index);
                            debug::info!("Removing blacklisted device at index {:?}: ", found_index);

                            <RoamingNetworkProfileBlacklistedDevices<T>>::mutate(roaming_network_profile_id, |v| {
                                if let Some(value) = v {
                                    // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.remove
                                    value.remove(found_index);
                                }
                            });

                            debug::info!("Removed blacklisted device");

                            debug::info!("Checking inserted values");
                            fetched_blacklisted_devices = <RoamingNetworkProfileBlacklistedDevices<T>>::get(roaming_network_profile_id);

                            if let Some(_blacklisted_devices) = fetched_blacklisted_devices {
                                debug::info!("Removed field roaming_device_id {:#?}", _blacklisted_devices);
                            }
                        }
                    }
                }
            }

            Self::deposit_event(RawEvent::RemovedRoamingNetworkProfileBlacklistedDevice(
                sender,
                roaming_network_profile_id,
                roaming_device_id
            ));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_network_profile_to_network(
            origin,
            roaming_network_profile_id: T::RoamingNetworkProfileIndex,
            roaming_network_id: T::RoamingNetworkIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_network = <roaming_networks::Module<T>>
                ::exists_roaming_network(roaming_network_id).is_ok();
            ensure!(is_roaming_network, "RoamingNetwork does not exist");

            // Ensure that caller of the function is the owner of the network id to assign the network_profile to
            ensure!(
                <roaming_networks::Module<T>>::is_roaming_network_owner(roaming_network_id, sender.clone()).is_ok(),
                "Only the roaming network owner can assign itself a roaming network profile"
            );

            Self::associate_network_profile_with_network(roaming_network_profile_id, roaming_network_id)
                .expect("Unable to associate network profile with network");

            // Ensure that the given network_profile id already exists
            let roaming_network_profile = Self::roaming_network_profile(roaming_network_profile_id);
            ensure!(roaming_network_profile.is_some(), "Invalid roaming_network_profile_id");

            // Ensure that the network_profile is not already owned by a different network
            // Unassign the network_profile from any existing network since it may only be owned by one network
            <RoamingNetworkProfileNetwork<T>>::remove(roaming_network_profile_id);

            // Assign the network_profile owner to the given network (even if already belongs to them)
            <RoamingNetworkProfileNetwork<T>>::insert(roaming_network_profile_id, roaming_network_id);

            Self::deposit_event(RawEvent::AssignedNetworkProfileToNetwork(sender, roaming_network_profile_id, roaming_network_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_network_profile_to_operator(
            origin,
            roaming_network_profile_id: T::RoamingNetworkProfileIndex,
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
                "Only the roaming operator owner can assign itself a roaming network profile"
            );

            Self::associate_network_profile_with_operator(roaming_network_profile_id, roaming_operator_id)
                .expect("Unable to associate network profile with operator");

            // Ensure that the given network_profile id already exists
            let roaming_network_profile = Self::roaming_network_profile(roaming_network_profile_id);
            ensure!(roaming_network_profile.is_some(), "Invalid roaming_network_profile_id");

            // Ensure that the network_profile is not already owned by a different operator
            // Unassign the network_profile from any existing operator since it may only be owned by one operator
            <RoamingNetworkProfileOperator<T>>::remove(roaming_network_profile_id);

            // Assign the network_profile owner to the given operator (even if already belongs to them)
            <RoamingNetworkProfileOperator<T>>::insert(roaming_network_profile_id, roaming_operator_id);

            Self::deposit_event(RawEvent::AssignedNetworkProfileToOperator(sender, roaming_network_profile_id, roaming_operator_id));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_roaming_network_profile_owner(
        roaming_network_profile_id: T::RoamingNetworkProfileIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_network_profile_owner(&roaming_network_profile_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of RoamingNetworkProfile"
        );
        Ok(())
    }

    pub fn is_owned_by_required_parent_relationship(
        roaming_network_profile_id: T::RoamingNetworkProfileIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        debug::info!("Get the network id associated with the network of the given network profile id");
        let network_profile_network_id = Self::roaming_network_profile_network(roaming_network_profile_id);

        if let Some(_network_profile_network_id) = network_profile_network_id {
            // Ensure that the caller is owner of the network id associated with the network profile
            ensure!(
                (<roaming_networks::Module<T>>::is_roaming_network_owner(
                    _network_profile_network_id.clone(),
                    sender.clone()
                ))
                .is_ok(),
                "Only owner of the network id associated with the given network profile can set an associated roaming \
                 network profile config"
            );
        } else {
            // There must be a network id associated with the network profile
            return Err(DispatchError::Other("RoamingNetworkProfileNetwork does not exist"));
        }
        Ok(())
    }

    pub fn exists_roaming_network_profile(
        roaming_network_profile_id: T::RoamingNetworkProfileIndex,
    ) -> Result<RoamingNetworkProfile, DispatchError> {
        match Self::roaming_network_profile(roaming_network_profile_id) {
            Some(roaming_network_profile) => Ok(roaming_network_profile),
            None => Err(DispatchError::Other("RoamingNetworkProfile does not exist")),
        }
    }

    pub fn has_value_for_network_profile_whitelisted_networks(
        roaming_network_profile_id: T::RoamingNetworkProfileIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if network_profile whitelisted network has a value that is defined");
        let fetched_network_profile_whitelisted_network =
            <RoamingNetworkProfileWhitelistedNetworks<T>>::get(roaming_network_profile_id);
        if let Some(_value) = fetched_network_profile_whitelisted_network {
            debug::info!("Found value for network_profile whitelisted network");
            return Ok(());
        }
        debug::info!("No value for network_profile whitelisted network");
        Err(DispatchError::Other("No value for network_profile whitelisted network"))
    }

    pub fn has_value_for_network_profile_blacklisted_devices(
        roaming_network_profile_id: T::RoamingNetworkProfileIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if network_profile blacklisted_devices has a value that is defined");
        let fetched_network_profile_blacklisted_devices =
            <RoamingNetworkProfileBlacklistedDevices<T>>::get(roaming_network_profile_id);
        if let Some(_value) = fetched_network_profile_blacklisted_devices {
            debug::info!("Found value for network_profile blacklisted_devices");
            return Ok(());
        }
        debug::info!("No value for network_profile blacklisted_devices");
        Err(DispatchError::Other("No value for network_profile blacklisted_devices"))
    }

    /// Only push the network profile id onto the end of the vector if it does not already exist
    pub fn associate_network_profile_with_network(
        roaming_network_profile_id: T::RoamingNetworkProfileIndex,
        roaming_network_id: T::RoamingNetworkIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given network id already exists as a key,
        // and where its corresponding value is a vector that already contains the given network profile id
        if let Some(network_network_profiles) = Self::roaming_network_network_profiles(roaming_network_id) {
            debug::info!("Network id key {:?} exists with value {:?}", roaming_network_id, network_network_profiles);
            let not_network_contains_network_profile = !network_network_profiles.contains(&roaming_network_profile_id);
            ensure!(not_network_contains_network_profile, "Network already contains the given network profile id");
            debug::info!("Network id key exists but its vector value does not contain the given network profile id");
            <RoamingNetworkNetworkProfiles<T>>::mutate(roaming_network_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_network_profile_id);
                }
            });
            debug::info!(
                "Associated network profile {:?} with network {:?}",
                roaming_network_profile_id,
                roaming_network_id
            );
            Ok(())
        } else {
            debug::info!(
                "Network id key does not yet exist. Creating the network key {:?} and appending the network profile \
                 id {:?} to its vector value",
                roaming_network_id,
                roaming_network_profile_id
            );
            <RoamingNetworkNetworkProfiles<T>>::insert(roaming_network_id, &vec![roaming_network_profile_id]);
            Ok(())
        }
    }

    /// Only push the network profile id onto the end of the vector if it does not already exist
    pub fn associate_network_profile_with_operator(
        roaming_network_profile_id: T::RoamingNetworkProfileIndex,
        roaming_operator_id: T::RoamingOperatorIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given operator id already exists as a key,
        // and where its corresponding value is a vector that already contains the given network profile id
        if let Some(operator_network_profiles) = Self::roaming_operator_network_profiles(roaming_operator_id) {
            debug::info!("Operator id key {:?} exists with value {:?}", roaming_operator_id, operator_network_profiles);
            let not_operator_contains_network_profile =
                !operator_network_profiles.contains(&roaming_network_profile_id);
            ensure!(not_operator_contains_network_profile, "Operator already contains the given network profile id");
            debug::info!("Operator id key exists but its vector value does not contain the given network profile id");
            <RoamingOperatorNetworkProfiles<T>>::mutate(roaming_operator_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_network_profile_id);
                }
            });
            debug::info!(
                "Associated network profile {:?} with operator {:?}",
                roaming_network_profile_id,
                roaming_operator_id
            );
            Ok(())
        } else {
            debug::info!(
                "Operator id key does not yet exist. Creating the operator key {:?} and appending the network profile \
                 id {:?} to its vector value",
                roaming_operator_id,
                roaming_network_profile_id
            );
            <RoamingOperatorNetworkProfiles<T>>::insert(roaming_operator_id, &vec![roaming_network_profile_id]);
            Ok(())
        }
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            <T as roaming_operators::Trait>::Randomness::random(&[0]),
            sender,
            <frame_system::Module<T>>::extrinsic_index(),
            <frame_system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_roaming_network_profile_id() -> Result<T::RoamingNetworkProfileIndex, DispatchError> {
        let roaming_network_profile_id = Self::roaming_network_profiles_count();
        if roaming_network_profile_id == <T::RoamingNetworkProfileIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingNetworkProfiles count overflow"));
        }
        Ok(roaming_network_profile_id)
    }

    fn insert_roaming_network_profile(
        owner: &T::AccountId,
        roaming_network_profile_id: T::RoamingNetworkProfileIndex,
        roaming_network_profile: RoamingNetworkProfile,
    ) {
        // Create and store roaming network_profile
        <RoamingNetworkProfiles<T>>::insert(roaming_network_profile_id, roaming_network_profile);
        <RoamingNetworkProfilesCount<T>>::put(roaming_network_profile_id + One::one());
        <RoamingNetworkProfileOwners<T>>::insert(roaming_network_profile_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_network_profile_id: T::RoamingNetworkProfileIndex) {
        <RoamingNetworkProfileOwners<T>>::insert(roaming_network_profile_id, to);
    }
}
