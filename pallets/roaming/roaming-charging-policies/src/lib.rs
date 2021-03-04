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
#[macro_use]
extern crate alloc; // Required to use Vec

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config: frame_system::Config + roaming_operators::Config + roaming_networks::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type RoamingChargingPolicyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingChargingPolicy(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
// Generic type parameters - Balance
pub struct RoamingChargingPolicyConfig<U, V> {
    pub policy_next_charging_at_block: U,
    pub policy_delay_after_billing_in_blocks: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Trait>::RoamingChargingPolicyIndex,
        <T as roaming_networks::Config>::RoamingNetworkIndex,
        <T as roaming_operators::Config>::RoamingOperatorIndex,
        <T as frame_system::Config>::BlockNumber,
    {
        /// A roaming charging_policy is created. (owner, roaming_charging_policy_id)
        Created(AccountId, RoamingChargingPolicyIndex),
        /// A roaming charging_policy is transferred. (from, to, roaming_charging_policy_id)
        Transferred(AccountId, AccountId, RoamingChargingPolicyIndex),
        /// A roaming charging_policy configuration
        RoamingChargingPolicyConfigSet(AccountId, RoamingChargingPolicyIndex, BlockNumber, BlockNumber),
        /// A roaming charging_policy is assigned to a operator. (owner of network, roaming_charging_policy_id, roaming_operator_id)
        AssignedChargingPolicyToOperator(AccountId, RoamingChargingPolicyIndex, RoamingOperatorIndex),
        /// A roaming charging_policy is assigned to a network. (owner of network, roaming_charging_policy_id, roaming_network_id)
        AssignedChargingPolicyToNetwork(AccountId, RoamingChargingPolicyIndex, RoamingNetworkIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingChargingPolicies {
        /// Stores all the roaming charging_policy, key is the roaming charging_policy id / index
        pub RoamingChargingPolicies get(fn roaming_charging_policy): map hasher(opaque_blake2_256) T::RoamingChargingPolicyIndex => Option<RoamingChargingPolicy>;

        /// Stores the total number of roaming charging_policies. i.e. the next roaming charging_policy index
        pub RoamingChargingPoliciesCount get(fn roaming_charging_policies_count): T::RoamingChargingPolicyIndex;

        /// Get roaming charging_policy owner
        pub RoamingChargingPolicyOwners get(fn roaming_charging_policy_owner): map hasher(opaque_blake2_256) T::RoamingChargingPolicyIndex => Option<T::AccountId>;

        /// Get roaming charging_policy config
        pub RoamingChargingPolicyConfigs get(fn roaming_charging_policy_configs): map hasher(opaque_blake2_256) T::RoamingChargingPolicyIndex => Option<RoamingChargingPolicyConfig<T::BlockNumber, T::BlockNumber>>;

        /// Get roaming charging_policy network
        pub RoamingChargingPolicyNetwork get(fn roaming_charging_policy_network): map hasher(opaque_blake2_256) T::RoamingChargingPolicyIndex => Option<T::RoamingNetworkIndex>;

        /// Get roaming network's charging policies
        pub RoamingNetworkChargingPolicies get(fn roaming_network_charging_policies): map hasher(opaque_blake2_256) T::RoamingNetworkIndex => Option<Vec<T::RoamingChargingPolicyIndex>>;

        /// Get roaming charging_policy operator
        pub RoamingChargingPolicyOperator get(fn roaming_charging_policy_operator): map hasher(opaque_blake2_256) T::RoamingChargingPolicyIndex => Option<T::RoamingOperatorIndex>;

        /// Get roaming operator's charging policies
        pub RoamingOperatorChargingPolicies get(fn roaming_operator_charging_policies): map hasher(opaque_blake2_256) T::RoamingOperatorIndex => Option<Vec<T::RoamingChargingPolicyIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming charging_policy
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_charging_policy_id = Self::next_roaming_charging_policy_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming charging_policy
            let roaming_charging_policy = RoamingChargingPolicy(unique_id);
            Self::insert_roaming_charging_policy(&sender, roaming_charging_policy_id, roaming_charging_policy);

            Self::deposit_event(RawEvent::Created(sender, roaming_charging_policy_id));
        }

        /// Transfer a roaming charging_policy to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, roaming_charging_policy_id: T::RoamingChargingPolicyIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_charging_policy_owner(roaming_charging_policy_id) == Some(sender.clone()), "Only owner can transfer roaming charging_policy");

            Self::update_owner(&to, roaming_charging_policy_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_charging_policy_id));
        }

        /// Set roaming charging_policy config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_config(
            origin,
            roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
            _policy_next_charging_at_block: Option<T::BlockNumber>,
            _policy_delay_after_billing_in_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming charging policy id whose config we want to change actually exists
            let is_roaming_charging_policy = Self::exists_roaming_charging_policy(roaming_charging_policy_id).is_ok();
            ensure!(is_roaming_charging_policy, "RoamingChargingPolicy does not exist");

            // Ensure that the caller is owner of the charging policy config they are trying to change
            ensure!(Self::roaming_charging_policy_owner(roaming_charging_policy_id) == Some(sender.clone()), "Only owner can set config for roaming charging_policy");

            // let is_owned_by_parent_relationship = Self::is_owned_by_required_parent_relationship(roaming_charging_policy_id, sender.clone()).is_ok();
            // ensure!(is_owned_by_parent_relationship, "Ownership by parent does not exist");

            let policy_next_charging_at_block = match _policy_next_charging_at_block {
                Some(value) => value,
                None => Default::default() // Default
            };
            let policy_delay_after_billing_in_blocks = match _policy_delay_after_billing_in_blocks {
                Some(value) => value,
                None => Default::default() // <timestamp::Module<T>>::get() // Default
            };

            // Check if a roaming charging policy config already exists with the given roaming charging policy id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_charging_policy_config_index(roaming_charging_policy_id).is_ok() {
                debug::info!("Mutating values");
                <RoamingChargingPolicyConfigs<T>>::mutate(roaming_charging_policy_id, |policy_config| {
                    if let Some(_policy_config) = policy_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _policy_config.policy_next_charging_at_block = policy_next_charging_at_block.clone();
                        _policy_config.policy_delay_after_billing_in_blocks = policy_delay_after_billing_in_blocks.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_policy_config = <RoamingChargingPolicyConfigs<T>>::get(roaming_charging_policy_id);
                if let Some(_policy_config) = fetched_policy_config {
                    debug::info!("Latest field policy_next_charging_at_block {:#?}", _policy_config.policy_next_charging_at_block);
                    debug::info!("Latest field policy_delay_after_billing_in_blocks {:#?}", _policy_config.policy_delay_after_billing_in_blocks);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new roaming charging_policy config instance with the input params
                let roaming_charging_policy_config_instance = RoamingChargingPolicyConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    policy_next_charging_at_block: policy_next_charging_at_block.clone(),
                    policy_delay_after_billing_in_blocks: policy_delay_after_billing_in_blocks.clone()
                };

                <RoamingChargingPolicyConfigs<T>>::insert(
                    roaming_charging_policy_id,
                    &roaming_charging_policy_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_policy_config = <RoamingChargingPolicyConfigs<T>>::get(roaming_charging_policy_id);
                if let Some(_policy_config) = fetched_policy_config {
                    debug::info!("Inserted field policy_next_charging_at_block {:#?}", _policy_config.policy_next_charging_at_block);
                    debug::info!("Inserted field policy_delay_after_billing_in_blocks {:#?}", _policy_config.policy_delay_after_billing_in_blocks);
                }
            }

            Self::deposit_event(RawEvent::RoamingChargingPolicyConfigSet(
                sender,
                roaming_charging_policy_id,
                policy_next_charging_at_block,
                policy_delay_after_billing_in_blocks
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_charging_policy_to_network(
            origin,
            roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
            roaming_network_id: T::RoamingNetworkIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_network = <roaming_networks::Module<T>>
                ::exists_roaming_network(roaming_network_id).is_ok();
            ensure!(is_roaming_network, "RoamingNetwork does not exist");

            // Ensure that caller of the function is the owner of the network id to assign the charging_policy to
            ensure!(
                <roaming_networks::Module<T>>::is_roaming_network_owner(roaming_network_id, sender.clone()).is_ok(),
                "Only the roaming network owner can assign itself a roaming charging policy"
            );

            Self::associate_charging_policy_with_network(roaming_charging_policy_id, roaming_network_id)
                .expect("Unable to associate charging policy with network");

            // Ensure that the given charging_policy id already exists
            let roaming_charging_policy = Self::roaming_charging_policy(roaming_charging_policy_id);
            ensure!(roaming_charging_policy.is_some(), "Invalid roaming_charging_policy_id");

            // Ensure that the charging_policy is not already owned by a different network
            // Unassign the charging_policy from any existing network since it may only be owned by one network
            <RoamingChargingPolicyNetwork<T>>::remove(roaming_charging_policy_id);

            // Assign the charging_policy owner to the given network (even if already belongs to them)
            <RoamingChargingPolicyNetwork<T>>::insert(roaming_charging_policy_id, roaming_network_id);

            Self::deposit_event(RawEvent::AssignedChargingPolicyToNetwork(sender, roaming_charging_policy_id, roaming_network_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_charging_policy_to_operator(
            origin,
            roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
            roaming_operator_id: T::RoamingOperatorIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_operator = <roaming_operators::Module<T>>
                ::exists_roaming_operator(roaming_operator_id).is_ok();
            ensure!(is_roaming_operator, "RoamingOperator does not exist");

            // Ensure that caller of the function is the owner of the operator id to assign the charging_policy to
            ensure!(
                <roaming_operators::Module<T>>::is_roaming_operator_owner(roaming_operator_id, sender.clone()).is_ok(),
                "Only the roaming operator owner can assign itself a roaming charging policy"
            );

            Self::associate_charging_policy_with_operator(roaming_charging_policy_id, roaming_operator_id)
                .expect("Unable to associate charging policy with operator");

            // Ensure that the given charging_policy id already exists
            let roaming_charging_policy = Self::roaming_charging_policy(roaming_charging_policy_id);
            ensure!(roaming_charging_policy.is_some(), "Invalid roaming_charging_policy_id");

            // Ensure that the charging_policy is not already owned by a different operator
            // Unassign the charging_policy from any existing operator since it may only be owned by one operator
            <RoamingChargingPolicyOperator<T>>::remove(roaming_charging_policy_id);

            // Assign the charging_policy owner to the given operator (even if already belongs to them)
            <RoamingChargingPolicyOperator<T>>::insert(roaming_charging_policy_id, roaming_operator_id);

            Self::deposit_event(RawEvent::AssignedChargingPolicyToOperator(sender, roaming_charging_policy_id, roaming_operator_id));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_roaming_charging_policy_owner(
        roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_charging_policy_owner(&roaming_charging_policy_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of RoamingChargingPolicy"
        );
        Ok(())
    }

    pub fn is_owned_by_required_parent_relationship(
        roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Get the charging policy operator id associated with the operator of the given charging policy id"
        );
        let charging_policy_operator_id = Self::roaming_charging_policy_operator(roaming_charging_policy_id);

        if let Some(_charging_policy_operator_id) = charging_policy_operator_id {
            // Ensure that the caller is owner of the operator id associated with the charging policy
            ensure!(
                (<roaming_operators::Module<T>>::is_roaming_operator_owner(
                    _charging_policy_operator_id.clone(),
                    sender.clone()
                ))
                .is_ok(),
                "Only owner of the operator id associated with the given charging policy can set an associated \
                 roaming charging policy config"
            );
        } else {
            // There must be a charging policy operator id associated with the charging policy
            return Err(DispatchError::Other("RoamingChargingPolicyOperator does not exist"));
        }
        Ok(())
    }

    pub fn exists_roaming_charging_policy(
        roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
    ) -> Result<RoamingChargingPolicy, DispatchError> {
        match Self::roaming_charging_policy(roaming_charging_policy_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("RoamingChargingPolicy does not exist")),
        }
    }

    pub fn exists_roaming_charging_policy_config(
        roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
    ) -> Result<(), DispatchError> {
        match Self::roaming_charging_policy_configs(roaming_charging_policy_id) {
            Some(_) => Ok(()),
            None => Err(DispatchError::Other("RoamingChargingPolicyConfig does not exist")),
        }
    }

    pub fn has_value_for_charging_policy_config_index(
        roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if charging policy config has a value that is defined");
        let fetched_policy_config = <RoamingChargingPolicyConfigs<T>>::get(roaming_charging_policy_id);
        if let Some(_) = fetched_policy_config {
            debug::info!("Found value for charging policy config");
            return Ok(());
        }
        debug::info!("No value for charging policy config");
        Err(DispatchError::Other("No value for charging policy config"))
    }

    /// Only push the charging policy id onto the end of the vector if it does not already exist
    pub fn associate_charging_policy_with_network(
        roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
        roaming_network_id: T::RoamingNetworkIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given network id already exists as a key,
        // and where its corresponding value is a vector that already contains the given charging policy id
        if let Some(network_charging_policies) = Self::roaming_network_charging_policies(roaming_network_id) {
            debug::info!("Network id key {:?} exists with value {:?}", roaming_network_id, network_charging_policies);
            let not_network_contains_charging_policy = !network_charging_policies.contains(&roaming_charging_policy_id);
            ensure!(not_network_contains_charging_policy, "Network already contains the given charging policy id");
            debug::info!("Network id key exists but its vector value does not contain the given charging policy id");
            <RoamingNetworkChargingPolicies<T>>::mutate(roaming_network_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_charging_policy_id);
                }
            });
            debug::info!(
                "Associated charging policy {:?} with network {:?}",
                roaming_charging_policy_id,
                roaming_network_id
            );
            Ok(())
        } else {
            debug::info!(
                "Network id key does not yet exist. Creating the network key {:?} and appending the charging policy \
                 id {:?} to its vector value",
                roaming_network_id,
                roaming_charging_policy_id
            );
            <RoamingNetworkChargingPolicies<T>>::insert(roaming_network_id, &vec![roaming_charging_policy_id]);
            Ok(())
        }
    }

    /// Only push the charging policy id onto the end of the vector if it does not already exist
    pub fn associate_charging_policy_with_operator(
        roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
        roaming_operator_id: T::RoamingOperatorIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given operator id already exists as a key,
        // and where its corresponding value is a vector that already contains the given charging policy id
        if let Some(operator_charging_policies) = Self::roaming_operator_charging_policies(roaming_operator_id) {
            debug::info!(
                "Operator id key {:?} exists with value {:?}",
                roaming_operator_id,
                operator_charging_policies
            );
            let not_operator_contains_charging_policy =
                !operator_charging_policies.contains(&roaming_charging_policy_id);
            ensure!(not_operator_contains_charging_policy, "Operator already contains the given charging policy id");
            debug::info!("Operator id key exists but its vector value does not contain the given charging policy id");
            <RoamingOperatorChargingPolicies<T>>::mutate(roaming_operator_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_charging_policy_id);
                }
            });
            debug::info!(
                "Associated charging policy {:?} with operator {:?}",
                roaming_charging_policy_id,
                roaming_operator_id
            );
            Ok(())
        } else {
            debug::info!(
                "Operator id key does not yet exist. Creating the operator key {:?} and appending the charging policy \
                 id {:?} to its vector value",
                roaming_operator_id,
                roaming_charging_policy_id
            );
            <RoamingOperatorChargingPolicies<T>>::insert(roaming_operator_id, &vec![roaming_charging_policy_id]);
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

    fn next_roaming_charging_policy_id() -> Result<T::RoamingChargingPolicyIndex, DispatchError> {
        let roaming_charging_policy_id = Self::roaming_charging_policies_count();
        if roaming_charging_policy_id == <T::RoamingChargingPolicyIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingChargingPolicies count overflow"));
        }
        Ok(roaming_charging_policy_id)
    }

    fn insert_roaming_charging_policy(
        owner: &T::AccountId,
        roaming_charging_policy_id: T::RoamingChargingPolicyIndex,
        roaming_charging_policy: RoamingChargingPolicy,
    ) {
        // Create and store roaming charging_policy
        <RoamingChargingPolicies<T>>::insert(roaming_charging_policy_id, roaming_charging_policy);
        <RoamingChargingPoliciesCount<T>>::put(roaming_charging_policy_id + One::one());
        <RoamingChargingPolicyOwners<T>>::insert(roaming_charging_policy_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_charging_policy_id: T::RoamingChargingPolicyIndex) {
        <RoamingChargingPolicyOwners<T>>::insert(roaming_charging_policy_id, to);
    }
}
