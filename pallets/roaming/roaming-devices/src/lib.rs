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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config:
    frame_system::Config + roaming_operators::Config + roaming_network_servers::Config + roaming_organizations::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RoamingDeviceIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> =
    <<T as roaming_operators::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingDevice(pub [u8; 16]);

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::RoamingDeviceIndex,
        <T as roaming_network_servers::Config>::RoamingNetworkServerIndex,
        <T as roaming_organizations::Config>::RoamingOrganizationIndex,
        Balance = BalanceOf<T>,
    {
        /// A roaming device is created. (owner, roaming_device_id)
        Created(AccountId, RoamingDeviceIndex),
        /// A roaming device is transferred. (from, to, roaming_device_id)
        Transferred(AccountId, AccountId, RoamingDeviceIndex),
        /// A roaming device is available for sale. (owner, roaming_device_id, price)
        PriceSet(AccountId, RoamingDeviceIndex, Option<Balance>),
        /// A roaming device is sold. (from, to, roaming_device_id, price)
        Sold(AccountId, AccountId, RoamingDeviceIndex, Balance),
        /// A roaming device is assigned to a network_server. (owner of network_server, roaming_device_id, roaming_network_server_id)
        AssignedDeviceToNetworkServer(AccountId, RoamingDeviceIndex, RoamingNetworkServerIndex),
        /// A roaming device is assigned to an organization. (owner of organization, roaming_device_id, roaming_organization_id)
        AssignedDeviceToOrganization(AccountId, RoamingDeviceIndex, RoamingOrganizationIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as RoamingDevices {
        /// Stores all the roaming devices, key is the roaming device id / index
        pub RoamingDevices get(fn roaming_device): map hasher(opaque_blake2_256) T::RoamingDeviceIndex => Option<RoamingDevice>;

        /// Stores the total number of roaming devices. i.e. the next roaming device index
        pub RoamingDevicesCount get(fn roaming_devices_count): T::RoamingDeviceIndex;

        /// Get roaming device owner
        pub RoamingDeviceOwners get(fn roaming_device_owner): map hasher(opaque_blake2_256) T::RoamingDeviceIndex => Option<T::AccountId>;

        /// Get roaming device price. None means not for sale.
        pub RoamingDevicePrices get(fn roaming_device_price): map hasher(opaque_blake2_256) T::RoamingDeviceIndex => Option<BalanceOf<T>>;

        /// Get roaming device network_server
        pub RoamingDeviceNetworkServers get(fn roaming_device_network_server): map hasher(opaque_blake2_256) T::RoamingDeviceIndex => Option<T::RoamingNetworkServerIndex>;

        /// Get roaming device organization
        pub RoamingDeviceOrganization get(fn roaming_device_organization): map hasher(opaque_blake2_256) T::RoamingDeviceIndex => Option<T::RoamingOrganizationIndex>;

        /// Get roaming network server's devices
        pub RoamingNetworkServerDevices get(fn roaming_network_server_devices): map hasher(opaque_blake2_256) T::RoamingNetworkServerIndex => Option<Vec<T::RoamingDeviceIndex>>;

        /// Get roaming organization's devices
        pub RoamingOrganizationDevices get(fn roaming_organization_devices): map hasher(opaque_blake2_256) T::RoamingOrganizationIndex => Option<Vec<T::RoamingDeviceIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming device
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_device_id = Self::next_roaming_device_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming device
            let roaming_device = RoamingDevice(unique_id);
            Self::insert_roaming_device(&sender, roaming_device_id, roaming_device);

            Self::deposit_event(RawEvent::Created(sender, roaming_device_id));
        }

        /// Transfer a roaming device to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_device_id: T::RoamingDeviceIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_device_owner(roaming_device_id) == Some(sender.clone()), "Only owner can transfer roaming device");

            Self::update_owner(&to, roaming_device_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_device_id));
        }

        /// Set a price for a roaming device for sale
        /// None to delist the roaming device
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_price(origin, roaming_device_id: T::RoamingDeviceIndex, price: Option<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_device_owner(roaming_device_id) == Some(sender.clone()), "Only owner can set price for roaming device");

            if let Some(ref price) = price {
                <RoamingDevicePrices<T>>::insert(roaming_device_id, price);
            } else {
                <RoamingDevicePrices<T>>::remove(roaming_device_id);
            }

            Self::deposit_event(RawEvent::PriceSet(sender, roaming_device_id, price));
        }

        /// Buy a roaming device with max price willing to pay
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn buy(origin, roaming_device_id: T::RoamingDeviceIndex, price: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            let owner = Self::roaming_device_owner(roaming_device_id);
            ensure!(owner.is_some(), "RoamingDevice owner does not exist");
            let owner = owner.unwrap();

            let roaming_device_price = Self::roaming_device_price(roaming_device_id);
            ensure!(roaming_device_price.is_some(), "RoamingDevice not for sale");

            let roaming_device_price = roaming_device_price.unwrap();
            ensure!(price >= roaming_device_price, "Price is too low");

            T::Currency::transfer(&sender, &owner, roaming_device_price, ExistenceRequirement::AllowDeath)?;

            <RoamingDevicePrices<T>>::remove(roaming_device_id);

            Self::update_owner(&sender, roaming_device_id);

            Self::deposit_event(RawEvent::Sold(owner, sender, roaming_device_id, roaming_device_price));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_device_to_network_server(
            origin,
            roaming_device_id: T::RoamingDeviceIndex,
            roaming_network_server_id: T::RoamingNetworkServerIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network server id already exists
            let is_roaming_network_server = <roaming_network_servers::Module<T>>
                ::exists_roaming_network_server(roaming_network_server_id).is_ok();
            ensure!(is_roaming_network_server, "RoamingNetworkServer does not exist");

            // Ensure that caller of the function is the owner of the network server id to assign the device to
            ensure!(
                <roaming_network_servers::Module<T>>::is_roaming_network_server_owner(roaming_network_server_id, sender.clone()).is_ok(),
                "Only the roaming network_server owner can assign itself a roaming device"
            );

            Self::associate_device_with_network_server(roaming_device_id, roaming_network_server_id)
                .expect("Unable to associate device with network server");

            // Ensure that the given device id already exists
            let roaming_device = Self::roaming_device(roaming_device_id);
            ensure!(roaming_device.is_some(), "Invalid roaming_device_id");

            // Ensure that the device is not already owned by a different network_server
            // Unassign the device from any existing network_server since it may only be owned by one network_server
            <RoamingDeviceNetworkServers<T>>::remove(roaming_device_id);

            // Assign the device owner to the given network_server (even if already belongs to them)
            <RoamingDeviceNetworkServers<T>>::insert(roaming_device_id, roaming_network_server_id);

            Self::deposit_event(RawEvent::AssignedDeviceToNetworkServer(sender, roaming_device_id, roaming_network_server_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_device_to_organization(
            origin,
            roaming_device_id: T::RoamingDeviceIndex,
            roaming_organization_id: T::RoamingOrganizationIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given organization id already exists
            let is_roaming_organization = <roaming_organizations::Module<T>>
                ::exists_roaming_organization(roaming_organization_id).is_ok();
            ensure!(is_roaming_organization, "RoamingOrganization does not exist");

            // Ensure that caller of the function is the owner of the organization id to assign the device to
            ensure!(
                <roaming_organizations::Module<T>>::is_roaming_organization_owner(roaming_organization_id, sender.clone()).is_ok(),
                "Only the roaming organization owner can assign itself a roaming device"
            );

            Self::associate_device_with_organization(roaming_device_id, roaming_organization_id)
                .expect("Unable to associate device with organization");

            // Ensure that the given device id already exists
            let roaming_device = Self::roaming_device(roaming_device_id);
            ensure!(roaming_device.is_some(), "Invalid roaming_device_id");

            // Ensure that the device is not already owned by a different organization
            // Unassign the device from any existing organization since it may only be owned by one organization
            <RoamingDeviceOrganization<T>>::remove(roaming_device_id);

            // Assign the device owner to the given organization (even if already belongs to them)
            <RoamingDeviceOrganization<T>>::insert(roaming_device_id, roaming_organization_id);

            Self::deposit_event(RawEvent::AssignedDeviceToOrganization(sender, roaming_device_id, roaming_organization_id));
        }
    }
}

impl<T: Config> Module<T> {
    pub fn is_roaming_device_owner(
        roaming_device_id: T::RoamingDeviceIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_device_owner(&roaming_device_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of RoamingDevice"
        );
        Ok(())
    }

    pub fn exists_roaming_device(roaming_device_id: T::RoamingDeviceIndex) -> Result<RoamingDevice, DispatchError> {
        match Self::roaming_device(roaming_device_id) {
            Some(roaming_device) => Ok(roaming_device),
            None => Err(DispatchError::Other("RoamingDevice does not exist")),
        }
    }

    /// Only push the device id onto the end of the vector if it does not already exist
    pub fn associate_device_with_network_server(
        roaming_device_id: T::RoamingDeviceIndex,
        roaming_network_server_id: T::RoamingNetworkServerIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given network server id already exists as a key,
        // and where its corresponding value is a vector that already contains the given device id
        if let Some(network_server_devices) = Self::roaming_network_server_devices(roaming_network_server_id) {
            debug::info!(
                "Network Server id key {:?} exists with value {:?}",
                roaming_network_server_id,
                network_server_devices
            );
            let not_network_server_contains_device = !network_server_devices.contains(&roaming_device_id);
            ensure!(not_network_server_contains_device, "Network Server already contains the given device id");
            debug::info!("Network Server id key exists but its vector value does not contain the given device id");
            <RoamingNetworkServerDevices<T>>::mutate(roaming_network_server_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_device_id);
                }
            });
            debug::info!(
                "Associated device {:?} with network server {:?}",
                roaming_device_id,
                roaming_network_server_id
            );
            Ok(())
        } else {
            debug::info!(
                "Network Server id key does not yet exist. Creating the network server key {:?} and appending the \
                 device id {:?} to its vector value",
                roaming_network_server_id,
                roaming_device_id
            );
            <RoamingNetworkServerDevices<T>>::insert(roaming_network_server_id, &vec![roaming_device_id]);
            Ok(())
        }
    }

    /// Only push the device id onto the end of the vector if it does not already exist
    pub fn associate_device_with_organization(
        roaming_device_id: T::RoamingDeviceIndex,
        roaming_organization_id: T::RoamingOrganizationIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given network server id already exists as a key,
        // and where its corresponding value is a vector that already contains the given device id
        if let Some(organization_devices) = Self::roaming_organization_devices(roaming_organization_id) {
            debug::info!(
                "Organization id key {:?} exists with value {:?}",
                roaming_organization_id,
                organization_devices
            );
            let not_organization_contains_device = !organization_devices.contains(&roaming_device_id);
            ensure!(not_organization_contains_device, "Organization already contains the given device id");
            debug::info!("Organization id key exists but its vector value does not contain the given device id");
            <RoamingOrganizationDevices<T>>::mutate(roaming_organization_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_device_id);
                }
            });
            debug::info!("Associated device {:?} with network server {:?}", roaming_device_id, roaming_organization_id);
            Ok(())
        } else {
            debug::info!(
                "Organization id key does not yet exist. Creating the network server key {:?} and appending the \
                 device id {:?} to its vector value",
                roaming_organization_id,
                roaming_device_id
            );
            <RoamingOrganizationDevices<T>>::insert(roaming_organization_id, &vec![roaming_device_id]);
            Ok(())
        }
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <frame_system::Module<T>>::extrinsic_index(),
            <frame_system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_roaming_device_id() -> Result<T::RoamingDeviceIndex, DispatchError> {
        let roaming_device_id = Self::roaming_devices_count();
        if roaming_device_id == <T::RoamingDeviceIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingDevices count overflow"));
        }
        Ok(roaming_device_id)
    }

    fn insert_roaming_device(
        owner: &T::AccountId,
        roaming_device_id: T::RoamingDeviceIndex,
        roaming_device: RoamingDevice,
    ) {
        // Create and store roaming device
        <RoamingDevices<T>>::insert(roaming_device_id, roaming_device);
        <RoamingDevicesCount<T>>::put(roaming_device_id + One::one());
        <RoamingDeviceOwners<T>>::insert(roaming_device_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_device_id: T::RoamingDeviceIndex) {
        <RoamingDeviceOwners<T>>::insert(roaming_device_id, to);
    }
}
