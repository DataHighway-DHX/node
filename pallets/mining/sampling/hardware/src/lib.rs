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
use mining_config_hardware;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config: frame_system::Config + roaming_operators::Config + mining_config_hardware::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningSamplingHardwareIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSamplingHardwareSampleHardwareOnline: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSamplingHardware(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSamplingHardwareSetting<U, V> {
    pub hardware_sample_block: U,
    pub hardware_sample_hardware_online: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::MiningSamplingHardwareIndex,
        <T as Config>::MiningSamplingHardwareSampleHardwareOnline,
        <T as mining_config_hardware::Config>::MiningSettingHardwareIndex,
        <T as frame_system::Config>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_sampling_hardware is created. (owner, mining_sampling_hardware_id)
        Created(AccountId, MiningSamplingHardwareIndex),
        /// A mining_samplings_hardware is transferred. (from, to, mining_samplings_hardware_id)
        Transferred(AccountId, AccountId, MiningSamplingHardwareIndex),
        MiningSamplingHardwareSettingSet(
            AccountId, MiningSettingHardwareIndex, MiningSamplingHardwareIndex,
            BlockNumber, MiningSamplingHardwareSampleHardwareOnline
        ),
        /// A mining_sampling_hardware is assigned to an mining_hardware.
        /// (owner of mining_hardware, mining_samplings_hardware_id, mining_config_hardware_id)
        AssignedHardwareSamplingToConfiguration(AccountId, MiningSamplingHardwareIndex, MiningSettingHardwareIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningSamplingHardware {
        /// Stores all the mining_samplings_hardwares, key is the mining_samplings_hardware id / index
        pub MiningSamplingHardwares get(fn mining_samplings_hardware): map hasher(opaque_blake2_256) T::MiningSamplingHardwareIndex => Option<MiningSamplingHardware>;

        /// Stores the total number of mining_samplings_hardwares. i.e. the next mining_samplings_hardware index
        pub MiningSamplingHardwareCount get(fn mining_samplings_hardware_count): T::MiningSamplingHardwareIndex;

        /// Stores mining_samplings_hardware owner
        pub MiningSamplingHardwareOwners get(fn mining_samplings_hardware_owner): map hasher(opaque_blake2_256) T::MiningSamplingHardwareIndex => Option<T::AccountId>;

        /// Stores mining_samplings_hardware_samplings_config
        pub MiningSamplingHardwareSettings get(fn mining_samplings_hardware_samplings_configs): map hasher(opaque_blake2_256) (T::MiningSettingHardwareIndex, T::MiningSamplingHardwareIndex) =>
            Option<MiningSamplingHardwareSetting<
                T::BlockNumber,
                T::MiningSamplingHardwareSampleHardwareOnline
            >>;

        /// Get mining_config_hardware_id belonging to a mining_samplings_hardware_id
        pub HardwareSamplingConfiguration get(fn hardware_sampling_configuration): map hasher(opaque_blake2_256) T::MiningSamplingHardwareIndex => Option<T::MiningSettingHardwareIndex>;

        /// Get mining_samplings_hardware_id's belonging to a mining_config_hardware_id
        pub HardwareSettingSamplings get(fn hardware_config_samplings): map hasher(opaque_blake2_256) T::MiningSettingHardwareIndex => Option<Vec<T::MiningSamplingHardwareIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_samplings_hardware
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_samplings_hardware_id = Self::next_mining_samplings_hardware_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_samplings_hardware
            let mining_samplings_hardware = MiningSamplingHardware(unique_id);
            Self::insert_mining_samplings_hardware(&sender, mining_samplings_hardware_id, mining_samplings_hardware);

            Self::deposit_event(RawEvent::Created(sender, mining_samplings_hardware_id));
        }

        /// Transfer a mining_samplings_hardware to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_samplings_hardware_id: T::MiningSamplingHardwareIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_samplings_hardware_owner(mining_samplings_hardware_id) == Some(sender.clone()), "Only owner can transfer mining mining_samplings_hardware");

            Self::update_owner(&to, mining_samplings_hardware_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_samplings_hardware_id));
        }

        /// Set mining_samplings_hardware_samplings_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_samplings_hardware_samplings_config(
            origin,
            mining_config_hardware_id: T::MiningSettingHardwareIndex,
            mining_samplings_hardware_id: T::MiningSamplingHardwareIndex,
            _hardware_sample_block: Option<T::BlockNumber>,
            _hardware_sample_hardware_online: Option<T::MiningSamplingHardwareSampleHardwareOnline>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_samplings_hardware_id whose config we want to change actually exists
            let is_mining_samplings_hardware = Self::exists_mining_samplings_hardware(mining_samplings_hardware_id).is_ok();
            ensure!(is_mining_samplings_hardware, "MiningSamplingHardware does not exist");

            // Ensure that the caller is owner of the mining_samplings_hardware_samplings_config they are trying to change
            ensure!(Self::mining_samplings_hardware_owner(mining_samplings_hardware_id) == Some(sender.clone()), "Only owner can set mining_samplings_hardware_samplings_config");

            // TODO - adjust default samplings
            let hardware_sample_block = match _hardware_sample_block.clone() {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            let hardware_sample_hardware_online = match _hardware_sample_hardware_online {
                Some(value) => value,
                None => 1u32.into() // Default
            };

            // Check if a mining_samplings_hardware_samplings_config already exists with the given mining_samplings_hardware_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_samplings_hardware_samplings_config_index(mining_config_hardware_id, mining_samplings_hardware_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSamplingHardwareSettings<T>>::mutate((mining_config_hardware_id, mining_samplings_hardware_id), |mining_samplings_hardware_samplings_config| {
                    if let Some(_mining_samplings_hardware_samplings_config) = mining_samplings_hardware_samplings_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_samplings_hardware_samplings_config.hardware_sample_block = hardware_sample_block.clone();
                        _mining_samplings_hardware_samplings_config.hardware_sample_hardware_online = hardware_sample_hardware_online.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_samplings_hardware_samplings_config = <MiningSamplingHardwareSettings<T>>::get((mining_config_hardware_id, mining_samplings_hardware_id));
                if let Some(_mining_samplings_hardware_samplings_config) = fetched_mining_samplings_hardware_samplings_config {
                    debug::info!("Latest field hardware_sample_block {:#?}", _mining_samplings_hardware_samplings_config.hardware_sample_block);
                    debug::info!("Latest field hardware_sample_hardware_online {:#?}", _mining_samplings_hardware_samplings_config.hardware_sample_hardware_online);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_samplings_hardware_samplings_config instance with the input params
                let mining_samplings_hardware_samplings_config_instance = MiningSamplingHardwareSetting {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_sample_block: hardware_sample_block.clone(),
                    hardware_sample_hardware_online: hardware_sample_hardware_online.clone(),
                };

                <MiningSamplingHardwareSettings<T>>::insert(
                    (mining_config_hardware_id, mining_samplings_hardware_id),
                    &mining_samplings_hardware_samplings_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_samplings_hardware_samplings_config = <MiningSamplingHardwareSettings<T>>::get((mining_config_hardware_id, mining_samplings_hardware_id));
                if let Some(_mining_samplings_hardware_samplings_config) = fetched_mining_samplings_hardware_samplings_config {
                    debug::info!("Inserted field hardware_sample_block {:#?}", _mining_samplings_hardware_samplings_config.hardware_sample_block);
                    debug::info!("Inserted field hardware_sample_hardware_online {:#?}", _mining_samplings_hardware_samplings_config.hardware_sample_hardware_online);
                }
            }

            Self::deposit_event(RawEvent::MiningSamplingHardwareSettingSet(
                sender,
                mining_config_hardware_id,
                mining_samplings_hardware_id,
                hardware_sample_block,
                hardware_sample_hardware_online,
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_sampling_to_configuration(
          origin,
          mining_samplings_hardware_id: T::MiningSamplingHardwareIndex,
          mining_config_hardware_id: T::MiningSettingHardwareIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_hardware = <mining_config_hardware::Module<T>>
                ::exists_mining_config_hardware(mining_config_hardware_id).is_ok();
            ensure!(is_configuration_hardware, "configuration_hardware does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the sampling to
            ensure!(
                <mining_config_hardware::Module<T>>::is_mining_config_hardware_owner(mining_config_hardware_id, sender.clone()).is_ok(),
                "Only the configuration_hardware owner can assign itself a sampling"
            );

            Self::associate_hardware_sampling_with_configuration(mining_samplings_hardware_id, mining_config_hardware_id)
                .expect("Unable to associate sampling with configuration");

            // Ensure that the given mining_samplings_hardware_id already exists
            let hardware_sampling = Self::mining_samplings_hardware(mining_samplings_hardware_id);
            ensure!(hardware_sampling.is_some(), "Invalid mining_samplings_hardware_id");

            // // Ensure that the sampling is not already owned by a different configuration
            // // Unassign the sampling from any existing configuration since it may only be owned by one configuration
            // <HardwareSamplingConfiguration<T>>::remove(mining_samplings_hardware_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <HardwareSamplingConfiguration<T>>::insert(mining_samplings_hardware_id, mining_config_hardware_id);

            Self::deposit_event(RawEvent::AssignedHardwareSamplingToConfiguration(sender, mining_samplings_hardware_id, mining_config_hardware_id));
            }
    }
}

impl<T: Config> Module<T> {
    pub fn is_mining_samplings_hardware_owner(
        mining_samplings_hardware_id: T::MiningSamplingHardwareIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_samplings_hardware_owner(&mining_samplings_hardware_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSamplingHardware"
        );
        Ok(())
    }

    pub fn exists_mining_samplings_hardware(
        mining_samplings_hardware_id: T::MiningSamplingHardwareIndex,
    ) -> Result<MiningSamplingHardware, DispatchError> {
        match Self::mining_samplings_hardware(mining_samplings_hardware_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSamplingHardware does not exist")),
        }
    }

    pub fn exists_mining_samplings_hardware_samplings_config(
        mining_config_hardware_id: T::MiningSettingHardwareIndex,
        mining_samplings_hardware_id: T::MiningSamplingHardwareIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_samplings_hardware_samplings_configs((
            mining_config_hardware_id,
            mining_samplings_hardware_id,
        )) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningSamplingHardwareSetting does not exist")),
        }
    }

    pub fn has_value_for_mining_samplings_hardware_samplings_config_index(
        mining_config_hardware_id: T::MiningSettingHardwareIndex,
        mining_samplings_hardware_id: T::MiningSamplingHardwareIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_samplings_hardware_samplings_config has a value that is defined");
        let fetched_mining_samplings_hardware_samplings_config =
            <MiningSamplingHardwareSettings<T>>::get((mining_config_hardware_id, mining_samplings_hardware_id));
        if let Some(_value) = fetched_mining_samplings_hardware_samplings_config {
            debug::info!("Found value for mining_samplings_hardware_samplings_config");
            return Ok(());
        }
        debug::info!("No value for mining_samplings_hardware_samplings_config");
        Err(DispatchError::Other("No value for mining_samplings_hardware_samplings_config"))
    }

    /// Only push the sampling id onto the end of the vector if it does not already exist
    pub fn associate_hardware_sampling_with_configuration(
        mining_samplings_hardware_id: T::MiningSamplingHardwareIndex,
        mining_config_hardware_id: T::MiningSettingHardwareIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given sampling id
        if let Some(configuration_samplings) = Self::hardware_config_samplings(mining_config_hardware_id) {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_config_hardware_id,
                configuration_samplings
            );
            let not_configuration_contains_sampling = !configuration_samplings.contains(&mining_samplings_hardware_id);
            ensure!(not_configuration_contains_sampling, "Configuration already contains the given sampling id");
            debug::info!("Configuration id key exists but its vector value does not contain the given sampling id");
            <HardwareSettingSamplings<T>>::mutate(mining_config_hardware_id, |v| {
                if let Some(value) = v {
                    value.push(mining_samplings_hardware_id);
                }
            });
            debug::info!(
                "Associated sampling {:?} with configuration {:?}",
                mining_samplings_hardware_id,
                mining_config_hardware_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 sampling id {:?} to its vector value",
                mining_config_hardware_id,
                mining_samplings_hardware_id
            );
            <HardwareSettingSamplings<T>>::insert(mining_config_hardware_id, &vec![mining_samplings_hardware_id]);
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

    fn next_mining_samplings_hardware_id() -> Result<T::MiningSamplingHardwareIndex, DispatchError> {
        let mining_samplings_hardware_id = Self::mining_samplings_hardware_count();
        if mining_samplings_hardware_id == <T::MiningSamplingHardwareIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningSamplingHardware count overflow"));
        }
        Ok(mining_samplings_hardware_id)
    }

    fn insert_mining_samplings_hardware(
        owner: &T::AccountId,
        mining_samplings_hardware_id: T::MiningSamplingHardwareIndex,
        mining_samplings_hardware: MiningSamplingHardware,
    ) {
        // Create and store mining mining_samplings_hardware
        <MiningSamplingHardwares<T>>::insert(mining_samplings_hardware_id, mining_samplings_hardware);
        <MiningSamplingHardwareCount<T>>::put(mining_samplings_hardware_id + One::one());
        <MiningSamplingHardwareOwners<T>>::insert(mining_samplings_hardware_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_samplings_hardware_id: T::MiningSamplingHardwareIndex) {
        <MiningSamplingHardwareOwners<T>>::insert(mining_samplings_hardware_id, to);
    }
}
