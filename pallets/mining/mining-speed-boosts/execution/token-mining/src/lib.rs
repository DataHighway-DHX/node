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
use mining_speed_boosts_eligibility_token_mining;
use mining_speed_boosts_lodgements_token_mining;
use mining_speed_boosts_rates_token_mining;
use mining_speed_boosts_sampling_token_mining;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + roaming_operators::Trait
    + mining_speed_boosts_configuration_token_mining::Trait
    + mining_speed_boosts_eligibility_token_mining::Trait
    + mining_speed_boosts_rates_token_mining::Trait
    + mining_speed_boosts_sampling_token_mining::Trait
    + mining_speed_boosts_lodgements_token_mining::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningSpeedBoostExecutionTokenMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostExecutionTokenMiningStartedDate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostExecutionTokenMiningEndedDate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningSpeedBoostExecutionTokenMiningExecutorAccountID: Parameter
    //     + Member
    //     + AtLeast32Bit
    //     + Bounded
    //     + Default
    //     + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostExecutionTokenMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostExecutionTokenMiningExecutionResult<U, V, W> {
    pub token_execution_executor_account_id: U,
    pub token_execution_started_date: V,
    pub token_execution_ended_date: W,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostExecutionTokenMiningIndex,
        // <T as Trait>::MiningSpeedBoostExecutionTokenMiningExecutorAccountID,
        <T as Trait>::MiningSpeedBoostExecutionTokenMiningStartedDate,
        <T as Trait>::MiningSpeedBoostExecutionTokenMiningEndedDate,
        <T as mining_speed_boosts_configuration_token_mining::Trait>::MiningSpeedBoostConfigurationTokenMiningIndex,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_execution_token_mining is created. (owner, mining_speed_boosts_execution_token_mining_id)
        Created(AccountId, MiningSpeedBoostExecutionTokenMiningIndex),
        /// A mining_speed_boosts_execution_token_mining is transferred. (from, to, mining_speed_boosts_execution_token_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostExecutionTokenMiningIndex),
        MiningSpeedBoostExecutionTokenMiningExecutionResultSet(
            AccountId, MiningSpeedBoostConfigurationTokenMiningIndex, MiningSpeedBoostExecutionTokenMiningIndex,
            AccountId, MiningSpeedBoostExecutionTokenMiningStartedDate,
            MiningSpeedBoostExecutionTokenMiningEndedDate
        ),
        /// A mining_speed_boosts_execution_token_mining is assigned to an mining_speed_boosts_token_mining.
        /// (owner of mining_speed_boosts_token_mining, mining_speed_boosts_execution_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
        AssignedTokenMiningExecutionToConfiguration(AccountId, MiningSpeedBoostExecutionTokenMiningIndex, MiningSpeedBoostConfigurationTokenMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostExecutionTokenMining {
        /// Stores all the mining_speed_boosts_execution_token_minings, key is the mining_speed_boosts_execution_token_mining id / index
        pub MiningSpeedBoostExecutionTokenMinings get(fn mining_speed_boosts_execution_token_mining): map hasher(opaque_blake2_256) T::MiningSpeedBoostExecutionTokenMiningIndex => Option<MiningSpeedBoostExecutionTokenMining>;

        /// Stores the total number of mining_speed_boosts_execution_token_minings. i.e. the next mining_speed_boosts_execution_token_mining index
        pub MiningSpeedBoostExecutionTokenMiningCount get(fn mining_speed_boosts_execution_token_mining_count): T::MiningSpeedBoostExecutionTokenMiningIndex;

        /// Stores mining_speed_boosts_execution_token_mining owner
        pub MiningSpeedBoostExecutionTokenMiningOwners get(fn mining_speed_boosts_execution_token_mining_owner): map hasher(opaque_blake2_256) T::MiningSpeedBoostExecutionTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_execution_token_mining_execution_result
        pub MiningSpeedBoostExecutionTokenMiningExecutionResults get(fn mining_speed_boosts_execution_token_mining_execution_results): map hasher(opaque_blake2_256) (T::MiningSpeedBoostConfigurationTokenMiningIndex, T::MiningSpeedBoostExecutionTokenMiningIndex) =>
            Option<MiningSpeedBoostExecutionTokenMiningExecutionResult<
                T::AccountId,
                T::MiningSpeedBoostExecutionTokenMiningStartedDate,
                T::MiningSpeedBoostExecutionTokenMiningEndedDate
            >>;

        /// Get mining_speed_boosts_configuration_token_mining_id belonging to a mining_speed_boosts_execution_token_mining_id
        pub TokenMiningExecutionConfiguration get(fn token_mining_execution_configuration): map hasher(opaque_blake2_256) T::MiningSpeedBoostExecutionTokenMiningIndex => Option<T::MiningSpeedBoostConfigurationTokenMiningIndex>;

        /// Get mining_speed_boosts_execution_token_mining_id's belonging to a mining_speed_boosts_configuration_token_mining_id
        pub TokenMiningConfigurationExecution get(fn token_mining_configuration_execution): map hasher(opaque_blake2_256) T::MiningSpeedBoostConfigurationTokenMiningIndex => Option<Vec<T::MiningSpeedBoostExecutionTokenMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_execution_token_mining
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_execution_token_mining_id = Self::next_mining_speed_boosts_execution_token_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_execution_token_mining
            let mining_speed_boosts_execution_token_mining = MiningSpeedBoostExecutionTokenMining(unique_id);
            Self::insert_mining_speed_boosts_execution_token_mining(&sender, mining_speed_boosts_execution_token_mining_id, mining_speed_boosts_execution_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_execution_token_mining_id));
        }

        /// Transfer a mining_speed_boosts_execution_token_mining to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_execution_token_mining_owner(mining_speed_boosts_execution_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_execution_token_mining");

            Self::update_owner(&to, mining_speed_boosts_execution_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_execution_token_mining_id));
        }

        // #[weight = 10_000 + T::DbWeight::get().writes(1)]
        // pub fn execution(
        //     origin,
        //     mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        //     mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
        //     mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
        // ) {
        //     let sender = ensure_signed(origin)?;

        //     // Ensure that the mining_speed_boosts_execution_token_mining_id whose config we want to change actually exists
        //     let is_mining_speed_boosts_execution_token_mining = Self::exists_mining_speed_boosts_execution_token_mining(mining_speed_boosts_execution_token_mining_id).is_ok();
        //     ensure!(is_mining_speed_boosts_execution_token_mining, "MiningSpeedBoostExecutionTokenMining does not exist");

        //     // Ensure that the caller is owner of the mining_speed_boosts_execution_token_mining_execution_result they are trying to change
        //     ensure!(Self::mining_speed_boosts_execution_token_mining_owner(mining_speed_boosts_execution_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_execution_token_mining_execution_result");

        //     // Check that only allow the owner of the configuration that the execution belongs to call this extrinsic
        //     // and execution their eligibility
        //     ensure!(
        //       <mining_speed_boosts_configuration_token_mining::Module<T>>::is_mining_speed_boosts_configuration_token_mining_owner(
        //         mining_speed_boosts_configuration_token_mining_id, sender.clone()
        //       ).is_ok(),
        //       "Only the configuration_token_mining owner can execution their associated eligibility"
        //     );

        //     // Check that the extrinsic call is made after the end date defined in the provided configuration

        //     // FIXME - add system time now
        //     let time_now = 123.into();
        //     // Get the config associated with the given configuration_token_mining
        //     if let Some(configuration_token_mining_config) = <mining_speed_boosts_configuration_token_mining::Module<T>>::mining_speed_boosts_configuration_token_mining_token_configs(mining_speed_boosts_configuration_token_mining_id) {
        //       if let _token_lock_period_end_date = configuration_token_mining_config.token_lock_period_end_date {
        //         // FIXME - get this to work when add system time
        //         // ensure!(time_now > token_lock_period_end_date, "Execution may not be made until after the end date of the lock period");
        //       } else {
        //         return Err(DispatchError::Other("Cannot find token_mining_config end_date associated with the execution"));
        //       }
        //     } else {
        //       return Err(DispatchError::Other("Cannot find token_mining_config associated with the execution"));
        //     }

        //     // Check that the provided eligibility amount has not already been executioned
        //     // i.e. there should only be a single execution instance for each configuration and eligibility in the MVP
        //     if let Some(token_mining_configuration_execution) = Self::token_mining_configuration_execution(mining_speed_boosts_configuration_token_mining_id) {
        //       ensure!(token_mining_configuration_execution.len() == 1, "Cannot have zero or more than one execution associated with configuration/eligibility");
        //     } else {
        //       return Err(DispatchError::Other("Cannot find configuration_execution associated with the execution"));
        //     }

        //     // Record the execution associated with their configuration/eligibility
        //     let token_execution_started_date: T::MiningSpeedBoostExecutionTokenMiningStartedDate = 0.into();
        //     let token_execution_ended_date: T::MiningSpeedBoostExecutionTokenMiningEndedDate = time_now;
        //     if let Some(eligibility_token_mining) = <mining_speed_boosts_eligibility_token_mining::Module<T>>::mining_speed_boosts_eligibility_token_mining_eligibility_results((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id)) {
        //       if let token_mining_calculated_eligibility = eligibility_token_mining.eligibility_token_mining_calculated_eligibility {
        //         ensure!(token_mining_calculated_eligibility > 0.into(), "Calculated eligibility is zero. Nothing to execution.");
        //         // FIXME - unable to figure out how to cast here!
        //         // token_execution_started_date = (token_mining_calculated_eligibility as T::MiningSpeedBoostExecutionTokenMiningStartedDate).clone();
        //       } else {
        //         return Err(DispatchError::Other("Cannot find token_mining_eligibility calculated_eligibility associated with the execution"));
        //       }
        //     } else {
        //       return Err(DispatchError::Other("Cannot find token_mining_eligibility associated with the execution"));
        //     }

        //     // Check if a mining_speed_boosts_execution_token_mining_execution_result already exists with the given mining_speed_boosts_execution_token_mining_id
        //     // to determine whether to insert new or mutate existing.
        //     if Self::has_value_for_mining_speed_boosts_execution_token_mining_execution_result_index(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id).is_ok() {
        //         debug::info!("Mutating values");
        //         <MiningSpeedBoostExecutionTokenMiningExecutionResults<T>>::mutate((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id), |mining_speed_boosts_execution_token_mining_execution_result| {
        //             if let Some(_mining_speed_boosts_execution_token_mining_execution_result) = mining_speed_boosts_execution_token_mining_execution_result {
        //                 // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
        //                 _mining_speed_boosts_execution_token_mining_execution_result.token_execution_started_date = token_execution_started_date.clone();
        //                 _mining_speed_boosts_execution_token_mining_execution_result.token_execution_ended_date = token_execution_ended_date.clone();
        //             }
        //         });
        //         debug::info!("Checking mutated values");
        //         let fetched_mining_speed_boosts_execution_token_mining_execution_result = <MiningSpeedBoostExecutionTokenMiningExecutionResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id));
        //         if let Some(_mining_speed_boosts_execution_token_mining_execution_result) = fetched_mining_speed_boosts_execution_token_mining_execution_result {
        //             debug::info!("Latest field token_execution_started_date {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_started_date);
        //             debug::info!("Latest field token_execution_ended_date {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_ended_date);
        //         }
        //     } else {
        //         debug::info!("Inserting values");

        //         // Create a new mining mining_speed_boosts_execution_token_mining_execution_result instance with the input params
        //         let mining_speed_boosts_execution_token_mining_execution_result_instance = MiningSpeedBoostExecutionTokenMiningExecutionResult {
        //             // Since each parameter passed into the function is optional (i.e. `Option`)
        //             // we will assign a default value if a parameter value is not provided.
        //             token_execution_started_date: token_execution_started_date.clone(),
        //             token_execution_ended_date: token_execution_ended_date.clone(),
        //         };

        //         <MiningSpeedBoostExecutionTokenMiningExecutionResults<T>>::insert(
        //             (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id),
        //             &mining_speed_boosts_execution_token_mining_execution_result_instance
        //         );

        //         debug::info!("Checking inserted values");
        //         let fetched_mining_speed_boosts_execution_token_mining_execution_result = <MiningSpeedBoostExecutionTokenMiningExecutionResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id));
        //         if let Some(_mining_speed_boosts_execution_token_mining_execution_result) = fetched_mining_speed_boosts_execution_token_mining_execution_result {
        //             debug::info!("Inserted field token_execution_started_date {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_started_date);
        //             debug::info!("Inserted field token_execution_ended_date {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_ended_date);
        //         }
        //     }

        //     // Self::deposit_event(RawEvent::MiningSpeedBoostExecutionTokenMiningExecutionResultSet(
        //     //     sender,
        //     //     mining_speed_boosts_configuration_token_mining_id,
        //     //     mining_speed_boosts_execution_token_mining_id,
        //     //     token_execution_started_date,
        //     //     token_execution_ended_date,
        //     // ));

        //     // After the execution is stored, then if the user wins a proportion of the block reward
        //     // through validating or nominating, then we will multiply that reward by their
        //     // executioned eligibility to determine what mining speed bonus they should be given.
        // }

        /// Set mining_speed_boosts_execution_token_mining_execution_result
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_speed_boosts_execution_token_mining_execution_result(
            origin,
            mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
            mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
            mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
            _token_execution_started_date: Option<T::MiningSpeedBoostExecutionTokenMiningStartedDate>,
            _token_execution_ended_date: Option<T::MiningSpeedBoostExecutionTokenMiningEndedDate>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_execution_token_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_execution_token_mining = Self::exists_mining_speed_boosts_execution_token_mining(mining_speed_boosts_execution_token_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_execution_token_mining, "MiningSpeedBoostExecutionTokenMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_execution_token_mining_execution_result they are trying to change
            ensure!(Self::mining_speed_boosts_execution_token_mining_owner(mining_speed_boosts_execution_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_execution_token_mining_execution_result");

            // TODO - adjust defaults
            let token_execution_executor_account_id = sender.clone();
            let token_execution_started_date = match _token_execution_started_date.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_execution_ended_date = match _token_execution_ended_date {
                Some(value) => value,
                None => 1.into() // Default
            };

            // Check if a mining_speed_boosts_execution_token_mining_execution_result already exists with the given mining_speed_boosts_execution_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_execution_token_mining_execution_result_index(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostExecutionTokenMiningExecutionResults<T>>::mutate((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id), |mining_speed_boosts_execution_token_mining_execution_result| {
                    if let Some(_mining_speed_boosts_execution_token_mining_execution_result) = mining_speed_boosts_execution_token_mining_execution_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_execution_token_mining_execution_result.token_execution_executor_account_id = token_execution_executor_account_id.clone();
                        _mining_speed_boosts_execution_token_mining_execution_result.token_execution_started_date = token_execution_started_date.clone();
                        _mining_speed_boosts_execution_token_mining_execution_result.token_execution_ended_date = token_execution_ended_date.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_execution_token_mining_execution_result = <MiningSpeedBoostExecutionTokenMiningExecutionResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id));
                if let Some(_mining_speed_boosts_execution_token_mining_execution_result) = fetched_mining_speed_boosts_execution_token_mining_execution_result {
                    debug::info!("Latest field token_execution_executor_account_id {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_executor_account_id);
                    debug::info!("Latest field token_execution_started_date {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_started_date);
                    debug::info!("Latest field token_execution_ended_date {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_ended_date);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_execution_token_mining_execution_result instance with the input params
                let mining_speed_boosts_execution_token_mining_execution_result_instance = MiningSpeedBoostExecutionTokenMiningExecutionResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_execution_executor_account_id: token_execution_executor_account_id.clone(),
                    token_execution_started_date: token_execution_started_date.clone(),
                    token_execution_ended_date: token_execution_ended_date.clone(),
                };

                <MiningSpeedBoostExecutionTokenMiningExecutionResults<T>>::insert(
                    (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id),
                    &mining_speed_boosts_execution_token_mining_execution_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_execution_token_mining_execution_result = <MiningSpeedBoostExecutionTokenMiningExecutionResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_execution_token_mining_id));
                if let Some(_mining_speed_boosts_execution_token_mining_execution_result) = fetched_mining_speed_boosts_execution_token_mining_execution_result {
                    debug::info!("Inserted field token_execution_executor_account_id {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_executor_account_id);
                    debug::info!("Inserted field token_execution_started_date {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_started_date);
                    debug::info!("Inserted field token_execution_ended_date {:#?}", _mining_speed_boosts_execution_token_mining_execution_result.token_execution_ended_date);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostExecutionTokenMiningExecutionResultSet(
                sender,
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_execution_token_mining_id,
                token_execution_executor_account_id,
                token_execution_started_date,
                token_execution_ended_date,
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_execution_to_configuration(
          origin,
          mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
          mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token_mining = <mining_speed_boosts_configuration_token_mining::Module<T>>
                ::exists_mining_speed_boosts_configuration_token_mining(mining_speed_boosts_configuration_token_mining_id).is_ok();
            ensure!(is_configuration_token_mining, "configuration_token_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the execution to
            ensure!(
                <mining_speed_boosts_configuration_token_mining::Module<T>>::is_mining_speed_boosts_configuration_token_mining_owner(mining_speed_boosts_configuration_token_mining_id, sender.clone()).is_ok(),
                "Only the configuration_token_mining owner can assign itself a execution"
            );

            Self::associate_token_execution_with_configuration(mining_speed_boosts_execution_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
                .expect("Unable to associate execution with configuration");

            // Ensure that the given mining_speed_boosts_execution_token_mining_id already exists
            let token_execution = Self::mining_speed_boosts_execution_token_mining(mining_speed_boosts_execution_token_mining_id);
            ensure!(token_execution.is_some(), "Invalid mining_speed_boosts_execution_token_mining_id");

            // // Ensure that the execution is not already owned by a different configuration
            // // Unassign the execution from any existing configuration since it may only be owned by one configuration
            // <TokenMiningExecutionConfiguration<T>>::remove(mining_speed_boosts_execution_token_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenMiningExecutionConfiguration<T>>::insert(mining_speed_boosts_execution_token_mining_id, mining_speed_boosts_configuration_token_mining_id);

            Self::deposit_event(RawEvent::AssignedTokenMiningExecutionToConfiguration(sender, mining_speed_boosts_execution_token_mining_id, mining_speed_boosts_configuration_token_mining_id));
            }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_speed_boosts_execution_token_mining_owner(
        mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_execution_token_mining_owner(&mining_speed_boosts_execution_token_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostExecutionTokenMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_execution_token_mining(
        mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
    ) -> Result<MiningSpeedBoostExecutionTokenMining, DispatchError> {
        match Self::mining_speed_boosts_execution_token_mining(mining_speed_boosts_execution_token_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostExecutionTokenMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_execution_token_mining_execution_result(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_execution_token_mining_execution_results((
            mining_speed_boosts_configuration_token_mining_id,
            mining_speed_boosts_execution_token_mining_id,
        )) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostExecutionTokenMiningExecutionResult does not exist")),
        }
    }

    pub fn has_value_for_mining_speed_boosts_execution_token_mining_execution_result_index(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_speed_boosts_execution_token_mining_execution_result has a value that is defined"
        );
        let fetched_mining_speed_boosts_execution_token_mining_execution_result =
            <MiningSpeedBoostExecutionTokenMiningExecutionResults<T>>::get((
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_execution_token_mining_id,
            ));
        if let Some(_value) = fetched_mining_speed_boosts_execution_token_mining_execution_result {
            debug::info!("Found value for mining_speed_boosts_execution_token_mining_execution_result");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_execution_token_mining_execution_result");
        Err(DispatchError::Other("No value for mining_speed_boosts_execution_token_mining_execution_result"))
    }

    /// Only push the execution id onto the end of the vector if it does not already exist
    pub fn associate_token_execution_with_configuration(
        mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given execution id
        if let Some(configuration_execution) =
            Self::token_mining_configuration_execution(mining_speed_boosts_configuration_token_mining_id)
        {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_speed_boosts_configuration_token_mining_id,
                configuration_execution
            );
            let not_configuration_contains_execution =
                !configuration_execution.contains(&mining_speed_boosts_execution_token_mining_id);
            ensure!(not_configuration_contains_execution, "Configuration already contains the given execution id");
            debug::info!("Configuration id key exists but its vector value does not contain the given execution id");
            <TokenMiningConfigurationExecution<T>>::mutate(mining_speed_boosts_configuration_token_mining_id, |v| {
                if let Some(value) = v {
                    value.push(mining_speed_boosts_execution_token_mining_id);
                }
            });
            debug::info!(
                "Associated execution {:?} with configuration {:?}",
                mining_speed_boosts_execution_token_mining_id,
                mining_speed_boosts_configuration_token_mining_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 execution id {:?} to its vector value",
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_execution_token_mining_id
            );
            <TokenMiningConfigurationExecution<T>>::insert(
                mining_speed_boosts_configuration_token_mining_id,
                &vec![mining_speed_boosts_execution_token_mining_id],
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

    fn next_mining_speed_boosts_execution_token_mining_id()
    -> Result<T::MiningSpeedBoostExecutionTokenMiningIndex, DispatchError> {
        let mining_speed_boosts_execution_token_mining_id = Self::mining_speed_boosts_execution_token_mining_count();
        if mining_speed_boosts_execution_token_mining_id ==
            <T::MiningSpeedBoostExecutionTokenMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostExecutionTokenMining count overflow"));
        }
        Ok(mining_speed_boosts_execution_token_mining_id)
    }

    fn insert_mining_speed_boosts_execution_token_mining(
        owner: &T::AccountId,
        mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
        mining_speed_boosts_execution_token_mining: MiningSpeedBoostExecutionTokenMining,
    ) {
        // Create and store mining mining_speed_boosts_execution_token_mining
        <MiningSpeedBoostExecutionTokenMinings<T>>::insert(
            mining_speed_boosts_execution_token_mining_id,
            mining_speed_boosts_execution_token_mining,
        );
        <MiningSpeedBoostExecutionTokenMiningCount<T>>::put(mining_speed_boosts_execution_token_mining_id + One::one());
        <MiningSpeedBoostExecutionTokenMiningOwners<T>>::insert(
            mining_speed_boosts_execution_token_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_execution_token_mining_id: T::MiningSpeedBoostExecutionTokenMiningIndex,
    ) {
        <MiningSpeedBoostExecutionTokenMiningOwners<T>>::insert(mining_speed_boosts_execution_token_mining_id, to);
    }
}
