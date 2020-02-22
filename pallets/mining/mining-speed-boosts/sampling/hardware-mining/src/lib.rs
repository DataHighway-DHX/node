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
use mining_speed_boosts_configuration_hardware_mining;
use roaming_operators;

/// The module's trait.
pub trait Trait:
    system::Trait + roaming_operators::Trait + mining_speed_boosts_configuration_hardware_mining::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MiningSpeedBoostSamplingHardwareMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostSamplingHardwareMiningSampleDate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostSamplingHardwareMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostSamplingHardwareMiningSamplingConfig<U, V> {
    pub hardware_sample_date: U,
    pub hardware_sample_hardware_online: V,
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostSamplingHardwareMiningIndex,
        <T as Trait>::MiningSpeedBoostSamplingHardwareMiningSampleDate,
        <T as Trait>::MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline,
        <T as mining_speed_boosts_configuration_hardware_mining::Trait>::MiningSpeedBoostConfigurationHardwareMiningIndex,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_sampling_hardware_mining is created. (owner, mining_speed_boosts_sampling_hardware_mining_id)
        Created(AccountId, MiningSpeedBoostSamplingHardwareMiningIndex),
        /// A mining_speed_boosts_samplings_hardware_mining is transferred. (from, to, mining_speed_boosts_samplings_hardware_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostSamplingHardwareMiningIndex),
        MiningSpeedBoostSamplingHardwareMiningSamplingConfigSet(
            AccountId, MiningSpeedBoostConfigurationHardwareMiningIndex, MiningSpeedBoostSamplingHardwareMiningIndex,
            MiningSpeedBoostSamplingHardwareMiningSampleDate, MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline
        ),
        /// A mining_speed_boosts_sampling_hardware_mining is assigned to an mining_speed_boosts_hardware_mining.
        /// (owner of mining_speed_boosts_hardware_mining, mining_speed_boosts_samplings_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id)
            AssignedHardwareMiningSamplingToConfiguration(AccountId, MiningSpeedBoostSamplingHardwareMiningIndex, MiningSpeedBoostConfigurationHardwareMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostSamplingHardwareMining {
        /// Stores all the mining_speed_boosts_samplings_hardware_minings, key is the mining_speed_boosts_samplings_hardware_mining id / index
        pub MiningSpeedBoostSamplingHardwareMinings get(fn mining_speed_boosts_samplings_hardware_mining): map hasher(blake2_256) T::MiningSpeedBoostSamplingHardwareMiningIndex => Option<MiningSpeedBoostSamplingHardwareMining>;

        /// Stores the total number of mining_speed_boosts_samplings_hardware_minings. i.e. the next mining_speed_boosts_samplings_hardware_mining index
        pub MiningSpeedBoostSamplingHardwareMiningCount get(fn mining_speed_boosts_samplings_hardware_mining_count): T::MiningSpeedBoostSamplingHardwareMiningIndex;

        /// Stores mining_speed_boosts_samplings_hardware_mining owner
        pub MiningSpeedBoostSamplingHardwareMiningOwners get(fn mining_speed_boosts_samplings_hardware_mining_owner): map hasher(blake2_256) T::MiningSpeedBoostSamplingHardwareMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_samplings_hardware_mining_samplings_config
        pub MiningSpeedBoostSamplingHardwareMiningSamplingConfigs get(fn mining_speed_boosts_samplings_hardware_mining_samplings_configs): map hasher(blake2_256) (T::MiningSpeedBoostConfigurationHardwareMiningIndex, T::MiningSpeedBoostSamplingHardwareMiningIndex) =>
            Option<MiningSpeedBoostSamplingHardwareMiningSamplingConfig<
                T::MiningSpeedBoostSamplingHardwareMiningSampleDate,
                T::MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline
            >>;

        /// Get mining_speed_boosts_configuration_hardware_mining_id belonging to a mining_speed_boosts_samplings_hardware_mining_id
        pub HardwareMiningSamplingConfiguration get(fn hardware_mining_sampling_configuration): map hasher(blake2_256) T::MiningSpeedBoostSamplingHardwareMiningIndex => Option<T::MiningSpeedBoostConfigurationHardwareMiningIndex>;

        /// Get mining_speed_boosts_samplings_hardware_mining_id's belonging to a mining_speed_boosts_configuration_hardware_mining_id
        pub HardwareMiningConfigurationSamplings get(fn hardware_mining_configuration_samplings): map hasher(blake2_256) T::MiningSpeedBoostConfigurationHardwareMiningIndex => Option<Vec<T::MiningSpeedBoostSamplingHardwareMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_samplings_hardware_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_samplings_hardware_mining_id = Self::next_mining_speed_boosts_samplings_hardware_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_samplings_hardware_mining
            let mining_speed_boosts_samplings_hardware_mining = MiningSpeedBoostSamplingHardwareMining(unique_id);
            Self::insert_mining_speed_boosts_samplings_hardware_mining(&sender, mining_speed_boosts_samplings_hardware_mining_id, mining_speed_boosts_samplings_hardware_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_samplings_hardware_mining_id));
        }

        /// Transfer a mining_speed_boosts_samplings_hardware_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_samplings_hardware_mining_owner(mining_speed_boosts_samplings_hardware_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_samplings_hardware_mining");

            Self::update_owner(&to, mining_speed_boosts_samplings_hardware_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_samplings_hardware_mining_id));
        }

        /// Set mining_speed_boosts_samplings_hardware_mining_samplings_config
        pub fn set_mining_speed_boosts_samplings_hardware_mining_samplings_config(
            origin,
            mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
            mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex,
            _hardware_sample_date: Option<T::MiningSpeedBoostSamplingHardwareMiningSampleDate>,
            _hardware_sample_hardware_online: Option<T::MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_samplings_hardware_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_samplings_hardware_mining = Self::exists_mining_speed_boosts_samplings_hardware_mining(mining_speed_boosts_samplings_hardware_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_samplings_hardware_mining, "MiningSpeedBoostSamplingHardwareMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_samplings_hardware_mining_samplings_config they are trying to change
            ensure!(Self::mining_speed_boosts_samplings_hardware_mining_owner(mining_speed_boosts_samplings_hardware_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_samplings_hardware_mining_samplings_config");

            // TODO - adjust default samplings
            let hardware_sample_date = match _hardware_sample_date.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let hardware_sample_hardware_online = match _hardware_sample_hardware_online {
                Some(value) => value,
                None => 1.into() // Default
            };

            // Check if a mining_speed_boosts_samplings_hardware_mining_samplings_config already exists with the given mining_speed_boosts_samplings_hardware_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_samplings_hardware_mining_samplings_config_index(mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_samplings_hardware_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostSamplingHardwareMiningSamplingConfigs<T>>::mutate((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_samplings_hardware_mining_id), |mining_speed_boosts_samplings_hardware_mining_samplings_config| {
                    if let Some(_mining_speed_boosts_samplings_hardware_mining_samplings_config) = mining_speed_boosts_samplings_hardware_mining_samplings_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_samplings_hardware_mining_samplings_config.hardware_sample_date = hardware_sample_date.clone();
                        _mining_speed_boosts_samplings_hardware_mining_samplings_config.hardware_sample_hardware_online = hardware_sample_hardware_online.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_samplings_hardware_mining_samplings_config = <MiningSpeedBoostSamplingHardwareMiningSamplingConfigs<T>>::get((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_samplings_hardware_mining_id));
                if let Some(_mining_speed_boosts_samplings_hardware_mining_samplings_config) = fetched_mining_speed_boosts_samplings_hardware_mining_samplings_config {
                    debug::info!("Latest field hardware_sample_date {:#?}", _mining_speed_boosts_samplings_hardware_mining_samplings_config.hardware_sample_date);
                    debug::info!("Latest field hardware_sample_hardware_online {:#?}", _mining_speed_boosts_samplings_hardware_mining_samplings_config.hardware_sample_hardware_online);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_samplings_hardware_mining_samplings_config instance with the input params
                let mining_speed_boosts_samplings_hardware_mining_samplings_config_instance = MiningSpeedBoostSamplingHardwareMiningSamplingConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_sample_date: hardware_sample_date.clone(),
                    hardware_sample_hardware_online: hardware_sample_hardware_online.clone(),
                };

                <MiningSpeedBoostSamplingHardwareMiningSamplingConfigs<T>>::insert(
                    (mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_samplings_hardware_mining_id),
                    &mining_speed_boosts_samplings_hardware_mining_samplings_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_samplings_hardware_mining_samplings_config = <MiningSpeedBoostSamplingHardwareMiningSamplingConfigs<T>>::get((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_samplings_hardware_mining_id));
                if let Some(_mining_speed_boosts_samplings_hardware_mining_samplings_config) = fetched_mining_speed_boosts_samplings_hardware_mining_samplings_config {
                    debug::info!("Inserted field hardware_sample_date {:#?}", _mining_speed_boosts_samplings_hardware_mining_samplings_config.hardware_sample_date);
                    debug::info!("Inserted field hardware_sample_hardware_online {:#?}", _mining_speed_boosts_samplings_hardware_mining_samplings_config.hardware_sample_hardware_online);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostSamplingHardwareMiningSamplingConfigSet(
                sender,
                mining_speed_boosts_configuration_hardware_mining_id,
                mining_speed_boosts_samplings_hardware_mining_id,
                hardware_sample_date,
                hardware_sample_hardware_online,
            ));
        }

        pub fn assign_sampling_to_configuration(
          origin,
          mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex,
          mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_hardware_mining = <mining_speed_boosts_configuration_hardware_mining::Module<T>>
                ::exists_mining_speed_boosts_configuration_hardware_mining(mining_speed_boosts_configuration_hardware_mining_id).is_ok();
            ensure!(is_configuration_hardware_mining, "configuration_hardware_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the sampling to
            ensure!(
                <mining_speed_boosts_configuration_hardware_mining::Module<T>>::is_mining_speed_boosts_configuration_hardware_mining_owner(mining_speed_boosts_configuration_hardware_mining_id, sender.clone()).is_ok(),
                "Only the configuration_hardware_mining owner can assign itself a sampling"
            );

            Self::associate_hardware_sampling_with_configuration(mining_speed_boosts_samplings_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id)
                .expect("Unable to associate sampling with configuration");

            // Ensure that the given mining_speed_boosts_samplings_hardware_mining_id already exists
            let hardware_sampling = Self::mining_speed_boosts_samplings_hardware_mining(mining_speed_boosts_samplings_hardware_mining_id);
            ensure!(hardware_sampling.is_some(), "Invalid mining_speed_boosts_samplings_hardware_mining_id");

            // // Ensure that the sampling is not already owned by a different configuration
            // // Unassign the sampling from any existing configuration since it may only be owned by one configuration
            // <HardwareMiningSamplingConfiguration<T>>::remove(mining_speed_boosts_samplings_hardware_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <HardwareMiningSamplingConfiguration<T>>::insert(mining_speed_boosts_samplings_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id);

            Self::deposit_event(RawEvent::AssignedHardwareMiningSamplingToConfiguration(sender, mining_speed_boosts_samplings_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id));
            }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_speed_boosts_samplings_hardware_mining_owner(
        mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_samplings_hardware_mining_owner(
                &mining_speed_boosts_samplings_hardware_mining_id
            )
            .map(|owner| owner == sender)
            .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostSamplingHardwareMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_samplings_hardware_mining(
        mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex,
    ) -> Result<MiningSpeedBoostSamplingHardwareMining, DispatchError> {
        match Self::mining_speed_boosts_samplings_hardware_mining(mining_speed_boosts_samplings_hardware_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostSamplingHardwareMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_samplings_hardware_mining_samplings_config(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
        mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_samplings_hardware_mining_samplings_configs((
            mining_speed_boosts_configuration_hardware_mining_id,
            mining_speed_boosts_samplings_hardware_mining_id,
        )) {
            Some(value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostSamplingHardwareMiningSamplingConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_speed_boosts_samplings_hardware_mining_samplings_config_index(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
        mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_speed_boosts_samplings_hardware_mining_samplings_config has a value that is defined"
        );
        let fetched_mining_speed_boosts_samplings_hardware_mining_samplings_config =
            <MiningSpeedBoostSamplingHardwareMiningSamplingConfigs<T>>::get((
                mining_speed_boosts_configuration_hardware_mining_id,
                mining_speed_boosts_samplings_hardware_mining_id,
            ));
        if let Some(value) = fetched_mining_speed_boosts_samplings_hardware_mining_samplings_config {
            debug::info!("Found value for mining_speed_boosts_samplings_hardware_mining_samplings_config");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_samplings_hardware_mining_samplings_config");
        Err(DispatchError::Other("No value for mining_speed_boosts_samplings_hardware_mining_samplings_config"))
    }

    /// Only push the sampling id onto the end of the vector if it does not already exist
    pub fn associate_hardware_sampling_with_configuration(
        mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex,
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given sampling id
        if let Some(configuration_samplings) =
            Self::hardware_mining_configuration_samplings(mining_speed_boosts_configuration_hardware_mining_id)
        {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_speed_boosts_configuration_hardware_mining_id,
                configuration_samplings
            );
            let not_configuration_contains_sampling =
                !configuration_samplings.contains(&mining_speed_boosts_samplings_hardware_mining_id);
            ensure!(not_configuration_contains_sampling, "Configuration already contains the given sampling id");
            debug::info!("Configuration id key exists but its vector value does not contain the given sampling id");
            <HardwareMiningConfigurationSamplings<T>>::mutate(
                mining_speed_boosts_configuration_hardware_mining_id,
                |v| {
                    if let Some(value) = v {
                        value.push(mining_speed_boosts_samplings_hardware_mining_id);
                    }
                },
            );
            debug::info!(
                "Associated sampling {:?} with configuration {:?}",
                mining_speed_boosts_samplings_hardware_mining_id,
                mining_speed_boosts_configuration_hardware_mining_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 sampling id {:?} to its vector value",
                mining_speed_boosts_configuration_hardware_mining_id,
                mining_speed_boosts_samplings_hardware_mining_id
            );
            <HardwareMiningConfigurationSamplings<T>>::insert(
                mining_speed_boosts_configuration_hardware_mining_id,
                &vec![mining_speed_boosts_samplings_hardware_mining_id],
            );
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

    fn next_mining_speed_boosts_samplings_hardware_mining_id()
    -> Result<T::MiningSpeedBoostSamplingHardwareMiningIndex, DispatchError> {
        let mining_speed_boosts_samplings_hardware_mining_id =
            Self::mining_speed_boosts_samplings_hardware_mining_count();
        if mining_speed_boosts_samplings_hardware_mining_id ==
            <T::MiningSpeedBoostSamplingHardwareMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostSamplingHardwareMining count overflow"));
        }
        Ok(mining_speed_boosts_samplings_hardware_mining_id)
    }

    fn insert_mining_speed_boosts_samplings_hardware_mining(
        owner: &T::AccountId,
        mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex,
        mining_speed_boosts_samplings_hardware_mining: MiningSpeedBoostSamplingHardwareMining,
    ) {
        // Create and store mining mining_speed_boosts_samplings_hardware_mining
        <MiningSpeedBoostSamplingHardwareMinings<T>>::insert(
            mining_speed_boosts_samplings_hardware_mining_id,
            mining_speed_boosts_samplings_hardware_mining,
        );
        <MiningSpeedBoostSamplingHardwareMiningCount<T>>::put(
            mining_speed_boosts_samplings_hardware_mining_id + One::one(),
        );
        <MiningSpeedBoostSamplingHardwareMiningOwners<T>>::insert(
            mining_speed_boosts_samplings_hardware_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_samplings_hardware_mining_id: T::MiningSpeedBoostSamplingHardwareMiningIndex,
    ) {
        <MiningSpeedBoostSamplingHardwareMiningOwners<T>>::insert(mining_speed_boosts_samplings_hardware_mining_id, to);
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
        type AccountData = balances::AccountData<u64>;
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
    parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
    }
    impl balances::Trait for Test {
        type AccountStore = System;
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
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
    impl mining_speed_boosts_configuration_hardware_mining::Trait for Test {
        type Event = ();
        type MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI = u64;
        // type MiningSpeedBoostConfigurationHardwareMiningHardwareType =
        // MiningSpeedBoostConfigurationHardwareMiningHardwareTypes;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareID = u64;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate = u64;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate = u64;
        // Mining Speed Boost Hardware Mining Config
        type MiningSpeedBoostConfigurationHardwareMiningHardwareSecure = bool;
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningSpeedBoostConfigurationHardwareMiningHardwareType = Vec<u8>;
        // FIXME - restore when stop temporarily using roaming-operators
        // type Currency = Balances;
        // type Randomness = RandomnessCollectiveFlip;
        type MiningSpeedBoostConfigurationHardwareMiningIndex = u64;
    }
    impl Trait for Test {
        type Event = ();
        type MiningSpeedBoostSamplingHardwareMiningIndex = u64;
        type MiningSpeedBoostSamplingHardwareMiningSampleDate = u64;
        type MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline = u64;
    }
    type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type MiningSpeedBoostSamplingHardwareMiningTestModule = Module<Test>;
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
}
