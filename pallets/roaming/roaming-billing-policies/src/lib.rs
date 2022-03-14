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
pub trait Config: frame_system::Config + roaming_operators::Config + roaming_networks::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RoamingBillingPolicyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingBillingPolicy(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
// Generic type parameters - Balance
pub struct RoamingBillingPolicySetting<U, V> {
    pub policy_next_billing_at_block: U,
    pub policy_frequency_in_blocks: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::RoamingBillingPolicyIndex,
        <T as roaming_networks::Config>::RoamingNetworkIndex,
        <T as roaming_operators::Config>::RoamingOperatorIndex,
        <T as frame_system::Config>::BlockNumber,
    {
        /// A roaming billing_policy is created. (owner, roaming_billing_policy_id)
        Created(AccountId, RoamingBillingPolicyIndex),
        /// A roaming billing_policy is transferred. (from, to, roaming_billing_policy_id)
        Transferred(AccountId, AccountId, RoamingBillingPolicyIndex),
        /// A roaming billing_policy configuration
        RoamingBillingPolicySettingSet(AccountId, RoamingBillingPolicyIndex, BlockNumber, BlockNumber),
        /// A roaming billing_policy is assigned to a operator. (owner of network, roaming_billing_policy_id, roaming_operator_id)
        AssignedBillingPolicyToOperator(AccountId, RoamingBillingPolicyIndex, RoamingOperatorIndex),
        /// A roaming billing_policy is assigned to a network. (owner of network, roaming_billing_policy_id, roaming_network_id)
        AssignedBillingPolicyToNetwork(AccountId, RoamingBillingPolicyIndex, RoamingNetworkIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as RoamingBillingPolicies {
        /// Stores all the roaming billing_policy, key is the roaming billing_policy id / index
        pub RoamingBillingPolicies get(fn roaming_billing_policy): map hasher(opaque_blake2_256) T::RoamingBillingPolicyIndex => Option<RoamingBillingPolicy>;

        /// Stores the total number of roaming billing_policies. i.e. the next roaming billing_policy index
        pub RoamingBillingPoliciesCount get(fn roaming_billing_policies_count): T::RoamingBillingPolicyIndex;

        /// Get roaming billing_policy owner
        pub RoamingBillingPolicyOwners get(fn roaming_billing_policy_owner): map hasher(opaque_blake2_256) T::RoamingBillingPolicyIndex => Option<T::AccountId>;

        /// Get roaming billing_policy config
        pub RoamingBillingPolicySettings get(fn roaming_billing_policy_settings): map hasher(opaque_blake2_256) T::RoamingBillingPolicyIndex => Option<RoamingBillingPolicySetting<T::BlockNumber, T::BlockNumber>>;

        /// Get roaming billing_policy network
        pub RoamingBillingPolicyNetwork get(fn roaming_billing_policy_network): map hasher(opaque_blake2_256) T::RoamingBillingPolicyIndex => Option<T::RoamingNetworkIndex>;

        /// Get roaming network's billing policies
        pub RoamingNetworkBillingPolicies get(fn roaming_network_billing_policies): map hasher(opaque_blake2_256) T::RoamingNetworkIndex => Option<Vec<T::RoamingBillingPolicyIndex>>;

        /// Get roaming billing_policy operator
        pub RoamingBillingPolicyOperator get(fn roaming_billing_policy_operator): map hasher(opaque_blake2_256) T::RoamingBillingPolicyIndex => Option<T::RoamingOperatorIndex>;

        /// Get roaming operator's billing policies
        pub RoamingOperatorBillingPolicies get(fn roaming_operator_billing_policies): map hasher(opaque_blake2_256) T::RoamingOperatorIndex => Option<Vec<T::RoamingBillingPolicyIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming billing_policy
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_billing_policy_id = Self::next_roaming_billing_policy_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming billing_policy
            let roaming_billing_policy = RoamingBillingPolicy(unique_id);
            Self::insert_roaming_billing_policy(&sender, roaming_billing_policy_id, roaming_billing_policy);

            Self::deposit_event(RawEvent::Created(sender, roaming_billing_policy_id));
        }

        /// Transfer a roaming billing_policy to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_billing_policy_id: T::RoamingBillingPolicyIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_billing_policy_owner(roaming_billing_policy_id) == Some(sender.clone()), "Only owner can transfer roaming billing_policy");

            Self::update_owner(&to, roaming_billing_policy_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_billing_policy_id));
        }

        /// Set roaming billing_policy config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_config(
            origin,
            roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
            _policy_next_billing_at_block: Option<T::BlockNumber>,
            _policy_frequency_in_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming billing policy id whose config we want to change actually exists
            let is_roaming_billing_policy = Self::exists_roaming_billing_policy(roaming_billing_policy_id).is_ok();
            ensure!(is_roaming_billing_policy, "RoamingBillingPolicy does not exist");

            // Ensure that the caller is owner of the billing policy config they are trying to change
            ensure!(Self::roaming_billing_policy_owner(roaming_billing_policy_id) == Some(sender.clone()), "Only owner can set config for roaming billing_policy");

            // let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_billing_policy_id, sender.clone()).is_ok();
            // ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            let policy_next_billing_at_block = match _policy_next_billing_at_block {
                Some(value) => value,
                None => Default::default() // Default
            };
            let policy_frequency_in_blocks = match _policy_frequency_in_blocks {
                Some(value) => value,
                None => Default::default() // <timestamp::Pallet<T>>::get() // Default
            };

            // Check if a roaming billing policy config already exists with the given roaming billing policy id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_billing_policy_setting_index(roaming_billing_policy_id).is_ok() {
                info!("Mutating values");
                <RoamingBillingPolicySettings<T>>::mutate(roaming_billing_policy_id, |policy_setting| {
                    if let Some(_policy_setting) = policy_setting {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _policy_setting.policy_next_billing_at_block = policy_next_billing_at_block.clone();
                        _policy_setting.policy_frequency_in_blocks = policy_frequency_in_blocks.clone();
                    }
                });
                info!("Checking mutated values");
                let fetched_policy_setting = <RoamingBillingPolicySettings<T>>::get(roaming_billing_policy_id);
                if let Some(_policy_setting) = fetched_policy_setting {
                    info!("Latest field policy_next_billing_at_block {:#?}", _policy_setting.policy_next_billing_at_block);
                    info!("Latest field policy_frequency_in_blocks {:#?}", _policy_setting.policy_frequency_in_blocks);
                }
            } else {
                info!("Inserting values");

                // Create a new roaming billing_policy config instance with the input params
                let roaming_billing_policy_setting_instance = RoamingBillingPolicySetting {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    policy_next_billing_at_block: policy_next_billing_at_block.clone(),
                    policy_frequency_in_blocks: policy_frequency_in_blocks.clone()
                };

                <RoamingBillingPolicySettings<T>>::insert(
                    roaming_billing_policy_id,
                    &roaming_billing_policy_setting_instance
                );

                info!("Checking inserted values");
                let fetched_policy_setting = <RoamingBillingPolicySettings<T>>::get(roaming_billing_policy_id);
                if let Some(_policy_setting) = fetched_policy_setting {
                    info!("Inserted field policy_next_billing_at_block {:#?}", _policy_setting.policy_next_billing_at_block);
                    info!("Inserted field policy_frequency_in_blocks {:#?}", _policy_setting.policy_frequency_in_blocks);
                }
            }

            Self::deposit_event(RawEvent::RoamingBillingPolicySettingSet(
                sender,
                roaming_billing_policy_id,
                policy_next_billing_at_block,
                policy_frequency_in_blocks
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_billing_policy_to_network(
            origin,
            roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
            roaming_network_id: T::RoamingNetworkIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_network = <roaming_networks::Pallet<T>>
                ::exists_roaming_network(roaming_network_id).is_ok();
            ensure!(is_roaming_network, "RoamingNetwork does not exist");

            // Ensure that caller of the function is the owner of the network id to assign the billing_policy to
            ensure!(
                <roaming_networks::Pallet<T>>::is_roaming_network_owner(roaming_network_id, sender.clone()).is_ok(),
                "Only the roaming network owner can assign itself a roaming billing policy"
            );

            Self::associate_billing_policy_with_network(roaming_billing_policy_id, roaming_network_id)
                .expect("Unable to associate billing policy with network");

            // Ensure that the given billing_policy id already exists
            let roaming_billing_policy = Self::roaming_billing_policy(roaming_billing_policy_id);
            ensure!(roaming_billing_policy.is_some(), "Invalid roaming_billing_policy_id");

            // Ensure that the billing_policy is not already owned by a different network
            // Unassign the billing_policy from any existing network since it may only be owned by one network
            <RoamingBillingPolicyNetwork<T>>::remove(roaming_billing_policy_id);

            // Assign the billing_policy owner to the given network (even if already belongs to them)
            <RoamingBillingPolicyNetwork<T>>::insert(roaming_billing_policy_id, roaming_network_id);

            Self::deposit_event(RawEvent::AssignedBillingPolicyToNetwork(sender, roaming_billing_policy_id, roaming_network_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_billing_policy_to_operator(
            origin,
            roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
            roaming_operator_id: T::RoamingOperatorIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_operator = <roaming_operators::Pallet<T>>
                ::exists_roaming_operator(roaming_operator_id).is_ok();
            ensure!(is_roaming_operator, "RoamingOperator does not exist");

            // Ensure that caller of the function is the owner of the operator id to assign the billing_policy to
            ensure!(
                <roaming_operators::Pallet<T>>::is_roaming_operator_owner(roaming_operator_id, sender.clone()).is_ok(),
                "Only the roaming operator owner can assign itself a roaming billing policy"
            );

            Self::associate_billing_policy_with_operator(roaming_billing_policy_id, roaming_operator_id)
                .expect("Unable to associate billing policy with operator");

            // Ensure that the given billing_policy id already exists
            let roaming_billing_policy = Self::roaming_billing_policy(roaming_billing_policy_id);
            ensure!(roaming_billing_policy.is_some(), "Invalid roaming_billing_policy_id");

            // Ensure that the billing_policy is not already owned by a different operator
            // Unassign the billing_policy from any existing operator since it may only be owned by one operator
            <RoamingBillingPolicyOperator<T>>::remove(roaming_billing_policy_id);

            // Assign the billing_policy owner to the given operator (even if already belongs to them)
            <RoamingBillingPolicyOperator<T>>::insert(roaming_billing_policy_id, roaming_operator_id);

            Self::deposit_event(RawEvent::AssignedBillingPolicyToOperator(sender, roaming_billing_policy_id, roaming_operator_id));
        }
    }
}

impl<T: Config> Module<T> {
    pub fn is_roaming_billing_policy_owner(
        roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_billing_policy_owner(&roaming_billing_policy_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of RoamingBillingPolicy"
        );
        Ok(())
    }

    // pub fn is_owned_by_required_parent_relationship(roaming_billing_policy_id: T::RoamingBillingPolicyIndex, sender:
    // T::AccountId) -> Result<(), DispatchError> {     info!("Get the billing policy operator id associated
    // with the operator of the given billing policy id");     let billing_policy_operator_id =
    // Self::roaming_billing_policy_operator(roaming_billing_policy_id);

    //     if let Some(_billing_policy_operator_id) = billing_policy_operator_id {
    //         // Ensure that the caller is owner of the operator id associated with the billing policy
    //         ensure!((<roaming_operators::Pallet<T>>::is_roaming_operator_owner(
    //                 _billing_policy_operator_id.clone(),
    //                 sender.clone()
    //             )).is_ok(), "Only owner of the operator id associated with the given billing policy can set an
    // associated roaming billing policy config"         );
    //     } else {
    //         // There must be a billing policy operator id associated with the billing policy
    //         return Err(DispatchError::Other("RoamingBillingPolicyOperator does not exist"));
    //     }
    //     Ok(())
    // }

    pub fn exists_roaming_billing_policy(
        roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
    ) -> Result<RoamingBillingPolicy, DispatchError> {
        match Self::roaming_billing_policy(roaming_billing_policy_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("RoamingBillingPolicy does not exist")),
        }
    }

    pub fn exists_roaming_billing_policy_setting(
        roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
    ) -> Result<(), DispatchError> {
        match Self::roaming_billing_policy_settings(roaming_billing_policy_id) {
            Some(_) => Ok(()),
            None => Err(DispatchError::Other("RoamingBillingPolicySetting does not exist")),
        }
    }

    pub fn has_value_for_billing_policy_setting_index(
        roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if billing policy config has a value that is defined");
        let fetched_policy_setting = <RoamingBillingPolicySettings<T>>::get(roaming_billing_policy_id);
        if let Some(_) = fetched_policy_setting {
            info!("Found value for billing policy config");
            return Ok(());
        }
        warn!("No value for billing policy config");
        Err(DispatchError::Other("No value for billing policy config"))
    }

    /// Only push the billing policy id onto the end of the vector if it does not already exist
    pub fn associate_billing_policy_with_network(
        roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
        roaming_network_id: T::RoamingNetworkIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given network id already exists as a key,
        // and where its corresponding value is a vector that already contains the given billing policy id
        if let Some(network_billing_policies) = Self::roaming_network_billing_policies(roaming_network_id) {
            info!("Network id key {:?} exists with value {:?}", roaming_network_id, network_billing_policies);
            let not_network_contains_billing_policy = !network_billing_policies.contains(&roaming_billing_policy_id);
            ensure!(not_network_contains_billing_policy, "Network already contains the given billing policy id");
            info!("Network id key exists but its vector value does not contain the given billing policy id");
            <RoamingNetworkBillingPolicies<T>>::mutate(roaming_network_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_billing_policy_id);
                }
            });
            info!(
                "Associated billing policy {:?} with network {:?}",
                roaming_billing_policy_id,
                roaming_network_id
            );
            Ok(())
        } else {
            info!(
                "Network id key does not yet exist. Creating the network key {:?} and appending the billing policy id \
                 {:?} to its vector value",
                roaming_network_id,
                roaming_billing_policy_id
            );
            <RoamingNetworkBillingPolicies<T>>::insert(roaming_network_id, &vec![roaming_billing_policy_id]);
            Ok(())
        }
    }

    /// Only push the billing policy id onto the end of the vector if it does not already exist
    pub fn associate_billing_policy_with_operator(
        roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
        roaming_operator_id: T::RoamingOperatorIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given operator id already exists as a key,
        // and where its corresponding value is a vector that already contains the given billing policy id
        if let Some(operator_billing_policies) = Self::roaming_operator_billing_policies(roaming_operator_id) {
            info!("Operator id key {:?} exists with value {:?}", roaming_operator_id, operator_billing_policies);
            let not_operator_contains_billing_policy = !operator_billing_policies.contains(&roaming_billing_policy_id);
            ensure!(not_operator_contains_billing_policy, "Operator already contains the given billing policy id");
            info!("Operator id key exists but its vector value does not contain the given billing policy id");
            <RoamingOperatorBillingPolicies<T>>::mutate(roaming_operator_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_billing_policy_id);
                }
            });
            info!(
                "Associated billing policy {:?} with operator {:?}",
                roaming_billing_policy_id,
                roaming_operator_id
            );
            Ok(())
        } else {
            info!(
                "Operator id key does not yet exist. Creating the operator key {:?} and appending the billing policy \
                 id {:?} to its vector value",
                roaming_operator_id,
                roaming_billing_policy_id
            );
            <RoamingOperatorBillingPolicies<T>>::insert(roaming_operator_id, &vec![roaming_billing_policy_id]);
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

    fn next_roaming_billing_policy_id() -> Result<T::RoamingBillingPolicyIndex, DispatchError> {
        let roaming_billing_policy_id = Self::roaming_billing_policies_count();
        if roaming_billing_policy_id == <T::RoamingBillingPolicyIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingBillingPolicies count overflow"));
        }
        Ok(roaming_billing_policy_id)
    }

    fn insert_roaming_billing_policy(
        owner: &T::AccountId,
        roaming_billing_policy_id: T::RoamingBillingPolicyIndex,
        roaming_billing_policy: RoamingBillingPolicy,
    ) {
        // Create and store roaming billing_policy
        <RoamingBillingPolicies<T>>::insert(roaming_billing_policy_id, roaming_billing_policy);
        <RoamingBillingPoliciesCount<T>>::put(roaming_billing_policy_id + One::one());
        <RoamingBillingPolicyOwners<T>>::insert(roaming_billing_policy_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_billing_policy_id: T::RoamingBillingPolicyIndex) {
        <RoamingBillingPolicyOwners<T>>::insert(roaming_billing_policy_id, to);
    }
}
