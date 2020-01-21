#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use runtime_io::hashing::{blake2_128};
use sr_primitives::traits::{Bounded, Member, One, SimpleArithmetic};
use support::traits::{Currency, ExistenceRequirement, Randomness};
/// A runtime module for managing non-fungible tokens
use support::{decl_event, decl_module, decl_storage, ensure, Parameter, debug};
use system::ensure_signed;
use rstd::prelude::*; // Imports Vec
#[macro_use]
extern crate alloc; // Required to use Vec

// use env;
use roaming_operators;
use roaming_networks;
use roaming_accounting_policies;

/// The module's configuration trait.
pub trait Trait: system::Trait +
                 roaming_operators::Trait +
                 roaming_networks::Trait +
                 roaming_accounting_policies::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type RoamingAgreementPolicyIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
	type RoamingAgreementPolicyActivationType: Parameter + Member + Default;
	type RoamingAgreementPolicyExpiry: Parameter + Member + Default;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingAgreementPolicy(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Debug, Decode, Default, Clone, PartialEq)]
// Generic type parameters - Balance
pub struct RoamingAgreementPolicyConfig<U, V> {
    pub policy_activation_type: U, // "passive" or "handover"
    pub policy_expiry: V,
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
        <T as Trait>::RoamingAgreementPolicyIndex,
        <T as Trait>::RoamingAgreementPolicyActivationType,
        <T as Trait>::RoamingAgreementPolicyExpiry,
        <T as roaming_accounting_policies::Trait>::RoamingAccountingPolicyIndex,
        <T as roaming_networks::Trait>::RoamingNetworkIndex,
	{
		/// A roaming agreement_policy is created. (owner, roaming_agreement_policy_id)
		Created(AccountId, RoamingAgreementPolicyIndex),
		/// A roaming agreement_policy is transferred. (from, to, roaming_agreement_policy_id)
		Transferred(AccountId, AccountId, RoamingAgreementPolicyIndex),
        /// A roaming agreement_policy configuration
        RoamingAgreementPolicyConfigSet(AccountId, RoamingAgreementPolicyIndex, RoamingAgreementPolicyActivationType, RoamingAgreementPolicyExpiry),
        /// A roaming agreement_policy is assigned to a accounting_policy. (owner of network, roaming_agreement_policy_id, roaming_accounting_policy_id)
        AssignedAgreementPolicyToAccountingPolicy(AccountId, RoamingAgreementPolicyIndex, RoamingAccountingPolicyIndex),
        /// A roaming agreement_policy is assigned to a network. (owner of network, roaming_agreement_policy_id, roaming_network_id)
        AssignedAgreementPolicyToNetwork(AccountId, RoamingAgreementPolicyIndex, RoamingNetworkIndex),
	}
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingAgreementPolicies {
        /// Stores all the roaming agreement_policy, key is the roaming agreement_policy id / index
        pub RoamingAgreementPolicies get(fn roaming_agreement_policy): map T::RoamingAgreementPolicyIndex => Option<RoamingAgreementPolicy>;

        /// Stores the total number of roaming agreement_policies. i.e. the next roaming agreement_policy index
        pub RoamingAgreementPoliciesCount get(fn roaming_agreement_policies_count): T::RoamingAgreementPolicyIndex;

        /// Get roaming agreement_policy owner
        pub RoamingAgreementPolicyOwners get(fn roaming_agreement_policy_owner): map T::RoamingAgreementPolicyIndex => Option<T::AccountId>;

        /// Get roaming agreement_policy config
        pub RoamingAgreementPolicyConfigs get(fn roaming_agreement_policy_configs): map T::RoamingAgreementPolicyIndex => Option<RoamingAgreementPolicyConfig<T::RoamingAgreementPolicyActivationType, T::RoamingAgreementPolicyExpiry>>;

        /// Get roaming agreement_policy network
        pub RoamingAgreementPolicyNetwork get(fn roaming_agreement_policy_network): map T::RoamingAgreementPolicyIndex => Option<T::RoamingNetworkIndex>;

        /// Get roaming network's agreement policies
        pub RoamingNetworkAgreementPolicies get(fn roaming_network_agreement_policies): map T::RoamingNetworkIndex => Option<Vec<T::RoamingAgreementPolicyIndex>>;

        /// Get roaming agreement_policy accounting_policy
        pub RoamingAgreementPolicyAccountingPolicy get(fn roaming_agreement_policy_accounting_policy): map T::RoamingAgreementPolicyIndex => Option<T::RoamingAccountingPolicyIndex>;

        /// Get roaming accounting_policy's agreement policies
        pub RoamingAccountingPolicyAgreementPolicies get(fn roaming_accounting_policy_agreement_policies): map T::RoamingAccountingPolicyIndex => Option<Vec<T::RoamingAgreementPolicyIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming agreement_policy
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_agreement_policy_id = Self::next_roaming_agreement_policy_id()?;

            let mut unique_id = Self::random_value(&sender);
            // if env::config::get_env() == "TEST" {
            //     unique_id = [0; 16];
            // } else {
                // Generate a random 128bit value
                unique_id = Self::random_value(&sender);
            // }

            // Create and store roaming agreement_policy
            let roaming_agreement_policy = RoamingAgreementPolicy(unique_id);
            Self::insert_roaming_agreement_policy(&sender, roaming_agreement_policy_id, roaming_agreement_policy);

            Self::deposit_event(RawEvent::Created(sender, roaming_agreement_policy_id));
        }

        /// Transfer a roaming agreement_policy to new owner
        pub fn transfer(origin, to: T::AccountId, roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_agreement_policy_owner(roaming_agreement_policy_id) == Some(sender.clone()), "Only owner can transfer roaming agreement_policy");

            Self::update_owner(&to, roaming_agreement_policy_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_agreement_policy_id));
        }

        /// Set roaming agreement_policy config
        pub fn set_config(
            origin,
            roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex,
            _policy_activation_type: Option<T::RoamingAgreementPolicyActivationType>, // "passive" or "handover"
            _policy_expiry: Option<T::RoamingAgreementPolicyExpiry>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the roaming agreement policy id whose config we want to change actually exists
            let is_roaming_agreement_policy = Self::exists_roaming_agreement_policy(roaming_agreement_policy_id).is_ok();
            ensure!(is_roaming_agreement_policy, "RoamingAgreementPolicy does not exist");

            // Ensure that the caller is owner of the agreement policy config they are trying to change
            ensure!(Self::roaming_agreement_policy_owner(roaming_agreement_policy_id) == Some(sender.clone()), "Only owner can set config for roaming agreement_policy");

            let policy_activation_type = match _policy_activation_type {
                Some(value) => value,
                None => Default::default() // Default
            };
            let policy_expiry = match _policy_expiry {
                Some(value) => value,
                None => Default::default() // <timestamp::Module<T>>::get() // Default
            };

            // Check if a roaming agreement policy config already exists with the given roaming agreement policy id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_agreement_policy_config_index(roaming_agreement_policy_id).is_ok() {
                debug::info!("Mutating values");
                <RoamingAgreementPolicyConfigs<T>>::mutate(roaming_agreement_policy_id, |policy_config| {
                    if let Some(_policy_config) = policy_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _policy_config.policy_activation_type = policy_activation_type.clone();
                        _policy_config.policy_expiry = policy_expiry.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_policy_config = <RoamingAgreementPolicyConfigs<T>>::get(roaming_agreement_policy_id);
                if let Some(_policy_config) = fetched_policy_config {
                    debug::info!("Latest field policy_activation_type {:#?}", _policy_config.policy_activation_type);
                    debug::info!("Latest field policy_expiry {:#?}", _policy_config.policy_expiry);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new roaming agreement_policy config instance with the input params
                let roaming_agreement_policy_config_instance = RoamingAgreementPolicyConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    policy_activation_type: policy_activation_type.clone(),
                    policy_expiry: policy_expiry.clone()
                };

                <RoamingAgreementPolicyConfigs<T>>::insert(
                    roaming_agreement_policy_id,
                    &roaming_agreement_policy_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_policy_config = <RoamingAgreementPolicyConfigs<T>>::get(roaming_agreement_policy_id);
                if let Some(_policy_config) = fetched_policy_config {
                    debug::info!("Inserted field policy_activation_type {:#?}", _policy_config.policy_activation_type);
                    debug::info!("Inserted field policy_expiry {:#?}", _policy_config.policy_expiry);
                }
            }

            Self::deposit_event(RawEvent::RoamingAgreementPolicyConfigSet(
                sender,
                roaming_agreement_policy_id,
                policy_activation_type,
                policy_expiry
            ));
        }

        // Optional and only used for organizational purposes to know which networks may want to use it.
        // Since we want users to be allowed to create and configure multiple policies and profiles for reuse.
        // They will then be associated with any specific networks when the user creates each roaming base profile.
        pub fn assign_agreement_policy_to_network(
            origin,
            roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex,
            roaming_network_id: T::RoamingNetworkIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_network = <roaming_networks::Module<T>>
                ::exists_roaming_network(roaming_network_id).is_ok();
            ensure!(is_roaming_network, "RoamingNetwork does not exist");

            // Ensure that caller of the function is the owner of the network id to assign the agreement_policy to
            ensure!(
                <roaming_networks::Module<T>>::is_roaming_network_owner(roaming_network_id, sender.clone()).is_ok(),
                "Only the roaming network owner can assign itself a roaming agreement policy"
            );

            Self::associate_agreement_policy_with_network(roaming_agreement_policy_id, roaming_network_id)
                .expect("Unable to associate agreement policy with network");

            // Ensure that the given agreement_policy id already exists
            let roaming_agreement_policy = Self::roaming_agreement_policy(roaming_agreement_policy_id);
            ensure!(roaming_agreement_policy.is_some(), "Invalid roaming_agreement_policy_id");

            // Ensure that the agreement_policy is not already owned by a different network
            // Unassign the agreement_policy from any existing network since it may only be owned by one network
            <RoamingAgreementPolicyNetwork<T>>::remove(roaming_agreement_policy_id);

            // Assign the agreement_policy owner to the given network (even if already belongs to them)
            <RoamingAgreementPolicyNetwork<T>>::insert(roaming_agreement_policy_id, roaming_network_id);

            Self::deposit_event(RawEvent::AssignedAgreementPolicyToNetwork(sender, roaming_agreement_policy_id, roaming_network_id));
        }

        pub fn assign_agreement_policy_to_accounting_policy(
            origin,
            roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex,
            roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_accounting_policy = <roaming_accounting_policies::Module<T>>
                ::exists_roaming_accounting_policy(roaming_accounting_policy_id).is_ok();
            ensure!(is_roaming_accounting_policy, "RoamingAccountingPolicy does not exist");

            // Ensure that caller of the function is the owner of the accounting_policy id to assign the agreement_policy to
            ensure!(
                <roaming_accounting_policies::Module<T>>::is_roaming_accounting_policy_owner(roaming_accounting_policy_id, sender.clone()).is_ok(),
                "Only the roaming accounting_policy owner can assign itself a roaming agreement policy"
            );

            Self::associate_agreement_policy_with_accounting_policy(roaming_agreement_policy_id, roaming_accounting_policy_id)
                .expect("Unable to associate agreement policy with accounting_policy");

            // Ensure that the given agreement_policy id already exists
            let roaming_agreement_policy = Self::roaming_agreement_policy(roaming_agreement_policy_id);
            ensure!(roaming_agreement_policy.is_some(), "Invalid roaming_agreement_policy_id");

            // Ensure that the agreement_policy is not already owned by a different accounting_policy
            // Unassign the agreement_policy from any existing accounting_policy since it may only be owned by one accounting_policy
            <RoamingAgreementPolicyAccountingPolicy<T>>::remove(roaming_agreement_policy_id);

            // Assign the agreement_policy owner to the given accounting_policy (even if already belongs to them)
            <RoamingAgreementPolicyAccountingPolicy<T>>::insert(roaming_agreement_policy_id, roaming_accounting_policy_id);

            Self::deposit_event(RawEvent::AssignedAgreementPolicyToAccountingPolicy(sender, roaming_agreement_policy_id, roaming_accounting_policy_id));
        }
    }
}

impl<T: Trait> Module<T> {
	pub fn is_roaming_agreement_policy_owner(roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex, sender: T::AccountId) -> Result<(), &'static str> {
        ensure!(
            Self::roaming_agreement_policy_owner(&roaming_agreement_policy_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of RoamingAgreementPolicy"
        );
        Ok(())
    }

    // Note: Not required
    // pub fn is_owned_by_required_parent_relationship(roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex, sender: T::AccountId) -> Result<(), &'static str> {
    //     debug::info!("Get the network id associated with the network of the given agreement policy id");
    //     let agreement_policy_network_id = Self::roaming_agreement_policy_network(roaming_agreement_policy_id);

    //     if let Some(_agreement_policy_network_id) = agreement_policy_network_id {
    //         // Ensure that the caller is owner of the network id associated with the agreement policy
    //         ensure!((<roaming_networks::Module<T>>::is_roaming_network_owner(
    //                 _agreement_policy_network_id.clone(),
    //                 sender.clone()
    //             )).is_ok(), "Only owner of the network id associated with the given agreement policy can set an associated roaming agreement policy config"
    //         );
    //     } else {
    //         // There must be a network id associated with the agreement policy
    //         return Err("RoamingAgreementPolicyNetwork does not exist");
    //     }
    //     Ok(())
    // }

    pub fn exists_roaming_agreement_policy(roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex) -> Result<RoamingAgreementPolicy, &'static str> {
        match Self::roaming_agreement_policy(roaming_agreement_policy_id) {
            Some(value) => Ok(value),
            None => Err("RoamingAgreementPolicy does not exist")
        }
    }

    pub fn exists_roaming_agreement_policy_config(roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex) -> Result<(), &'static str> {
        match Self::roaming_agreement_policy_configs(roaming_agreement_policy_id) {
            Some(value) => Ok(()),
            None => Err("RoamingAgreementPolicyConfig does not exist")
        }
    }

    pub fn has_value_for_agreement_policy_config_index(roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex)
        -> Result<(), &'static str> {
        debug::info!("Checking if agreement policy config has a value that is defined");
        let fetched_policy_config = <RoamingAgreementPolicyConfigs<T>>::get(roaming_agreement_policy_id);
        if let Some(value) = fetched_policy_config {
            debug::info!("Found value for agreement policy config");
            return Ok(());
        }
        debug::info!("No value for agreement policy config");
        Err("No value for agreement policy config")
    }

    /// Only push the agreement policy id onto the end of the vector if it does not already exist
    pub fn associate_agreement_policy_with_network(
        roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex,
        roaming_network_id: T::RoamingNetworkIndex
    ) -> Result<(), &'static str>
    {
        // Early exit with error since do not want to append if the given network id already exists as a key,
        // and where its corresponding value is a vector that already contains the given agreement policy id
        if let Some(network_agreement_policies) = Self::roaming_network_agreement_policies(roaming_network_id) {
            debug::info!("Network id key {:?} exists with value {:?}", roaming_network_id, network_agreement_policies);
            let not_network_contains_agreement_policy = !network_agreement_policies.contains(&roaming_agreement_policy_id);
            ensure!(not_network_contains_agreement_policy, "Network already contains the given agreement policy id");
            debug::info!("Network id key exists but its vector value does not contain the given agreement policy id");
            <RoamingNetworkAgreementPolicies<T>>::mutate(roaming_network_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_agreement_policy_id);
                }
            });
            debug::info!("Associated agreement policy {:?} with network {:?}", roaming_agreement_policy_id, roaming_network_id);
            Ok(())
        } else {
            debug::info!("Network id key does not yet exist. Creating the network key {:?} and appending the agreement policy id {:?} to its vector value", roaming_network_id, roaming_agreement_policy_id);
            <RoamingNetworkAgreementPolicies<T>>::insert(roaming_network_id, &vec![roaming_agreement_policy_id]);
            Ok(())
        }
    }

    /// Only push the agreement policy id onto the end of the vector if it does not already exist
    pub fn associate_agreement_policy_with_accounting_policy(
        roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex,
        roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex
    ) -> Result<(), &'static str>
    {
        // Early exit with error since do not want to append if the given accounting_policy id already exists as a key,
        // and where its corresponding value is a vector that already contains the given agreement policy id
        if let Some(accounting_policy_agreement_policies) = Self::roaming_accounting_policy_agreement_policies(roaming_accounting_policy_id) {
            debug::info!("AccountingPolicy id key {:?} exists with value {:?}", roaming_accounting_policy_id, accounting_policy_agreement_policies);
            let not_accounting_policy_contains_agreement_policy = !accounting_policy_agreement_policies.contains(&roaming_agreement_policy_id);
            ensure!(not_accounting_policy_contains_agreement_policy, "AccountingPolicy already contains the given agreement policy id");
            debug::info!("AccountingPolicy id key exists but its vector value does not contain the given agreement policy id");
            <RoamingAccountingPolicyAgreementPolicies<T>>::mutate(roaming_accounting_policy_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_agreement_policy_id);
                }
            });
            debug::info!("Associated agreement policy {:?} with accounting_policy {:?}", roaming_agreement_policy_id, roaming_accounting_policy_id);
            Ok(())
        } else {
            debug::info!("AccountingPolicy id key does not yet exist. Creating the accounting_policy key {:?} and appending the agreement policy id {:?} to its vector value", roaming_accounting_policy_id, roaming_agreement_policy_id);
            <RoamingAccountingPolicyAgreementPolicies<T>>::insert(roaming_accounting_policy_id, &vec![roaming_agreement_policy_id]);
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

    fn next_roaming_agreement_policy_id() -> Result<T::RoamingAgreementPolicyIndex, &'static str> {
        let roaming_agreement_policy_id = Self::roaming_agreement_policies_count();
        if roaming_agreement_policy_id == <T::RoamingAgreementPolicyIndex as Bounded>::max_value() {
            return Err("RoamingAgreementPolicies count overflow");
        }
        Ok(roaming_agreement_policy_id)
    }

    fn insert_roaming_agreement_policy(owner: &T::AccountId, roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex, roaming_agreement_policy: RoamingAgreementPolicy) {
        // Create and store roaming agreement_policy
        <RoamingAgreementPolicies<T>>::insert(roaming_agreement_policy_id, roaming_agreement_policy);
        <RoamingAgreementPoliciesCount<T>>::put(roaming_agreement_policy_id + One::one());
        <RoamingAgreementPolicyOwners<T>>::insert(roaming_agreement_policy_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_agreement_policy_id: T::RoamingAgreementPolicyIndex) {
        <RoamingAgreementPolicyOwners<T>>::insert(roaming_agreement_policy_id, to);
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{H256};
    use sr_primitives::Perbill;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{assert_noop, assert_ok, impl_outer_origin, parameter_types, weights::Weight};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ();
        type TransferFee = ();
        type CreationFee = ();
    }
    impl transaction_payment::Trait for Test {
        type Currency = Balances;
        type OnTransactionPayment = ();
        type TransactionBaseFee = ();
        type TransactionByteFee = ();
        type WeightToFee = ();
        type FeeMultiplierUpdate = ();
    }

    impl roaming_operators::Trait for Test {
        type Event = ();
        type Currency = Balances;
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl roaming_networks::Trait for Test {
        type Event = ();
        type RoamingNetworkIndex = u64;
    }
    impl roaming_accounting_policies::Trait for Test {
        type Event = ();
        type RoamingAccountingPolicyIndex = u64;
        type RoamingAccountingPolicyType = Vec<u8>;
        type RoamingAccountingPolicyUplinkFeeFactor = u32;
        type RoamingAccountingPolicyDownlinkFeeFactor = u32;
    }
    impl Trait for Test {
        type Event = ();
        type RoamingAgreementPolicyIndex = u64;
        type RoamingAgreementPolicyActivationType = Vec<u8>;
        type RoamingAgreementPolicyExpiry = u64;
    }
    //type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type RoamingAgreementPolicyModule = Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        runtime_io::TestExternalities::new(t)
    }

    #[test]
    fn basic_setup_works() {
        new_test_ext().execute_with(|| {
            // Verify Initial Storage
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policies_count(), 0);
            assert!(RoamingAgreementPolicyModule::roaming_agreement_policy(0).is_none());
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policy_owner(0), None);
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
        });
    }

    #[test]
    fn create_works() {
        new_test_ext().execute_with(|| {
            // Call Functions
            assert_ok!(RoamingAgreementPolicyModule::create(Origin::signed(1)));
            // Verify Storage
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policies_count(), 1);
            assert!(RoamingAgreementPolicyModule::roaming_agreement_policy(0).is_some());
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policy_owner(0), Some(1));
        });
    }

    #[test]
    fn create_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            <RoamingAgreementPoliciesCount<Test>>::put(u64::max_value());
            // Call Functions
            assert_noop!(
                RoamingAgreementPolicyModule::create(Origin::signed(1)),
                "RoamingAgreementPolicies count overflow"
            );
            // Verify Storage
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policies_count(), u64::max_value());
            assert!(RoamingAgreementPolicyModule::roaming_agreement_policy(0).is_none());
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policy_owner(0), None);
        });
    }

    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingAgreementPolicyModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingAgreementPolicyModule::transfer(Origin::signed(1), 2, 0));
            // Verify Storage
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policies_count(), 1);
            assert!(RoamingAgreementPolicyModule::roaming_agreement_policy(0).is_some());
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policy_owner(0), Some(2));
        });
    }

    #[test]
    fn transfer_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingAgreementPolicyModule::create(Origin::signed(1)));
            // Call Functions
            assert_noop!(
                RoamingAgreementPolicyModule::transfer(Origin::signed(2), 2, 0),
                "Only owner can transfer roaming agreement_policy"
            );
            assert_noop!(
                RoamingAgreementPolicyModule::transfer(Origin::signed(1), 2, 1),
                "Only owner can transfer roaming agreement_policy"
            );
            // Verify Storage
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policies_count(), 1);
            assert!(RoamingAgreementPolicyModule::roaming_agreement_policy(0).is_some());
            assert_eq!(RoamingAgreementPolicyModule::roaming_agreement_policy_owner(0), Some(1));
        });
    }
}
