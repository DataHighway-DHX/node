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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config: frame_system::Config + roaming_operators::Config + roaming_network_servers::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RoamingServiceProfileIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingServiceProfileUplinkRate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingServiceProfileDownlinkRate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingServiceProfile(pub [u8; 16]);

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::RoamingServiceProfileIndex,
        <T as Config>::RoamingServiceProfileUplinkRate,
        <T as Config>::RoamingServiceProfileDownlinkRate,
        <T as roaming_network_servers::Config>::RoamingNetworkServerIndex,
    {
        /// A roaming service_profile is created. (owner, roaming_service_profile_id)
        Created(AccountId, RoamingServiceProfileIndex),
        /// A roaming service_profile is transferred. (from, to, roaming_service_profile_id)
        Transferred(AccountId, AccountId, RoamingServiceProfileIndex),
        /// A roaming service_profile is assigned an uplink rate. (owner, roaming_service_profile_id, uplink rate)
        UplinkRateSet(AccountId, RoamingServiceProfileIndex, Option<RoamingServiceProfileUplinkRate>),
        /// A roaming service_profile is assigned an downlink rate. (owner, roaming_service_profile_id, downlink rate)
        DownlinkRateSet(AccountId, RoamingServiceProfileIndex, Option<RoamingServiceProfileDownlinkRate>),
        /// A roaming service_profile is assigned to a network_server. (owner of network_server, roaming_service_profile_id, roaming_network_server_id)
        AssignedServiceProfileToNetworkServer(AccountId, RoamingServiceProfileIndex, RoamingNetworkServerIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as RoamingServiceProfiles {
        /// Stores all the roaming service_profiles, key is the roaming service_profile id / index
        pub RoamingServiceProfiles get(fn roaming_service_profile): map hasher(opaque_blake2_256) T::RoamingServiceProfileIndex => Option<RoamingServiceProfile>;

        /// Stores the total number of roaming service_profiles. i.e. the next roaming service_profile index
        pub RoamingServiceProfilesCount get(fn roaming_service_profiles_count): T::RoamingServiceProfileIndex;

        /// Get roaming service_profile owner
        pub RoamingServiceProfileOwners get(fn roaming_service_profile_owner): map hasher(opaque_blake2_256) T::RoamingServiceProfileIndex => Option<T::AccountId>;

        /// Get roaming service_profile uplink rate.
        pub RoamingServiceProfileUplinkRates get(fn roaming_service_profile_uplink_rate): map hasher(opaque_blake2_256) T::RoamingServiceProfileIndex => Option<T::RoamingServiceProfileUplinkRate>;

        /// Get roaming service_profile downlink rate.
        pub RoamingServiceProfileDownlinkRates get(fn roaming_service_profile_downlink_rate): map hasher(opaque_blake2_256) T::RoamingServiceProfileIndex => Option<T::RoamingServiceProfileDownlinkRate>;

        /// Get roaming service_profile network_server
        pub RoamingServiceProfileNetworkServer get(fn roaming_service_profile_network_server): map hasher(opaque_blake2_256) T::RoamingServiceProfileIndex => Option<T::RoamingNetworkServerIndex>;

        /// Get roaming network_server service_profiles
        pub RoamingNetworkServerServiceProfiles get(fn roaming_network_server_service_profiles): map hasher(opaque_blake2_256) T::RoamingNetworkServerIndex => Option<Vec<T::RoamingServiceProfileIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming service_profile
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_service_profile_id = Self::next_roaming_service_profile_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming service_profile
            let roaming_service_profile = RoamingServiceProfile(unique_id);
            Self::insert_roaming_service_profile(&sender, roaming_service_profile_id, roaming_service_profile);

            Self::deposit_event(RawEvent::Created(sender, roaming_service_profile_id));
        }

        /// Transfer a roaming service_profile to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_service_profile_id: T::RoamingServiceProfileIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_service_profile_owner(roaming_service_profile_id) == Some(sender.clone()), "Only owner can transfer roaming service_profile");

            Self::update_owner(&to, roaming_service_profile_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_service_profile_id));
        }

        /// Set uplink_rate for a roaming service_profile
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_uplink_rate(origin, roaming_service_profile_id: T::RoamingServiceProfileIndex, uplink_rate: Option<T::RoamingServiceProfileUplinkRate>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_service_profile_owner(roaming_service_profile_id) == Some(sender.clone()), "Only owner can set uplink_rate for roaming service_profile");

            // let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_service_profile_id, sender.clone()).is_ok();
            // ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            if let Some(ref uplink_rate) = uplink_rate {
                <RoamingServiceProfileUplinkRates<T>>::insert(roaming_service_profile_id, uplink_rate);
            } else {
                <RoamingServiceProfileUplinkRates<T>>::remove(roaming_service_profile_id);
            }

            Self::deposit_event(RawEvent::UplinkRateSet(sender, roaming_service_profile_id, uplink_rate));
        }

        /// Set downlink_rate for a roaming service_profile
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_downlink_rate(origin, roaming_service_profile_id: T::RoamingServiceProfileIndex, downlink_rate: Option<T::RoamingServiceProfileDownlinkRate>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_service_profile_owner(roaming_service_profile_id) == Some(sender.clone()), "Only owner can set downlink_rate for roaming service_profile");

            // let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_service_profile_id, sender.clone()).is_ok();
            // ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            if let Some(ref downlink_rate) = downlink_rate {
                <RoamingServiceProfileDownlinkRates<T>>::insert(roaming_service_profile_id, downlink_rate);
            } else {
                <RoamingServiceProfileDownlinkRates<T>>::remove(roaming_service_profile_id);
            }

            Self::deposit_event(RawEvent::DownlinkRateSet(sender, roaming_service_profile_id, downlink_rate));
        }

        // Optional: Service Profile is assigned to Network (Roaming Base) Profile, which is associated with a network.
        // This is an override to associate it with a specific Network Server rather than entire networks.
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_service_profile_to_network_server(
            origin,
            roaming_service_profile_id: T::RoamingServiceProfileIndex,
            roaming_network_server_id: T::RoamingNetworkServerIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network_server id already exists
            let is_roaming_network_server = <roaming_network_servers::Module<T>>
                ::exists_roaming_network_server(roaming_network_server_id).is_ok();
            ensure!(is_roaming_network_server, "RoamingNetworkServer does not exist");

            // Ensure that caller of the function is the owner of the network_server id to assign the service_profile to
            ensure!(
                <roaming_network_servers::Module<T>>::is_roaming_network_server_owner(roaming_network_server_id, sender.clone()).is_ok(),
                "Only the roaming network_server owner can assign itself a roaming service_profile"
            );

            Self::associate_service_profile_with_network_server(roaming_service_profile_id, roaming_network_server_id)
                .expect("Unable to associate service_profile with network_server");

            // Ensure that the given service_profile id already exists
            let roaming_service_profile = Self::roaming_service_profile(roaming_service_profile_id);
            ensure!(roaming_service_profile.is_some(), "Invalid roaming_service_profile_id");

            // Ensure that the service_profile is not already owned by a different network_server
            // Unassign the service_profile from any existing network_server since it may only be owned by one network_server
            <RoamingServiceProfileNetworkServer<T>>::remove(roaming_service_profile_id);

            // Assign the service_profile owner to the given network_server (even if already belongs to them)
            <RoamingServiceProfileNetworkServer<T>>::insert(roaming_service_profile_id, roaming_network_server_id);

            Self::deposit_event(RawEvent::AssignedServiceProfileToNetworkServer(sender, roaming_service_profile_id, roaming_network_server_id));
        }
    }
}

impl<T: Config> Module<T> {
    pub fn exists_roaming_service_profile(
        roaming_service_profile_id: T::RoamingServiceProfileIndex,
    ) -> Result<RoamingServiceProfile, DispatchError> {
        match Self::roaming_service_profile(roaming_service_profile_id) {
            Some(roaming_service_profile) => Ok(roaming_service_profile),
            None => Err(DispatchError::Other("RoamingServiceProfile does not exist")),
        }
    }

    // pub fn is_owned_by_required_parent_relationship(roaming_service_profile_id: T::RoamingServiceProfileIndex,
    // sender: T::AccountId) -> Result<(), DispatchError> {     info!("Get the network_server id associated
    // with the network_server of the given service profile id");     let service_profile_network_server_id =
    // Self::roaming_service_profile_network_server(roaming_service_profile_id);

    //     if let Some(_service_profile_network_server_id) = service_profile_network_server_id {
    //         // Ensure that the caller is owner of the network_server id associated with the service profile
    //         ensure!((<roaming_network_servers::Module<T>>::is_roaming_network_server_owner(
    //                 _service_profile_network_server_id.clone(),
    //                 sender.clone()
    //             )).is_ok(), "Only owner of the network_server id associated with the given service profile can set an
    // associated roaming service profile config"         );
    //     } else {
    //         // There must be a network_server id associated with the service profile
    //         return Err(DispatchError::Other("RoamingServiceProfileNetworkServer does not exist"));
    //     }
    //     Ok(())
    // }

    /// Only push the service_profile id onto the end of the vector if it does not already exist
    pub fn associate_service_profile_with_network_server(
        roaming_service_profile_id: T::RoamingServiceProfileIndex,
        roaming_network_server_id: T::RoamingNetworkServerIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given network_server id already exists as a key,
        // and where its corresponding value is a vector that already contains the given service_profile id
        if let Some(network_server_service_profiles) =
            Self::roaming_network_server_service_profiles(roaming_network_server_id)
        {
            info!(
                "NetworkServer id key {:?} exists with value {:?}",
                roaming_network_server_id,
                network_server_service_profiles
            );
            let not_network_server_contains_service_profile =
                !network_server_service_profiles.contains(&roaming_service_profile_id);
            ensure!(
                not_network_server_contains_service_profile,
                "NetworkServer already contains the given service_profile id"
            );
            info!(
                "NetworkServer id key exists but its vector value does not contain the given service_profile id"
            );
            <RoamingNetworkServerServiceProfiles<T>>::mutate(roaming_network_server_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_service_profile_id);
                }
            });
            info!(
                "Associated service_profile {:?} with network_server {:?}",
                roaming_service_profile_id,
                roaming_network_server_id
            );
            Ok(())
        } else {
            info!(
                "NetworkServer id key does not yet exist. Creating the network_server key {:?} and appending the \
                 service_profile id {:?} to its vector value",
                roaming_network_server_id,
                roaming_service_profile_id
            );
            <RoamingNetworkServerServiceProfiles<T>>::insert(
                roaming_network_server_id,
                &vec![roaming_service_profile_id],
            );
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

    fn next_roaming_service_profile_id() -> Result<T::RoamingServiceProfileIndex, DispatchError> {
        let roaming_service_profile_id = Self::roaming_service_profiles_count();
        if roaming_service_profile_id == <T::RoamingServiceProfileIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingServiceProfiles count overflow"));
        }
        Ok(roaming_service_profile_id)
    }

    fn insert_roaming_service_profile(
        owner: &T::AccountId,
        roaming_service_profile_id: T::RoamingServiceProfileIndex,
        roaming_service_profile: RoamingServiceProfile,
    ) {
        // Create and store roaming service_profile
        <RoamingServiceProfiles<T>>::insert(roaming_service_profile_id, roaming_service_profile);
        <RoamingServiceProfilesCount<T>>::put(roaming_service_profile_id + One::one());
        <RoamingServiceProfileOwners<T>>::insert(roaming_service_profile_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_service_profile_id: T::RoamingServiceProfileIndex) {
        <RoamingServiceProfileOwners<T>>::insert(roaming_service_profile_id, to);
    }
}
