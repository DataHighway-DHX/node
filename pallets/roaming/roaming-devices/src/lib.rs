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
use roaming_operators; // Only for access to Currency trait
use roaming_organizations;

/// The module's configuration trait.
pub trait Trait:
    system::Trait + roaming_operators::Trait + roaming_network_servers::Trait + roaming_organizations::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type RoamingDeviceIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingDevice(pub [u8; 16]);

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::RoamingDeviceIndex,
        <T as roaming_network_servers::Trait>::RoamingNetworkServerIndex,
        <T as roaming_organizations::Trait>::RoamingOrganizationIndex,
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
    trait Store for Module<T: Trait> as RoamingDevices {
        /// Stores all the roaming devices, key is the roaming device id / index
        pub RoamingDevices get(fn roaming_device): map hasher(blake2_256) T::RoamingDeviceIndex => Option<RoamingDevice>;

        /// Stores the total number of roaming devices. i.e. the next roaming device index
        pub RoamingDevicesCount get(fn roaming_devices_count): T::RoamingDeviceIndex;

        /// Get roaming device owner
        pub RoamingDeviceOwners get(fn roaming_device_owner): map hasher(blake2_256) T::RoamingDeviceIndex => Option<T::AccountId>;

        /// Get roaming device price. None means not for sale.
        pub RoamingDevicePrices get(fn roaming_device_price): map hasher(blake2_256) T::RoamingDeviceIndex => Option<BalanceOf<T>>;

        /// Get roaming device network_server
        pub RoamingDeviceNetworkServers get(fn roaming_device_network_server): map hasher(blake2_256) T::RoamingDeviceIndex => Option<T::RoamingNetworkServerIndex>;

        /// Get roaming device organization
        pub RoamingDeviceOrganization get(fn roaming_device_organization): map hasher(blake2_256) T::RoamingDeviceIndex => Option<T::RoamingOrganizationIndex>;

        /// Get roaming network server's devices
        pub RoamingNetworkServerDevices get(fn roaming_network_server_devices): map hasher(blake2_256) T::RoamingNetworkServerIndex => Option<Vec<T::RoamingDeviceIndex>>;

        /// Get roaming organization's devices
        pub RoamingOrganizationDevices get(fn roaming_organization_devices): map hasher(blake2_256) T::RoamingOrganizationIndex => Option<Vec<T::RoamingDeviceIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming device
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
        pub fn transfer(origin, to: T::AccountId, roaming_device_id: T::RoamingDeviceIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_device_owner(roaming_device_id) == Some(sender.clone()), "Only owner can transfer roaming device");

            Self::update_owner(&to, roaming_device_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_device_id));
        }

        /// Set a price for a roaming device for sale
        /// None to delist the roaming device
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

impl<T: Trait> Module<T> {
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
            <system::Module<T>>::extrinsic_index(),
            <system::Module<T>>::block_number(),
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
        type AccountStore = ();
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
    impl roaming_network_servers::Trait for Test {
        type Event = ();
        type RoamingNetworkServerIndex = u64;
    }
    impl roaming_organizations::Trait for Test {
        type Event = ();
        type RoamingOrganizationIndex = u64;
    }
    impl Trait for Test {
        type Event = ();
        type RoamingDeviceIndex = u64;
    }
    // type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type RoamingDeviceModule = Module<Test>;
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
            assert_eq!(RoamingDeviceModule::roaming_devices_count(), 0);
            assert!(RoamingDeviceModule::roaming_device(0).is_none());
            assert_eq!(RoamingDeviceModule::roaming_device_owner(0), None);
            assert_eq!(RoamingDeviceModule::roaming_device_price(0), None);
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
        });
    }

    #[test]
    fn create_works() {
        new_test_ext().execute_with(|| {
            // Call Functions
            assert_ok!(RoamingDeviceModule::create(Origin::signed(1)));
            // Verify Storage
            assert_eq!(RoamingDeviceModule::roaming_devices_count(), 1);
            assert!(RoamingDeviceModule::roaming_device(0).is_some());
            assert_eq!(RoamingDeviceModule::roaming_device_owner(0), Some(1));
            assert_eq!(RoamingDeviceModule::roaming_device_price(0), None);
        });
    }

    #[test]
    fn create_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            <RoamingDevicesCount<Test>>::put(u64::max_value());
            // Call Functions
            assert_noop!(RoamingDeviceModule::create(Origin::signed(1)), "RoamingDevices count overflow");
            // Verify Storage
            assert_eq!(RoamingDeviceModule::roaming_devices_count(), u64::max_value());
            assert!(RoamingDeviceModule::roaming_device(0).is_none());
            assert_eq!(RoamingDeviceModule::roaming_device_owner(0), None);
            assert_eq!(RoamingDeviceModule::roaming_device_price(0), None);
        });
    }

    #[test]
    fn transfer_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingDeviceModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingDeviceModule::transfer(Origin::signed(1), 2, 0));
            // Verify Storage
            assert_eq!(RoamingDeviceModule::roaming_devices_count(), 1);
            assert!(RoamingDeviceModule::roaming_device(0).is_some());
            assert_eq!(RoamingDeviceModule::roaming_device_owner(0), Some(2));
            assert_eq!(RoamingDeviceModule::roaming_device_price(0), None);
        });
    }

    #[test]
    fn transfer_handles_basic_errors() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingDeviceModule::create(Origin::signed(1)));
            // Call Functions
            assert_noop!(
                RoamingDeviceModule::transfer(Origin::signed(2), 2, 0),
                "Only owner can transfer roaming device"
            );
            assert_noop!(
                RoamingDeviceModule::transfer(Origin::signed(1), 2, 1),
                "Only owner can transfer roaming device"
            );
            // Verify Storage
            assert_eq!(RoamingDeviceModule::roaming_devices_count(), 1);
            assert!(RoamingDeviceModule::roaming_device(0).is_some());
            assert_eq!(RoamingDeviceModule::roaming_device_owner(0), Some(1));
            assert_eq!(RoamingDeviceModule::roaming_device_price(0), None);
        });
    }

    #[test]
    fn set_price_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingDeviceModule::create(Origin::signed(1)));
            // Call Functions
            assert_ok!(RoamingDeviceModule::set_price(Origin::signed(1), 0, Some(10)));
            // Verify Storage
            assert_eq!(RoamingDeviceModule::roaming_devices_count(), 1);
            assert!(RoamingDeviceModule::roaming_device(0).is_some());
            assert_eq!(RoamingDeviceModule::roaming_device_owner(0), Some(1));
            assert_eq!(RoamingDeviceModule::roaming_device_price(0), Some(10));
        });
    }

    #[test]
    fn buy_works() {
        new_test_ext().execute_with(|| {
            // Setup
            assert_ok!(RoamingDeviceModule::create(Origin::signed(1)));
            assert_ok!(RoamingDeviceModule::set_price(Origin::signed(1), 0, Some(10)));
            // Call Functions
            assert_ok!(RoamingDeviceModule::buy(Origin::signed(2), 0, 10));
            // Verify Storage
            assert_eq!(RoamingDeviceModule::roaming_devices_count(), 1);
            assert!(RoamingDeviceModule::roaming_device(0).is_some());
            assert_eq!(RoamingDeviceModule::roaming_device_owner(0), Some(2));
            assert_eq!(RoamingDeviceModule::roaming_device_price(0), None);
            assert_eq!(Balances::free_balance(1), 20);
            assert_eq!(Balances::free_balance(2), 10);
        });
    }
}
