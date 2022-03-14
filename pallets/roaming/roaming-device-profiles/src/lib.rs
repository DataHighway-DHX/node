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
pub trait Config: frame_system::Config + roaming_operators::Config + roaming_devices::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RoamingDeviceProfileIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingDeviceProfileDevAddr: Parameter + Member + Default;
    type RoamingDeviceProfileDevEUI: Parameter + Member + Default;
    type RoamingDeviceProfileJoinEUI: Parameter + Member + Default;
    type RoamingDeviceProfileVendorID: Parameter + Member + Default;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingDeviceProfile(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
// Generic type parameters - Balance
pub struct RoamingDeviceProfileSetting<U, V, W, X> {
    pub device_profile_devaddr: U,
    pub device_profile_deveui: V,
    pub device_profile_joineui: W,
    pub device_profile_vendorid: X,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::RoamingDeviceProfileIndex,
        <T as Config>::RoamingDeviceProfileDevAddr,
        <T as Config>::RoamingDeviceProfileDevEUI,
        <T as Config>::RoamingDeviceProfileJoinEUI,
        <T as Config>::RoamingDeviceProfileVendorID,
        <T as roaming_devices::Config>::RoamingDeviceIndex,
    {
        /// A roaming device_profile is created. (owner, roaming_device_profile_id)
        Created(AccountId, RoamingDeviceProfileIndex),
        /// A roaming device_profile is transferred. (from, to, roaming_device_profile_id)
        Transferred(AccountId, AccountId, RoamingDeviceProfileIndex),
        /// A roaming device_profile configuration
        RoamingDeviceProfileSettingSet(AccountId, RoamingDeviceProfileIndex, RoamingDeviceProfileDevAddr, RoamingDeviceProfileDevEUI, RoamingDeviceProfileJoinEUI, RoamingDeviceProfileVendorID),
        /// A roaming device_profile is assigned to a device. (owner of device, roaming_device_profile_id, roaming_device_id)
        AssignedDeviceProfileToDevice(AccountId, RoamingDeviceProfileIndex, RoamingDeviceIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as RoamingDeviceProfiles {
        /// Stores all the roaming device_profiles, key is the roaming device_profile id / index
        pub RoamingDeviceProfiles get(fn roaming_device_profile): map hasher(opaque_blake2_256) T::RoamingDeviceProfileIndex => Option<RoamingDeviceProfile>;

        /// Stores the total number of roaming device_profiles. i.e. the next roaming device_profile index
        pub RoamingDeviceProfilesCount get(fn roaming_device_profiles_count): T::RoamingDeviceProfileIndex;

        /// Get roaming device_profile owner
        pub RoamingDeviceProfileOwners get(fn roaming_device_profile_owner): map hasher(opaque_blake2_256) T::RoamingDeviceProfileIndex => Option<T::AccountId>;

        /// Get roaming device_profile config
        pub RoamingDeviceProfileSettings get(fn roaming_device_profile_settings): map hasher(opaque_blake2_256) T::RoamingDeviceProfileIndex => Option<RoamingDeviceProfileSetting<T::RoamingDeviceProfileDevAddr, T::RoamingDeviceProfileDevEUI, T::RoamingDeviceProfileJoinEUI, T::RoamingDeviceProfileVendorID>>;

        /// Get roaming device_profile device
        pub RoamingDeviceProfileDevice get(fn roaming_device_profile_device): map hasher(opaque_blake2_256) T::RoamingDeviceProfileIndex => Option<T::RoamingDeviceIndex>;

        /// Get roaming device device_profiles
        pub RoamingDeviceDeviceProfiles get(fn roaming_device_device_profiles): map hasher(opaque_blake2_256) T::RoamingDeviceIndex => Option<Vec<T::RoamingDeviceProfileIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming device_profile
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_device_profile_id = Self::next_roaming_device_profile_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming device_profile
            let roaming_device_profile = RoamingDeviceProfile(unique_id);
            Self::insert_roaming_device_profile(&sender, roaming_device_profile_id, roaming_device_profile);

            Self::deposit_event(RawEvent::Created(sender, roaming_device_profile_id));
        }

        /// Transfer a roaming device_profile to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_device_profile_id: T::RoamingDeviceProfileIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_device_profile_owner(roaming_device_profile_id) == Some(sender.clone()), "Only owner can transfer roaming device_profile");

            Self::update_owner(&to, roaming_device_profile_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_device_profile_id));
        }

        /// Set roaming device_profile config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_config(
            origin,
            roaming_device_profile_id: T::RoamingDeviceProfileIndex,
            _device_profile_devaddr: Option<T::RoamingDeviceProfileDevAddr>,
            _device_profile_deveui: Option<T::RoamingDeviceProfileDevEUI>,
            _device_profile_joineui: Option<T::RoamingDeviceProfileJoinEUI>,
            _device_profile_vendorid: Option<T::RoamingDeviceProfileVendorID>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming device profile id whose config we want to change actually exists
            let is_roaming_device_profile = Self::exists_roaming_device_profile(roaming_device_profile_id).is_ok();
            ensure!(is_roaming_device_profile, "RoamingDeviceProfile does not exist");

            // Ensure that the caller is owner of the device profile config they are trying to change
            ensure!(Self::roaming_device_profile_owner(roaming_device_profile_id) == Some(sender.clone()), "Only owner can set config for roaming device_profile");

            let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_device_profile_id, sender.clone()).is_ok();
            ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            let device_profile_devaddr = match _device_profile_devaddr {
                Some(value) => value,
                None => Default::default() // Default
            };
            let device_profile_deveui = match _device_profile_deveui {
                Some(value) => value,
                None => Default::default()
            };
            let device_profile_joineui = match _device_profile_joineui {
                Some(value) => value,
                None => Default::default()
            };
            let device_profile_vendorid = match _device_profile_vendorid {
                Some(value) => value,
                None => Default::default()
            };

            // Check if a roaming device profile config already exists with the given roaming device profile id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_device_profile_setting_index(roaming_device_profile_id).is_ok() {
                info!("Mutating values");
                <RoamingDeviceProfileSettings<T>>::mutate(roaming_device_profile_id, |profile_setting| {
                    if let Some(_profile_setting) = profile_setting {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _profile_setting.device_profile_devaddr = device_profile_devaddr.clone();
                        _profile_setting.device_profile_deveui = device_profile_deveui.clone();
                        _profile_setting.device_profile_joineui = device_profile_joineui.clone();
                        _profile_setting.device_profile_vendorid = device_profile_vendorid.clone();
                    }
                });
                info!("Checking mutated values");
                let fetched_profile_setting = <RoamingDeviceProfileSettings<T>>::get(roaming_device_profile_id);
                if let Some(_profile_setting) = fetched_profile_setting {
                    info!("Latest field device_profile_devaddr {:#?}", _profile_setting.device_profile_devaddr);
                    info!("Latest field device_profile_deveui {:#?}", _profile_setting.device_profile_deveui);
                    info!("Latest field device_profile_joineui {:#?}", _profile_setting.device_profile_joineui);
                    info!("Latest field device_profile_vendorid {:#?}", _profile_setting.device_profile_vendorid);
                }
            } else {
                info!("Inserting values");

                // Create a new roaming device_profile config instance with the input params
                let roaming_device_profile_setting_instance = RoamingDeviceProfileSetting {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    device_profile_devaddr: device_profile_devaddr.clone(),
                    device_profile_deveui: device_profile_deveui.clone(),
                    device_profile_joineui: device_profile_joineui.clone(),
                    device_profile_vendorid: device_profile_vendorid.clone()
                };

                <RoamingDeviceProfileSettings<T>>::insert(
                    roaming_device_profile_id,
                    &roaming_device_profile_setting_instance
                );

                info!("Checking inserted values");
                let fetched_profile_setting = <RoamingDeviceProfileSettings<T>>::get(roaming_device_profile_id);
                if let Some(_profile_setting) = fetched_profile_setting {
                    info!("Inserted field device_profile_devaddr {:#?}", _profile_setting.device_profile_devaddr);
                    info!("Inserted field device_profile_deveui {:#?}", _profile_setting.device_profile_deveui);
                    info!("Inserted field device_profile_joineui {:#?}", _profile_setting.device_profile_joineui);
                    info!("Inserted field device_profile_vendorid {:#?}", _profile_setting.device_profile_vendorid);
                }
            }

            Self::deposit_event(RawEvent::RoamingDeviceProfileSettingSet(
                sender,
                roaming_device_profile_id,
                device_profile_devaddr,
                device_profile_deveui,
                device_profile_joineui,
                device_profile_vendorid
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_device_profile_to_device(
            origin,
            roaming_device_profile_id: T::RoamingDeviceProfileIndex,
            roaming_device_id: T::RoamingDeviceIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given device id already exists
            let is_roaming_device = <roaming_devices::Pallet<T>>
                ::exists_roaming_device(roaming_device_id).is_ok();
            ensure!(is_roaming_device, "RoamingDevice does not exist");

            // Ensure that caller of the function is the owner of the device id to assign the device_profile to
            ensure!(
                <roaming_devices::Pallet<T>>::is_roaming_device_owner(roaming_device_id, sender.clone()).is_ok(),
                "Only the roaming device owner can assign itself a roaming device_profile"
            );

            Self::associate_device_profile_with_device(roaming_device_profile_id, roaming_device_id)
                .expect("Unable to associate device_profile with device");

            // Ensure that the given device_profile id already exists
            let roaming_device_profile = Self::roaming_device_profile(roaming_device_profile_id);
            ensure!(roaming_device_profile.is_some(), "Invalid roaming_device_profile_id");

            // Ensure that the device_profile is not already owned by a different device
            // Unassign the device_profile from any existing device since it may only be owned by one device
            <RoamingDeviceProfileDevice<T>>::remove(roaming_device_profile_id);

            // Assign the device_profile owner to the given device (even if already belongs to them)
            <RoamingDeviceProfileDevice<T>>::insert(roaming_device_profile_id, roaming_device_id);

            Self::deposit_event(RawEvent::AssignedDeviceProfileToDevice(sender, roaming_device_profile_id, roaming_device_id));
        }
    }
}

impl<T: Config> Module<T> {
    pub fn exists_roaming_device_profile(
        roaming_device_profile_id: T::RoamingDeviceProfileIndex,
    ) -> Result<RoamingDeviceProfile, DispatchError> {
        match Self::roaming_device_profile(roaming_device_profile_id) {
            Some(roaming_device_profile) => Ok(roaming_device_profile),
            None => Err(DispatchError::Other("RoamingDeviceProfile does not exist")),
        }
    }

    pub fn is_owned_by_required_parent_relationship(
        roaming_device_profile_id: T::RoamingDeviceProfileIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        info!("Get the device id associated with the device of the given device profile id");
        let device_profile_device_id = Self::roaming_device_profile_device(roaming_device_profile_id);

        if let Some(_device_profile_device_id) = device_profile_device_id {
            // Ensure that the caller is owner of the device id associated with the device profile
            ensure!(
                (<roaming_devices::Pallet<T>>::is_roaming_device_owner(
                    _device_profile_device_id.clone(),
                    sender.clone()
                ))
                .is_ok(),
                "Only owner of the device id associated with the given device profile can set an associated roaming \
                 device profile config"
            );
        } else {
            // There must be a device id associated with the device profile
            return Err(DispatchError::Other("RoamingDeviceProfileDevice does not exist"));
        }
        Ok(())
    }

    pub fn exists_roaming_device_profile_setting(
        roaming_device_profile_id: T::RoamingDeviceProfileIndex,
    ) -> Result<(), DispatchError> {
        match Self::roaming_device_profile_settings(roaming_device_profile_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("RoamingDeviceProfileSetting does not exist")),
        }
    }

    pub fn has_value_for_device_profile_setting_index(
        roaming_device_profile_id: T::RoamingDeviceProfileIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if device profile config has a value that is defined");
        let fetched_profile_setting = <RoamingDeviceProfileSettings<T>>::get(roaming_device_profile_id);
        if let Some(_value) = fetched_profile_setting {
            info!("Found value for device profile config");
            return Ok(());
        }
        warn!("No value for device profile config");
        Err(DispatchError::Other("No value for device profile config"))
    }

    /// Only push the device_profile id onto the end of the vector if it does not already exist
    pub fn associate_device_profile_with_device(
        roaming_device_profile_id: T::RoamingDeviceProfileIndex,
        roaming_device_id: T::RoamingDeviceIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given device id already exists as a key,
        // and where its corresponding value is a vector that already contains the given device_profile id
        if let Some(device_device_profiles) = Self::roaming_device_device_profiles(roaming_device_id) {
            info!("Device id key {:?} exists with value {:?}", roaming_device_id, device_device_profiles);
            let not_device_contains_device_profile = !device_device_profiles.contains(&roaming_device_profile_id);
            ensure!(not_device_contains_device_profile, "Device already contains the given device_profile id");
            info!("Device id key exists but its vector value does not contain the given device_profile id");
            <RoamingDeviceDeviceProfiles<T>>::mutate(roaming_device_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_device_profile_id);
                }
            });
            info!(
                "Associated device_profile {:?} with device {:?}",
                roaming_device_profile_id,
                roaming_device_id
            );
            Ok(())
        } else {
            info!(
                "Device id key does not yet exist. Creating the device key {:?} and appending the device_profile id \
                 {:?} to its vector value",
                roaming_device_id,
                roaming_device_profile_id
            );
            <RoamingDeviceDeviceProfiles<T>>::insert(roaming_device_id, &vec![roaming_device_profile_id]);
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

    fn next_roaming_device_profile_id() -> Result<T::RoamingDeviceProfileIndex, DispatchError> {
        let roaming_device_profile_id = Self::roaming_device_profiles_count();
        if roaming_device_profile_id == <T::RoamingDeviceProfileIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingDeviceProfiles count overflow"));
        }
        Ok(roaming_device_profile_id)
    }

    fn insert_roaming_device_profile(
        owner: &T::AccountId,
        roaming_device_profile_id: T::RoamingDeviceProfileIndex,
        roaming_device_profile: RoamingDeviceProfile,
    ) {
        // Create and store roaming device_profile
        <RoamingDeviceProfiles<T>>::insert(roaming_device_profile_id, roaming_device_profile);
        <RoamingDeviceProfilesCount<T>>::put(roaming_device_profile_id + One::one());
        <RoamingDeviceProfileOwners<T>>::insert(roaming_device_profile_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_device_profile_id: T::RoamingDeviceProfileIndex) {
        <RoamingDeviceProfileOwners<T>>::insert(roaming_device_profile_id, to);
    }
}
