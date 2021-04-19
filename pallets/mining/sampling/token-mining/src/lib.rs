#![cfg_attr(not(feature = "std"), no_std)]

use codec::{
    Decode,
    Encode,
};
use frame_support::{
    log,
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
use mining_setting_token;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config:
    frame_system::Config + roaming_operators::Config + mining_setting_token::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningSamplingTokenIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSamplingTokenSampleLockedAmount: Parameter
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
pub struct MiningSamplingToken(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSamplingTokenSetting<U, V> {
    pub token_sample_block: U,
    pub token_sample_locked_amount: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::MiningSamplingTokenIndex,
        <T as Config>::MiningSamplingTokenSampleLockedAmount,
        <T as mining_setting_token::Config>::MiningSettingTokenIndex,
        <T as frame_system::Config>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_sampling_token is created. (owner, mining_sampling_token_id)
        Created(AccountId, MiningSamplingTokenIndex),
        /// A mining_samplings_token is transferred. (from, to, mining_samplings_token_id)
        Transferred(AccountId, AccountId, MiningSamplingTokenIndex),
        MiningSamplingTokenSettingSet(
            AccountId, MiningSettingTokenIndex, MiningSamplingTokenIndex,
            BlockNumber, MiningSamplingTokenSampleLockedAmount
        ),
        /// A mining_sampling_token is assigned to an mining_token.
        /// (owner of mining_token, mining_samplings_token_id, mining_setting_token_id)
        AssignedTokenSamplingToConfiguration(AccountId, MiningSamplingTokenIndex, MiningSettingTokenIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningSamplingToken {
        /// Stores all the mining_samplings_tokens, key is the mining_samplings_token id / index
        pub MiningSamplingTokens get(fn mining_samplings_token): map hasher(opaque_blake2_256) T::MiningSamplingTokenIndex => Option<MiningSamplingToken>;

        /// Stores the total number of mining_samplings_tokens. i.e. the next mining_samplings_token index
        pub MiningSamplingTokenCount get(fn mining_samplings_token_count): T::MiningSamplingTokenIndex;

        /// Stores mining_samplings_token owner
        pub MiningSamplingTokenOwners get(fn mining_samplings_token_owner): map hasher(opaque_blake2_256) T::MiningSamplingTokenIndex => Option<T::AccountId>;

        /// Stores mining_samplings_token_samplings_config
        pub MiningSamplingTokenSettings get(fn mining_samplings_token_samplings_configs): map hasher(opaque_blake2_256) (T::MiningSettingTokenIndex, T::MiningSamplingTokenIndex) =>
            Option<MiningSamplingTokenSetting<
                T::BlockNumber,
                T::MiningSamplingTokenSampleLockedAmount
            >>;

        /// Get mining_setting_token_id belonging to a mining_samplings_token_id
        pub TokenSamplingConfiguration get(fn token_sampling_configuration): map hasher(opaque_blake2_256) T::MiningSamplingTokenIndex => Option<T::MiningSettingTokenIndex>;

        /// Get mining_samplings_token_id's belonging to a mining_setting_token_id
        pub TokenSettingSamplings get(fn token_setting_samplings): map hasher(opaque_blake2_256) T::MiningSettingTokenIndex => Option<Vec<T::MiningSamplingTokenIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_samplings_token
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_samplings_token_id = Self::next_mining_samplings_token_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_samplings_token
            let mining_samplings_token = MiningSamplingToken(unique_id);
            Self::insert_mining_samplings_token(&sender, mining_samplings_token_id, mining_samplings_token);

            Self::deposit_event(RawEvent::Created(sender, mining_samplings_token_id));
        }

        /// Transfer a mining_samplings_token to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_samplings_token_id: T::MiningSamplingTokenIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_samplings_token_owner(mining_samplings_token_id) == Some(sender.clone()), "Only owner can transfer mining mining_samplings_token");

            Self::update_owner(&to, mining_samplings_token_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_samplings_token_id));
        }

        /// Set mining_samplings_token_samplings_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_samplings_token_samplings_config(
            origin,
            mining_setting_token_id: T::MiningSettingTokenIndex,
            mining_samplings_token_id: T::MiningSamplingTokenIndex,
            _token_sample_block: Option<T::BlockNumber>,
            _token_sample_locked_amount: Option<T::MiningSamplingTokenSampleLockedAmount>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_samplings_token_id whose config we want to change actually exists
            let is_mining_samplings_token = Self::exists_mining_samplings_token(mining_samplings_token_id).is_ok();
            ensure!(is_mining_samplings_token, "MiningSamplingToken does not exist");

            // Ensure that the caller is owner of the mining_samplings_token_samplings_config they are trying to change
            ensure!(Self::mining_samplings_token_owner(mining_samplings_token_id) == Some(sender.clone()), "Only owner can set mining_samplings_token_samplings_config");

            // TODO - adjust default samplings
            let token_sample_block = match _token_sample_block.clone() {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            let token_sample_locked_amount = match _token_sample_locked_amount {
                Some(value) => value,
                None => 1u32.into() // Default
            };

            // Check if a mining_samplings_token_samplings_config already exists with the given mining_samplings_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_samplings_token_samplings_config_index(mining_setting_token_id, mining_samplings_token_id).is_ok() {
                log::info!("Mutating values");
                <MiningSamplingTokenSettings<T>>::mutate((mining_setting_token_id, mining_samplings_token_id), |mining_samplings_token_samplings_config| {
                    if let Some(_mining_samplings_token_samplings_config) = mining_samplings_token_samplings_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_samplings_token_samplings_config.token_sample_block = token_sample_block.clone();
                        _mining_samplings_token_samplings_config.token_sample_locked_amount = token_sample_locked_amount.clone();
                    }
                });
                log::info!("Checking mutated values");
                let fetched_mining_samplings_token_samplings_config = <MiningSamplingTokenSettings<T>>::get((mining_setting_token_id, mining_samplings_token_id));
                if let Some(_mining_samplings_token_samplings_config) = fetched_mining_samplings_token_samplings_config {
                    log::info!("Latest field token_sample_block {:#?}", _mining_samplings_token_samplings_config.token_sample_block);
                    log::info!("Latest field token_sample_locked_amount {:#?}", _mining_samplings_token_samplings_config.token_sample_locked_amount);
                }
            } else {
                log::info!("Inserting values");

                // Create a new mining mining_samplings_token_samplings_config instance with the input params
                let mining_samplings_token_samplings_config_instance = MiningSamplingTokenSetting {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_sample_block: token_sample_block.clone(),
                    token_sample_locked_amount: token_sample_locked_amount.clone(),
                };

                <MiningSamplingTokenSettings<T>>::insert(
                    (mining_setting_token_id, mining_samplings_token_id),
                    &mining_samplings_token_samplings_config_instance
                );

                log::info!("Checking inserted values");
                let fetched_mining_samplings_token_samplings_config = <MiningSamplingTokenSettings<T>>::get((mining_setting_token_id, mining_samplings_token_id));
                if let Some(_mining_samplings_token_samplings_config) = fetched_mining_samplings_token_samplings_config {
                    log::info!("Inserted field token_sample_block {:#?}", _mining_samplings_token_samplings_config.token_sample_block);
                    log::info!("Inserted field token_sample_locked_amount {:#?}", _mining_samplings_token_samplings_config.token_sample_locked_amount);
                }
            }

            Self::deposit_event(RawEvent::MiningSamplingTokenSettingSet(
                sender,
                mining_setting_token_id,
                mining_samplings_token_id,
                token_sample_block,
                token_sample_locked_amount,
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_sampling_to_configuration(
          origin,
          mining_samplings_token_id: T::MiningSamplingTokenIndex,
          mining_setting_token_id: T::MiningSettingTokenIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token = <mining_setting_token::Module<T>>
                ::exists_mining_setting_token(mining_setting_token_id).is_ok();
            ensure!(is_configuration_token, "configuration_token does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the sampling to
            ensure!(
                <mining_setting_token::Module<T>>::is_mining_setting_token_owner(mining_setting_token_id, sender.clone()).is_ok(),
                "Only the configuration_token owner can assign itself a sampling"
            );

            Self::associate_token_sampling_with_configuration(mining_samplings_token_id, mining_setting_token_id)
                .expect("Unable to associate sampling with configuration");

            // Ensure that the given mining_samplings_token_id already exists
            let token_sampling = Self::mining_samplings_token(mining_samplings_token_id);
            ensure!(token_sampling.is_some(), "Invalid mining_samplings_token_id");

            // // Ensure that the sampling is not already owned by a different configuration
            // // Unassign the sampling from any existing configuration since it may only be owned by one configuration
            // <TokenSamplingConfiguration<T>>::remove(mining_samplings_token_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenSamplingConfiguration<T>>::insert(mining_samplings_token_id, mining_setting_token_id);

            Self::deposit_event(RawEvent::AssignedTokenSamplingToConfiguration(sender, mining_samplings_token_id, mining_setting_token_id));
            }
    }
}

impl<T: Config> Module<T> {
    pub fn is_mining_samplings_token_owner(
        mining_samplings_token_id: T::MiningSamplingTokenIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_samplings_token_owner(&mining_samplings_token_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSamplingToken"
        );
        Ok(())
    }

    pub fn exists_mining_samplings_token(
        mining_samplings_token_id: T::MiningSamplingTokenIndex,
    ) -> Result<MiningSamplingToken, DispatchError> {
        match Self::mining_samplings_token(mining_samplings_token_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSamplingToken does not exist")),
        }
    }

    pub fn exists_mining_samplings_token_samplings_config(
        mining_setting_token_id: T::MiningSettingTokenIndex,
        mining_samplings_token_id: T::MiningSamplingTokenIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_samplings_token_samplings_configs((
            mining_setting_token_id,
            mining_samplings_token_id,
        )) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningSamplingTokenSetting does not exist")),
        }
    }

    pub fn has_value_for_mining_samplings_token_samplings_config_index(
        mining_setting_token_id: T::MiningSettingTokenIndex,
        mining_samplings_token_id: T::MiningSamplingTokenIndex,
    ) -> Result<(), DispatchError> {
        log::info!(
            "Checking if mining_samplings_token_samplings_config has a value that is defined"
        );
        let fetched_mining_samplings_token_samplings_config =
            <MiningSamplingTokenSettings<T>>::get((
                mining_setting_token_id,
                mining_samplings_token_id,
            ));
        if let Some(_value) = fetched_mining_samplings_token_samplings_config {
            log::info!("Found value for mining_samplings_token_samplings_config");
            return Ok(());
        }
        log::info!("No value for mining_samplings_token_samplings_config");
        Err(DispatchError::Other("No value for mining_samplings_token_samplings_config"))
    }

    /// Only push the sampling id onto the end of the vector if it does not already exist
    pub fn associate_token_sampling_with_configuration(
        mining_samplings_token_id: T::MiningSamplingTokenIndex,
        mining_setting_token_id: T::MiningSettingTokenIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given sampling id
        if let Some(configuration_samplings) =
            Self::token_setting_samplings(mining_setting_token_id)
        {
            log::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_setting_token_id,
                configuration_samplings
            );
            let not_configuration_contains_sampling =
                !configuration_samplings.contains(&mining_samplings_token_id);
            ensure!(not_configuration_contains_sampling, "Configuration already contains the given sampling id");
            log::info!("Configuration id key exists but its vector value does not contain the given sampling id");
            <TokenSettingSamplings<T>>::mutate(mining_setting_token_id, |v| {
                if let Some(value) = v {
                    value.push(mining_samplings_token_id);
                }
            });
            log::info!(
                "Associated sampling {:?} with configuration {:?}",
                mining_samplings_token_id,
                mining_setting_token_id
            );
            Ok(())
        } else {
            log::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 sampling id {:?} to its vector value",
                mining_setting_token_id,
                mining_samplings_token_id
            );
            <TokenSettingSamplings<T>>::insert(
                mining_setting_token_id,
                &vec![mining_samplings_token_id],
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

    fn next_mining_samplings_token_id()
    -> Result<T::MiningSamplingTokenIndex, DispatchError> {
        let mining_samplings_token_id = Self::mining_samplings_token_count();
        if mining_samplings_token_id ==
            <T::MiningSamplingTokenIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSamplingToken count overflow"));
        }
        Ok(mining_samplings_token_id)
    }

    fn insert_mining_samplings_token(
        owner: &T::AccountId,
        mining_samplings_token_id: T::MiningSamplingTokenIndex,
        mining_samplings_token: MiningSamplingToken,
    ) {
        // Create and store mining mining_samplings_token
        <MiningSamplingTokens<T>>::insert(
            mining_samplings_token_id,
            mining_samplings_token,
        );
        <MiningSamplingTokenCount<T>>::put(mining_samplings_token_id + One::one());
        <MiningSamplingTokenOwners<T>>::insert(
            mining_samplings_token_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_samplings_token_id: T::MiningSamplingTokenIndex,
    ) {
        <MiningSamplingTokenOwners<T>>::insert(mining_samplings_token_id, to);
    }
}
