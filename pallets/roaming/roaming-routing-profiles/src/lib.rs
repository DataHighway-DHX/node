#![cfg_attr(not(feature = "std"), no_std)]

use log::{warn, info};
use codec::{
    Decode,
    Encode,
    MaxEncodedLen,
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
pub trait Config: frame_system::Config + roaming_operators::Config + roaming_devices::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RoamingRoutingProfileIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingRoutingProfileAppServer: Parameter + Member + Default;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingRoutingProfile(pub [u8; 16]);

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::RoamingRoutingProfileIndex,
        <T as Config>::RoamingRoutingProfileAppServer,
        <T as roaming_devices::Config>::RoamingDeviceIndex,
    {
        /// A roaming routing_profile is created. (owner, roaming_routing_profile_id)
        Created(AccountId, RoamingRoutingProfileIndex),
        /// A roaming routing_profile is transferred. (from, to, roaming_routing_profile_id)
        Transferred(AccountId, AccountId, RoamingRoutingProfileIndex),
        /// A roaming routing_profile is assigned an app server. (owner, roaming_routing_profile_id, app server)
        AppServerSet(AccountId, RoamingRoutingProfileIndex, Option<RoamingRoutingProfileAppServer>),
        /// A roaming routing_profile is assigned to a device. (owner of device, roaming_routing_profile_id, roaming_device_id)
        AssignedRoutingProfileToDevice(AccountId, RoamingRoutingProfileIndex, RoamingDeviceIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as RoamingRoutingProfiles {
        /// Stores all the roaming routing_profiles, key is the roaming routing_profile id / index
        pub RoamingRoutingProfiles get(fn roaming_routing_profile): map hasher(opaque_blake2_256) T::RoamingRoutingProfileIndex => Option<RoamingRoutingProfile>;

        /// Stores the total number of roaming routing_profiles. i.e. the next roaming routing_profile index
        pub RoamingRoutingProfilesCount get(fn roaming_routing_profiles_count): T::RoamingRoutingProfileIndex;

        /// Get roaming routing_profile owner
        pub RoamingRoutingProfileOwners get(fn roaming_routing_profile_owner): map hasher(opaque_blake2_256) T::RoamingRoutingProfileIndex => Option<T::AccountId>;

        /// Get roaming routing_profile app server.
        pub RoamingRoutingProfileAppServers get(fn roaming_routing_profile_app_server): map hasher(opaque_blake2_256) T::RoamingRoutingProfileIndex => Option<T::RoamingRoutingProfileAppServer>;

        /// Get roaming routing_profile device
        pub RoamingRoutingProfileDevices get(fn roaming_routing_profile_device): map hasher(opaque_blake2_256) T::RoamingRoutingProfileIndex => Option<T::RoamingDeviceIndex>;

        /// Get roaming device routing_profiles
        pub RoamingDeviceRoutingProfiles get(fn roaming_device_routing_profiles): map hasher(opaque_blake2_256) T::RoamingDeviceIndex => Option<Vec<T::RoamingRoutingProfileIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming routing_profile
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_routing_profile_id = Self::next_roaming_routing_profile_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming routing_profile
            let roaming_routing_profile = RoamingRoutingProfile(unique_id);
            Self::insert_roaming_routing_profile(&sender, roaming_routing_profile_id, roaming_routing_profile);

            Self::deposit_event(RawEvent::Created(sender, roaming_routing_profile_id));
        }

        /// Transfer a roaming routing_profile to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_routing_profile_id: T::RoamingRoutingProfileIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_routing_profile_owner(roaming_routing_profile_id) == Some(sender.clone()), "Only owner can transfer roaming routing_profile");

            Self::update_owner(&to, roaming_routing_profile_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_routing_profile_id));
        }

        /// Set app server for a roaming routing_profile
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_app_server(origin, roaming_routing_profile_id: T::RoamingRoutingProfileIndex, app_server: Option<T::RoamingRoutingProfileAppServer>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_routing_profile_owner(roaming_routing_profile_id) == Some(sender.clone()), "Only owner can set app server for roaming routing_profile");

            // let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_routing_profile_id, sender.clone()).is_ok();
            // ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            if let Some(ref app_server) = app_server {
                <RoamingRoutingProfileAppServers<T>>::insert(roaming_routing_profile_id, app_server);
            } else {
                <RoamingRoutingProfileAppServers<T>>::remove(roaming_routing_profile_id);
            }

            Self::deposit_event(RawEvent::AppServerSet(sender, roaming_routing_profile_id, app_server));
        }

        // Note: This is wrong, routing profile shouldn't be assigned to a device.
        // Instead it should be "optionally" be assigned to a network server, which is the "home" network server
        // of one or more devices. But we associated the routing profile with a network server when
        // we create a network (roaming base) profile.
        // pub fn assign_routing_profile_to_device(
        //     origin,
        //     roaming_routing_profile_id: T::RoamingRoutingProfileIndex,
        //     roaming_device_id: T::RoamingDeviceIndex
        // ) {
        //     let sender = ensure_signed(origin)?;

        //     // Ensure that the given device id already exists
        //     let is_roaming_device = <roaming_devices::Module<T>>
        //         ::exists_roaming_device(roaming_device_id).is_ok();
        //     ensure!(is_roaming_device, "RoamingDevice does not exist");

        //     // Ensure that caller of the function is the owner of the device id to assign the routing_profile to
        //     ensure!(
        //         <roaming_devices::Module<T>>::is_roaming_device_owner(roaming_device_id, sender.clone()).is_ok(),
        //         "Only the roaming device owner can assign itself a roaming routing_profile"
        //     );

        //     Self::associate_routing_profile_with_device(roaming_routing_profile_id, roaming_device_id)
        //         .expect("Unable to associate routing_profile with device");

        //     // Ensure that the given routing_profile id already exists
        //     let roaming_routing_profile = Self::roaming_routing_profile(roaming_routing_profile_id);
        //     ensure!(roaming_routing_profile.is_some(), "Invalid roaming_routing_profile_id");

        //     // Ensure that the routing_profile is not already owned by a different device
        //     // Unassign the routing_profile from any existing device since it may only be owned by one device
        //     <RoamingRoutingProfileDevices<T>>::remove(roaming_routing_profile_id);

        //     // Assign the routing_profile owner to the given device (even if already belongs to them)
        //     <RoamingRoutingProfileDevices<T>>::insert(roaming_routing_profile_id, roaming_device_id);

        //     Self::deposit_event(RawEvent::AssignedRoutingProfileToDevice(sender, roaming_routing_profile_id, roaming_device_id));
        // }
    }
}

impl<T: Config> Module<T> {
    pub fn exists_roaming_routing_profile(
        roaming_routing_profile_id: T::RoamingRoutingProfileIndex,
    ) -> Result<RoamingRoutingProfile, DispatchError> {
        match Self::roaming_routing_profile(roaming_routing_profile_id) {
            Some(roaming_routing_profile) => Ok(roaming_routing_profile),
            None => Err(DispatchError::Other("RoamingRoutingProfile does not exist")),
        }
    }

    // pub fn is_owned_by_required_parent_relationship(roaming_routing_profile_id: T::RoamingRoutingProfileIndex,
    // sender: T::AccountId) -> Result<(), DispatchError> {     info!("Get the device id associated with the
    // device of the given routing profile id");     let routing_profile_device_id =
    // Self::roaming_routing_profile_device(roaming_routing_profile_id);

    //     if let Some(_routing_profile_device_id) = routing_profile_device_id {
    //         // Ensure that the caller is owner of the device id associated with the routing profile
    //         ensure!((<roaming_devices::Module<T>>::is_roaming_device_owner(
    //                 _routing_profile_device_id.clone(),
    //                 sender.clone()
    //             )).is_ok(), "Only owner of the device id associated with the given routing profile can set an
    // associated roaming routing profile config"         );
    //     } else {
    //         // There must be a device id associated with the routing profile
    //         return Err(DispatchError::Other("RoamingRoutingProfileDevice does not exist"));
    //     }
    //     Ok(())
    // }

    /// Only push the routing_profile id onto the end of the vector if it does not already exist
    pub fn associate_routing_profile_with_device(
        roaming_routing_profile_id: T::RoamingRoutingProfileIndex,
        roaming_device_id: T::RoamingDeviceIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given device id already exists as a key,
        // and where its corresponding value is a vector that already contains the given routing_profile id
        if let Some(device_routing_profiles) = Self::roaming_device_routing_profiles(roaming_device_id) {
            info!("Device id key {:?} exists with value {:?}", roaming_device_id, device_routing_profiles);
            let not_device_contains_routing_profile = !device_routing_profiles.contains(&roaming_routing_profile_id);
            ensure!(not_device_contains_routing_profile, "Device already contains the given routing_profile id");
            info!("Device id key exists but its vector value does not contain the given routing_profile id");
            <RoamingDeviceRoutingProfiles<T>>::mutate(roaming_device_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_routing_profile_id);
                }
            });
            info!(
                "Associated routing_profile {:?} with device {:?}",
                roaming_routing_profile_id,
                roaming_device_id
            );
            Ok(())
        } else {
            info!(
                "Device id key does not yet exist. Creating the device key {:?} and appending the routing_profile id \
                 {:?} to its vector value",
                roaming_device_id,
                roaming_routing_profile_id
            );
            <RoamingDeviceRoutingProfiles<T>>::insert(roaming_device_id, &vec![roaming_routing_profile_id]);
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

    fn next_roaming_routing_profile_id() -> Result<T::RoamingRoutingProfileIndex, DispatchError> {
        let roaming_routing_profile_id = Self::roaming_routing_profiles_count();
        if roaming_routing_profile_id == <T::RoamingRoutingProfileIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingRoutingProfiles count overflow"));
        }
        Ok(roaming_routing_profile_id)
    }

    fn insert_roaming_routing_profile(
        owner: &T::AccountId,
        roaming_routing_profile_id: T::RoamingRoutingProfileIndex,
        roaming_routing_profile: RoamingRoutingProfile,
    ) {
        // Create and store roaming routing_profile
        <RoamingRoutingProfiles<T>>::insert(roaming_routing_profile_id, roaming_routing_profile);
        <RoamingRoutingProfilesCount<T>>::put(roaming_routing_profile_id + One::one());
        <RoamingRoutingProfileOwners<T>>::insert(roaming_routing_profile_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_routing_profile_id: T::RoamingRoutingProfileIndex) {
        <RoamingRoutingProfileOwners<T>>::insert(roaming_routing_profile_id, to);
    }
}
