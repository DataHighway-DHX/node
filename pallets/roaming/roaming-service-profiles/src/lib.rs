#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_io::hashing::{blake2_128};
use sr_primitives::traits::{Bounded, Member, One, SimpleArithmetic};
use frame_support::traits::{Currency, ExistenceRequirement, Randomness};
/// A runtime module for managing non-fungible tokens
use frame_support::{decl_event, decl_error, dispatch, decl_module, decl_storage, ensure, Parameter, debug};
use system::ensure_signed;
use sp-std::prelude::*; // Imports Vec

use roaming_network_servers;

/// The module's configuration trait.
pub trait Trait: system::Trait + roaming_operators::Trait + roaming_network_servers::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type RoamingServiceProfileIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    type RoamingServiceProfileUplinkRate: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
	type RoamingServiceProfileDownlinkRate: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingServiceProfile(pub [u8; 16]);

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
        <T as Trait>::RoamingServiceProfileIndex,
        <T as Trait>::RoamingServiceProfileUplinkRate,
        <T as Trait>::RoamingServiceProfileDownlinkRate,
        <T as roaming_network_servers::Trait>::RoamingNetworkServerIndex,
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
    trait Store for Module<T: Trait> as RoamingServiceProfiles {
        /// Stores all the roaming service_profiles, key is the roaming service_profile id / index
        pub RoamingServiceProfiles get(fn roaming_service_profile): map T::RoamingServiceProfileIndex => Option<RoamingServiceProfile>;

        /// Stores the total number of roaming service_profiles. i.e. the next roaming service_profile index
        pub RoamingServiceProfilesCount get(fn roaming_service_profiles_count): T::RoamingServiceProfileIndex;

        /// Get roaming service_profile owner
        pub RoamingServiceProfileOwners get(fn roaming_service_profile_owner): map T::RoamingServiceProfileIndex => Option<T::AccountId>;

        /// Get roaming service_profile uplink rate.
        pub RoamingServiceProfileUplinkRates get(fn roaming_service_profile_uplink_rate): map T::RoamingServiceProfileIndex => Option<T::RoamingServiceProfileUplinkRate>;

        /// Get roaming service_profile downlink rate.
        pub RoamingServiceProfileDownlinkRates get(fn roaming_service_profile_downlink_rate): map T::RoamingServiceProfileIndex => Option<T::RoamingServiceProfileDownlinkRate>;

        /// Get roaming service_profile network_server
        pub RoamingServiceProfileNetworkServer get(fn roaming_service_profile_network_server): map T::RoamingServiceProfileIndex => Option<T::RoamingNetworkServerIndex>;

        /// Get roaming network_server service_profiles
        pub RoamingNetworkServerServiceProfiles get(fn roaming_network_server_service_profiles): map T::RoamingNetworkServerIndex => Option<Vec<T::RoamingServiceProfileIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming service_profile
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
        pub fn transfer(origin, to: T::AccountId, roaming_service_profile_id: T::RoamingServiceProfileIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_service_profile_owner(roaming_service_profile_id) == Some(sender.clone()), "Only owner can transfer roaming service_profile");

            Self::update_owner(&to, roaming_service_profile_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_service_profile_id));
        }

        /// Set uplink_rate for a roaming service_profile
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

impl<T: Trait> Module<T> {
    pub fn exists_roaming_service_profile(roaming_service_profile_id: T::RoamingServiceProfileIndex) -> Result<RoamingServiceProfile, &'static str> {
        match Self::roaming_service_profile(roaming_service_profile_id) {
            Some(roaming_service_profile) => Ok(roaming_service_profile),
            None => Err("RoamingServiceProfile does not exist")
        }
    }

    // pub fn is_owned_by_required_parent_relationship(roaming_service_profile_id: T::RoamingServiceProfileIndex, sender: T::AccountId) -> Result<(), &'static str> {
    //     debug::info!("Get the network_server id associated with the network_server of the given service profile id");
    //     let service_profile_network_server_id = Self::roaming_service_profile_network_server(roaming_service_profile_id);

    //     if let Some(_service_profile_network_server_id) = service_profile_network_server_id {
    //         // Ensure that the caller is owner of the network_server id associated with the service profile
    //         ensure!((<roaming_network_servers::Module<T>>::is_roaming_network_server_owner(
    //                 _service_profile_network_server_id.clone(),
    //                 sender.clone()
    //             )).is_ok(), "Only owner of the network_server id associated with the given service profile can set an associated roaming service profile config"
    //         );
    //     } else {
    //         // There must be a network_server id associated with the service profile
    //         return Err("RoamingServiceProfileNetworkServer does not exist");
    //     }
    //     Ok(())
    // }

    /// Only push the service_profile id onto the end of the vector if it does not already exist
    pub fn associate_service_profile_with_network_server(
        roaming_service_profile_id: T::RoamingServiceProfileIndex,
        roaming_network_server_id: T::RoamingNetworkServerIndex,
    ) -> Result<(), &'static str>
    {
        // Early exit with error since do not want to append if the given network_server id already exists as a key,
        // and where its corresponding value is a vector that already contains the given service_profile id
        if let Some(network_server_service_profiles) = Self::roaming_network_server_service_profiles(roaming_network_server_id) {
            debug::info!("NetworkServer id key {:?} exists with value {:?}", roaming_network_server_id, network_server_service_profiles);
            let not_network_server_contains_service_profile = !network_server_service_profiles.contains(&roaming_service_profile_id);
            ensure!(not_network_server_contains_service_profile, "NetworkServer already contains the given service_profile id");
            debug::info!("NetworkServer id key exists but its vector value does not contain the given service_profile id");
            <RoamingNetworkServerServiceProfiles<T>>::mutate(roaming_network_server_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_service_profile_id);
                }
            });
            debug::info!("Associated service_profile {:?} with network_server {:?}", roaming_service_profile_id, roaming_network_server_id);
            Ok(())
        } else {
            debug::info!("NetworkServer id key does not yet exist. Creating the network_server key {:?} and appending the service_profile id {:?} to its vector value", roaming_network_server_id, roaming_service_profile_id);
            <RoamingNetworkServerServiceProfiles<T>>::insert(roaming_network_server_id, &vec![roaming_service_profile_id]);
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

    fn next_roaming_service_profile_id() -> Result<T::RoamingServiceProfileIndex, &'static str> {
        let roaming_service_profile_id = Self::roaming_service_profiles_count();
        if roaming_service_profile_id == <T::RoamingServiceProfileIndex as Bounded>::max_value() {
            return Err("RoamingServiceProfiles count overflow");
        }
        Ok(roaming_service_profile_id)
    }

    fn insert_roaming_service_profile(owner: &T::AccountId, roaming_service_profile_id: T::RoamingServiceProfileIndex, roaming_service_profile: RoamingServiceProfile) {
        // Create and store roaming service_profile
        <RoamingServiceProfiles<T>>::insert(roaming_service_profile_id, roaming_service_profile);
        <RoamingServiceProfilesCount<T>>::put(roaming_service_profile_id + One::one());
        <RoamingServiceProfileOwners<T>>::insert(roaming_service_profile_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_service_profile_id: T::RoamingServiceProfileIndex) {
        <RoamingServiceProfileOwners<T>>::insert(roaming_service_profile_id, to);
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

	use sp_core::H256;
	use frame_support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
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
        type ModuleToIndex = ();
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
    impl roaming_network_servers::Trait for Test {
        type Event = ();
        type RoamingNetworkServerIndex = u64;
    }
    impl Trait for Test {
        type Event = ();
        type RoamingServiceProfileIndex = u64;
        type RoamingServiceProfileUplinkRate = u32;
        type RoamingServiceProfileDownlinkRate = u32;
    }

    //type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type RoamingServiceProfileModule = Module<Test>;
    type RoamingNetworkServerModule = roaming_network_servers::Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }

    #[test]
    fn basic_setup_works() {
        new_test_ext().execute_with(|| {
            // Verify Initial Storage
            assert_eq!(RoamingServiceProfileModule::roaming_service_profiles_count(), 0);
            assert!(RoamingServiceProfileModule::roaming_service_profile(0).is_none());
            assert_eq!(RoamingServiceProfileModule::roaming_service_profile_owner(0), None);
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
        });
    }

    #[test]
    fn create_works() {
        new_test_ext().execute_with(|| {
            // Call Functions
            assert_ok!(RoamingServiceProfileModule::create(Origin::signed(1)));
            // Verify Storage
            assert_eq!(RoamingServiceProfileModule::roaming_service_profiles_count(), 1);
            assert!(RoamingServiceProfileModule::roaming_service_profile(0).is_some());
            assert_eq!(RoamingServiceProfileModule::roaming_service_profile_owner(0), Some(1));
        });
    }

    #[test]
    fn create_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            <RoamingServiceProfilesCount<Test>>::put(u64::max_value());
            // Call Functions
            assert_noop!(
                RoamingServiceProfileModule::create(Origin::signed(1)),
                "RoamingServiceProfiles count overflow"
            );
            // Verify Storage
            assert_eq!(RoamingServiceProfileModule::roaming_service_profiles_count(), u64::max_value());
            assert!(RoamingServiceProfileModule::roaming_service_profile(0).is_none());
            assert_eq!(RoamingServiceProfileModule::roaming_service_profile_owner(0), None);
        });
    }

    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingServiceProfileModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingServiceProfileModule::transfer(Origin::signed(1), 2, 0));
            // Verify Storage
            assert_eq!(RoamingServiceProfileModule::roaming_service_profiles_count(), 1);
            assert!(RoamingServiceProfileModule::roaming_service_profile(0).is_some());
            assert_eq!(RoamingServiceProfileModule::roaming_service_profile_owner(0), Some(2));
        });
    }

    #[test]
    fn transfer_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingServiceProfileModule::create(Origin::signed(1)));
            // Call Functions
            assert_noop!(
                RoamingServiceProfileModule::transfer(Origin::signed(2), 2, 0),
                "Only owner can transfer roaming service_profile"
            );
            assert_noop!(
                RoamingServiceProfileModule::transfer(Origin::signed(1), 2, 1),
                "Only owner can transfer roaming service_profile"
            );
            // Verify Storage
            assert_eq!(RoamingServiceProfileModule::roaming_service_profiles_count(), 1);
            assert!(RoamingServiceProfileModule::roaming_service_profile(0).is_some());
            assert_eq!(RoamingServiceProfileModule::roaming_service_profile_owner(0), Some(1));
        });
    }
}
