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
        Zero,
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
pub trait Trait: frame_system::Trait + roaming_operators::Trait + mining_rates_token::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningConfigTokenIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // Mining Speed Boost Token Mining Config
    type MiningConfigTokenType: Parameter + Member + Default;
    type MiningConfigTokenLockAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type Currency: Currency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningConfigToken(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningConfigTokenConfig<U, V, W, X> {
    pub token_type: U,
    pub token_lock_amount: V,
    pub token_lock_start_block: W,
    pub token_lock_interval_blocks: X, // FIXME - why need end date if already have start date and period
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningConfigTokenRequirementsConfig<U, V, W> {
    pub token_type: U,
    pub token_lock_min_amount: V, /* Balance used instead of
                                   * MiningConfigTokenTokenLockMinAmount */
    pub token_lock_min_blocks: W,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningConfigTokenExecutionResult<U, V, W> {
    pub token_execution_executor_account_id: U,
    pub token_execution_started_block: V,
    pub token_execution_interval_blocks: W,
}

decl_event!(
    pub enum Event<T> where
        AccountId = <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningConfigTokenIndex,
        <T as Trait>::MiningConfigTokenType,
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
        Balance = BalanceOf<T>,
    {
        /// A mining_config_token is created. (owner, mining_config_token_id)
        Created(AccountId, MiningConfigTokenIndex),
        /// A mining_config_token is transferred. (from, to, mining_config_token_id)
        Transferred(AccountId, AccountId, MiningConfigTokenIndex),
        MiningConfigTokenConfigSet(
            AccountId, MiningConfigTokenIndex, MiningConfigTokenType, Balance, BlockNumber, BlockNumber
        ),
        MiningConfigTokenRequirementsConfigSet(
            AccountId, MiningConfigTokenIndex, MiningConfigTokenType, Balance,
            BlockNumber
        ),
        MiningConfigTokenExecutionResultSet(
            AccountId, MiningConfigTokenIndex,AccountId, BlockNumber, BlockNumber
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningConfigToken {
        /// Stores all the mining_config_tokens, key is the mining_config_token id / index
        pub MiningConfigTokens get(fn mining_config_token): map hasher(opaque_blake2_256) T::MiningConfigTokenIndex => Option<MiningConfigToken>;

        /// Stores the total number of mining_config_tokens. i.e. the next mining_config_token index
        pub MiningConfigTokenCount get(fn mining_config_token_count): T::MiningConfigTokenIndex;

        /// Stores mining_config_token owner
        pub MiningConfigTokenOwners get(fn mining_config_token_owner): map hasher(opaque_blake2_256) T::MiningConfigTokenIndex => Option<T::AccountId>;

        /// Stores mining_config_token_token_config
        pub MiningConfigTokenConfigs get(fn mining_config_token_configs): map hasher(opaque_blake2_256) T::MiningConfigTokenIndex =>
            Option<MiningConfigTokenConfig<T::MiningConfigTokenType, BalanceOf<T>, T::BlockNumber, T::BlockNumber>>;

        /// Stores mining_config_token_token_cooldown_config
        pub MiningConfigTokenRequirementsConfigs get(fn mining_config_token_cooldown_configs): map hasher(opaque_blake2_256) T::MiningConfigTokenIndex =>
            Option<MiningConfigTokenRequirementsConfig<T::MiningConfigTokenType, BalanceOf<T>, T::BlockNumber>>;

        /// Stores mining_config_token_execution_result
        pub MiningConfigTokenExecutionResults get(fn mining_config_token_execution_results): map hasher(opaque_blake2_256) T::MiningConfigTokenIndex =>
            Option<MiningConfigTokenExecutionResult<
                T::AccountId,
                T::BlockNumber,
                T::BlockNumber
            >>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        // TODO - automatically checks through all the accounts that have
        // successfully been locked, whether it is the end of their cooldown period and if so sample the balance, to
        // determine their elegibility, and perform the lodgement for reward and unlock their tokens
        fn on_finalize(current_block_number: T::BlockNumber) {
            debug::info!("execution/token-mining - on_finalize");
            debug::info!("current block number {:#?}", current_block_number);

            let config_token_count = Self::mining_config_token_count();

            // FIXME - is there an upper bound on the size of these sets and
            // the computation of this nested loop? what max size or custom weight function?
            // See https://substrate.dev/recipes/map-set.html
            //
            // Loop through all mining_config_token_id
            for idx_c in 0..config_token_count.into() {
                let fetched_mining_execution_token_result = <MiningConfigTokenExecutionResults<T>>::get(idx_c);

                if let Some(_mining_execution_token_result) = fetched_mining_execution_token_result {
                    debug::info!("token_execution_executor_account_id {:#?}", _mining_execution_token_result.token_execution_executor_account_id);
                    debug::info!("token_execution_started_block {:#?}", _mining_execution_token_result.token_execution_started_block);
                    debug::info!("token_execution_interval_blocks {:#?}", _mining_execution_token_result.token_execution_interval_blocks);

                    let fetched_mining_config_token_cooldown_config = Self::mining_config_token_cooldown_configs(idx_c);
                    if let Some(_mining_config_token_cooldown_config) = fetched_mining_config_token_cooldown_config {
                        // debug::info!("token_type {:#?}", _mining_config_token_cooldown_config.token_type);
                        debug::info!("token_lock_min_blocks {:#?}", _mining_config_token_cooldown_config.token_lock_min_blocks);

                        if let token_lock_min_blocks = _mining_config_token_cooldown_config.token_lock_min_blocks {
                            if let Some(configuration_token) = Self::mining_config_token_configs((idx_c)) {
                                if let token_lock_amount = configuration_token.token_lock_amount {

                                    // FIXME - remove hard-coded and integrate
                                    // const MINING_REQUESTED_END_BLOCK = One::one() * 10;

                                    // If the end of the mining period has been reached, then stop giving them rewards,
                                    // and unlock their bonded tokens after waiting the cooldown period of _mining_config_token_cooldown_config.token_lock_min_blocks
                                    // after the end of their mining period.

                                    if <frame_system::Module<T>>::block_number() >= _mining_execution_token_result.token_execution_interval_blocks {
                                        if <frame_system::Module<T>>::block_number() > _mining_execution_token_result.token_execution_interval_blocks + token_lock_min_blocks {
                                        // if <frame_system::Module<T>>::block_number() > MINING_REQUESTED_END_BLOCK + token_lock_min_blocks {
                                            // TODO - Unlock the funds. Store updated status
                                            // We only want to unlock the rewards they have earned.
                                            // The amount of tokens that they originally locked to mine the rewards will remain locked until the end of their mining period
                                            // (FIXME - or until they request to stop mining, which hasn't been implemented yet),
                                            // and then they cannot move those locked tokens for the cooldown period and receive no further rewards.

                                            if let Some(mining_config_token) = Self::mining_config_token(idx_c) {
                                                // <T as Trait>::Currency::remove_lock(
                                                //     mining_config_token, // where idx_c is mining_config_token_id
                                                //     &_mining_execution_token_result.token_execution_executor_account_id,
                                                // );
                                            }
                                        }
                                    } else if <frame_system::Module<T>>::block_number() < _mining_execution_token_result.token_execution_interval_blocks {
                                        // Check if cooldown period has been reached before start distributing rewards.
                                        // If so then we unlock and transfer the reward tokens to the user from the treasury.
                                        // Reference: https://github.com/hicommonwealth/edgeware-node/blob/master/modules/edge-treasury-reward/src/lib.rs#L42
                                        if <frame_system::Module<T>>::block_number() % token_lock_min_blocks == Zero::zero() {
                                            // FIXME - assumes there is only one rates config index so hard-coded 0, but we could have many
                                            let fetched_mining_rates_token_rates_config = <mining_rates_token::Module<T>>::mining_rates_token_rates_configs(0.into());
                                            if let Some(_mining_rates_token_rates_config) = fetched_mining_rates_token_rates_config {
                                                debug::info!("token_execution_interval_blocks {:#?}", _mining_execution_token_result.token_execution_interval_blocks);

                                                // TODO - choose the token rate that corresponds to the _mining_config_token_cooldown_config.token_type
                                                // and use this to determine the reward ratio.
                                                // in the meantime until this is fixed, we will just assume the user is mining MXC and choose the rate for that

                                                // Reward ratio
                                                let reward_ratio = _mining_rates_token_rates_config.token_token_mxc;

                                                // Calculate the reward based on the reward ratio (i.e. 1 DHX per 10 DHX that was locked)
                                                // e.g. (1.1 - 1) * 10 DHX, where 1.1 is ratio of mining reward for the MXC token

                                                // let reward = (reward_ratio - 1.into()) * token_lock_amount;

                                                // Distribute the reward to the account that has locked the funds

                                                // <T as Trait>::Currency::transfer(
                                                //     &<pallet_treasury::Module<T>>::account_id(),
                                                //     &_mining_execution_token_result.token_execution_executor_account_id,
                                                //     reward,
                                                //     ExistenceRequirement::KeepAlive
                                                // );

                                                // Emit event since treasury unlocked locked tokens and rewarded customer the reward ratio

                                                // Self::deposit_event(RawEvent::TreasuryRewardTokenMiningPostCooldown(
                                                //     <pallet_balances::Module<T>>::free_balance(_mining_execution_token_result.token_execution_executor_account_id),
                                                //     <frame_system::Module<T>>::block_number(),
                                                //     _mining_execution_token_result.token_execution_executor_account_id
                                                // ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        /// Create a new mining mining_config_token
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_config_token_id = Self::next_mining_config_token_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_config_token
            let mining_config_token = MiningConfigToken(unique_id);
            Self::insert_mining_config_token(&sender, mining_config_token_id, mining_config_token);

            Self::deposit_event(RawEvent::Created(sender, mining_config_token_id));
        }

        /// Transfer a mining_config_token to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_config_token_id: T::MiningConfigTokenIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_config_token_owner(mining_config_token_id) == Some(sender.clone()), "Only owner can transfer mining mining_config_token");

            Self::update_owner(&to, mining_config_token_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_config_token_id));
        }

        /// Set mining_config_token_token_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_config_token_config(
            origin,
            mining_config_token_id: T::MiningConfigTokenIndex,
            _token_type: Option<T::MiningConfigTokenType>,
            _token_lock_amount: Option<BalanceOf<T>>,
            _token_lock_start_block: Option<T::BlockNumber>,
            _token_lock_interval_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_config_token_id whose config we want to change actually exists
            let is_mining_config_token = Self::exists_mining_config_token(mining_config_token_id).is_ok();
            ensure!(is_mining_config_token, "MiningConfigToken does not exist");

            // Ensure that the caller is owner of the mining_config_token_token_config they are trying to change
            ensure!(Self::mining_config_token_owner(mining_config_token_id) == Some(sender.clone()), "Only owner can set mining_config_token_token_config");

            let mut default_token_type = Default::default();
            let mut default_token_lock_min_amount = Default::default();
            let mut default_token_lock_min_blocks = Default::default();
            let mut fetched_mining_config_token_token_cooldown_config = <MiningConfigTokenRequirementsConfigs<T>>::get(mining_config_token_id);
            if let Some(_mining_config_token_token_cooldown_config) = fetched_mining_config_token_token_cooldown_config {
                default_token_type = _mining_config_token_token_cooldown_config.token_type;
                default_token_lock_min_amount = _mining_config_token_token_cooldown_config.token_lock_min_amount;
                default_token_lock_min_blocks = _mining_config_token_token_cooldown_config.token_lock_min_blocks;
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

            // Check if a mining_config_token_token_config already exists with the given mining_config_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_config_token_token_config_index(mining_config_token_id).is_ok() {
                debug::info!("Mutating values");
                <MiningConfigTokenConfigs<T>>::mutate(mining_config_token_id, |mining_config_token_token_config| {
                    if let Some(_mining_config_token_token_config) = mining_config_token_token_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_config_token_token_config.token_type = token_type.clone();
                        _mining_config_token_token_config.token_lock_amount = token_lock_amount.clone();
                        _mining_config_token_token_config.token_lock_start_block = token_lock_start_block.clone();
                        _mining_config_token_token_config.token_lock_interval_blocks = token_lock_interval_blocks.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_config_token_token_config = <MiningConfigTokenConfigs<T>>::get(mining_config_token_id);
                if let Some(_mining_config_token_token_config) = fetched_mining_config_token_token_config {
                    debug::info!("Latest field token_type {:#?}", _mining_config_token_token_config.token_type);
                    debug::info!("Latest field token_lock_amount {:#?}", _mining_config_token_token_config.token_lock_amount);
                    debug::info!("Latest field token_lock_start_block {:#?}", _mining_config_token_token_config.token_lock_start_block);
                    debug::info!("Latest field token_lock_interval_blocks {:#?}", _mining_config_token_token_config.token_lock_interval_blocks);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_config_token_token_config instance with the input params
                let mining_config_token_token_config_instance = MiningConfigTokenConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_type: token_type.clone(),
                    token_lock_amount: token_lock_amount.clone(),
                    token_lock_start_block: token_lock_start_block.clone(),
                    token_lock_interval_blocks: token_lock_interval_blocks.clone()
                };

                <MiningConfigTokenConfigs<T>>::insert(
                    mining_config_token_id,
                    &mining_config_token_token_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_config_token_token_config = <MiningConfigTokenConfigs<T>>::get(mining_config_token_id);
                if let Some(_mining_config_token_token_config) = fetched_mining_config_token_token_config {
                    debug::info!("Inserted field token_type {:#?}", _mining_config_token_token_config.token_type);
                    debug::info!("Inserted field token_lock_amount {:#?}", _mining_config_token_token_config.token_lock_amount);
                    debug::info!("Inserted field token_lock_start_block {:#?}", _mining_config_token_token_config.token_lock_start_block);
                    debug::info!("Inserted field token_lock_interval_blocks {:#?}", _mining_config_token_token_config.token_lock_interval_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningConfigTokenConfigSet(
                sender,
                mining_config_token_id,
                token_type,
                token_lock_amount,
                token_lock_start_block,
                token_lock_interval_blocks
            ));
        }


        /// Set mining_config_token_token_cooldown_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_config_token_cooldown_config(
            origin,
            mining_config_token_id: T::MiningConfigTokenIndex,
            _token_type: Option<T::MiningConfigTokenType>,
            _token_lock_min_amount: Option<BalanceOf<T>>,
            _token_lock_min_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_config_token_id whose config we want to change actually exists
            let is_mining_config_token = Self::exists_mining_config_token(mining_config_token_id).is_ok();
            ensure!(is_mining_config_token, "MiningConfigToken does not exist");

            // Ensure that the caller is owner of the mining_config_token_token_config they are trying to change
            ensure!(Self::mining_config_token_owner(mining_config_token_id) == Some(sender.clone()), "Only owner can set mining_config_token_token_cooldown_config");

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

            // Check if a mining_config_token_token_cooldown_config already exists with the given mining_config_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_config_token_token_cooldown_config_index(mining_config_token_id).is_ok() {
                debug::info!("Mutating values");
                <MiningConfigTokenRequirementsConfigs<T>>::mutate(mining_config_token_id, |mining_config_token_token_cooldown_config| {
                    if let Some(_mining_config_token_token_cooldown_config) = mining_config_token_token_cooldown_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_config_token_token_cooldown_config.token_type = token_type.clone();
                        _mining_config_token_token_cooldown_config.token_lock_min_amount = token_lock_min_amount.clone();
                        _mining_config_token_token_cooldown_config.token_lock_min_blocks = token_lock_min_blocks.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_config_token_token_cooldown_config = <MiningConfigTokenRequirementsConfigs<T>>::get(mining_config_token_id);
                if let Some(_mining_config_token_token_cooldown_config) = fetched_mining_config_token_token_cooldown_config {
                    debug::info!("Latest field token_type {:#?}", _mining_config_token_token_cooldown_config.token_type);
                    debug::info!("Latest field token_lock_min_amount {:#?}", _mining_config_token_token_cooldown_config.token_lock_min_amount);
                    debug::info!("Latest field token_lock_min_blocks {:#?}", _mining_config_token_token_cooldown_config.token_lock_min_blocks);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_config_token_token_cooldown_config instance with the input params
                let mining_config_token_token_cooldown_config_instance = MiningConfigTokenRequirementsConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_type: token_type.clone(),
                    token_lock_min_amount: token_lock_min_amount.clone(),
                    token_lock_min_blocks: token_lock_min_blocks.clone(),
                };

                <MiningConfigTokenRequirementsConfigs<T>>::insert(
                    mining_config_token_id,
                    &mining_config_token_token_cooldown_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_config_token_token_cooldown_config = <MiningConfigTokenRequirementsConfigs<T>>::get(mining_config_token_id);
                if let Some(_mining_config_token_token_cooldown_config) = fetched_mining_config_token_token_cooldown_config {
                    debug::info!("Inserted field token_type {:#?}", _mining_config_token_token_cooldown_config.token_type);
                    debug::info!("Inserted field token_lock_min_amount {:#?}", _mining_config_token_token_cooldown_config.token_lock_min_amount);
                    debug::info!("Inserted field token_lock_min_blocks {:#?}", _mining_config_token_token_cooldown_config.token_lock_min_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningConfigTokenRequirementsConfigSet(
                sender,
                mining_config_token_id,
                token_type,
                token_lock_min_amount,
                token_lock_min_blocks,
            ));
        }

        /// Set mining_config_token_execution_result
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_config_token_execution_result(
            origin,
            mining_config_token_id: T::MiningConfigTokenIndex,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_config_token_id whose config we want to change actually exists
            let is_mining_config_token = Self::exists_mining_config_token(mining_config_token_id).is_ok();
            ensure!(is_mining_config_token, "MiningConfigToken does not exist");

            // Ensure that the caller is owner of the mining_config_token_token_config they are trying to change
            ensure!(Self::mining_config_token_owner(mining_config_token_id) == Some(sender.clone()), "Only owner can set mining_config_token_token_config");

            // Check that only allow the owner of the configuration that the execution belongs to call this extrinsic to set and execute
            ensure!(
                Self::is_mining_config_token_owner(
                    mining_config_token_id, sender.clone()
                ).is_ok(),
                "Only the configuration_token owner can execute their associated execution"
            );

            // Assign config values to the execution values
            let token_execution_executor_account_id = sender.clone();
            let token_execution_started_block;
            let token_execution_interval_blocks;

            if let Some(configuration_token_config) =
                Self::mining_config_token_configs(mining_config_token_id)
            {
                if let _token_lock_start_block = configuration_token_config.token_lock_start_block {
                    token_execution_started_block = _token_lock_start_block.clone();
                } else {
                    return Err(DispatchError::Other("Cannot find token_lock_start_block associated with the config"));
                }

                if let _token_lock_interval_blocks = configuration_token_config.token_lock_interval_blocks {
                    token_execution_interval_blocks = _token_lock_interval_blocks.clone();
                } else {
                    return Err(DispatchError::Other("Cannot find token_lock_interval_blocks associated with the config"));
                }
            } else {
                return Err(DispatchError::Other("Cannot find token_config associated with the execution"));
            }

            // TODO - we could just use the token_execution_started_block that we queried already instead of calling it again within this function
            // Ensure that the associated token configuration has a token_execution_started_block > current_block
            let is_token_execution_started_block_greater_than_current_block = Self::token_execution_started_block_greater_than_current_block(mining_config_token_id).is_ok();
            ensure!(is_token_execution_started_block_greater_than_current_block, "token execution does not have a token_execution_started_block > current_block");

            // Ensure that the associated token configuration has a token_lock_interval_blocks > token_lock_min_blocks
            let is_token_lock_interval_blocks_greater_than_token_lock_min_blocks = Self::token_lock_interval_blocks_greater_than_token_lock_min_blocks(mining_config_token_id).is_ok();
            ensure!(is_token_lock_interval_blocks_greater_than_token_lock_min_blocks, "token configuration does not have a token_lock_interval_blocks > token_lock_min_blocks");

            // Ensure that the associated token configuration has a token_lock_amount > token_lock_min_amount
            let is_token_lock_amount_greater_than_token_lock_min_amount = Self::token_lock_amount_greater_than_token_lock_min_amount(mining_config_token_id).is_ok();
            ensure!(is_token_lock_amount_greater_than_token_lock_min_amount, "token configuration does not have a token_lock_amount > token_lock_min_amount");

            Self::execution(
                sender.clone(),
                mining_config_token_id,
                token_execution_executor_account_id.clone(),
                token_execution_started_block,
                token_execution_interval_blocks,
            );
            debug::info!("Executed");

            ensure!(Self::execution(
                sender.clone(),
                mining_config_token_id,
                token_execution_executor_account_id.clone(),
                token_execution_started_block,
                token_execution_interval_blocks,
            ).is_ok(), "Cannot execute");

            // Check if a mining_config_token_execution_result already exists with the given mining_config_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_config_token_execution_result_index(mining_config_token_id).is_ok() {
                debug::info!("Mutating values");
                <MiningConfigTokenExecutionResults<T>>::mutate((mining_config_token_id), |mining_config_token_execution_result| {
                    if let Some(_mining_config_token_execution_result) = mining_config_token_execution_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_config_token_execution_result.token_execution_executor_account_id = token_execution_executor_account_id.clone();
                        _mining_config_token_execution_result.token_execution_started_block = token_execution_started_block.clone();
                        _mining_config_token_execution_result.token_execution_interval_blocks = token_execution_interval_blocks.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_config_token_execution_result = <MiningConfigTokenExecutionResults<T>>::get((mining_config_token_id));
                if let Some(_mining_config_token_execution_result) = fetched_mining_config_token_execution_result {
                    debug::info!("Latest field token_execution_executor_account_id {:#?}", _mining_config_token_execution_result.token_execution_executor_account_id);
                    debug::info!("Latest field token_execution_started_block {:#?}", _mining_config_token_execution_result.token_execution_started_block);
                    debug::info!("Latest field token_execution_interval_blocks {:#?}", _mining_config_token_execution_result.token_execution_interval_blocks);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_config_token_execution_result instance with the input params
                let mining_config_token_execution_result_instance = MiningConfigTokenExecutionResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_execution_executor_account_id: token_execution_executor_account_id.clone(),
                    token_execution_started_block: token_execution_started_block.clone(),
                    token_execution_interval_blocks: token_execution_interval_blocks.clone(),
                };

                <MiningConfigTokenExecutionResults<T>>::insert(
                    mining_config_token_id,
                    &mining_config_token_execution_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_config_token_execution_result = <MiningConfigTokenExecutionResults<T>>::get(mining_config_token_id);
                if let Some(_mining_config_token_execution_result) = fetched_mining_config_token_execution_result {
                    debug::info!("Inserted field token_execution_executor_account_id {:#?}", _mining_config_token_execution_result.token_execution_executor_account_id);
                    debug::info!("Inserted field token_execution_started_block {:#?}", _mining_config_token_execution_result.token_execution_started_block);
                    debug::info!("Inserted field token_execution_interval_blocks {:#?}", _mining_config_token_execution_result.token_execution_interval_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningConfigTokenExecutionResultSet(
                sender.clone(),
                mining_config_token_id,
                token_execution_executor_account_id.clone(),
                token_execution_started_block,
                token_execution_interval_blocks,
            ));

            if Self::execution(
                sender.clone(),
                mining_config_token_id,
                token_execution_executor_account_id.clone(),
                token_execution_started_block,
                token_execution_interval_blocks,
            ).is_ok() {
                debug::info!("Executed");
            } else {
                debug::info!("Cannot execute");
            }
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_config_token_owner(
        mining_config_token_id: T::MiningConfigTokenIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_config_token_owner(&mining_config_token_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of Mining"
        );
        Ok(())
    }

    pub fn exists_mining_config_token(
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<MiningConfigToken, DispatchError> {
        match Self::mining_config_token(mining_config_token_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningConfigToken does not exist")),
        }
    }

    pub fn exists_mining_config_token_token_config(
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_config_token_configs(mining_config_token_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningConfigTokenConfig does not exist")),
        }
    }

    // Check that the token execution has a token_execution_started_block > current_block
    pub fn token_execution_started_block_greater_than_current_block(
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        // Check that the extrinsic call is made after the start date defined in the provided configuration

        let current_block = <frame_system::Module<T>>::block_number();
        // Get the config associated with the given configuration_token
        if let Some(configuration_token_config) =
            Self::mining_config_token_configs(mining_config_token_id)
        {
            if let _token_lock_start_block = configuration_token_config.token_lock_start_block {
                ensure!(
                    current_block > _token_lock_start_block,
                    "Execution may not be made until after the start block of the lock period in the configuration"
                );
                Ok(())
            } else {
                return Err(DispatchError::Other("Cannot find token_config start_date associated with the execution"));
            }
        } else {
            return Err(DispatchError::Other("Cannot find token_config associated with the execution"));
        }
    }

    // Check that the associated token configuration has a token_lock_interval_blocks > token_lock_min_blocks
    pub fn token_lock_interval_blocks_greater_than_token_lock_min_blocks(
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        if let Some(configuration_token) =
            Self::mining_config_token_configs((mining_config_token_id))
        {
            if let Some(cooldown_configuration_token) =
                Self::mining_config_token_cooldown_configs((mining_config_token_id))
            {
                if let token_lock_interval_blocks = configuration_token.token_lock_interval_blocks {
                    if let token_lock_min_blocks = cooldown_configuration_token.token_lock_min_blocks {
                        ensure!(
                            token_lock_interval_blocks > token_lock_min_blocks,
                            "Lock period must be longer than the minimum lock period of the cooldown config. Cannot \
                             execute."
                        );
                        Ok(())
                    } else {
                        return Err(DispatchError::Other(
                            "Cannot find token_config with token_lock_min_blocks associated with the execution",
                        ));
                    }
                } else {
                    return Err(DispatchError::Other(
                        "Cannot find token_config with token_lock_interval_blocks associated with the execution",
                    ));
                }
            } else {
                return Err(DispatchError::Other("Cannot find token_cooldown_config associated with the execution"));
            }
        } else {
            return Err(DispatchError::Other("Cannot find token_config associated with the execution"));
        }
    }

    // Check that the associated token configuration has a token_lock_amount > token_lock_min_amount
    pub fn token_lock_amount_greater_than_token_lock_min_amount(
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        if let Some(configuration_token) =
            Self::mining_config_token_configs((mining_config_token_id))
        {
            if let Some(cooldown_configuration_token) =
                Self::mining_config_token_cooldown_configs((mining_config_token_id))
            {
                if let lock_amount = configuration_token.token_lock_amount {
                    if let lock_min_amount = cooldown_configuration_token.token_lock_min_amount {
                        ensure!(
                            lock_amount > lock_min_amount,
                            "Locked amount must be larger than the minimum locked amount of the cooldown config. \
                             Cannot execute."
                        );
                        Ok(())
                    } else {
                        return Err(DispatchError::Other(
                            "Cannot find token_config with token_lock_min_blocks associated with the execution",
                        ));
                    }
                } else {
                    return Err(DispatchError::Other(
                        "Cannot find token_config with token_lock_interval_blocks associated with the execution",
                    ));
                }
            } else {
                return Err(DispatchError::Other("Cannot find token_cooldown_config associated with the execution"));
            }
        } else {
            return Err(DispatchError::Other("Cannot find token_config associated with the execution"));
        }
    }

    pub fn has_value_for_mining_config_token_token_config_index(
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_config_token_token_config has a value that is defined");
        let fetched_mining_config_token_token_config = <MiningConfigTokenConfigs<T>>::get(mining_config_token_id);
        if let Some(_value) = fetched_mining_config_token_token_config {
            debug::info!("Found value for mining_config_token_token_config");
            return Ok(());
        }
        debug::info!("No value for mining_config_token_token_config");
        Err(DispatchError::Other("No value for mining_config_token_token_config"))
    }

    pub fn has_value_for_mining_config_token_token_cooldown_config_index(
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_config_token_token_cooldown_config has a value that is defined");
        let fetched_mining_config_token_token_cooldown_config =
            <MiningConfigTokenRequirementsConfigs<T>>::get(mining_config_token_id);
        if let Some(_value) = fetched_mining_config_token_token_cooldown_config {
            debug::info!("Found value for mining_config_token_token_cooldown_config");
            return Ok(());
        }
        debug::info!("No value for mining_config_token_token_cooldown_config");
        Err(DispatchError::Other("No value for mining_config_token_token_cooldown_config"))
    }

    pub fn has_value_for_mining_config_token_execution_result_index(
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_config_token_execution_result has a value that is defined");
        let fetched_mining_config_token_execution_result =
            <MiningConfigTokenExecutionResults<T>>::get(mining_config_token_id);
        if let Some(_value) = fetched_mining_config_token_execution_result {
            debug::info!("Found value for mining_config_token_execution_result");
            return Ok(());
        }
        debug::info!("No value for mining_config_token_execution_result");
        Err(DispatchError::Other("No value for mining_config_token_execution_result"))
    }

    pub fn execution(
        sender: T::AccountId,
        mining_config_token_id: T::MiningConfigTokenIndex,
        _token_execution_executor_account_id: T::AccountId,
        _token_execution_started_block: T::BlockNumber,
        _token_execution_interval_blocks: T::BlockNumber,
    ) -> Result<(), DispatchError> {
        return Ok(());
        // const EXAMPLE_ID: LockIdentifier = *b"example ";

        // TODO - Lock the token_lock_amount for the token_lock_interval_blocks using the Balances module

        // TODO - Setup a function in on_finalize that automatically checks through all the accounts that have
        // successfully been locked, whether it is the end of their cooldown period and if so sample the balance, to
        // determine their elegibility, and perform the claim for reward and unlock their tokens
        // TODO - Update tests for the above
        if let Some(configuration_token) =
            Self::mining_config_token_configs((mining_config_token_id))
        {
            if let lock_amount = configuration_token.token_lock_amount {
                if let Some(execution_results) = Self::mining_config_token_execution_results(mining_config_token_id) {
                    // <T as Trait>::Currency::set_lock(
                    //     execution_token, // EXAMPLE_ID,
                    //     &_token_execution_executor_account_id,
                    //     lock_amount,
                    //     WithdrawReasons::all(),
                    // );
                    return Ok(());
                } else {
                    return Err(DispatchError::Other(
                        "Cannot find mining_config_token_id associated with the execution",
                    ));
                }
            } else {
                return Err(DispatchError::Other(
                    "Cannot find token_mining_config with token_lock_period associated with the execution",
                ));
            }
        } else {
            return Err(DispatchError::Other("Cannot find token_mining_config associated with the execution"));
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

    fn next_mining_config_token_id() -> Result<T::MiningConfigTokenIndex, DispatchError> {
        let mining_config_token_id = Self::mining_config_token_count();
        if mining_config_token_id == <T::MiningConfigTokenIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningConfigToken count overflow"));
        }
        Ok(mining_config_token_id)
    }

    fn insert_mining_config_token(
        owner: &T::AccountId,
        mining_config_token_id: T::MiningConfigTokenIndex,
        mining_config_token: MiningConfigToken,
    ) {
        // Create and store mining mining_config_token
        <MiningConfigTokens<T>>::insert(mining_config_token_id, mining_config_token);
        <MiningConfigTokenCount<T>>::put(mining_config_token_id + One::one());
        <MiningConfigTokenOwners<T>>::insert(mining_config_token_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_config_token_id: T::MiningConfigTokenIndex) {
        <MiningConfigTokenOwners<T>>::insert(mining_config_token_id, to);
    }
}
