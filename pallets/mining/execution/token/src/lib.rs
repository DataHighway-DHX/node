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
use mining_claims_token;
use mining_config_token;
use mining_eligibility_token;
use mining_rates_token;
use mining_sampling_token;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

/// The module's configuration trait.
pub trait Config:
    frame_system::Config
    + roaming_operators::Config
    + mining_config_token::Config
    + mining_eligibility_token::Config
    + mining_rates_token::Config
    + mining_sampling_token::Config
    + mining_claims_token::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningExecutionTokenIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningExecutionTokenExecutorAccountID: Parameter
    //     + Member
    //     + AtLeast32Bit
    //     + Bounded
    //     + Default
    //     + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningExecutionToken(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningExecutionTokenExecutionResult<U, V, W> {
    pub token_execution_executor_account_id: U,
    pub token_execution_started_block: V,
    pub token_execution_ended_block: W,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::MiningExecutionTokenIndex,
        // <T as Config>::MiningExecutionTokenExecutorAccountID,
        <T as mining_config_token::Config>::MiningConfigTokenIndex,
        <T as frame_system::Config>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_execution_token is created. (owner, mining_execution_token_id)
        Created(AccountId, MiningExecutionTokenIndex),
        /// A mining_execution_token is transferred. (from, to, mining_execution_token_id)
        Transferred(AccountId, AccountId, MiningExecutionTokenIndex),
        MiningExecutionTokenExecutionResultSet(
            AccountId, MiningConfigTokenIndex, MiningExecutionTokenIndex,
            AccountId, BlockNumber, BlockNumber
        ),
        /// A mining_execution_token is assigned to an mining_token.
        /// (owner of mining_token, mining_execution_token_id, mining_config_token_id)
        AssignedTokenExecutionToConfiguration(AccountId, MiningExecutionTokenIndex, MiningConfigTokenIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningExecutionToken {
        /// Stores all the mining_execution_tokens, key is the mining_execution_token id / index
        pub MiningExecutionTokens get(fn mining_execution_token): map hasher(opaque_blake2_256) T::MiningExecutionTokenIndex => Option<MiningExecutionToken>;

        /// Stores the total number of mining_execution_tokens. i.e. the next mining_execution_token index
        pub MiningExecutionTokenCount get(fn mining_execution_token_count): T::MiningExecutionTokenIndex;

        /// Stores mining_execution_token owner
        pub MiningExecutionTokenOwners get(fn mining_execution_token_owner): map hasher(opaque_blake2_256) T::MiningExecutionTokenIndex => Option<T::AccountId>;

        /// Stores mining_execution_token_execution_result
        pub MiningExecutionTokenExecutionResults get(fn mining_execution_token_execution_results): map hasher(opaque_blake2_256) (T::MiningConfigTokenIndex, T::MiningExecutionTokenIndex) =>
            Option<MiningExecutionTokenExecutionResult<
                T::AccountId,
                T::BlockNumber,
                T::BlockNumber
            >>;

        /// Get mining_config_token_id belonging to a mining_execution_token_id
        pub TokenExecutionConfiguration get(fn token_execution_configuration): map hasher(opaque_blake2_256) T::MiningExecutionTokenIndex => Option<T::MiningConfigTokenIndex>;

        /// Get mining_execution_token_id's belonging to a mining_config_token_id
        pub TokenConfigExecution get(fn token_config_execution): map hasher(opaque_blake2_256) T::MiningConfigTokenIndex => Option<Vec<T::MiningExecutionTokenIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_execution_token
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_execution_token_id = Self::next_mining_execution_token_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_execution_token
            let mining_execution_token = MiningExecutionToken(unique_id);
            Self::insert_mining_execution_token(&sender, mining_execution_token_id, mining_execution_token);

            Self::deposit_event(RawEvent::Created(sender, mining_execution_token_id));
        }

        /// Transfer a mining_execution_token to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_execution_token_id: T::MiningExecutionTokenIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_execution_token_owner(mining_execution_token_id) == Some(sender.clone()), "Only owner can transfer mining mining_execution_token");

            Self::update_owner(&to, mining_execution_token_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_execution_token_id));
        }

        /// Set mining_execution_token_execution_result
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_execution_token_execution_result(
            origin,
            mining_config_token_id: T::MiningConfigTokenIndex,
            mining_execution_token_id: T::MiningExecutionTokenIndex,
            _token_execution_started_block: Option<T::BlockNumber>,
            _token_execution_ended_block: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_execution_token_id whose config we want to change actually exists
            let is_mining_execution_token = Self::exists_mining_execution_token(mining_execution_token_id).is_ok();
            ensure!(is_mining_execution_token, "MiningExecutionToken does not exist");

            // Ensure that the caller is owner of the mining_execution_token_execution_result they are trying to change
            ensure!(Self::mining_execution_token_owner(mining_execution_token_id) == Some(sender.clone()), "Only owner can set mining_execution_token_execution_result");

            // Check that only allow the owner of the configuration that the execution belongs to call this extrinsic to set and execute
            ensure!(
                <mining_config_token::Module<T>>::is_mining_config_token_owner(
                    mining_config_token_id, sender.clone()
                ).is_ok(),
                "Only the configuration_token owner can execute their associated execution"
            );

            // TODO - adjust defaults
            let token_execution_executor_account_id = sender.clone();
            let token_execution_started_block = match _token_execution_started_block.clone() {
                Some(value) => value,
                None => <frame_system::Module<T>>::block_number()
            };
            let token_execution_ended_block = match _token_execution_ended_block {
                Some(value) => value,
                None => <frame_system::Module<T>>::block_number() + 1u32.into() // Default
            };

            // FIXME
            // // Ensure that the associated token configuration has a token_execution_started_block > current_block
            // let is_token_execution_started_block_greater_than_current_block = Self::token_execution_started_block_greater_than_current_block(mining_execution_token_id, mining_config_token_id).is_ok();
            // ensure!(is_token_execution_started_block_greater_than_current_block, "token execution does not have a token_execution_started_block > current_block");

            // FIXME
            // // Ensure that the associated token configuration has a token_lock_interval_blocks > token_lock_min_blocks
            // let is_token_lock_interval_blocks_greater_than_token_lock_min_blocks = Self::token_lock_interval_blocks_greater_than_token_lock_min_blocks(mining_execution_token_id, mining_config_token_id).is_ok();
            // ensure!(is_token_lock_interval_blocks_greater_than_token_lock_min_blocks, "token configuration does not have a token_lock_interval_blocks > token_lock_min_blocks");

            // Ensure that the associated token configuration has a token_lock_amount > token_lock_min_amount
            let is_token_lock_amount_greater_than_token_lock_min_amount = Self::token_lock_amount_greater_than_token_lock_min_amount(mining_execution_token_id, mining_config_token_id).is_ok();
            ensure!(is_token_lock_amount_greater_than_token_lock_min_amount, "token configuration does not have a token_lock_amount > token_lock_min_amount");

            // Check if a mining_execution_token_execution_result already exists with the given mining_execution_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_execution_token_execution_result_index(mining_config_token_id, mining_execution_token_id).is_ok() {
                debug::info!("Mutating values");
                <MiningExecutionTokenExecutionResults<T>>::mutate((mining_config_token_id, mining_execution_token_id), |mining_execution_token_execution_result| {
                    if let Some(_mining_execution_token_execution_result) = mining_execution_token_execution_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_execution_token_execution_result.token_execution_executor_account_id = token_execution_executor_account_id.clone();
                        _mining_execution_token_execution_result.token_execution_started_block = token_execution_started_block.clone();
                        _mining_execution_token_execution_result.token_execution_ended_block = token_execution_ended_block.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_execution_token_execution_result = <MiningExecutionTokenExecutionResults<T>>::get((mining_config_token_id, mining_execution_token_id));
                if let Some(_mining_execution_token_execution_result) = fetched_mining_execution_token_execution_result {
                    debug::info!("Latest field token_execution_executor_account_id {:#?}", _mining_execution_token_execution_result.token_execution_executor_account_id);
                    debug::info!("Latest field token_execution_started_block {:#?}", _mining_execution_token_execution_result.token_execution_started_block);
                    debug::info!("Latest field token_execution_ended_block {:#?}", _mining_execution_token_execution_result.token_execution_ended_block);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_execution_token_execution_result instance with the input params
                let mining_execution_token_execution_result_instance = MiningExecutionTokenExecutionResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_execution_executor_account_id: token_execution_executor_account_id.clone(),
                    token_execution_started_block: token_execution_started_block.clone(),
                    token_execution_ended_block: token_execution_ended_block.clone(),
                };

                <MiningExecutionTokenExecutionResults<T>>::insert(
                    (mining_config_token_id, mining_execution_token_id),
                    &mining_execution_token_execution_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_execution_token_execution_result = <MiningExecutionTokenExecutionResults<T>>::get((mining_config_token_id, mining_execution_token_id));
                if let Some(_mining_execution_token_execution_result) = fetched_mining_execution_token_execution_result {
                    debug::info!("Inserted field token_execution_executor_account_id {:#?}", _mining_execution_token_execution_result.token_execution_executor_account_id);
                    debug::info!("Inserted field token_execution_started_block {:#?}", _mining_execution_token_execution_result.token_execution_started_block);
                    debug::info!("Inserted field token_execution_ended_block {:#?}", _mining_execution_token_execution_result.token_execution_ended_block);
                }
            }

            Self::deposit_event(RawEvent::MiningExecutionTokenExecutionResultSet(
                sender.clone(),
                mining_config_token_id,
                mining_execution_token_id,
                token_execution_executor_account_id.clone(),
                token_execution_started_block,
                token_execution_ended_block,
            ));



            if Self::execution(
                sender.clone(),
                mining_config_token_id,
                mining_execution_token_id,
                token_execution_executor_account_id.clone(),
                token_execution_started_block,
                token_execution_ended_block,
            ).is_ok() {
                debug::info!("Executed");
            } else {
                debug::info!("Cannot execute");
            }
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_execution_to_configuration(
          origin,
          mining_execution_token_id: T::MiningExecutionTokenIndex,
          mining_config_token_id: T::MiningConfigTokenIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token = <mining_config_token::Module<T>>
                ::exists_mining_config_token(mining_config_token_id).is_ok();
            ensure!(is_configuration_token, "configuration_token does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the execution to
            ensure!(
                <mining_config_token::Module<T>>::is_mining_config_token_owner(mining_config_token_id, sender.clone()).is_ok(),
                "Only the configuration_token owner can assign itself a execution"
            );

            Self::associate_token_execution_with_configuration(mining_execution_token_id, mining_config_token_id)
                .expect("Unable to associate execution with configuration");

            // Ensure that the given mining_execution_token_id already exists
            let token_execution = Self::mining_execution_token(mining_execution_token_id);
            ensure!(token_execution.is_some(), "Invalid mining_execution_token_id");

            // // Ensure that the execution is not already owned by a different configuration
            // // Unassign the execution from any existing configuration since it may only be owned by one configuration
            // <TokenExecutionConfiguration<T>>::remove(mining_execution_token_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenExecutionConfiguration<T>>::insert(mining_execution_token_id, mining_config_token_id);

            Self::deposit_event(RawEvent::AssignedTokenExecutionToConfiguration(sender, mining_execution_token_id, mining_config_token_id));
            }
    }
}

impl<T: Config> Module<T> {
    pub fn is_mining_execution_token_owner(
        mining_execution_token_id: T::MiningExecutionTokenIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_execution_token_owner(&mining_execution_token_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningExecutionToken"
        );
        Ok(())
    }

    pub fn exists_mining_execution_token(
        mining_execution_token_id: T::MiningExecutionTokenIndex,
    ) -> Result<MiningExecutionToken, DispatchError> {
        match Self::mining_execution_token(mining_execution_token_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningExecutionToken does not exist")),
        }
    }

    pub fn exists_mining_execution_token_execution_result(
        mining_config_token_id: T::MiningConfigTokenIndex,
        mining_execution_token_id: T::MiningExecutionTokenIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_execution_token_execution_results((mining_config_token_id, mining_execution_token_id)) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningExecutionTokenExecutionResult does not exist")),
        }
    }

    // Check that the token execution has a token_execution_started_block > current_block
    pub fn token_execution_started_block_greater_than_current_block(
        mining_execution_token_id: T::MiningExecutionTokenIndex,
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        // Check that the extrinsic call is made after the start date defined in the provided configuration

        let current_block = <frame_system::Module<T>>::block_number();
        // Get the config associated with the given configuration_token
        if let Some(configuration_token_config) =
            <mining_config_token::Module<T>>::mining_config_token_token_configs(mining_config_token_id)
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
        mining_execution_token_id: T::MiningExecutionTokenIndex,
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        if let Some(configuration_token) =
            <mining_config_token::Module<T>>::mining_config_token_token_configs((mining_config_token_id))
        {
            if let Some(cooldown_configuration_token) =
                <mining_config_token::Module<T>>::mining_config_token_token_cooldown_configs((mining_config_token_id))
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
        mining_execution_token_id: T::MiningExecutionTokenIndex,
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        if let Some(configuration_token) =
            <mining_config_token::Module<T>>::mining_config_token_token_configs((mining_config_token_id))
        {
            if let Some(cooldown_configuration_token) =
                <mining_config_token::Module<T>>::mining_config_token_token_cooldown_configs((mining_config_token_id))
            {
                if let locked_amount = configuration_token.token_lock_amount {
                    if let lock_min_amount = cooldown_configuration_token.token_lock_min_amount {
                        ensure!(
                            locked_amount > lock_min_amount,
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

    pub fn execution(
        sender: T::AccountId,
        mining_config_token_id: T::MiningConfigTokenIndex,
        mining_execution_token_id: T::MiningExecutionTokenIndex,
        _token_execution_executor_account_id: T::AccountId,
        _token_execution_started_block: T::BlockNumber,
        _token_execution_ended_block: T::BlockNumber,
    ) -> Result<(), DispatchError> {
        return Ok(());

        // TODO - Lock the token_lock_amount for the token_lock_interval_blocks using the Balances module

        // TODO - Setup a function in on_finalize that automatically checks through all the accounts that have
        // successfully been locked, whether it is the end of their cooldown period and if so sample the balance, to
        // determine their elegibility, and perform the claim for reward and unlock their tokens
        // TODO - Update tests for the above
    }

    pub fn has_value_for_mining_execution_token_execution_result_index(
        mining_config_token_id: T::MiningConfigTokenIndex,
        mining_execution_token_id: T::MiningExecutionTokenIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_execution_token_execution_result has a value that is defined");
        let fetched_mining_execution_token_execution_result =
            <MiningExecutionTokenExecutionResults<T>>::get((mining_config_token_id, mining_execution_token_id));
        if let Some(_value) = fetched_mining_execution_token_execution_result {
            debug::info!("Found value for mining_execution_token_execution_result");
            return Ok(());
        }
        debug::info!("No value for mining_execution_token_execution_result");
        Err(DispatchError::Other("No value for mining_execution_token_execution_result"))
    }

    /// Only push the execution id onto the end of the vector if it does not already exist
    pub fn associate_token_execution_with_configuration(
        mining_execution_token_id: T::MiningExecutionTokenIndex,
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given execution id
        if let Some(configuration_execution) = Self::token_config_execution(mining_config_token_id) {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_config_token_id,
                configuration_execution
            );
            let not_configuration_contains_execution = !configuration_execution.contains(&mining_execution_token_id);
            ensure!(not_configuration_contains_execution, "Configuration already contains the given execution id");
            debug::info!("Configuration id key exists but its vector value does not contain the given execution id");
            <TokenConfigExecution<T>>::mutate(mining_config_token_id, |v| {
                if let Some(value) = v {
                    value.push(mining_execution_token_id);
                }
            });
            debug::info!(
                "Associated execution {:?} with configuration {:?}",
                mining_execution_token_id,
                mining_config_token_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 execution id {:?} to its vector value",
                mining_config_token_id,
                mining_execution_token_id
            );
            <TokenConfigExecution<T>>::insert(mining_config_token_id, &vec![mining_execution_token_id]);
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

    fn next_mining_execution_token_id() -> Result<T::MiningExecutionTokenIndex, DispatchError> {
        let mining_execution_token_id = Self::mining_execution_token_count();
        if mining_execution_token_id == <T::MiningExecutionTokenIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningExecutionToken count overflow"));
        }
        Ok(mining_execution_token_id)
    }

    fn insert_mining_execution_token(
        owner: &T::AccountId,
        mining_execution_token_id: T::MiningExecutionTokenIndex,
        mining_execution_token: MiningExecutionToken,
    ) {
        // Create and store mining mining_execution_token
        <MiningExecutionTokens<T>>::insert(mining_execution_token_id, mining_execution_token);
        <MiningExecutionTokenCount<T>>::put(mining_execution_token_id + One::one());
        <MiningExecutionTokenOwners<T>>::insert(mining_execution_token_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_execution_token_id: T::MiningExecutionTokenIndex) {
        <MiningExecutionTokenOwners<T>>::insert(mining_execution_token_id, to);
    }
}
