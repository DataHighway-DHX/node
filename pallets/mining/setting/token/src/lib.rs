#![cfg_attr(not(feature = "std"), no_std)]

use log::{warn, info};
use codec::{
    Decode,
    Encode,
};
use frame_support::{
    decl_event,
    decl_module,
    decl_storage,
    ensure,
    traits::{
        Currency,
        Get,
        Randomness,
    },
    Parameter,
};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config: frame_system::Config + roaming_operators::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningSettingTokenIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // Mining Speed Boost Token Mining Config
    type MiningSettingTokenType: Parameter + Member + Default;
    type MiningSettingTokenLockAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> =
    <<T as roaming_operators::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSettingToken(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct MiningSettingTokenSetting<U, V, W, X> {
    pub token_type: U,
    pub token_lock_amount: V,
    pub token_lock_start_block: W,
    pub token_lock_interval_blocks: X, // FIXME - why need end date if already have start date and period
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct MiningSettingTokenRequirementsSetting<U, V, W> {
    pub token_type: U,
    pub token_lock_min_amount: V, /* Balance used instead of
                                   * MiningSettingTokenTokenLockMinAmount */
    pub token_lock_min_blocks: W,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::MiningSettingTokenIndex,
        <T as Config>::MiningSettingTokenType,
        <T as frame_system::Config>::BlockNumber,
        Balance = BalanceOf<T>,
    {
        /// A mining_setting_token is created. (owner, mining_setting_token_id)
        Created(AccountId, MiningSettingTokenIndex),
        /// A mining_setting_token is transferred. (from, to, mining_setting_token_id)
        Transferred(AccountId, AccountId, MiningSettingTokenIndex),
        MiningSettingTokenSettingSet(
            AccountId, MiningSettingTokenIndex, MiningSettingTokenType, Balance, BlockNumber, BlockNumber
        ),
        MiningSettingTokenRequirementsSettingSet(
            AccountId, MiningSettingTokenIndex, MiningSettingTokenType, Balance,
            BlockNumber
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningSettingToken {
        /// Stores all the mining_setting_tokens, key is the mining_setting_token id / index
        pub MiningSettingTokens get(fn mining_setting_token): map hasher(opaque_blake2_256) T::MiningSettingTokenIndex => Option<MiningSettingToken>;

        /// Stores the total number of mining_setting_tokens. i.e. the next mining_setting_token index
        pub MiningSettingTokenCount get(fn mining_setting_token_count): T::MiningSettingTokenIndex;

        /// Stores mining_setting_token owner
        pub MiningSettingTokenOwners get(fn mining_setting_token_owner): map hasher(opaque_blake2_256) T::MiningSettingTokenIndex => Option<T::AccountId>;

        /// Stores mining_setting_token_token_setting
        pub MiningSettingTokenSettings get(fn mining_setting_token_token_settings): map hasher(opaque_blake2_256) T::MiningSettingTokenIndex =>
            Option<MiningSettingTokenSetting<T::MiningSettingTokenType, BalanceOf<T>, T::BlockNumber, T::BlockNumber>>;

        /// Stores mining_setting_token_token_cooldown_config
        pub MiningSettingTokenRequirementsSettings get(fn mining_setting_token_token_cooldown_configs): map hasher(opaque_blake2_256) T::MiningSettingTokenIndex =>
            Option<MiningSettingTokenRequirementsSetting<T::MiningSettingTokenType, BalanceOf<T>, T::BlockNumber>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_setting_token
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_setting_token_id = Self::next_mining_setting_token_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_setting_token
            let mining_setting_token = MiningSettingToken(unique_id);
            Self::insert_mining_setting_token(&sender, mining_setting_token_id, mining_setting_token);

            Self::deposit_event(RawEvent::Created(sender, mining_setting_token_id));
        }

        /// Transfer a mining_setting_token to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_setting_token_id: T::MiningSettingTokenIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_setting_token_owner(mining_setting_token_id) == Some(sender.clone()), "Only owner can transfer mining mining_setting_token");

            Self::update_owner(&to, mining_setting_token_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_setting_token_id));
        }

        /// Set mining_setting_token_token_setting
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_setting_token_token_setting(
            origin,
            mining_setting_token_id: T::MiningSettingTokenIndex,
            _token_type: Option<T::MiningSettingTokenType>,
            _token_lock_amount: Option<BalanceOf<T>>,
            _token_lock_start_block: Option<T::BlockNumber>,
            _token_lock_interval_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_setting_token_id whose config we want to change actually exists
            let is_mining_setting_token = Self::exists_mining_setting_token(mining_setting_token_id).is_ok();
            ensure!(is_mining_setting_token, "MiningSettingToken does not exist");

            // Ensure that the caller is owner of the mining_setting_token_token_setting they are trying to change
            ensure!(Self::mining_setting_token_owner(mining_setting_token_id) == Some(sender.clone()), "Only owner can set mining_setting_token_token_setting");

            let mut default_token_type = Default::default();
            let mut default_token_lock_min_amount = Default::default();
            let mut default_token_lock_min_blocks = Default::default();
            let mut fetched_mining_setting_token_token_cooldown_config = <MiningSettingTokenRequirementsSettings<T>>::get(mining_setting_token_id);
            if let Some(_mining_setting_token_token_cooldown_config) = fetched_mining_setting_token_token_cooldown_config {
                default_token_type = _mining_setting_token_token_cooldown_config.token_type;
                default_token_lock_min_amount = _mining_setting_token_token_cooldown_config.token_lock_min_amount;
                default_token_lock_min_blocks = _mining_setting_token_token_cooldown_config.token_lock_min_blocks;
            }

            let token_type = match _token_type.clone() {
                Some(value) => value,
                None => default_token_type
            };
            let token_lock_amount = match _token_lock_amount {
                Some(value) => value,
                None => default_token_lock_min_amount
            };
            let token_lock_start_block = match _token_lock_start_block {
                Some(value) => value,
                None => <frame_system::Pallet<T>>::block_number()
            };
            let token_lock_interval_blocks = match _token_lock_interval_blocks {
                Some(value) => value,
                None => default_token_lock_min_blocks
            };

            // Check if a mining_setting_token_token_setting already exists with the given mining_setting_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_setting_token_token_setting_index(mining_setting_token_id).is_ok() {
                info!("Mutating values");
                <MiningSettingTokenSettings<T>>::mutate(mining_setting_token_id, |mining_setting_token_token_setting| {
                    if let Some(_mining_setting_token_token_setting) = mining_setting_token_token_setting {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_setting_token_token_setting.token_type = token_type.clone();
                        _mining_setting_token_token_setting.token_lock_amount = token_lock_amount.clone();
                        _mining_setting_token_token_setting.token_lock_start_block = token_lock_start_block.clone();
                        _mining_setting_token_token_setting.token_lock_interval_blocks = token_lock_interval_blocks.clone();
                    }
                });
                info!("Checking mutated values");
                let fetched_mining_setting_token_token_setting = <MiningSettingTokenSettings<T>>::get(mining_setting_token_id);
                if let Some(_mining_setting_token_token_setting) = fetched_mining_setting_token_token_setting {
                    info!("Latest field token_type {:#?}", _mining_setting_token_token_setting.token_type);
                    info!("Latest field token_lock_amount {:#?}", _mining_setting_token_token_setting.token_lock_amount);
                    info!("Latest field token_lock_start_block {:#?}", _mining_setting_token_token_setting.token_lock_start_block);
                    info!("Latest field token_lock_interval_blocks {:#?}", _mining_setting_token_token_setting.token_lock_interval_blocks);
                }
            } else {
                info!("Inserting values");

                // Create a new mining mining_setting_token_token_setting instance with the input params
                let mining_setting_token_token_setting_instance = MiningSettingTokenSetting {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_type: token_type.clone(),
                    token_lock_amount: token_lock_amount.clone(),
                    token_lock_start_block: token_lock_start_block.clone(),
                    token_lock_interval_blocks: token_lock_interval_blocks.clone()
                };

                <MiningSettingTokenSettings<T>>::insert(
                    mining_setting_token_id,
                    &mining_setting_token_token_setting_instance
                );

                info!("Checking inserted values");
                let fetched_mining_setting_token_token_setting = <MiningSettingTokenSettings<T>>::get(mining_setting_token_id);
                if let Some(_mining_setting_token_token_setting) = fetched_mining_setting_token_token_setting {
                    info!("Inserted field token_type {:#?}", _mining_setting_token_token_setting.token_type);
                    info!("Inserted field token_lock_amount {:#?}", _mining_setting_token_token_setting.token_lock_amount);
                    info!("Inserted field token_lock_start_block {:#?}", _mining_setting_token_token_setting.token_lock_start_block);
                    info!("Inserted field token_lock_interval_blocks {:#?}", _mining_setting_token_token_setting.token_lock_interval_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningSettingTokenSettingSet(
                sender,
                mining_setting_token_id,
                token_type,
                token_lock_amount,
                token_lock_start_block,
                token_lock_interval_blocks
            ));
        }


        /// Set mining_setting_token_token_cooldown_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_setting_token_token_cooldown_config(
            origin,
            mining_setting_token_id: T::MiningSettingTokenIndex,
            _token_type: Option<T::MiningSettingTokenType>,
            _token_lock_min_amount: Option<BalanceOf<T>>,
            _token_lock_min_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_setting_token_id whose config we want to change actually exists
            let is_mining_setting_token = Self::exists_mining_setting_token(mining_setting_token_id).is_ok();
            ensure!(is_mining_setting_token, "MiningSettingToken does not exist");

            // Ensure that the caller is owner of the mining_setting_token_token_setting they are trying to change
            ensure!(Self::mining_setting_token_owner(mining_setting_token_id) == Some(sender.clone()), "Only owner can set mining_setting_token_token_cooldown_config");

            let token_type = match _token_type.clone() {
                Some(value) => value,
                None => Default::default() // Default
            };
            let token_lock_min_amount = match _token_lock_min_amount {
                Some(value) => value,
                None => 10u32.into() // Default
            };
            let token_lock_min_blocks = match _token_lock_min_blocks {
                Some(value) => value,
                None => 7u32.into() // Default
            };

            // Check if a mining_setting_token_token_cooldown_config already exists with the given mining_setting_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_setting_token_token_cooldown_config_index(mining_setting_token_id).is_ok() {
                info!("Mutating values");
                <MiningSettingTokenRequirementsSettings<T>>::mutate(mining_setting_token_id, |mining_setting_token_token_cooldown_config| {
                    if let Some(_mining_setting_token_token_cooldown_config) = mining_setting_token_token_cooldown_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_setting_token_token_cooldown_config.token_type = token_type.clone();
                        _mining_setting_token_token_cooldown_config.token_lock_min_amount = token_lock_min_amount.clone();
                        _mining_setting_token_token_cooldown_config.token_lock_min_blocks = token_lock_min_blocks.clone();
                    }
                });
                info!("Checking mutated values");
                let fetched_mining_setting_token_token_cooldown_config = <MiningSettingTokenRequirementsSettings<T>>::get(mining_setting_token_id);
                if let Some(_mining_setting_token_token_cooldown_config) = fetched_mining_setting_token_token_cooldown_config {
                    info!("Latest field token_type {:#?}", _mining_setting_token_token_cooldown_config.token_type);
                    info!("Latest field token_lock_min_amount {:#?}", _mining_setting_token_token_cooldown_config.token_lock_min_amount);
                    info!("Latest field token_lock_min_blocks {:#?}", _mining_setting_token_token_cooldown_config.token_lock_min_blocks);
                }
            } else {
                info!("Inserting values");

                // Create a new mining mining_setting_token_token_cooldown_config instance with the input params
                let mining_setting_token_token_cooldown_config_instance = MiningSettingTokenRequirementsSetting {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_type: token_type.clone(),
                    token_lock_min_amount: token_lock_min_amount.clone(),
                    token_lock_min_blocks: token_lock_min_blocks.clone(),
                };

                <MiningSettingTokenRequirementsSettings<T>>::insert(
                    mining_setting_token_id,
                    &mining_setting_token_token_cooldown_config_instance
                );

                info!("Checking inserted values");
                let fetched_mining_setting_token_token_cooldown_config = <MiningSettingTokenRequirementsSettings<T>>::get(mining_setting_token_id);
                if let Some(_mining_setting_token_token_cooldown_config) = fetched_mining_setting_token_token_cooldown_config {
                    info!("Inserted field token_type {:#?}", _mining_setting_token_token_cooldown_config.token_type);
                    info!("Inserted field token_lock_min_amount {:#?}", _mining_setting_token_token_cooldown_config.token_lock_min_amount);
                    info!("Inserted field token_lock_min_blocks {:#?}", _mining_setting_token_token_cooldown_config.token_lock_min_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningSettingTokenRequirementsSettingSet(
                sender,
                mining_setting_token_id,
                token_type,
                token_lock_min_amount,
                token_lock_min_blocks,
            ));
        }
    }
}

impl<T: Config> Module<T> {
    pub fn is_mining_setting_token_owner(
        mining_setting_token_id: T::MiningSettingTokenIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_setting_token_owner(&mining_setting_token_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of Mining"
        );
        Ok(())
    }

    pub fn exists_mining_setting_token(
        mining_setting_token_id: T::MiningSettingTokenIndex,
    ) -> Result<MiningSettingToken, DispatchError> {
        match Self::mining_setting_token(mining_setting_token_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSettingToken does not exist")),
        }
    }

    pub fn exists_mining_setting_token_token_setting(
        mining_setting_token_id: T::MiningSettingTokenIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_setting_token_token_settings(mining_setting_token_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningSettingTokenSetting does not exist")),
        }
    }

    pub fn has_value_for_mining_setting_token_token_setting_index(
        mining_setting_token_id: T::MiningSettingTokenIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if mining_setting_token_token_setting has a value that is defined");
        let fetched_mining_setting_token_token_setting = <MiningSettingTokenSettings<T>>::get(mining_setting_token_id);
        if let Some(_value) = fetched_mining_setting_token_token_setting {
            info!("Found value for mining_setting_token_token_setting");
            return Ok(());
        }
        warn!("No value for mining_setting_token_token_setting");
        Err(DispatchError::Other("No value for mining_setting_token_token_setting"))
    }

    pub fn has_value_for_mining_setting_token_token_cooldown_config_index(
        mining_setting_token_id: T::MiningSettingTokenIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if mining_setting_token_token_cooldown_config has a value that is defined");
        let fetched_mining_setting_token_token_cooldown_config =
            <MiningSettingTokenRequirementsSettings<T>>::get(mining_setting_token_id);
        if let Some(_value) = fetched_mining_setting_token_token_cooldown_config {
            info!("Found value for mining_setting_token_token_cooldown_config");
            return Ok(());
        }
        warn!("No value for mining_setting_token_token_cooldown_config");
        Err(DispatchError::Other("No value for mining_setting_token_token_cooldown_config"))
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <frame_system::Pallet<T>>::extrinsic_index(),
            <frame_system::Pallet<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_mining_setting_token_id() -> Result<T::MiningSettingTokenIndex, DispatchError> {
        let mining_setting_token_id = Self::mining_setting_token_count();
        if mining_setting_token_id == <T::MiningSettingTokenIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningSettingToken count overflow"));
        }
        Ok(mining_setting_token_id)
    }

    fn insert_mining_setting_token(
        owner: &T::AccountId,
        mining_setting_token_id: T::MiningSettingTokenIndex,
        mining_setting_token: MiningSettingToken,
    ) {
        // Create and store mining mining_setting_token
        <MiningSettingTokens<T>>::insert(mining_setting_token_id, mining_setting_token);
        <MiningSettingTokenCount<T>>::put(mining_setting_token_id + One::one());
        <MiningSettingTokenOwners<T>>::insert(mining_setting_token_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_setting_token_id: T::MiningSettingTokenIndex) {
        <MiningSettingTokenOwners<T>>::insert(mining_setting_token_id, to);
    }
}
