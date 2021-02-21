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

// FIXME - remove roaming_operators here, only use this approach since do not know how to use BalanceOf using only
// mining runtime module
use mining_config_hardware_mining;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait + roaming_operators::Trait + mining_config_hardware_mining::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningSamplingHardwareMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSamplingHardwareMiningSampleHardwareOnline: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSamplingHardwareMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSamplingHardwareMiningSamplingConfig<U, V> {
    pub hardware_sample_block: U,
    pub hardware_sample_hardware_online: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningSamplingHardwareMiningIndex,
        <T as Trait>::MiningSamplingHardwareMiningSampleHardwareOnline,
        <T as mining_config_hardware_mining::Trait>::MiningConfigHardwareMiningIndex,
        <T as frame_system::Trait>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_sampling_hardware_mining is created. (owner, mining_sampling_hardware_mining_id)
        Created(AccountId, MiningSamplingHardwareMiningIndex),
        /// A mining_samplings_hardware_mining is transferred. (from, to, mining_samplings_hardware_mining_id)
        Transferred(AccountId, AccountId, MiningSamplingHardwareMiningIndex),
        MiningSamplingHardwareMiningSamplingConfigSet(
            AccountId, MiningConfigHardwareMiningIndex, MiningSamplingHardwareMiningIndex,
            BlockNumber, MiningSamplingHardwareMiningSampleHardwareOnline
        ),
        /// A mining_sampling_hardware_mining is assigned to an mining_hardware_mining.
        /// (owner of mining_hardware_mining, mining_samplings_hardware_mining_id, mining_config_hardware_mining_id)
        AssignedHardwareMiningSamplingToConfiguration(AccountId, MiningSamplingHardwareMiningIndex, MiningConfigHardwareMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSamplingHardwareMining {
        /// Stores all the mining_samplings_hardware_minings, key is the mining_samplings_hardware_mining id / index
        pub MiningSamplingHardwareMinings get(fn mining_samplings_hardware_mining): map hasher(opaque_blake2_256) T::MiningSamplingHardwareMiningIndex => Option<MiningSamplingHardwareMining>;

        /// Stores the total number of mining_samplings_hardware_minings. i.e. the next mining_samplings_hardware_mining index
        pub MiningSamplingHardwareMiningCount get(fn mining_samplings_hardware_mining_count): T::MiningSamplingHardwareMiningIndex;

        /// Stores mining_samplings_hardware_mining owner
        pub MiningSamplingHardwareMiningOwners get(fn mining_samplings_hardware_mining_owner): map hasher(opaque_blake2_256) T::MiningSamplingHardwareMiningIndex => Option<T::AccountId>;

        /// Stores mining_samplings_hardware_mining_samplings_config
        pub MiningSamplingHardwareMiningSamplingConfigs get(fn mining_samplings_hardware_mining_samplings_configs): map hasher(opaque_blake2_256) (T::MiningConfigHardwareMiningIndex, T::MiningSamplingHardwareMiningIndex) =>
            Option<MiningSamplingHardwareMiningSamplingConfig<
                T::BlockNumber,
                T::MiningSamplingHardwareMiningSampleHardwareOnline
            >>;

        /// Get mining_config_hardware_mining_id belonging to a mining_samplings_hardware_mining_id
        pub HardwareMiningSamplingConfiguration get(fn hardware_mining_sampling_configuration): map hasher(opaque_blake2_256) T::MiningSamplingHardwareMiningIndex => Option<T::MiningConfigHardwareMiningIndex>;

        /// Get mining_samplings_hardware_mining_id's belonging to a mining_config_hardware_mining_id
        pub HardwareMiningConfigSamplings get(fn hardware_mining_config_samplings): map hasher(opaque_blake2_256) T::MiningConfigHardwareMiningIndex => Option<Vec<T::MiningSamplingHardwareMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_samplings_hardware_mining
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_samplings_hardware_mining_id = Self::next_mining_samplings_hardware_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_samplings_hardware_mining
            let mining_samplings_hardware_mining = MiningSamplingHardwareMining(unique_id);
            Self::insert_mining_samplings_hardware_mining(&sender, mining_samplings_hardware_mining_id, mining_samplings_hardware_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_samplings_hardware_mining_id));
        }

        /// Transfer a mining_samplings_hardware_mining to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_samplings_hardware_mining_owner(mining_samplings_hardware_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_samplings_hardware_mining");

            Self::update_owner(&to, mining_samplings_hardware_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_samplings_hardware_mining_id));
        }

        /// Set mining_samplings_hardware_mining_samplings_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_samplings_hardware_mining_samplings_config(
            origin,
            mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
            mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex,
            _hardware_sample_block: Option<T::BlockNumber>,
            _hardware_sample_hardware_online: Option<T::MiningSamplingHardwareMiningSampleHardwareOnline>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_samplings_hardware_mining_id whose config we want to change actually exists
            let is_mining_samplings_hardware_mining = Self::exists_mining_samplings_hardware_mining(mining_samplings_hardware_mining_id).is_ok();
            ensure!(is_mining_samplings_hardware_mining, "MiningSamplingHardwareMining does not exist");

            // Ensure that the caller is owner of the mining_samplings_hardware_mining_samplings_config they are trying to change
            ensure!(Self::mining_samplings_hardware_mining_owner(mining_samplings_hardware_mining_id) == Some(sender.clone()), "Only owner can set mining_samplings_hardware_mining_samplings_config");

            // TODO - adjust default samplings
            let hardware_sample_block = match _hardware_sample_block.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let hardware_sample_hardware_online = match _hardware_sample_hardware_online {
                Some(value) => value,
                None => 1.into() // Default
            };

            // Check if a mining_samplings_hardware_mining_samplings_config already exists with the given mining_samplings_hardware_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_samplings_hardware_mining_samplings_config_index(mining_config_hardware_mining_id, mining_samplings_hardware_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSamplingHardwareMiningSamplingConfigs<T>>::mutate((mining_config_hardware_mining_id, mining_samplings_hardware_mining_id), |mining_samplings_hardware_mining_samplings_config| {
                    if let Some(_mining_samplings_hardware_mining_samplings_config) = mining_samplings_hardware_mining_samplings_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_samplings_hardware_mining_samplings_config.hardware_sample_block = hardware_sample_block.clone();
                        _mining_samplings_hardware_mining_samplings_config.hardware_sample_hardware_online = hardware_sample_hardware_online.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_samplings_hardware_mining_samplings_config = <MiningSamplingHardwareMiningSamplingConfigs<T>>::get((mining_config_hardware_mining_id, mining_samplings_hardware_mining_id));
                if let Some(_mining_samplings_hardware_mining_samplings_config) = fetched_mining_samplings_hardware_mining_samplings_config {
                    debug::info!("Latest field hardware_sample_block {:#?}", _mining_samplings_hardware_mining_samplings_config.hardware_sample_block);
                    debug::info!("Latest field hardware_sample_hardware_online {:#?}", _mining_samplings_hardware_mining_samplings_config.hardware_sample_hardware_online);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_samplings_hardware_mining_samplings_config instance with the input params
                let mining_samplings_hardware_mining_samplings_config_instance = MiningSamplingHardwareMiningSamplingConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_sample_block: hardware_sample_block.clone(),
                    hardware_sample_hardware_online: hardware_sample_hardware_online.clone(),
                };

                <MiningSamplingHardwareMiningSamplingConfigs<T>>::insert(
                    (mining_config_hardware_mining_id, mining_samplings_hardware_mining_id),
                    &mining_samplings_hardware_mining_samplings_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_samplings_hardware_mining_samplings_config = <MiningSamplingHardwareMiningSamplingConfigs<T>>::get((mining_config_hardware_mining_id, mining_samplings_hardware_mining_id));
                if let Some(_mining_samplings_hardware_mining_samplings_config) = fetched_mining_samplings_hardware_mining_samplings_config {
                    debug::info!("Inserted field hardware_sample_block {:#?}", _mining_samplings_hardware_mining_samplings_config.hardware_sample_block);
                    debug::info!("Inserted field hardware_sample_hardware_online {:#?}", _mining_samplings_hardware_mining_samplings_config.hardware_sample_hardware_online);
                }
            }

            Self::deposit_event(RawEvent::MiningSamplingHardwareMiningSamplingConfigSet(
                sender,
                mining_config_hardware_mining_id,
                mining_samplings_hardware_mining_id,
                hardware_sample_block,
                hardware_sample_hardware_online,
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_sampling_to_configuration(
          origin,
          mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex,
          mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_hardware_mining = <mining_config_hardware_mining::Module<T>>
                ::exists_mining_config_hardware_mining(mining_config_hardware_mining_id).is_ok();
            ensure!(is_configuration_hardware_mining, "configuration_hardware_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the sampling to
            ensure!(
                <mining_config_hardware_mining::Module<T>>::is_mining_config_hardware_mining_owner(mining_config_hardware_mining_id, sender.clone()).is_ok(),
                "Only the configuration_hardware_mining owner can assign itself a sampling"
            );

            Self::associate_hardware_sampling_with_configuration(mining_samplings_hardware_mining_id, mining_config_hardware_mining_id)
                .expect("Unable to associate sampling with configuration");

            // Ensure that the given mining_samplings_hardware_mining_id already exists
            let hardware_sampling = Self::mining_samplings_hardware_mining(mining_samplings_hardware_mining_id);
            ensure!(hardware_sampling.is_some(), "Invalid mining_samplings_hardware_mining_id");

            // // Ensure that the sampling is not already owned by a different configuration
            // // Unassign the sampling from any existing configuration since it may only be owned by one configuration
            // <HardwareMiningSamplingConfiguration<T>>::remove(mining_samplings_hardware_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <HardwareMiningSamplingConfiguration<T>>::insert(mining_samplings_hardware_mining_id, mining_config_hardware_mining_id);

            Self::deposit_event(RawEvent::AssignedHardwareMiningSamplingToConfiguration(sender, mining_samplings_hardware_mining_id, mining_config_hardware_mining_id));
            }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_samplings_hardware_mining_owner(
        mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_samplings_hardware_mining_owner(
                &mining_samplings_hardware_mining_id
            )
            .map(|owner| owner == sender)
            .unwrap_or(false),
            "Sender is not owner of MiningSamplingHardwareMining"
        );
        Ok(())
    }

    pub fn exists_mining_samplings_hardware_mining(
        mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex,
    ) -> Result<MiningSamplingHardwareMining, DispatchError> {
        match Self::mining_samplings_hardware_mining(mining_samplings_hardware_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSamplingHardwareMining does not exist")),
        }
    }

    pub fn exists_mining_samplings_hardware_mining_samplings_config(
        mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
        mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_samplings_hardware_mining_samplings_configs((
            mining_config_hardware_mining_id,
            mining_samplings_hardware_mining_id,
        )) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningSamplingHardwareMiningSamplingConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_samplings_hardware_mining_samplings_config_index(
        mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
        mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_samplings_hardware_mining_samplings_config has a value that is defined"
        );
        let fetched_mining_samplings_hardware_mining_samplings_config =
            <MiningSamplingHardwareMiningSamplingConfigs<T>>::get((
                mining_config_hardware_mining_id,
                mining_samplings_hardware_mining_id,
            ));
        if let Some(_value) = fetched_mining_samplings_hardware_mining_samplings_config {
            debug::info!("Found value for mining_samplings_hardware_mining_samplings_config");
            return Ok(());
        }
        debug::info!("No value for mining_samplings_hardware_mining_samplings_config");
        Err(DispatchError::Other("No value for mining_samplings_hardware_mining_samplings_config"))
    }

    /// Only push the sampling id onto the end of the vector if it does not already exist
    pub fn associate_hardware_sampling_with_configuration(
        mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex,
        mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given sampling id
        if let Some(configuration_samplings) =
            Self::hardware_mining_config_samplings(mining_config_hardware_mining_id)
        {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_config_hardware_mining_id,
                configuration_samplings
            );
            let not_configuration_contains_sampling =
                !configuration_samplings.contains(&mining_samplings_hardware_mining_id);
            ensure!(not_configuration_contains_sampling, "Configuration already contains the given sampling id");
            debug::info!("Configuration id key exists but its vector value does not contain the given sampling id");
            <HardwareMiningConfigSamplings<T>>::mutate(
                mining_config_hardware_mining_id,
                |v| {
                    if let Some(value) = v {
                        value.push(mining_samplings_hardware_mining_id);
                    }
                },
            );
            debug::info!(
                "Associated sampling {:?} with configuration {:?}",
                mining_samplings_hardware_mining_id,
                mining_config_hardware_mining_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 sampling id {:?} to its vector value",
                mining_config_hardware_mining_id,
                mining_samplings_hardware_mining_id
            );
            <HardwareMiningConfigSamplings<T>>::insert(
                mining_config_hardware_mining_id,
                &vec![mining_samplings_hardware_mining_id],
            );
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

    fn next_mining_samplings_hardware_mining_id()
    -> Result<T::MiningSamplingHardwareMiningIndex, DispatchError> {
        let mining_samplings_hardware_mining_id =
            Self::mining_samplings_hardware_mining_count();
        if mining_samplings_hardware_mining_id ==
            <T::MiningSamplingHardwareMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSamplingHardwareMining count overflow"));
        }
        Ok(mining_samplings_hardware_mining_id)
    }

    fn insert_mining_samplings_hardware_mining(
        owner: &T::AccountId,
        mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex,
        mining_samplings_hardware_mining: MiningSamplingHardwareMining,
    ) {
        // Create and store mining mining_samplings_hardware_mining
        <MiningSamplingHardwareMinings<T>>::insert(
            mining_samplings_hardware_mining_id,
            mining_samplings_hardware_mining,
        );
        <MiningSamplingHardwareMiningCount<T>>::put(
            mining_samplings_hardware_mining_id + One::one(),
        );
        <MiningSamplingHardwareMiningOwners<T>>::insert(
            mining_samplings_hardware_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_samplings_hardware_mining_id: T::MiningSamplingHardwareMiningIndex,
    ) {
        <MiningSamplingHardwareMiningOwners<T>>::insert(mining_samplings_hardware_mining_id, to);
    }
}
