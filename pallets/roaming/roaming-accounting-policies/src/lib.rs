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

use roaming_networks;
use roaming_operators;

/// The module's configuration trait.
pub trait Trait: system::Trait + roaming_operators::Trait + roaming_networks::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type RoamingAccountingPolicyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingAccountingPolicyType: Parameter + Member + Default;
    type RoamingAccountingPolicyUplinkFeeFactor: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingAccountingPolicyDownlinkFeeFactor: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingAccountingPolicy(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
// Generic type parameters - Balance
pub struct RoamingAccountingPolicyConfig<U, V, W, X> {
    pub policy_type: U, // "adhoc" or "subscription"
    pub subscription_fee: V,
    pub uplink_fee_factor: W,
    pub downlink_fee_factor: X,
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::RoamingAccountingPolicyIndex,
        <T as Trait>::RoamingAccountingPolicyType,
        <T as Trait>::RoamingAccountingPolicyUplinkFeeFactor,
        <T as Trait>::RoamingAccountingPolicyDownlinkFeeFactor,
        <T as roaming_networks::Trait>::RoamingNetworkIndex,
        Balance = BalanceOf<T>,
    {
        /// A roaming accounting_policy is created. (owner, roaming_accounting_policy_id)
        Created(AccountId, RoamingAccountingPolicyIndex),
        /// A roaming accounting_policy is transferred. (from, to, roaming_accounting_policy_id)
        Transferred(AccountId, AccountId, RoamingAccountingPolicyIndex),
        /// A roaming accounting_policy configuration
        RoamingAccountingPolicyConfigSet(AccountId, RoamingAccountingPolicyIndex, RoamingAccountingPolicyType, Balance, RoamingAccountingPolicyUplinkFeeFactor, RoamingAccountingPolicyDownlinkFeeFactor),
        /// A roaming accounting_policy is assigned to a network. (owner of network, roaming_accounting_policy_id, roaming_network_id)
        AssignedAccountingPolicyToNetwork(AccountId, RoamingAccountingPolicyIndex, RoamingNetworkIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingAccountingPolicies {
        /// Stores all the roaming accounting_policies, key is the roaming accounting_policy id / index
        pub RoamingAccountingPolicies get(fn roaming_accounting_policy): map hasher(blake2_256) T::RoamingAccountingPolicyIndex => Option<RoamingAccountingPolicy>;

        /// Stores the total number of roaming accounting_policies. i.e. the next roaming accounting_policy index
        pub RoamingAccountingPoliciesCount get(fn roaming_accounting_policies_count): T::RoamingAccountingPolicyIndex;

        /// Get roaming accounting_policy owner
        pub RoamingAccountingPolicyOwners get(fn roaming_accounting_policy_owner): map hasher(blake2_256) T::RoamingAccountingPolicyIndex => Option<T::AccountId>;

        /// Get roaming accounting_policy config
        pub RoamingAccountingPolicyConfigs get(fn roaming_accounting_policy_configs): map hasher(blake2_256) T::RoamingAccountingPolicyIndex => Option<RoamingAccountingPolicyConfig<T::RoamingAccountingPolicyType, BalanceOf<T>, T::RoamingAccountingPolicyUplinkFeeFactor, T::RoamingAccountingPolicyDownlinkFeeFactor>>;

        /// Get roaming accounting_policy network
        pub RoamingAccountingPolicyNetwork get(fn roaming_accounting_policy_network): map hasher(blake2_256) T::RoamingAccountingPolicyIndex => Option<T::RoamingNetworkIndex>;

        /// Get roaming network's accounting policies
        pub RoamingNetworkAccountingPolicies get(fn roaming_network_accounting_policies): map hasher(blake2_256) T::RoamingNetworkIndex => Option<Vec<T::RoamingAccountingPolicyIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming accounting_policy
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
        pub fn transfer(origin, to: T::AccountId, roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_accounting_policy_owner(roaming_accounting_policy_id) == Some(sender.clone()), "Only owner can transfer roaming accounting_policy");

            Self::update_owner(&to, roaming_accounting_policy_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_accounting_policy_id));
        }

        /// Set roaming account_policy config
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
                None => 1.into() // Default
            };
            let uplink_fee_factor = match _uplink_fee_factor {
                Some(value) => value,
                None => 1.into() // Default
            };
            let downlink_fee_factor = match _downlink_fee_factor {
                Some(value) => value,
                None => 1.into() // Default
            };

            // Check if a roaming accounting policy config already exists with the given roaming accounting policy id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_accounting_policy_config_index(roaming_accounting_policy_id).is_ok() {
                debug::info!("Mutating values");
                <RoamingAccountingPolicyConfigs<T>>::mutate(roaming_accounting_policy_id, |policy_config| {
                    if let Some(_policy_config) = policy_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _policy_config.policy_type = policy_type.clone();
                        _policy_config.subscription_fee = subscription_fee.clone();
                        _policy_config.uplink_fee_factor = uplink_fee_factor.clone();
                        _policy_config.downlink_fee_factor = downlink_fee_factor.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_policy_config = <RoamingAccountingPolicyConfigs<T>>::get(roaming_accounting_policy_id);
                if let Some(_policy_config) = fetched_policy_config {
                    debug::info!("Latest field policy_type {:#?}", _policy_config.policy_type);
                    debug::info!("Latest field subscription_fee {:#?}", _policy_config.subscription_fee);
                    debug::info!("Latest field uplink_fee_factor {:#?}", _policy_config.uplink_fee_factor);
                    debug::info!("Latest field downlink_fee_factor {:#?}", _policy_config.downlink_fee_factor);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new roaming accounting_policy config instance with the input params
                let roaming_accounting_policy_config_instance = RoamingAccountingPolicyConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    policy_type: policy_type.clone(),
                    subscription_fee: subscription_fee.clone(),
                    uplink_fee_factor: uplink_fee_factor.clone(),
                    downlink_fee_factor: downlink_fee_factor.clone()
                };

                <RoamingAccountingPolicyConfigs<T>>::insert(
                    roaming_accounting_policy_id,
                    &roaming_accounting_policy_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_policy_config = <RoamingAccountingPolicyConfigs<T>>::get(roaming_accounting_policy_id);
                if let Some(_policy_config) = fetched_policy_config {
                    debug::info!("Inserted field policy_type {:#?}", _policy_config.policy_type);
                    debug::info!("Inserted field subscription_fee {:#?}", _policy_config.subscription_fee);
                    debug::info!("Inserted field uplink_fee_factor {:#?}", _policy_config.uplink_fee_factor);
                    debug::info!("Inserted field downlink_fee_factor {:#?}", _policy_config.downlink_fee_factor);
                }
            }

            Self::deposit_event(RawEvent::RoamingAccountingPolicyConfigSet(
                sender,
                roaming_accounting_policy_id,
                policy_type,
                subscription_fee,
                uplink_fee_factor,
                downlink_fee_factor
            ));
        }

        pub fn assign_accounting_policy_to_network(
            origin,
            roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
            roaming_network_id: T::RoamingNetworkIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given network id already exists
            let is_roaming_network = <roaming_networks::Module<T>>
                ::exists_roaming_network(roaming_network_id).is_ok();
            ensure!(is_roaming_network, "RoamingNetwork does not exist");

            // Ensure that caller of the function is the owner of the network id to assign the accounting_policy to
            ensure!(
                <roaming_networks::Module<T>>::is_roaming_network_owner(roaming_network_id, sender.clone()).is_ok(),
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

impl<T: Trait> Module<T> {
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
    // sender: T::AccountId) -> Result<(), DispatchError> {     debug::info!("Get the network id associated with the
    // network of the given accounting policy id");     let accounting_policy_network_id =
    // Self::roaming_accounting_policy_network(roaming_accounting_policy_id);

    //     if let Some(_accounting_policy_network_id) = accounting_policy_network_id {
    //         // Ensure that the caller is owner of the network id associated with the accounting policy
    //         ensure!((<roaming_networks::Module<T>>::is_roaming_network_owner(
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

    pub fn exists_roaming_accounting_policy_config(
        roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
    ) -> Result<(), DispatchError> {
        match Self::roaming_accounting_policy_configs(roaming_accounting_policy_id) {
            Some(value) => Ok(()),
            None => Err(DispatchError::Other("RoamingAccountingPolicyConfig does not exist")),
        }
    }

    pub fn has_value_for_accounting_policy_config_index(
        roaming_accounting_policy_id: T::RoamingAccountingPolicyIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if accounting policy config has a value that is defined");
        let fetched_policy_config = <RoamingAccountingPolicyConfigs<T>>::get(roaming_accounting_policy_id);
        if let Some(value) = fetched_policy_config {
            debug::info!("Found value for accounting policy config");
            return Ok(());
        }
        debug::info!("No value for accounting policy config");
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
            debug::info!("Network id key {:?} exists with value {:?}", roaming_network_id, network_accounting_policies);
            let not_network_contains_accounting_policy =
                !network_accounting_policies.contains(&roaming_accounting_policy_id);
            ensure!(not_network_contains_accounting_policy, "Network already contains the given accounting policy id");
            debug::info!("Network id key exists but its vector value does not contain the given accounting policy id");
            <RoamingNetworkAccountingPolicies<T>>::mutate(roaming_network_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_accounting_policy_id);
                }
            });
            debug::info!(
                "Associated accounting policy {:?} with network {:?}",
                roaming_accounting_policy_id,
                roaming_network_id
            );
            Ok(())
        } else {
            debug::info!(
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
            <system::Module<T>>::extrinsic_index(),
            <system::Module<T>>::block_number(),
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

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_ok,
        impl_outer_origin,
        parameter_types,
        weights::Weight,
    };
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{
            BlakeTwo256,
            IdentityLookup,
        },
        Perbill,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type AccountId = u64;
        type AvailableBlockRatio = AvailableBlockRatio;
        type BlockHashCount = BlockHashCount;
        type BlockNumber = u64;
        type Call = ();
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Header = Header;
        type Index = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type MaximumBlockLength = MaximumBlockLength;
        type MaximumBlockWeight = MaximumBlockWeight;
        type ModuleToIndex = ();
        type Origin = Origin;
        type Version = ();
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ();
    }
    impl transaction_payment::Trait for Test {
        type Currency = Balances;
        type FeeMultiplierUpdate = ();
        type OnTransactionPayment = ();
        type TransactionBaseFee = ();
        type TransactionByteFee = ();
        type WeightToFee = ();
    }
    impl roaming_operators::Trait for Test {
        type Currency = Balances;
        type Event = ();
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl roaming_networks::Trait for Test {
        type Event = ();
        type RoamingNetworkIndex = u64;
    }
    impl Trait for Test {
        type Event = ();
        type RoamingAccountingPolicyDownlinkFeeFactor = u32;
        type RoamingAccountingPolicyIndex = u64;
        type RoamingAccountingPolicyType = Vec<u8>;
        type RoamingAccountingPolicyUplinkFeeFactor = u32;
    }
    // type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type RoamingAccountingPolicyModule = Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }

    #[test]
    fn basic_setup_works() {
        new_test_ext().execute_with(|| {
            // Verify Initial Storage
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), 0);
            assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_none());
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), None);
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
        });
    }

    #[test]
    fn create_works() {
        new_test_ext().execute_with(|| {
            // Call Functions
            assert_ok!(RoamingAccountingPolicyModule::create(Origin::signed(1)));
            // Verify Storage
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), 1);
            assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_some());
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), Some(1));
        });
    }

    #[test]
    fn create_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            <RoamingAccountingPoliciesCount<Test>>::put(u64::max_value());
            // Call Functions
            assert_noop!(
                RoamingAccountingPolicyModule::create(Origin::signed(1)),
                "RoamingAccountingPolicies count overflow"
            );
            // Verify Storage
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), u64::max_value());
            assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_none());
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), None);
        });
    }

    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingAccountingPolicyModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingAccountingPolicyModule::transfer(Origin::signed(1), 2, 0));
            // Verify Storage
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), 1);
            assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_some());
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), Some(2));
        });
    }

    #[test]
    fn transfer_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingAccountingPolicyModule::create(Origin::signed(1)));
            // Call Functions
            assert_noop!(
                RoamingAccountingPolicyModule::transfer(Origin::signed(2), 2, 0),
                "Only owner can transfer roaming accounting_policy"
            );
            assert_noop!(
                RoamingAccountingPolicyModule::transfer(Origin::signed(1), 2, 1),
                "Only owner can transfer roaming accounting_policy"
            );
            // Verify Storage
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), 1);
            assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_some());
            assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), Some(1));
        });
    }
}
