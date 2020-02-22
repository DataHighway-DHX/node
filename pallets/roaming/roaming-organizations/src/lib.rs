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

use roaming_network_servers;
use roaming_operators;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait: system::Trait + roaming_operators::Trait + roaming_network_servers::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type RoamingOrganizationIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingOrganization(pub [u8; 16]);

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::RoamingOrganizationIndex,
        <T as roaming_network_servers::Trait>::RoamingNetworkServerIndex,
        Balance = BalanceOf<T>,
    {
        /// A roaming organization is created. (owner, roaming_organization_id)
        Created(AccountId, RoamingOrganizationIndex),
        /// A roaming organization is transferred. (from, to, roaming_organization_id)
        Transferred(AccountId, AccountId, RoamingOrganizationIndex),
        /// A roaming organization is available for sale. (owner, roaming_organization_id, price)
        PriceSet(AccountId, RoamingOrganizationIndex, Option<Balance>),
        /// A roaming organization is sold. (from, to, roaming_organization_id, price)
        Sold(AccountId, AccountId, RoamingOrganizationIndex, Balance),
        /// A roaming organization is assigned to a network server. (owner of network server, roaming_organization_id, roaming_network_server_id)
        AssignedOrganizationToNetworkServer(AccountId, RoamingOrganizationIndex, RoamingNetworkServerIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingOrganizations {
        /// Stores all the roaming organizations, key is the roaming organization id / index
        pub RoamingOrganizations get(fn roaming_organization): map hasher(blake2_256) T::RoamingOrganizationIndex => Option<RoamingOrganization>;

        /// Stores the total number of roaming organizations. i.e. the next roaming organization index
        pub RoamingOrganizationsCount get(fn roaming_organizations_count): T::RoamingOrganizationIndex;

        /// Get roaming organization owner
        pub RoamingOrganizationOwners get(fn roaming_organization_owner): map hasher(blake2_256) T::RoamingOrganizationIndex => Option<T::AccountId>;

        /// Get roaming organization price. None means not for sale.
        pub RoamingOrganizationPrices get(fn roaming_organization_price): map hasher(blake2_256) T::RoamingOrganizationIndex => Option<BalanceOf<T>>;

        /// Get roaming organization network server
        pub RoamingOrganizationNetworkServers get(fn roaming_organization_network_server): map hasher(blake2_256) T::RoamingOrganizationIndex => Option<T::RoamingNetworkServerIndex>;

        /// Get roaming network server organizations
        pub RoamingNetworkServerOrganizations get(fn roaming_network_server_organizations): map hasher(blake2_256) T::RoamingNetworkServerIndex => Option<Vec<T::RoamingOrganizationIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming organization
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_organization_id = Self::next_roaming_organization_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming organization
            let roaming_organization = RoamingOrganization(unique_id);
            Self::insert_roaming_organization(&sender, roaming_organization_id, roaming_organization);

            Self::deposit_event(RawEvent::Created(sender, roaming_organization_id));
        }

        /// Transfer a roaming organization to new owner
        pub fn transfer(origin, to: T::AccountId, roaming_organization_id: T::RoamingOrganizationIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_organization_owner(roaming_organization_id) == Some(sender.clone()), "Only owner can transfer roaming organization");

            Self::update_owner(&to, roaming_organization_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_organization_id));
        }

        /// Set a price for a roaming organization for sale
        /// None to delist the roaming organization
        pub fn set_price(origin, roaming_organization_id: T::RoamingOrganizationIndex, price: Option<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_organization_owner(roaming_organization_id) == Some(sender.clone()), "Only owner can set price for roaming organization");

            if let Some(ref price) = price {
                <RoamingOrganizationPrices<T>>::insert(roaming_organization_id, price);
            } else {
                <RoamingOrganizationPrices<T>>::remove(roaming_organization_id);
            }

            Self::deposit_event(RawEvent::PriceSet(sender, roaming_organization_id, price));
        }

        /// Buy a roaming organization with max price willing to pay
        pub fn buy(origin, roaming_organization_id: T::RoamingOrganizationIndex, price: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            let owner = Self::roaming_organization_owner(roaming_organization_id);
            ensure!(owner.is_some(), "RoamingOrganization owner does not exist");
            let owner = owner.unwrap();

            let roaming_organization_price = Self::roaming_organization_price(roaming_organization_id);
            ensure!(roaming_organization_price.is_some(), "RoamingOrganization not for sale");

            let roaming_organization_price = roaming_organization_price.unwrap();
            ensure!(price >= roaming_organization_price, "Price is too low");

            T::Currency::transfer(&sender, &owner, roaming_organization_price, ExistenceRequirement::AllowDeath)?;

            <RoamingOrganizationPrices<T>>::remove(roaming_organization_id);

            Self::update_owner(&sender, roaming_organization_id);

            Self::deposit_event(RawEvent::Sold(owner, sender, roaming_organization_id, roaming_organization_price));
        }

        pub fn assign_organization_to_network_server(
            origin,
            roaming_organization_id: T::RoamingOrganizationIndex,
            roaming_network_server_id: T::RoamingNetworkServerIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network server id already exists
            let is_roaming_network_server = <roaming_network_servers::Module<T>>
                ::exists_roaming_network_server(roaming_network_server_id).is_ok();
            ensure!(is_roaming_network_server, "RoamingNetworkServer does not exist");

            // Ensure that caller of the function is the owner of the network server id to assign the organization to
            ensure!(
                <roaming_network_servers::Module<T>>::is_roaming_network_server_owner(roaming_network_server_id, sender.clone()).is_ok(),
                "Only the roaming network_server owner can assign itself a roaming organization"
            );

            Self::associate_organization_with_network_server(roaming_organization_id, roaming_network_server_id)
                .expect("Unable to associate organization with network server");

            // Ensure that the given organization id already exists
            let roaming_organization = Self::roaming_organization(roaming_organization_id);
            ensure!(roaming_organization.is_some(), "Invalid roaming_organization_id");

            // Ensure that the organization is not already owned by a different network server
            // Unassign the organization from any existing network since it may only be owned by one network server
            <RoamingOrganizationNetworkServers<T>>::remove(roaming_organization_id);

            // Assign the organization owner to the given network server (even if already belongs to them)
            <RoamingOrganizationNetworkServers<T>>::insert(roaming_organization_id, roaming_network_server_id);

            Self::deposit_event(RawEvent::AssignedOrganizationToNetworkServer(sender, roaming_organization_id, roaming_network_server_id));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_roaming_organization_owner(
        roaming_organization_id: T::RoamingOrganizationIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_organization_owner(&roaming_organization_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of RoamingOrganization"
        );
        Ok(())
    }

    pub fn exists_roaming_organization(
        roaming_organization_id: T::RoamingOrganizationIndex,
    ) -> Result<RoamingOrganization, DispatchError> {
        match Self::roaming_organization(roaming_organization_id) {
            Some(roaming_organization) => Ok(roaming_organization),
            None => Err(DispatchError::Other("RoamingOrganization does not exist")),
        }
    }

    /// Only push the organization id onto the end of the vector if it does not already exist
    pub fn associate_organization_with_network_server(
        roaming_organization_id: T::RoamingOrganizationIndex,
        roaming_network_server_id: T::RoamingNetworkServerIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given network server id already exists as a key,
        // and where its corresponding value is a vector that already contains the given organization id
        if let Some(network_server_organizations) =
            Self::roaming_network_server_organizations(roaming_network_server_id)
        {
            debug::info!(
                "Network Server id key {:?} exists with value {:?}",
                roaming_network_server_id,
                network_server_organizations
            );
            let not_network_server_contains_organization =
                !network_server_organizations.contains(&roaming_organization_id);
            ensure!(
                not_network_server_contains_organization,
                "Network Server already contains the given organization id"
            );
            debug::info!(
                "Network Server id key exists but its vector value does not contain the given organization id"
            );
            <RoamingNetworkServerOrganizations<T>>::mutate(roaming_network_server_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_organization_id);
                }
            });
            debug::info!(
                "Associated organization {:?} with network server {:?}",
                roaming_organization_id,
                roaming_network_server_id
            );
            Ok(())
        } else {
            debug::info!(
                "Network Server id key does not yet exist. Creating the network server key {:?} and appending the \
                 organization id {:?} to its vector value",
                roaming_network_server_id,
                roaming_organization_id
            );
            <RoamingNetworkServerOrganizations<T>>::insert(roaming_network_server_id, &vec![roaming_organization_id]);
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

    fn next_roaming_organization_id() -> Result<T::RoamingOrganizationIndex, DispatchError> {
        let roaming_organization_id = Self::roaming_organizations_count();
        if roaming_organization_id == <T::RoamingOrganizationIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingOrganizations count overflow"));
        }
        Ok(roaming_organization_id)
    }

    fn insert_roaming_organization(
        owner: &T::AccountId,
        roaming_organization_id: T::RoamingOrganizationIndex,
        roaming_organization: RoamingOrganization,
    ) {
        // Create and store roaming organization
        <RoamingOrganizations<T>>::insert(roaming_organization_id, roaming_organization);
        <RoamingOrganizationsCount<T>>::put(roaming_organization_id + One::one());
        <RoamingOrganizationOwners<T>>::insert(roaming_organization_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_organization_id: T::RoamingOrganizationIndex) {
        <RoamingOrganizationOwners<T>>::insert(roaming_organization_id, to);
    }
}
