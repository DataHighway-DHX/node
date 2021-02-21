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
        Currency,
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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + roaming_operators::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningConfigTokenMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // Mining Speed Boost Token Mining Config
    type MiningConfigTokenMiningTokenType: Parameter + Member + Default;
    type MiningConfigTokenMiningTokenLockAmount: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
}

type BalanceOf<T> =
    <<T as roaming_operators::Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningConfigTokenMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningConfigTokenMiningTokenConfig<U, V, W, X> {
    pub token_type: U,
    pub token_lock_amount: V,
    pub token_lock_start_block: W,
    pub token_lock_interval_blocks: X, // FIXME - why need end date if already have start date and period
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningConfigTokenMiningTokenRequirementsConfig<U, V, W> {
    pub token_type: U,
    pub token_lock_min_amount: V, /* Balance used instead of
                                   * MiningConfigTokenMiningTokenLockMinAmount */
    pub token_lock_min_blocks: W,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningConfigTokenMiningIndex,
        <T as Trait>::MiningConfigTokenMiningTokenType,
        <T as frame_system::Trait>::BlockNumber,
        Balance = BalanceOf<T>,
    {
        /// A mining_config_token_mining is created. (owner, mining_config_token_mining_id)
        Created(AccountId, MiningConfigTokenMiningIndex),
        /// A mining_config_token_mining is transferred. (from, to, mining_config_token_mining_id)
        Transferred(AccountId, AccountId, MiningConfigTokenMiningIndex),
        MiningConfigTokenMiningTokenConfigSet(
            AccountId, MiningConfigTokenMiningIndex, MiningConfigTokenMiningTokenType, Balance, BlockNumber, BlockNumber
        ),
        MiningConfigTokenMiningTokenRequirementsConfigSet(
            AccountId, MiningConfigTokenMiningIndex, MiningConfigTokenMiningTokenType, Balance,
            BlockNumber
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningConfigTokenMining {
        /// Stores all the mining_config_token_minings, key is the mining_config_token_mining id / index
        pub MiningConfigTokenMinings get(fn mining_config_token_mining): map hasher(opaque_blake2_256) T::MiningConfigTokenMiningIndex => Option<MiningConfigTokenMining>;

        /// Stores the total number of mining_config_token_minings. i.e. the next mining_config_token_mining index
        pub MiningConfigTokenMiningCount get(fn mining_config_token_mining_count): T::MiningConfigTokenMiningIndex;

        /// Stores mining_config_token_mining owner
        pub MiningConfigTokenMiningOwners get(fn mining_config_token_mining_owner): map hasher(opaque_blake2_256) T::MiningConfigTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_config_token_mining_token_config
        pub MiningConfigTokenMiningTokenConfigs get(fn mining_config_token_mining_token_configs): map hasher(opaque_blake2_256) T::MiningConfigTokenMiningIndex =>
            Option<MiningConfigTokenMiningTokenConfig<T::MiningConfigTokenMiningTokenType, BalanceOf<T>, T::BlockNumber, T::BlockNumber>>;

        /// Stores mining_config_token_mining_token_cooldown_config
        pub MiningConfigTokenMiningTokenRequirementsConfigs get(fn mining_config_token_mining_token_cooldown_configs): map hasher(opaque_blake2_256) T::MiningConfigTokenMiningIndex =>
            Option<MiningConfigTokenMiningTokenRequirementsConfig<T::MiningConfigTokenMiningTokenType, BalanceOf<T>, T::BlockNumber>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_config_token_mining
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_config_token_mining_id = Self::next_mining_config_token_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_config_token_mining
            let mining_config_token_mining = MiningConfigTokenMining(unique_id);
            Self::insert_mining_config_token_mining(&sender, mining_config_token_mining_id, mining_config_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_config_token_mining_id));
        }

        /// Transfer a mining_config_token_mining to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_config_token_mining_id: T::MiningConfigTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_config_token_mining_owner(mining_config_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_config_token_mining");

            Self::update_owner(&to, mining_config_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_config_token_mining_id));
        }

        /// Set mining_config_token_mining_token_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_config_token_mining_token_config(
            origin,
            mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
            _token_type: Option<T::MiningConfigTokenMiningTokenType>,
            _token_lock_amount: Option<BalanceOf<T>>,
            _token_lock_start_block: Option<T::BlockNumber>,
            _token_lock_interval_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_config_token_mining_id whose config we want to change actually exists
            let is_mining_config_token_mining = Self::exists_mining_config_token_mining(mining_config_token_mining_id).is_ok();
            ensure!(is_mining_config_token_mining, "MiningConfigTokenMining does not exist");

            // Ensure that the caller is owner of the mining_config_token_mining_token_config they are trying to change
            ensure!(Self::mining_config_token_mining_owner(mining_config_token_mining_id) == Some(sender.clone()), "Only owner can set mining_config_token_mining_token_config");

            let mut default_token_type = Default::default();
            let mut default_token_lock_min_amount = Default::default();
            let mut default_token_lock_min_blocks = Default::default();
            let mut fetched_mining_config_token_mining_token_cooldown_config = <MiningConfigTokenMiningTokenRequirementsConfigs<T>>::get(mining_config_token_mining_id);
            if let Some(_mining_config_token_mining_token_cooldown_config) = fetched_mining_config_token_mining_token_cooldown_config {
                default_token_type = _mining_config_token_mining_token_cooldown_config.token_type;
                default_token_lock_min_amount = _mining_config_token_mining_token_cooldown_config.token_lock_min_amount;
                default_token_lock_min_blocks = _mining_config_token_mining_token_cooldown_config.token_lock_min_blocks;
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
                None => <frame_system::Module<T>>::block_number()
            };
            let token_lock_interval_blocks = match _token_lock_interval_blocks {
                Some(value) => value,
                None => default_token_lock_min_blocks
            };

            // Check if a mining_config_token_mining_token_config already exists with the given mining_config_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_config_token_mining_token_config_index(mining_config_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningConfigTokenMiningTokenConfigs<T>>::mutate(mining_config_token_mining_id, |mining_config_token_mining_token_config| {
                    if let Some(_mining_config_token_mining_token_config) = mining_config_token_mining_token_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_config_token_mining_token_config.token_type = token_type.clone();
                        _mining_config_token_mining_token_config.token_lock_amount = token_lock_amount.clone();
                        _mining_config_token_mining_token_config.token_lock_start_block = token_lock_start_block.clone();
                        _mining_config_token_mining_token_config.token_lock_interval_blocks = token_lock_interval_blocks.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_config_token_mining_token_config = <MiningConfigTokenMiningTokenConfigs<T>>::get(mining_config_token_mining_id);
                if let Some(_mining_config_token_mining_token_config) = fetched_mining_config_token_mining_token_config {
                    debug::info!("Latest field token_type {:#?}", _mining_config_token_mining_token_config.token_type);
                    debug::info!("Latest field token_lock_amount {:#?}", _mining_config_token_mining_token_config.token_lock_amount);
                    debug::info!("Latest field token_lock_start_block {:#?}", _mining_config_token_mining_token_config.token_lock_start_block);
                    debug::info!("Latest field token_lock_interval_blocks {:#?}", _mining_config_token_mining_token_config.token_lock_interval_blocks);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_config_token_mining_token_config instance with the input params
                let mining_config_token_mining_token_config_instance = MiningConfigTokenMiningTokenConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_type: token_type.clone(),
                    token_lock_amount: token_lock_amount.clone(),
                    token_lock_start_block: token_lock_start_block.clone(),
                    token_lock_interval_blocks: token_lock_interval_blocks.clone()
                };

                <MiningConfigTokenMiningTokenConfigs<T>>::insert(
                    mining_config_token_mining_id,
                    &mining_config_token_mining_token_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_config_token_mining_token_config = <MiningConfigTokenMiningTokenConfigs<T>>::get(mining_config_token_mining_id);
                if let Some(_mining_config_token_mining_token_config) = fetched_mining_config_token_mining_token_config {
                    debug::info!("Inserted field token_type {:#?}", _mining_config_token_mining_token_config.token_type);
                    debug::info!("Inserted field token_lock_amount {:#?}", _mining_config_token_mining_token_config.token_lock_amount);
                    debug::info!("Inserted field token_lock_start_block {:#?}", _mining_config_token_mining_token_config.token_lock_start_block);
                    debug::info!("Inserted field token_lock_interval_blocks {:#?}", _mining_config_token_mining_token_config.token_lock_interval_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningConfigTokenMiningTokenConfigSet(
                sender,
                mining_config_token_mining_id,
                token_type,
                token_lock_amount,
                token_lock_start_block,
                token_lock_interval_blocks
            ));
        }


        /// Set mining_config_token_mining_token_cooldown_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_config_token_mining_token_cooldown_config(
            origin,
            mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
            _token_type: Option<T::MiningConfigTokenMiningTokenType>,
            _token_lock_min_amount: Option<BalanceOf<T>>,
            _token_lock_min_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_config_token_mining_id whose config we want to change actually exists
            let is_mining_config_token_mining = Self::exists_mining_config_token_mining(mining_config_token_mining_id).is_ok();
            ensure!(is_mining_config_token_mining, "MiningConfigTokenMining does not exist");

            // Ensure that the caller is owner of the mining_config_token_mining_token_config they are trying to change
            ensure!(Self::mining_config_token_mining_owner(mining_config_token_mining_id) == Some(sender.clone()), "Only owner can set mining_config_token_mining_token_cooldown_config");

            let token_type = match _token_type.clone() {
                Some(value) => value,
                None => Default::default() // Default
            };
            let token_lock_min_amount = match _token_lock_min_amount {
                Some(value) => value,
                None => 10.into() // Default
            };
            let token_lock_min_blocks = match _token_lock_min_blocks {
                Some(value) => value,
                None => 7.into() // Default
            };

            // Check if a mining_config_token_mining_token_cooldown_config already exists with the given mining_config_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_config_token_mining_token_cooldown_config_index(mining_config_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningConfigTokenMiningTokenRequirementsConfigs<T>>::mutate(mining_config_token_mining_id, |mining_config_token_mining_token_cooldown_config| {
                    if let Some(_mining_config_token_mining_token_cooldown_config) = mining_config_token_mining_token_cooldown_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_config_token_mining_token_cooldown_config.token_type = token_type.clone();
                        _mining_config_token_mining_token_cooldown_config.token_lock_min_amount = token_lock_min_amount.clone();
                        _mining_config_token_mining_token_cooldown_config.token_lock_min_blocks = token_lock_min_blocks.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_config_token_mining_token_cooldown_config = <MiningConfigTokenMiningTokenRequirementsConfigs<T>>::get(mining_config_token_mining_id);
                if let Some(_mining_config_token_mining_token_cooldown_config) = fetched_mining_config_token_mining_token_cooldown_config {
                    debug::info!("Latest field token_type {:#?}", _mining_config_token_mining_token_cooldown_config.token_type);
                    debug::info!("Latest field token_lock_min_amount {:#?}", _mining_config_token_mining_token_cooldown_config.token_lock_min_amount);
                    debug::info!("Latest field token_lock_min_blocks {:#?}", _mining_config_token_mining_token_cooldown_config.token_lock_min_blocks);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_config_token_mining_token_cooldown_config instance with the input params
                let mining_config_token_mining_token_cooldown_config_instance = MiningConfigTokenMiningTokenRequirementsConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_type: token_type.clone(),
                    token_lock_min_amount: token_lock_min_amount.clone(),
                    token_lock_min_blocks: token_lock_min_blocks.clone(),
                };

                <MiningConfigTokenMiningTokenRequirementsConfigs<T>>::insert(
                    mining_config_token_mining_id,
                    &mining_config_token_mining_token_cooldown_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_config_token_mining_token_cooldown_config = <MiningConfigTokenMiningTokenRequirementsConfigs<T>>::get(mining_config_token_mining_id);
                if let Some(_mining_config_token_mining_token_cooldown_config) = fetched_mining_config_token_mining_token_cooldown_config {
                    debug::info!("Inserted field token_type {:#?}", _mining_config_token_mining_token_cooldown_config.token_type);
                    debug::info!("Inserted field token_lock_min_amount {:#?}", _mining_config_token_mining_token_cooldown_config.token_lock_min_amount);
                    debug::info!("Inserted field token_lock_min_blocks {:#?}", _mining_config_token_mining_token_cooldown_config.token_lock_min_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningConfigTokenMiningTokenRequirementsConfigSet(
                sender,
                mining_config_token_mining_id,
                token_type,
                token_lock_min_amount,
                token_lock_min_blocks,
            ));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_config_token_mining_owner(
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_config_token_mining_owner(
                &mining_config_token_mining_id
            )
            .map(|owner| owner == sender)
            .unwrap_or(false),
            "Sender is not owner of Mining"
        );
        Ok(())
    }

    pub fn exists_mining_config_token_mining(
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
    ) -> Result<MiningConfigTokenMining, DispatchError> {
        match Self::mining_config_token_mining(mining_config_token_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningConfigTokenMining does not exist")),
        }
    }

    pub fn exists_mining_config_token_mining_token_config(
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_config_token_mining_token_configs(
            mining_config_token_mining_id,
        ) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningConfigTokenMiningTokenConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_config_token_mining_token_config_index(
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_config_token_mining_token_config has a value that is defined"
        );
        let fetched_mining_config_token_mining_token_config =
            <MiningConfigTokenMiningTokenConfigs<T>>::get(
                mining_config_token_mining_id,
            );
        if let Some(_value) = fetched_mining_config_token_mining_token_config {
            debug::info!("Found value for mining_config_token_mining_token_config");
            return Ok(());
        }
        debug::info!("No value for mining_config_token_mining_token_config");
        Err(DispatchError::Other("No value for mining_config_token_mining_token_config"))
    }

    pub fn has_value_for_mining_config_token_mining_token_cooldown_config_index(
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_config_token_mining_token_cooldown_config has a value that is \
             defined"
        );
        let fetched_mining_config_token_mining_token_cooldown_config =
            <MiningConfigTokenMiningTokenRequirementsConfigs<T>>::get(
                mining_config_token_mining_id,
            );
        if let Some(_value) = fetched_mining_config_token_mining_token_cooldown_config {
            debug::info!("Found value for mining_config_token_mining_token_cooldown_config");
            return Ok(());
        }
        debug::info!("No value for mining_config_token_mining_token_cooldown_config");
        Err(DispatchError::Other("No value for mining_config_token_mining_token_cooldown_config"))
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

    fn next_mining_config_token_mining_id()
    -> Result<T::MiningConfigTokenMiningIndex, DispatchError> {
        let mining_config_token_mining_id =
            Self::mining_config_token_mining_count();
        if mining_config_token_mining_id ==
            <T::MiningConfigTokenMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningConfigTokenMining count overflow"));
        }
        Ok(mining_config_token_mining_id)
    }

    fn insert_mining_config_token_mining(
        owner: &T::AccountId,
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
        mining_config_token_mining: MiningConfigTokenMining,
    ) {
        // Create and store mining mining_config_token_mining
        <MiningConfigTokenMinings<T>>::insert(
            mining_config_token_mining_id,
            mining_config_token_mining,
        );
        <MiningConfigTokenMiningCount<T>>::put(
            mining_config_token_mining_id + One::one(),
        );
        <MiningConfigTokenMiningOwners<T>>::insert(
            mining_config_token_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
    ) {
        <MiningConfigTokenMiningOwners<T>>::insert(
            mining_config_token_mining_id,
            to,
        );
    }
}
