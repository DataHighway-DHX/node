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
use mining_config_token_mining;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait + roaming_operators::Trait + mining_config_token_mining::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningSamplingTokenMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSamplingTokenMiningSampleLockedAmount: Parameter
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
pub struct MiningSamplingTokenMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSamplingTokenMiningSamplingConfig<U, V> {
    pub token_sample_block: U,
    pub token_sample_locked_amount: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningSamplingTokenMiningIndex,
        <T as Trait>::MiningSamplingTokenMiningSampleLockedAmount,
        <T as mining_config_token_mining::Trait>::MiningConfigTokenMiningIndex,
        <T as frame_system::Trait>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_sampling_token_mining is created. (owner, mining_sampling_token_mining_id)
        Created(AccountId, MiningSamplingTokenMiningIndex),
        /// A mining_samplings_token_mining is transferred. (from, to, mining_samplings_token_mining_id)
        Transferred(AccountId, AccountId, MiningSamplingTokenMiningIndex),
        MiningSamplingTokenMiningSamplingConfigSet(
            AccountId, MiningConfigTokenMiningIndex, MiningSamplingTokenMiningIndex,
            BlockNumber, MiningSamplingTokenMiningSampleLockedAmount
        ),
        /// A mining_sampling_token_mining is assigned to an mining_token_mining.
        /// (owner of mining_token_mining, mining_samplings_token_mining_id, mining_config_token_mining_id)
        AssignedTokenMiningSamplingToConfiguration(AccountId, MiningSamplingTokenMiningIndex, MiningConfigTokenMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSamplingTokenMining {
        /// Stores all the mining_samplings_token_minings, key is the mining_samplings_token_mining id / index
        pub MiningSamplingTokenMinings get(fn mining_samplings_token_mining): map hasher(opaque_blake2_256) T::MiningSamplingTokenMiningIndex => Option<MiningSamplingTokenMining>;

        /// Stores the total number of mining_samplings_token_minings. i.e. the next mining_samplings_token_mining index
        pub MiningSamplingTokenMiningCount get(fn mining_samplings_token_mining_count): T::MiningSamplingTokenMiningIndex;

        /// Stores mining_samplings_token_mining owner
        pub MiningSamplingTokenMiningOwners get(fn mining_samplings_token_mining_owner): map hasher(opaque_blake2_256) T::MiningSamplingTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_samplings_token_mining_samplings_config
        pub MiningSamplingTokenMiningSamplingConfigs get(fn mining_samplings_token_mining_samplings_configs): map hasher(opaque_blake2_256) (T::MiningConfigTokenMiningIndex, T::MiningSamplingTokenMiningIndex) =>
            Option<MiningSamplingTokenMiningSamplingConfig<
                T::BlockNumber,
                T::MiningSamplingTokenMiningSampleLockedAmount
            >>;

        /// Get mining_config_token_mining_id belonging to a mining_samplings_token_mining_id
        pub TokenMiningSamplingConfiguration get(fn token_mining_sampling_configuration): map hasher(opaque_blake2_256) T::MiningSamplingTokenMiningIndex => Option<T::MiningConfigTokenMiningIndex>;

        /// Get mining_samplings_token_mining_id's belonging to a mining_config_token_mining_id
        pub TokenMiningConfigSamplings get(fn token_mining_config_samplings): map hasher(opaque_blake2_256) T::MiningConfigTokenMiningIndex => Option<Vec<T::MiningSamplingTokenMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_samplings_token_mining
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_samplings_token_mining_id = Self::next_mining_samplings_token_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_samplings_token_mining
            let mining_samplings_token_mining = MiningSamplingTokenMining(unique_id);
            Self::insert_mining_samplings_token_mining(&sender, mining_samplings_token_mining_id, mining_samplings_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_samplings_token_mining_id));
        }

        /// Transfer a mining_samplings_token_mining to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_samplings_token_mining_owner(mining_samplings_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_samplings_token_mining");

            Self::update_owner(&to, mining_samplings_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_samplings_token_mining_id));
        }

        /// Set mining_samplings_token_mining_samplings_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_samplings_token_mining_samplings_config(
            origin,
            mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
            mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex,
            _token_sample_block: Option<T::BlockNumber>,
            _token_sample_locked_amount: Option<T::MiningSamplingTokenMiningSampleLockedAmount>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_samplings_token_mining_id whose config we want to change actually exists
            let is_mining_samplings_token_mining = Self::exists_mining_samplings_token_mining(mining_samplings_token_mining_id).is_ok();
            ensure!(is_mining_samplings_token_mining, "MiningSamplingTokenMining does not exist");

            // Ensure that the caller is owner of the mining_samplings_token_mining_samplings_config they are trying to change
            ensure!(Self::mining_samplings_token_mining_owner(mining_samplings_token_mining_id) == Some(sender.clone()), "Only owner can set mining_samplings_token_mining_samplings_config");

            // TODO - adjust default samplings
            let token_sample_block = match _token_sample_block.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_sample_locked_amount = match _token_sample_locked_amount {
                Some(value) => value,
                None => 1.into() // Default
            };

            // Check if a mining_samplings_token_mining_samplings_config already exists with the given mining_samplings_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_samplings_token_mining_samplings_config_index(mining_config_token_mining_id, mining_samplings_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSamplingTokenMiningSamplingConfigs<T>>::mutate((mining_config_token_mining_id, mining_samplings_token_mining_id), |mining_samplings_token_mining_samplings_config| {
                    if let Some(_mining_samplings_token_mining_samplings_config) = mining_samplings_token_mining_samplings_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_samplings_token_mining_samplings_config.token_sample_block = token_sample_block.clone();
                        _mining_samplings_token_mining_samplings_config.token_sample_locked_amount = token_sample_locked_amount.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_samplings_token_mining_samplings_config = <MiningSamplingTokenMiningSamplingConfigs<T>>::get((mining_config_token_mining_id, mining_samplings_token_mining_id));
                if let Some(_mining_samplings_token_mining_samplings_config) = fetched_mining_samplings_token_mining_samplings_config {
                    debug::info!("Latest field token_sample_block {:#?}", _mining_samplings_token_mining_samplings_config.token_sample_block);
                    debug::info!("Latest field token_sample_locked_amount {:#?}", _mining_samplings_token_mining_samplings_config.token_sample_locked_amount);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_samplings_token_mining_samplings_config instance with the input params
                let mining_samplings_token_mining_samplings_config_instance = MiningSamplingTokenMiningSamplingConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_sample_block: token_sample_block.clone(),
                    token_sample_locked_amount: token_sample_locked_amount.clone(),
                };

                <MiningSamplingTokenMiningSamplingConfigs<T>>::insert(
                    (mining_config_token_mining_id, mining_samplings_token_mining_id),
                    &mining_samplings_token_mining_samplings_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_samplings_token_mining_samplings_config = <MiningSamplingTokenMiningSamplingConfigs<T>>::get((mining_config_token_mining_id, mining_samplings_token_mining_id));
                if let Some(_mining_samplings_token_mining_samplings_config) = fetched_mining_samplings_token_mining_samplings_config {
                    debug::info!("Inserted field token_sample_block {:#?}", _mining_samplings_token_mining_samplings_config.token_sample_block);
                    debug::info!("Inserted field token_sample_locked_amount {:#?}", _mining_samplings_token_mining_samplings_config.token_sample_locked_amount);
                }
            }

            Self::deposit_event(RawEvent::MiningSamplingTokenMiningSamplingConfigSet(
                sender,
                mining_config_token_mining_id,
                mining_samplings_token_mining_id,
                token_sample_block,
                token_sample_locked_amount,
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_sampling_to_configuration(
          origin,
          mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex,
          mining_config_token_mining_id: T::MiningConfigTokenMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token_mining = <mining_config_token_mining::Module<T>>
                ::exists_mining_config_token_mining(mining_config_token_mining_id).is_ok();
            ensure!(is_configuration_token_mining, "configuration_token_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the sampling to
            ensure!(
                <mining_config_token_mining::Module<T>>::is_mining_config_token_mining_owner(mining_config_token_mining_id, sender.clone()).is_ok(),
                "Only the configuration_token_mining owner can assign itself a sampling"
            );

            Self::associate_token_sampling_with_configuration(mining_samplings_token_mining_id, mining_config_token_mining_id)
                .expect("Unable to associate sampling with configuration");

            // Ensure that the given mining_samplings_token_mining_id already exists
            let token_sampling = Self::mining_samplings_token_mining(mining_samplings_token_mining_id);
            ensure!(token_sampling.is_some(), "Invalid mining_samplings_token_mining_id");

            // // Ensure that the sampling is not already owned by a different configuration
            // // Unassign the sampling from any existing configuration since it may only be owned by one configuration
            // <TokenMiningSamplingConfiguration<T>>::remove(mining_samplings_token_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenMiningSamplingConfiguration<T>>::insert(mining_samplings_token_mining_id, mining_config_token_mining_id);

            Self::deposit_event(RawEvent::AssignedTokenMiningSamplingToConfiguration(sender, mining_samplings_token_mining_id, mining_config_token_mining_id));
            }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_samplings_token_mining_owner(
        mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_samplings_token_mining_owner(&mining_samplings_token_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSamplingTokenMining"
        );
        Ok(())
    }

    pub fn exists_mining_samplings_token_mining(
        mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex,
    ) -> Result<MiningSamplingTokenMining, DispatchError> {
        match Self::mining_samplings_token_mining(mining_samplings_token_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSamplingTokenMining does not exist")),
        }
    }

    pub fn exists_mining_samplings_token_mining_samplings_config(
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
        mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_samplings_token_mining_samplings_configs((
            mining_config_token_mining_id,
            mining_samplings_token_mining_id,
        )) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningSamplingTokenMiningSamplingConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_samplings_token_mining_samplings_config_index(
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
        mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_samplings_token_mining_samplings_config has a value that is defined"
        );
        let fetched_mining_samplings_token_mining_samplings_config =
            <MiningSamplingTokenMiningSamplingConfigs<T>>::get((
                mining_config_token_mining_id,
                mining_samplings_token_mining_id,
            ));
        if let Some(_value) = fetched_mining_samplings_token_mining_samplings_config {
            debug::info!("Found value for mining_samplings_token_mining_samplings_config");
            return Ok(());
        }
        debug::info!("No value for mining_samplings_token_mining_samplings_config");
        Err(DispatchError::Other("No value for mining_samplings_token_mining_samplings_config"))
    }

    /// Only push the sampling id onto the end of the vector if it does not already exist
    pub fn associate_token_sampling_with_configuration(
        mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex,
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given sampling id
        if let Some(configuration_samplings) =
            Self::token_mining_config_samplings(mining_config_token_mining_id)
        {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_config_token_mining_id,
                configuration_samplings
            );
            let not_configuration_contains_sampling =
                !configuration_samplings.contains(&mining_samplings_token_mining_id);
            ensure!(not_configuration_contains_sampling, "Configuration already contains the given sampling id");
            debug::info!("Configuration id key exists but its vector value does not contain the given sampling id");
            <TokenMiningConfigSamplings<T>>::mutate(mining_config_token_mining_id, |v| {
                if let Some(value) = v {
                    value.push(mining_samplings_token_mining_id);
                }
            });
            debug::info!(
                "Associated sampling {:?} with configuration {:?}",
                mining_samplings_token_mining_id,
                mining_config_token_mining_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 sampling id {:?} to its vector value",
                mining_config_token_mining_id,
                mining_samplings_token_mining_id
            );
            <TokenMiningConfigSamplings<T>>::insert(
                mining_config_token_mining_id,
                &vec![mining_samplings_token_mining_id],
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

    fn next_mining_samplings_token_mining_id()
    -> Result<T::MiningSamplingTokenMiningIndex, DispatchError> {
        let mining_samplings_token_mining_id = Self::mining_samplings_token_mining_count();
        if mining_samplings_token_mining_id ==
            <T::MiningSamplingTokenMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSamplingTokenMining count overflow"));
        }
        Ok(mining_samplings_token_mining_id)
    }

    fn insert_mining_samplings_token_mining(
        owner: &T::AccountId,
        mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex,
        mining_samplings_token_mining: MiningSamplingTokenMining,
    ) {
        // Create and store mining mining_samplings_token_mining
        <MiningSamplingTokenMinings<T>>::insert(
            mining_samplings_token_mining_id,
            mining_samplings_token_mining,
        );
        <MiningSamplingTokenMiningCount<T>>::put(mining_samplings_token_mining_id + One::one());
        <MiningSamplingTokenMiningOwners<T>>::insert(
            mining_samplings_token_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_samplings_token_mining_id: T::MiningSamplingTokenMiningIndex,
    ) {
        <MiningSamplingTokenMiningOwners<T>>::insert(mining_samplings_token_mining_id, to);
    }
}
