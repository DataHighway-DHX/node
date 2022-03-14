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
pub trait Config: frame_system::Config + roaming_operators::Config + roaming_networks::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RoamingAccountingPolicyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingAccountingPolicyType: Parameter + Member + Default;
    type RoamingAccountingPolicyUplinkFeeFactor: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingAccountingPolicyDownlinkFeeFactor: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> =
    <<T as roaming_operators::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingAccountingPolicy(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
// Generic type parameters - Balance
pub struct RoamingAccountingPolicySetting<U, V, W, X> {
    pub policy_type: U, // "adhoc" or "subscription"
    pub subscription_fee: V,
    pub uplink_fee_factor: W,
    pub downlink_fee_factor: X,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::RoamingAccountingPolicyIndex,
        <T as Config>::RoamingAccountingPolicyType,
        <T as Config>::RoamingAccountingPolicyUplinkFeeFactor,
        <T as Config>::RoamingAccountingPolicyDownlinkFeeFactor,
        <T as roaming_networks::Config>::RoamingNetworkIndex,
        Balance = BalanceOf<T>,
    {
        /// A roaming accounting_policy is created. (owner, roaming_accounting_policy_id)
        Created(AccountId, RoamingAccountingPolicyIndex),
        /// A roaming accounting_policy is transferred. (from, to, roaming_accounting_policy_id)
        Transferred(AccountId, AccountId, RoamingAccountingPolicyIndex),
        /// A roaming accounting_policy configuration
        RoamingAccountingPolicySettingSet(AccountId, RoamingAccountingPolicyIndex, RoamingAccountingPolicyType, Balance, RoamingAccountingPolicyUplinkFeeFactor, RoamingAccountingPolicyDownlinkFeeFactor),
        /// A roaming accounting_policy is assigned to a network. (owner of network, roaming_accounting_policy_id, roaming_network_id)
        AssignedAccountingPolicyToNetwork(AccountId, RoamingAccountingPolicyIndex, RoamingNetworkIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as RoamingAccountingPolicies {
        /// Stores all the roaming accounting_policies, key is the roaming accounting_policy id / index
        pub RoamingAccountingPolicies get(fn roaming_accounting_policy): map hasher(opaque_blake2_256) T::RoamingAccountingPolicyIndex => Option<RoamingAccountingPolicy>;

        /// Stores the total number of roaming accounting_policies. i.e. the next roaming accounting_policy index
        pub RoamingAccountingPoliciesCount get(fn roaming_accounting_policies_count): T::RoamingAccountingPolicyIndex;

        /// Get roaming accounting_policy owner
        pub RoamingAccountingPolicyOwners get(fn roaming_accounting_policy_owner): map hasher(opaque_blake2_256) T::RoamingAccountingPolicyIndex => Option<T::AccountId>;

        /// Get roaming accounting_policy config
        pub RoamingAccountingPolicySettings get(fn roaming_accounting_policy_settings): map hasher(opaque_blake2_256) T::RoamingAccountingPolicyIndex => Option<RoamingAccountingPolicySetting<T::RoamingAccountingPolicyType, BalanceOf<T>, T::RoamingAccountingPolicyUplinkFeeFactor, T::RoamingAccountingPolicyDownlinkFeeFactor>>;

        /// Get roaming accounting_policy network
        pub RoamingAccountingPolicyNetwork get(fn roaming_accounting_policy_network): map hasher(opaque_blake2_256) T::RoamingAccountingPolicyIndex => Option<T::RoamingNetworkIndex>;

        /// Get roaming network's accounting policies
        pub RoamingNetworkAccountingPolicies get(fn roaming_network_accounting_policies): map hasher(opaque_blake2_256) T::RoamingNetworkIndex => Option<Vec<T::RoamingAccountingPolicyIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming accounting_policy
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_accounting_policy_id = Self::next_roaming_accounting_policy_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming accounting_policy
            let roaming_accounting_policy = RoamingAccountingPolicy(unique_id);
            Self::insert_roaming_accounting_policy(&sender, roaming_accounting_policy_id, roaming_accounting_policy);

            Self::deposit_event(RawEvent::Created(sender, roaming_accounting_policy_id));
        }

        /// Transfer a roaming accounting_policy to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_accounting_policy_owner(roaming_accounting_policy_id) == Some(sender.clone()), "Only owner can transfer roaming accounting_policy");

            Self::update_owner(&to, roaming_accounting_policy_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_accounting_policy_id));
        }

        /// Set roaming account_policy config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_config(
            origin,
            roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
            _policy_type: Option<T::RoamingAccountingPolicyType>, // "adhoc" or "subscription"
            _subscription_fee: Option<BalanceOf<T>>,
            _uplink_fee_factor: Option<T::RoamingAccountingPolicyUplinkFeeFactor>,
            _downlink_fee_factor: Option<T::RoamingAccountingPolicyDownlinkFeeFactor>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming accounting policy id whose config we want to change actually exists
            let is_roaming_accounting_policy = Self::exists_roaming_accounting_policy(roaming_accounting_policy_id).is_ok();
            ensure!(is_roaming_accounting_policy, "RoamingAccountingPolicy does not exist");

            // Ensure that the caller is owner of the accounting policy config they are trying to change
            ensure!(Self::roaming_accounting_policy_owner(roaming_accounting_policy_id) == Some(sender.clone()), "Only owner can set config for roaming accounting_policy");

            // let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_accounting_policy_id, sender.clone()).is_ok();
            // ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            let policy_type = match _policy_type.clone() {
                Some(value) => value,
                None => Default::default() // Default
            };
            let subscription_fee = match _subscription_fee {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            let uplink_fee_factor = match _uplink_fee_factor {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            let downlink_fee_factor = match _downlink_fee_factor {
                Some(value) => value,
                None => 1u32.into() // Default
            };

            // Check if a roaming accounting policy config already exists with the given roaming accounting policy id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_accounting_policy_setting_index(roaming_accounting_policy_id).is_ok() {
                info!("Mutating values");
                <RoamingAccountingPolicySettings<T>>::mutate(roaming_accounting_policy_id, |policy_setting| {
                    if let Some(_policy_setting) = policy_setting {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _policy_setting.policy_type = policy_type.clone();
                        _policy_setting.subscription_fee = subscription_fee.clone();
                        _policy_setting.uplink_fee_factor = uplink_fee_factor.clone();
                        _policy_setting.downlink_fee_factor = downlink_fee_factor.clone();
                    }
                });
                info!("Checking mutated values");
                let fetched_policy_setting = <RoamingAccountingPolicySettings<T>>::get(roaming_accounting_policy_id);
                if let Some(_policy_setting) = fetched_policy_setting {
                    info!("Latest field policy_type {:#?}", _policy_setting.policy_type);
                    info!("Latest field subscription_fee {:#?}", _policy_setting.subscription_fee);
                    info!("Latest field uplink_fee_factor {:#?}", _policy_setting.uplink_fee_factor);
                    info!("Latest field downlink_fee_factor {:#?}", _policy_setting.downlink_fee_factor);
                }
            } else {
                info!("Inserting values");

                // Create a new roaming accounting_policy config instance with the input params
                let roaming_accounting_policy_setting_instance = RoamingAccountingPolicySetting {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    policy_type: policy_type.clone(),
                    subscription_fee: subscription_fee.clone(),
                    uplink_fee_factor: uplink_fee_factor.clone(),
                    downlink_fee_factor: downlink_fee_factor.clone()
                };

                <RoamingAccountingPolicySettings<T>>::insert(
                    roaming_accounting_policy_id,
                    &roaming_accounting_policy_setting_instance
                );

                info!("Checking inserted values");
                let fetched_policy_setting = <RoamingAccountingPolicySettings<T>>::get(roaming_accounting_policy_id);
                if let Some(_policy_setting) = fetched_policy_setting {
                    info!("Inserted field policy_type {:#?}", _policy_setting.policy_type);
                    info!("Inserted field subscription_fee {:#?}", _policy_setting.subscription_fee);
                    info!("Inserted field uplink_fee_factor {:#?}", _policy_setting.uplink_fee_factor);
                    info!("Inserted field downlink_fee_factor {:#?}", _policy_setting.downlink_fee_factor);
                }
            }

            Self::deposit_event(RawEvent::RoamingAccountingPolicySettingSet(
                sender,
                roaming_accounting_policy_id,
                policy_type,
                subscription_fee,
                uplink_fee_factor,
                downlink_fee_factor
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_accounting_policy_to_network(
            origin,
            roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
            roaming_network_id: T::RoamingNetworkIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_network = <roaming_networks::Pallet<T>>
                ::exists_roaming_network(roaming_network_id).is_ok();
            ensure!(is_roaming_network, "RoamingNetwork does not exist");

            // Ensure that caller of the function is the owner of the network id to assign the accounting_policy to
            ensure!(
                <roaming_networks::Pallet<T>>::is_roaming_network_owner(roaming_network_id, sender.clone()).is_ok(),
                "Only the roaming network owner can assign itself a roaming accounting policy"
            );

            Self::associate_accounting_policy_with_network(roaming_accounting_policy_id, roaming_network_id)
                .expect("Unable to associate accounting policy with network");

            // Ensure that the given accounting_policy id already exists
            let roaming_accounting_policy = Self::roaming_accounting_policy(roaming_accounting_policy_id);
            ensure!(roaming_accounting_policy.is_some(), "Invalid roaming_accounting_policy_id");

            // Ensure that the accounting_policy is not already owned by a different network
            // Unassign the accounting_policy from any existing network since it may only be owned by one network
            <RoamingAccountingPolicyNetwork<T>>::remove(roaming_accounting_policy_id);

            // Assign the accounting_policy owner to the given network (even if already belongs to them)
            <RoamingAccountingPolicyNetwork<T>>::insert(roaming_accounting_policy_id, roaming_network_id);

            Self::deposit_event(RawEvent::AssignedAccountingPolicyToNetwork(sender, roaming_accounting_policy_id, roaming_network_id));
        }
    }
}

impl<T: Config> Module<T> {
    pub fn is_roaming_accounting_policy_owner(
        roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_accounting_policy_owner(&roaming_accounting_policy_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of RoamingAccountingPolicy"
        );
        Ok(())
    }

    // Note: Not required
    // pub fn is_owned_by_required_parent_relationship(roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
    // sender: T::AccountId) -> Result<(), DispatchError> {     info!("Get the network id associated with the
    // network of the given accounting policy id");     let accounting_policy_network_id =
    // Self::roaming_accounting_policy_network(roaming_accounting_policy_id);

    //     if let Some(_accounting_policy_network_id) = accounting_policy_network_id {
    //         // Ensure that the caller is owner of the network id associated with the accounting policy
    //         ensure!((<roaming_networks::Pallet<T>>::is_roaming_network_owner(
    //                 _accounting_policy_network_id.clone(),
    //                 sender.clone()
    //             )).is_ok(), "Only owner of the network id associated with the given accounting policy can set an
    // associated roaming accounting policy config"         );
    //     } else {
    //         // There must be a network id associated with the accounting policy
    //         return Err(DispatchError::Other("RoamingAccountingPolicyNetwork does not exist"));
    //     }
    //     Ok(())
    // }

    pub fn exists_roaming_accounting_policy(
        roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
    ) -> Result<RoamingAccountingPolicy, DispatchError> {
        match Self::roaming_accounting_policy(roaming_accounting_policy_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("RoamingAccountingPolicy does not exist")),
        }
    }

    pub fn exists_roaming_accounting_policy_setting(
        roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
    ) -> Result<(), DispatchError> {
        match Self::roaming_accounting_policy_settings(roaming_accounting_policy_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("RoamingAccountingPolicySetting does not exist")),
        }
    }

    pub fn has_value_for_accounting_policy_setting_index(
        roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if accounting policy config has a value that is defined");
        let fetched_policy_setting = <RoamingAccountingPolicySettings<T>>::get(roaming_accounting_policy_id);
        if let Some(_value) = fetched_policy_setting {
            info!("Found value for accounting policy config");
            return Ok(());
        }
        warn!("No value for accounting policy config");
        Err(DispatchError::Other("No value for accounting policy config"))
    }

    /// Only push the accounting policy id onto the end of the vector if it does not already exist
    pub fn associate_accounting_policy_with_network(
        roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
        roaming_network_id: T::RoamingNetworkIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given network id already exists as a key,
        // and where its corresponding value is a vector that already contains the given accounting policy id
        if let Some(network_accounting_policies) = Self::roaming_network_accounting_policies(roaming_network_id) {
            info!("Network id key {:?} exists with value {:?}", roaming_network_id, network_accounting_policies);
            let not_network_contains_accounting_policy =
                !network_accounting_policies.contains(&roaming_accounting_policy_id);
            ensure!(not_network_contains_accounting_policy, "Network already contains the given accounting policy id");
            info!("Network id key exists but its vector value does not contain the given accounting policy id");
            <RoamingNetworkAccountingPolicies<T>>::mutate(roaming_network_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_accounting_policy_id);
                }
            });
            info!(
                "Associated accounting policy {:?} with network {:?}",
                roaming_accounting_policy_id,
                roaming_network_id
            );
            Ok(())
        } else {
            info!(
                "Network id key does not yet exist. Creating the network key {:?} and appending the accounting policy \
                 id {:?} to its vector value",
                roaming_network_id,
                roaming_accounting_policy_id
            );
            <RoamingNetworkAccountingPolicies<T>>::insert(roaming_network_id, &vec![roaming_accounting_policy_id]);
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

    fn next_roaming_accounting_policy_id() -> Result<T::RoamingAccountingPolicyIndex, DispatchError> {
        let roaming_accounting_policy_id = Self::roaming_accounting_policies_count();
        if roaming_accounting_policy_id == <T::RoamingAccountingPolicyIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingAccountingPolicies count overflow"));
        }
        Ok(roaming_accounting_policy_id)
    }

    fn insert_roaming_accounting_policy(
        owner: &T::AccountId,
        roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
        roaming_accounting_policy: RoamingAccountingPolicy,
    ) {
        // Create and store roaming accounting_policy
        <RoamingAccountingPolicies<T>>::insert(roaming_accounting_policy_id, roaming_accounting_policy);
        <RoamingAccountingPoliciesCount<T>>::put(roaming_accounting_policy_id + One::one());
        <RoamingAccountingPolicyOwners<T>>::insert(roaming_accounting_policy_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex) {
        <RoamingAccountingPolicyOwners<T>>::insert(roaming_accounting_policy_id, to);
    }
}
