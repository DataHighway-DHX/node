#![cfg_attr(not(feature = "std"), no_std)]

use codec::{
    Decode,
    Encode,
};
use frame_support::traits::Randomness;
/// A runtime module for managing non-fungible tokens
use frame_support::{
    debug,
    decl_event,
    decl_module,
    decl_storage,
    ensure,
    traits::Get,
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
// mining-speed-boosts runtime module
use mining_speed_boosts_configuration_token_mining;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config:
    frame_system::Config + roaming_operators::Config + mining_speed_boosts_configuration_token_mining::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningSpeedBoostSamplingTokenMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostSamplingTokenMiningSampleDate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostSamplingTokenMiningSampleTokensLocked: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostSamplingTokenMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostSamplingTokenMiningSamplingConfig<U, V> {
    pub token_sample_date: U,
    pub token_sample_tokens_locked: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::MiningSpeedBoostSamplingTokenMiningIndex,
        <T as Config>::MiningSpeedBoostSamplingTokenMiningSampleDate,
        <T as Config>::MiningSpeedBoostSamplingTokenMiningSampleTokensLocked,
        <T as mining_speed_boosts_configuration_token_mining::Config>::MiningSpeedBoostConfigurationTokenMiningIndex,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_sampling_token_mining is created. (owner, mining_speed_boosts_sampling_token_mining_id)
        Created(AccountId, MiningSpeedBoostSamplingTokenMiningIndex),
        /// A mining_speed_boosts_samplings_token_mining is transferred. (from, to, mining_speed_boosts_samplings_token_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostSamplingTokenMiningIndex),
        MiningSpeedBoostSamplingTokenMiningSamplingConfigSet(
            AccountId, MiningSpeedBoostConfigurationTokenMiningIndex, MiningSpeedBoostSamplingTokenMiningIndex,
            MiningSpeedBoostSamplingTokenMiningSampleDate, MiningSpeedBoostSamplingTokenMiningSampleTokensLocked
        ),
        /// A mining_speed_boosts_sampling_token_mining is assigned to an mining_speed_boosts_token_mining.
        /// (owner of mining_speed_boosts_token_mining, mining_speed_boosts_samplings_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
            AssignedTokenMiningSamplingToConfiguration(AccountId, MiningSpeedBoostSamplingTokenMiningIndex, MiningSpeedBoostConfigurationTokenMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningSpeedBoostSamplingTokenMining {
        /// Stores all the mining_speed_boosts_samplings_token_minings, key is the mining_speed_boosts_samplings_token_mining id / index
        pub MiningSpeedBoostSamplingTokenMinings get(fn mining_speed_boosts_samplings_token_mining): map hasher(opaque_blake2_256) T::MiningSpeedBoostSamplingTokenMiningIndex => Option<MiningSpeedBoostSamplingTokenMining>;

        /// Stores the total number of mining_speed_boosts_samplings_token_minings. i.e. the next mining_speed_boosts_samplings_token_mining index
        pub MiningSpeedBoostSamplingTokenMiningCount get(fn mining_speed_boosts_samplings_token_mining_count): T::MiningSpeedBoostSamplingTokenMiningIndex;

        /// Stores mining_speed_boosts_samplings_token_mining owner
        pub MiningSpeedBoostSamplingTokenMiningOwners get(fn mining_speed_boosts_samplings_token_mining_owner): map hasher(opaque_blake2_256) T::MiningSpeedBoostSamplingTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_samplings_token_mining_samplings_config
        pub MiningSpeedBoostSamplingTokenMiningSamplingConfigs get(fn mining_speed_boosts_samplings_token_mining_samplings_configs): map hasher(opaque_blake2_256) (T::MiningSpeedBoostConfigurationTokenMiningIndex, T::MiningSpeedBoostSamplingTokenMiningIndex) =>
            Option<MiningSpeedBoostSamplingTokenMiningSamplingConfig<
                T::MiningSpeedBoostSamplingTokenMiningSampleDate,
                T::MiningSpeedBoostSamplingTokenMiningSampleTokensLocked
            >>;

        /// Get mining_speed_boosts_configuration_token_mining_id belonging to a mining_speed_boosts_samplings_token_mining_id
        pub TokenMiningSamplingConfiguration get(fn token_mining_sampling_configuration): map hasher(opaque_blake2_256) T::MiningSpeedBoostSamplingTokenMiningIndex => Option<T::MiningSpeedBoostConfigurationTokenMiningIndex>;

        /// Get mining_speed_boosts_samplings_token_mining_id's belonging to a mining_speed_boosts_configuration_token_mining_id
        pub TokenMiningConfigurationSamplings get(fn token_mining_configuration_samplings): map hasher(opaque_blake2_256) T::MiningSpeedBoostConfigurationTokenMiningIndex => Option<Vec<T::MiningSpeedBoostSamplingTokenMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_samplings_token_mining
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_samplings_token_mining_id = Self::next_mining_speed_boosts_samplings_token_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_samplings_token_mining
            let mining_speed_boosts_samplings_token_mining = MiningSpeedBoostSamplingTokenMining(unique_id);
            Self::insert_mining_speed_boosts_samplings_token_mining(&sender, mining_speed_boosts_samplings_token_mining_id, mining_speed_boosts_samplings_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_samplings_token_mining_id));
        }

        /// Transfer a mining_speed_boosts_samplings_token_mining to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_samplings_token_mining_owner(mining_speed_boosts_samplings_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_samplings_token_mining");

            Self::update_owner(&to, mining_speed_boosts_samplings_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_samplings_token_mining_id));
        }

        /// Set mining_speed_boosts_samplings_token_mining_samplings_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_speed_boosts_samplings_token_mining_samplings_config(
            origin,
            mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
            mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex,
            _token_sample_date: Option<T::MiningSpeedBoostSamplingTokenMiningSampleDate>,
            _token_sample_tokens_locked: Option<T::MiningSpeedBoostSamplingTokenMiningSampleTokensLocked>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_samplings_token_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_samplings_token_mining = Self::exists_mining_speed_boosts_samplings_token_mining(mining_speed_boosts_samplings_token_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_samplings_token_mining, "MiningSpeedBoostSamplingTokenMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_samplings_token_mining_samplings_config they are trying to change
            ensure!(Self::mining_speed_boosts_samplings_token_mining_owner(mining_speed_boosts_samplings_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_samplings_token_mining_samplings_config");

            // TODO - adjust default samplings
            let token_sample_date = match _token_sample_date.clone() {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            let token_sample_tokens_locked = match _token_sample_tokens_locked {
                Some(value) => value,
                None => 1u32.into() // Default
            };

            // Check if a mining_speed_boosts_samplings_token_mining_samplings_config already exists with the given mining_speed_boosts_samplings_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_samplings_token_mining_samplings_config_index(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_samplings_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostSamplingTokenMiningSamplingConfigs<T>>::mutate((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_samplings_token_mining_id), |mining_speed_boosts_samplings_token_mining_samplings_config| {
                    if let Some(_mining_speed_boosts_samplings_token_mining_samplings_config) = mining_speed_boosts_samplings_token_mining_samplings_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_samplings_token_mining_samplings_config.token_sample_date = token_sample_date.clone();
                        _mining_speed_boosts_samplings_token_mining_samplings_config.token_sample_tokens_locked = token_sample_tokens_locked.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_samplings_token_mining_samplings_config = <MiningSpeedBoostSamplingTokenMiningSamplingConfigs<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_samplings_token_mining_id));
                if let Some(_mining_speed_boosts_samplings_token_mining_samplings_config) = fetched_mining_speed_boosts_samplings_token_mining_samplings_config {
                    debug::info!("Latest field token_sample_date {:#?}", _mining_speed_boosts_samplings_token_mining_samplings_config.token_sample_date);
                    debug::info!("Latest field token_sample_tokens_locked {:#?}", _mining_speed_boosts_samplings_token_mining_samplings_config.token_sample_tokens_locked);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_samplings_token_mining_samplings_config instance with the input params
                let mining_speed_boosts_samplings_token_mining_samplings_config_instance = MiningSpeedBoostSamplingTokenMiningSamplingConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_sample_date: token_sample_date.clone(),
                    token_sample_tokens_locked: token_sample_tokens_locked.clone(),
                };

                <MiningSpeedBoostSamplingTokenMiningSamplingConfigs<T>>::insert(
                    (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_samplings_token_mining_id),
                    &mining_speed_boosts_samplings_token_mining_samplings_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_samplings_token_mining_samplings_config = <MiningSpeedBoostSamplingTokenMiningSamplingConfigs<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_samplings_token_mining_id));
                if let Some(_mining_speed_boosts_samplings_token_mining_samplings_config) = fetched_mining_speed_boosts_samplings_token_mining_samplings_config {
                    debug::info!("Inserted field token_sample_date {:#?}", _mining_speed_boosts_samplings_token_mining_samplings_config.token_sample_date);
                    debug::info!("Inserted field token_sample_tokens_locked {:#?}", _mining_speed_boosts_samplings_token_mining_samplings_config.token_sample_tokens_locked);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostSamplingTokenMiningSamplingConfigSet(
                sender,
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_samplings_token_mining_id,
                token_sample_date,
                token_sample_tokens_locked,
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_sampling_to_configuration(
          origin,
          mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex,
          mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token_mining = <mining_speed_boosts_configuration_token_mining::Module<T>>
                ::exists_mining_speed_boosts_configuration_token_mining(mining_speed_boosts_configuration_token_mining_id).is_ok();
            ensure!(is_configuration_token_mining, "configuration_token_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the sampling to
            ensure!(
                <mining_speed_boosts_configuration_token_mining::Module<T>>::is_mining_speed_boosts_configuration_token_mining_owner(mining_speed_boosts_configuration_token_mining_id, sender.clone()).is_ok(),
                "Only the configuration_token_mining owner can assign itself a sampling"
            );

            Self::associate_token_sampling_with_configuration(mining_speed_boosts_samplings_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
                .expect("Unable to associate sampling with configuration");

            // Ensure that the given mining_speed_boosts_samplings_token_mining_id already exists
            let token_sampling = Self::mining_speed_boosts_samplings_token_mining(mining_speed_boosts_samplings_token_mining_id);
            ensure!(token_sampling.is_some(), "Invalid mining_speed_boosts_samplings_token_mining_id");

            // // Ensure that the sampling is not already owned by a different configuration
            // // Unassign the sampling from any existing configuration since it may only be owned by one configuration
            // <TokenMiningSamplingConfiguration<T>>::remove(mining_speed_boosts_samplings_token_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenMiningSamplingConfiguration<T>>::insert(mining_speed_boosts_samplings_token_mining_id, mining_speed_boosts_configuration_token_mining_id);

            Self::deposit_event(RawEvent::AssignedTokenMiningSamplingToConfiguration(sender, mining_speed_boosts_samplings_token_mining_id, mining_speed_boosts_configuration_token_mining_id));
            }
    }
}

impl<T: Config> Module<T> {
    pub fn is_mining_speed_boosts_samplings_token_mining_owner(
        mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_samplings_token_mining_owner(&mining_speed_boosts_samplings_token_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostSamplingTokenMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_samplings_token_mining(
        mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex,
    ) -> Result<MiningSpeedBoostSamplingTokenMining, DispatchError> {
        match Self::mining_speed_boosts_samplings_token_mining(mining_speed_boosts_samplings_token_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostSamplingTokenMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_samplings_token_mining_samplings_config(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_samplings_token_mining_samplings_configs((
            mining_speed_boosts_configuration_token_mining_id,
            mining_speed_boosts_samplings_token_mining_id,
        )) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostSamplingTokenMiningSamplingConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_speed_boosts_samplings_token_mining_samplings_config_index(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_speed_boosts_samplings_token_mining_samplings_config has a value that is defined"
        );
        let fetched_mining_speed_boosts_samplings_token_mining_samplings_config =
            <MiningSpeedBoostSamplingTokenMiningSamplingConfigs<T>>::get((
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_samplings_token_mining_id,
            ));
        if let Some(_value) = fetched_mining_speed_boosts_samplings_token_mining_samplings_config {
            debug::info!("Found value for mining_speed_boosts_samplings_token_mining_samplings_config");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_samplings_token_mining_samplings_config");
        Err(DispatchError::Other("No value for mining_speed_boosts_samplings_token_mining_samplings_config"))
    }

    /// Only push the sampling id onto the end of the vector if it does not already exist
    pub fn associate_token_sampling_with_configuration(
        mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex,
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given sampling id
        if let Some(configuration_samplings) =
            Self::token_mining_configuration_samplings(mining_speed_boosts_configuration_token_mining_id)
        {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_speed_boosts_configuration_token_mining_id,
                configuration_samplings
            );
            let not_configuration_contains_sampling =
                !configuration_samplings.contains(&mining_speed_boosts_samplings_token_mining_id);
            ensure!(not_configuration_contains_sampling, "Configuration already contains the given sampling id");
            debug::info!("Configuration id key exists but its vector value does not contain the given sampling id");
            <TokenMiningConfigurationSamplings<T>>::mutate(mining_speed_boosts_configuration_token_mining_id, |v| {
                if let Some(value) = v {
                    value.push(mining_speed_boosts_samplings_token_mining_id);
                }
            });
            debug::info!(
                "Associated sampling {:?} with configuration {:?}",
                mining_speed_boosts_samplings_token_mining_id,
                mining_speed_boosts_configuration_token_mining_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 sampling id {:?} to its vector value",
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_samplings_token_mining_id
            );
            <TokenMiningConfigurationSamplings<T>>::insert(
                mining_speed_boosts_configuration_token_mining_id,
                &vec![mining_speed_boosts_samplings_token_mining_id],
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

    fn next_mining_speed_boosts_samplings_token_mining_id()
    -> Result<T::MiningSpeedBoostSamplingTokenMiningIndex, DispatchError> {
        let mining_speed_boosts_samplings_token_mining_id = Self::mining_speed_boosts_samplings_token_mining_count();
        if mining_speed_boosts_samplings_token_mining_id ==
            <T::MiningSpeedBoostSamplingTokenMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostSamplingTokenMining count overflow"));
        }
        Ok(mining_speed_boosts_samplings_token_mining_id)
    }

    fn insert_mining_speed_boosts_samplings_token_mining(
        owner: &T::AccountId,
        mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex,
        mining_speed_boosts_samplings_token_mining: MiningSpeedBoostSamplingTokenMining,
    ) {
        // Create and store mining mining_speed_boosts_samplings_token_mining
        <MiningSpeedBoostSamplingTokenMinings<T>>::insert(
            mining_speed_boosts_samplings_token_mining_id,
            mining_speed_boosts_samplings_token_mining,
        );
        <MiningSpeedBoostSamplingTokenMiningCount<T>>::put(mining_speed_boosts_samplings_token_mining_id + One::one());
        <MiningSpeedBoostSamplingTokenMiningOwners<T>>::insert(
            mining_speed_boosts_samplings_token_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_samplings_token_mining_id: T::MiningSpeedBoostSamplingTokenMiningIndex,
    ) {
        <MiningSpeedBoostSamplingTokenMiningOwners<T>>::insert(mining_speed_boosts_samplings_token_mining_id, to);
    }
}
