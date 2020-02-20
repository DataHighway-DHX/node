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

// FIXME - remove roaming_operators here, only use this approach since do not know how to use BalanceOf using only
// mining-speed-boosts runtime module
use roaming_operators;

/// The module's rates trait.
pub trait Trait: system::Trait + roaming_operators::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MiningSpeedBoostRatesHardwareMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesHardwareMiningHardwareSecure: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesHardwareMiningHardwareInsecure: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningSpeedBoostRatesHardwareMiningMaxHardware: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostRatesHardwareMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostRatesHardwareMiningRatesConfig<U, V, W> {
    pub hardware_hardware_secure: U,
    pub hardware_hardware_insecure: V,
    pub hardware_max_hardware: W,
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningIndex,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningHardwareSecure,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningHardwareInsecure,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningMaxHardware,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_rates_hardware_mining is created. (owner, mining_speed_boosts_rates_hardware_mining_id)
        Created(AccountId, MiningSpeedBoostRatesHardwareMiningIndex),
        /// A mining_speed_boosts_rates_hardware_mining is transferred. (from, to, mining_speed_boosts_rates_hardware_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostRatesHardwareMiningIndex),
        MiningSpeedBoostRatesHardwareMiningRatesConfigSet(
            AccountId, MiningSpeedBoostRatesHardwareMiningIndex, MiningSpeedBoostRatesHardwareMiningHardwareSecure,
            MiningSpeedBoostRatesHardwareMiningHardwareInsecure, MiningSpeedBoostRatesHardwareMiningMaxHardware
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostRatesHardwareMining {
        /// Stores all the mining_speed_boosts_rates_hardware_minings, key is the mining_speed_boosts_rates_hardware_mining id / index
        pub MiningSpeedBoostRatesHardwareMinings get(fn mining_speed_boosts_rates_hardware_mining): map hasher(blake2_256) T::MiningSpeedBoostRatesHardwareMiningIndex => Option<MiningSpeedBoostRatesHardwareMining>;

        /// Stores the total number of mining_speed_boosts_rates_hardware_minings. i.e. the next mining_speed_boosts_rates_hardware_mining index
        pub MiningSpeedBoostRatesHardwareMiningCount get(fn mining_speed_boosts_rates_hardware_mining_count): T::MiningSpeedBoostRatesHardwareMiningIndex;

        /// Stores mining_speed_boosts_rates_hardware_mining owner
        pub MiningSpeedBoostRatesHardwareMiningOwners get(fn mining_speed_boosts_rates_hardware_mining_owner): map hasher(blake2_256) T::MiningSpeedBoostRatesHardwareMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_rates_hardware_mining_rates_config
        pub MiningSpeedBoostRatesHardwareMiningRatesConfigs get(fn mining_speed_boosts_rates_hardware_mining_rates_configs): map hasher(blake2_256) T::MiningSpeedBoostRatesHardwareMiningIndex =>
            Option<MiningSpeedBoostRatesHardwareMiningRatesConfig<T::MiningSpeedBoostRatesHardwareMiningHardwareSecure,
            T::MiningSpeedBoostRatesHardwareMiningHardwareInsecure, T::MiningSpeedBoostRatesHardwareMiningMaxHardware>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_rates_hardware_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_rates_hardware_mining_id = Self::next_mining_speed_boosts_rates_hardware_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_rates_hardware_mining
            let mining_speed_boosts_rates_hardware_mining = MiningSpeedBoostRatesHardwareMining(unique_id);
            Self::insert_mining_speed_boosts_rates_hardware_mining(&sender, mining_speed_boosts_rates_hardware_mining_id, mining_speed_boosts_rates_hardware_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_rates_hardware_mining_id));
        }

        /// Transfer a mining_speed_boosts_rates_hardware_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_rates_hardware_mining_owner(mining_speed_boosts_rates_hardware_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_rates_hardware_mining");

            Self::update_owner(&to, mining_speed_boosts_rates_hardware_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_rates_hardware_mining_id));
        }

        /// Set mining_speed_boosts_rates_hardware_mining_rates_config
        pub fn set_mining_speed_boosts_rates_hardware_mining_rates_config(
            origin,
            mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
            _hardware_hardware_secure: Option<T::MiningSpeedBoostRatesHardwareMiningHardwareSecure>,
            _hardware_hardware_insecure: Option<T::MiningSpeedBoostRatesHardwareMiningHardwareInsecure>,
            _hardware_max_hardware: Option<T::MiningSpeedBoostRatesHardwareMiningMaxHardware>
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_rates_hardware_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_rates_hardware_mining = Self::exists_mining_speed_boosts_rates_hardware_mining(mining_speed_boosts_rates_hardware_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_rates_hardware_mining, "MiningSpeedBoostRatesHardwareMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_rates_hardware_mining_rates_config they are trying to change
            ensure!(Self::mining_speed_boosts_rates_hardware_mining_owner(mining_speed_boosts_rates_hardware_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_rates_hardware_mining_rates_config");

            // TODO - adjust default rates
            let hardware_hardware_secure = match _hardware_hardware_secure.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let hardware_hardware_insecure = match _hardware_hardware_insecure {
                Some(value) => value,
                None => 1.into() // Default
            };
            let hardware_max_hardware = match _hardware_max_hardware {
              Some(value) => value,
              None => 1.into() // Default
            };

            // Check if a mining_speed_boosts_rates_hardware_mining_rates_config already exists with the given mining_speed_boosts_rates_hardware_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_rates_hardware_mining_rates_config_index(mining_speed_boosts_rates_hardware_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::mutate(mining_speed_boosts_rates_hardware_mining_id, |mining_speed_boosts_rates_hardware_mining_rates_config| {
                    if let Some(_mining_speed_boosts_rates_hardware_mining_rates_config) = mining_speed_boosts_rates_hardware_mining_rates_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_secure = hardware_hardware_secure.clone();
                        _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_insecure = hardware_hardware_insecure.clone();
                        _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_max_hardware = hardware_max_hardware.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_rates_hardware_mining_rates_config = <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::get(mining_speed_boosts_rates_hardware_mining_id);
                if let Some(_mining_speed_boosts_rates_hardware_mining_rates_config) = fetched_mining_speed_boosts_rates_hardware_mining_rates_config {
                    debug::info!("Latest field hardware_hardware_secure {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_secure);
                    debug::info!("Latest field hardware_hardware_insecure {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_insecure);
                    debug::info!("Latest field hardware_max_hardware {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_max_hardware);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_rates_hardware_mining_rates_config instance with the input params
                let mining_speed_boosts_rates_hardware_mining_rates_config_instance = MiningSpeedBoostRatesHardwareMiningRatesConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_hardware_secure: hardware_hardware_secure.clone(),
                    hardware_hardware_insecure: hardware_hardware_insecure.clone(),
                    hardware_max_hardware: hardware_max_hardware.clone(),
                };

                <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::insert(
                    mining_speed_boosts_rates_hardware_mining_id,
                    &mining_speed_boosts_rates_hardware_mining_rates_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_rates_hardware_mining_rates_config = <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::get(mining_speed_boosts_rates_hardware_mining_id);
                if let Some(_mining_speed_boosts_rates_hardware_mining_rates_config) = fetched_mining_speed_boosts_rates_hardware_mining_rates_config {
                    debug::info!("Inserted field hardware_hardware_secure {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_secure);
                    debug::info!("Inserted field hardware_hardware_insecure {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_insecure);
                    debug::info!("Inserted field hardware_max_hardware {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_max_hardware);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostRatesHardwareMiningRatesConfigSet(
                sender,
                mining_speed_boosts_rates_hardware_mining_id,
                hardware_hardware_secure,
                hardware_hardware_insecure,
                hardware_max_hardware,
            ));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_speed_boosts_rates_hardware_mining_owner(
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_rates_hardware_mining_owner(&mining_speed_boosts_rates_hardware_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostRatesHardwareMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_rates_hardware_mining(
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
    ) -> Result<MiningSpeedBoostRatesHardwareMining, DispatchError> {
        match Self::mining_speed_boosts_rates_hardware_mining(mining_speed_boosts_rates_hardware_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostRatesHardwareMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_rates_hardware_mining_rates_config(
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_rates_hardware_mining_rates_configs(
            mining_speed_boosts_rates_hardware_mining_id,
        ) {
            Some(value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostRatesHardwareMiningRatesConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_speed_boosts_rates_hardware_mining_rates_config_index(
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_speed_boosts_rates_hardware_mining_rates_config has a value that is defined");
        let fetched_mining_speed_boosts_rates_hardware_mining_rates_config =
            <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::get(mining_speed_boosts_rates_hardware_mining_id);
        if let Some(value) = fetched_mining_speed_boosts_rates_hardware_mining_rates_config {
            debug::info!("Found value for mining_speed_boosts_rates_hardware_mining_rates_config");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_rates_hardware_mining_rates_config");
        Err(DispatchError::Other("No value for mining_speed_boosts_rates_hardware_mining_rates_config"))
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

    fn next_mining_speed_boosts_rates_hardware_mining_id()
    -> Result<T::MiningSpeedBoostRatesHardwareMiningIndex, DispatchError> {
        let mining_speed_boosts_rates_hardware_mining_id = Self::mining_speed_boosts_rates_hardware_mining_count();
        if mining_speed_boosts_rates_hardware_mining_id ==
            <T::MiningSpeedBoostRatesHardwareMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostRatesHardwareMining count overflow"));
        }
        Ok(mining_speed_boosts_rates_hardware_mining_id)
    }

    fn insert_mining_speed_boosts_rates_hardware_mining(
        owner: &T::AccountId,
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
        mining_speed_boosts_rates_hardware_mining: MiningSpeedBoostRatesHardwareMining,
    ) {
        // Create and store mining mining_speed_boosts_rates_hardware_mining
        <MiningSpeedBoostRatesHardwareMinings<T>>::insert(
            mining_speed_boosts_rates_hardware_mining_id,
            mining_speed_boosts_rates_hardware_mining,
        );
        <MiningSpeedBoostRatesHardwareMiningCount<T>>::put(mining_speed_boosts_rates_hardware_mining_id + One::one());
        <MiningSpeedBoostRatesHardwareMiningOwners<T>>::insert(
            mining_speed_boosts_rates_hardware_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
    ) {
        <MiningSpeedBoostRatesHardwareMiningOwners<T>>::insert(mining_speed_boosts_rates_hardware_mining_id, to);
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
    // FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
    impl roaming_operators::Trait for Test {
        type Currency = Balances;
        type Event = ();
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl Trait for Test {
        type Event = ();
        type MiningSpeedBoostRatesHardwareMiningHardwareInsecure = u32;
        type MiningSpeedBoostRatesHardwareMiningHardwareSecure = u32;
        type MiningSpeedBoostRatesHardwareMiningIndex = u64;
        type MiningSpeedBoostRatesHardwareMiningMaxHardware = u32;
    }
    // type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type MiningSpeedBoostRatesHardwareMiningTestModule = Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }
}
