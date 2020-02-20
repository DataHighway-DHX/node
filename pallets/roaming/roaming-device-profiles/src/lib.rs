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

use roaming_devices;

/// The module's configuration trait.
pub trait Trait: system::Trait + roaming_operators::Trait + roaming_devices::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type RoamingDeviceProfileIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RoamingDeviceProfileDevAddr: Parameter + Member + Default;
    type RoamingDeviceProfileDevEUI: Parameter + Member + Default;
    type RoamingDeviceProfileJoinEUI: Parameter + Member + Default;
    type RoamingDeviceProfileVendorID: Parameter + Member + Default;
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingDeviceProfile(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
// Generic type parameters - Balance
pub struct RoamingDeviceProfileConfig<U, V, W, X> {
    pub device_profile_devaddr: U,
    pub device_profile_deveui: V,
    pub device_profile_joineui: W,
    pub device_profile_vendorid: X,
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::RoamingDeviceProfileIndex,
        <T as Trait>::RoamingDeviceProfileDevAddr,
        <T as Trait>::RoamingDeviceProfileDevEUI,
        <T as Trait>::RoamingDeviceProfileJoinEUI,
        <T as Trait>::RoamingDeviceProfileVendorID,
        <T as roaming_devices::Trait>::RoamingDeviceIndex,
    {
        /// A roaming device_profile is created. (owner, roaming_device_profile_id)
        Created(AccountId, RoamingDeviceProfileIndex),
        /// A roaming device_profile is transferred. (from, to, roaming_device_profile_id)
        Transferred(AccountId, AccountId, RoamingDeviceProfileIndex),
        /// A roaming device_profile configuration
        RoamingDeviceProfileConfigSet(AccountId, RoamingDeviceProfileIndex, RoamingDeviceProfileDevAddr, RoamingDeviceProfileDevEUI, RoamingDeviceProfileJoinEUI, RoamingDeviceProfileVendorID),
        /// A roaming device_profile is assigned to a device. (owner of device, roaming_device_profile_id, roaming_device_id)
        AssignedDeviceProfileToDevice(AccountId, RoamingDeviceProfileIndex, RoamingDeviceIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingDeviceProfiles {
        /// Stores all the roaming device_profiles, key is the roaming device_profile id / index
        pub RoamingDeviceProfiles get(fn roaming_device_profile): map hasher(blake2_256) T::RoamingDeviceProfileIndex => Option<RoamingDeviceProfile>;

        /// Stores the total number of roaming device_profiles. i.e. the next roaming device_profile index
        pub RoamingDeviceProfilesCount get(fn roaming_device_profiles_count): T::RoamingDeviceProfileIndex;

        /// Get roaming device_profile owner
        pub RoamingDeviceProfileOwners get(fn roaming_device_profile_owner): map hasher(blake2_256) T::RoamingDeviceProfileIndex => Option<T::AccountId>;

        /// Get roaming device_profile config
        pub RoamingDeviceProfileConfigs get(fn roaming_device_profile_configs): map hasher(blake2_256) T::RoamingDeviceProfileIndex => Option<RoamingDeviceProfileConfig<T::RoamingDeviceProfileDevAddr, T::RoamingDeviceProfileDevEUI, T::RoamingDeviceProfileJoinEUI, T::RoamingDeviceProfileVendorID>>;

        /// Get roaming device_profile device
        pub RoamingDeviceProfileDevice get(fn roaming_device_profile_device): map hasher(blake2_256) T::RoamingDeviceProfileIndex => Option<T::RoamingDeviceIndex>;

        /// Get roaming device device_profiles
        pub RoamingDeviceDeviceProfiles get(fn roaming_device_device_profiles): map hasher(blake2_256) T::RoamingDeviceIndex => Option<Vec<T::RoamingDeviceProfileIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming device_profile
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
        pub fn transfer(origin, to: T::AccountId, roaming_device_profile_id: T::RoamingDeviceProfileIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_device_profile_owner(roaming_device_profile_id) == Some(sender.clone()), "Only owner can transfer roaming device_profile");

            Self::update_owner(&to, roaming_device_profile_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_device_profile_id));
        }

        /// Set roaming device_profile config
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
            if Self::has_value_for_device_profile_config_index(roaming_device_profile_id).is_ok() {
                debug::info!("Mutating values");
                <RoamingDeviceProfileConfigs<T>>::mutate(roaming_device_profile_id, |profile_config| {
                    if let Some(_profile_config) = profile_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _profile_config.device_profile_devaddr = device_profile_devaddr.clone();
                        _profile_config.device_profile_deveui = device_profile_deveui.clone();
                        _profile_config.device_profile_joineui = device_profile_joineui.clone();
                        _profile_config.device_profile_vendorid = device_profile_vendorid.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_profile_config = <RoamingDeviceProfileConfigs<T>>::get(roaming_device_profile_id);
                if let Some(_profile_config) = fetched_profile_config {
                    debug::info!("Latest field device_profile_devaddr {:#?}", _profile_config.device_profile_devaddr);
                    debug::info!("Latest field device_profile_deveui {:#?}", _profile_config.device_profile_deveui);
                    debug::info!("Latest field device_profile_joineui {:#?}", _profile_config.device_profile_joineui);
                    debug::info!("Latest field device_profile_vendorid {:#?}", _profile_config.device_profile_vendorid);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new roaming device_profile config instance with the input params
                let roaming_device_profile_config_instance = RoamingDeviceProfileConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    device_profile_devaddr: device_profile_devaddr.clone(),
                    device_profile_deveui: device_profile_deveui.clone(),
                    device_profile_joineui: device_profile_joineui.clone(),
                    device_profile_vendorid: device_profile_vendorid.clone()
                };

                <RoamingDeviceProfileConfigs<T>>::insert(
                    roaming_device_profile_id,
                    &roaming_device_profile_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_profile_config = <RoamingDeviceProfileConfigs<T>>::get(roaming_device_profile_id);
                if let Some(_profile_config) = fetched_profile_config {
                    debug::info!("Inserted field device_profile_devaddr {:#?}", _profile_config.device_profile_devaddr);
                    debug::info!("Inserted field device_profile_deveui {:#?}", _profile_config.device_profile_deveui);
                    debug::info!("Inserted field device_profile_joineui {:#?}", _profile_config.device_profile_joineui);
                    debug::info!("Inserted field device_profile_vendorid {:#?}", _profile_config.device_profile_vendorid);
                }
            }

            Self::deposit_event(RawEvent::RoamingDeviceProfileConfigSet(
                sender,
                roaming_device_profile_id,
                device_profile_devaddr,
                device_profile_deveui,
                device_profile_joineui,
                device_profile_vendorid
            ));
        }

        pub fn assign_device_profile_to_device(
            origin,
            roaming_device_profile_id: T::RoamingDeviceProfileIndex,
            roaming_device_id: T::RoamingDeviceIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given device id already exists
            let is_roaming_device = <roaming_devices::Module<T>>
                ::exists_roaming_device(roaming_device_id).is_ok();
            ensure!(is_roaming_device, "RoamingDevice does not exist");

            // Ensure that caller of the function is the owner of the device id to assign the device_profile to
            ensure!(
                <roaming_devices::Module<T>>::is_roaming_device_owner(roaming_device_id, sender.clone()).is_ok(),
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

impl<T: Trait> Module<T> {
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
        debug::info!("Get the device id associated with the device of the given device profile id");
        let device_profile_device_id = Self::roaming_device_profile_device(roaming_device_profile_id);

        if let Some(_device_profile_device_id) = device_profile_device_id {
            // Ensure that the caller is owner of the device id associated with the device profile
            ensure!(
                (<roaming_devices::Module<T>>::is_roaming_device_owner(
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

    pub fn exists_roaming_device_profile_config(
        roaming_device_profile_id: T::RoamingDeviceProfileIndex,
    ) -> Result<(), DispatchError> {
        match Self::roaming_device_profile_configs(roaming_device_profile_id) {
            Some(value) => Ok(()),
            None => Err(DispatchError::Other("RoamingDeviceProfileConfig does not exist")),
        }
    }

    pub fn has_value_for_device_profile_config_index(
        roaming_device_profile_id: T::RoamingDeviceProfileIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if device profile config has a value that is defined");
        let fetched_profile_config = <RoamingDeviceProfileConfigs<T>>::get(roaming_device_profile_id);
        if let Some(value) = fetched_profile_config {
            debug::info!("Found value for device profile config");
            return Ok(());
        }
        debug::info!("No value for device profile config");
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
            debug::info!("Device id key {:?} exists with value {:?}", roaming_device_id, device_device_profiles);
            let not_device_contains_device_profile = !device_device_profiles.contains(&roaming_device_profile_id);
            ensure!(not_device_contains_device_profile, "Device already contains the given device_profile id");
            debug::info!("Device id key exists but its vector value does not contain the given device_profile id");
            <RoamingDeviceDeviceProfiles<T>>::mutate(roaming_device_id, |v| {
                if let Some(value) = v {
                    value.push(roaming_device_profile_id);
                }
            });
            debug::info!(
                "Associated device_profile {:?} with device {:?}",
                roaming_device_profile_id,
                roaming_device_id
            );
            Ok(())
        } else {
            debug::info!(
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
            <system::Module<T>>::extrinsic_index(),
            <system::Module<T>>::block_number(),
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
        type AccountData = ();
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
        type OnNewAccount = ();
        type OnReapAccount = ();
        type Origin = Origin;
        type Version = ();
    }
    impl balances::Trait for Test {
        type AccountData = ();
        type AccountStore = ();
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ();
        type OnNewAccount = ();
        type OnReapAccount = ();
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
    impl roaming_network_servers::Trait for Test {
        type Event = ();
        type RoamingNetworkServerIndex = u64;
    }
    impl roaming_networks::Trait for Test {
        type Event = ();
        type RoamingNetworkIndex = u64;
    }
    impl roaming_organizations::Trait for Test {
        type Event = ();
        type RoamingOrganizationIndex = u64;
    }
    impl roaming_devices::Trait for Test {
        type Event = ();
        type RoamingDeviceIndex = u64;
    }
    impl Trait for Test {
        type Event = ();
        type RoamingDeviceProfileDevAddr = Vec<u8>;
        type RoamingDeviceProfileDevEUI = Vec<u8>;
        type RoamingDeviceProfileIndex = u64;
        type RoamingDeviceProfileJoinEUI = Vec<u8>;
        type RoamingDeviceProfileVendorID = Vec<u8>;
    }
    // type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type RoamingDeviceProfileModule = Module<Test>;
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
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profiles_count(), 0);
            assert!(RoamingDeviceProfileModule::roaming_device_profile(0).is_none());
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profile_owner(0), None);
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
        });
    }

    #[test]
    fn create_works() {
        new_test_ext().execute_with(|| {
            // Call Functions
            assert_ok!(RoamingDeviceProfileModule::create(Origin::signed(1)));
            // Verify Storage
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profiles_count(), 1);
            assert!(RoamingDeviceProfileModule::roaming_device_profile(0).is_some());
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profile_owner(0), Some(1));
        });
    }

    #[test]
    fn create_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            <RoamingDeviceProfilesCount<Test>>::put(u64::max_value());
            // Call Functions
            assert_noop!(RoamingDeviceProfileModule::create(Origin::signed(1)), "RoamingDeviceProfiles count overflow");
            // Verify Storage
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profiles_count(), u64::max_value());
            assert!(RoamingDeviceProfileModule::roaming_device_profile(0).is_none());
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profile_owner(0), None);
        });
    }

    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingDeviceProfileModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingDeviceProfileModule::transfer(Origin::signed(1), 2, 0));
            // Verify Storage
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profiles_count(), 1);
            assert!(RoamingDeviceProfileModule::roaming_device_profile(0).is_some());
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profile_owner(0), Some(2));
        });
    }

    #[test]
    fn transfer_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingDeviceProfileModule::create(Origin::signed(1)));
            // Call Functions
            assert_noop!(
                RoamingDeviceProfileModule::transfer(Origin::signed(2), 2, 0),
                "Only owner can transfer roaming device_profile"
            );
            assert_noop!(
                RoamingDeviceProfileModule::transfer(Origin::signed(1), 2, 1),
                "Only owner can transfer roaming device_profile"
            );
            // Verify Storage
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profiles_count(), 1);
            assert!(RoamingDeviceProfileModule::roaming_device_profile(0).is_some());
            assert_eq!(RoamingDeviceProfileModule::roaming_device_profile_owner(0), Some(1));
        });
    }
}
