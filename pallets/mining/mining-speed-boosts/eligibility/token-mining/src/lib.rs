#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_io::hashing::{blake2_128};
use sp_runtime::traits::{Bounded, Member, One, AtLeast32Bit};
use frame_support::traits::{Currency, ExistenceRequirement, Randomness};
/// A runtime module for managing non-fungible tokens
use frame_support::{decl_event, decl_module, decl_storage, ensure, Parameter, debug};
use system::ensure_signed;
use sp_runtime::DispatchError;
use sp_std::prelude::*; // Imports Vec

// FIXME - remove this, only used this approach since do not know how to use BalanceOf using only mining-speed-boosts runtime module
use roaming_operators;
use mining_speed_boosts_rates_token_mining;
use mining_speed_boosts_configuration_token_mining;
use mining_speed_boosts_sampling_token_mining;

/// The module's eligibilitys trait.
pub trait Trait: system::Trait +
    roaming_operators::Trait +
    mining_speed_boosts_rates_token_mining::Trait +
    mining_speed_boosts_configuration_token_mining::Trait +
    mining_speed_boosts_sampling_token_mining::Trait
  {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MiningSpeedBoostEligibilityTokenMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningSpeedBoostEligibilityTokenMiningDateAudited: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningSpeedBoostEligibilityTokenMiningAuditorAccountID: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
  }

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostEligibilityTokenMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostEligibilityTokenMiningEligibilityResult<U, V> {
    pub eligibility_token_mining_calculated_eligibility: U,
    pub eligibility_token_mining_token_locked_percentage: V,
    // pub eligibility_token_mining_date_audited: W,
    // pub eligibility_token_mining_auditor_account_id: X,
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostEligibilityTokenMiningIndex,
        <T as Trait>::MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility,
        <T as Trait>::MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage,
        // <T as Trait>::MiningSpeedBoostEligibilityTokenMiningDateAudited,
        // <T as Trait>::MiningSpeedBoostEligibilityTokenMiningAuditorAccountID,
        <T as mining_speed_boosts_configuration_token_mining::Trait>::MiningSpeedBoostConfigurationTokenMiningIndex,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_eligibility_token_mining is created. (owner, mining_speed_boosts_eligibility_token_mining_id)
        Created(AccountId, MiningSpeedBoostEligibilityTokenMiningIndex),
        /// A mining_speed_boosts_eligibility_token_mining is transferred. (from, to, mining_speed_boosts_eligibility_token_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostEligibilityTokenMiningIndex),
        // MiningSpeedBoostEligibilityTokenMiningEligibilityResultSet(
        //   AccountId, MiningSpeedBoostConfigurationTokenMiningIndex, MiningSpeedBoostEligibilityTokenMiningIndex,
        //   MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility, MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage,
        //   MiningSpeedBoostEligibilityTokenMiningDateAudited, MiningSpeedBoostEligibilityTokenMiningAuditorAccountID
        // ),
        MiningSpeedBoostEligibilityTokenMiningEligibilityResultSet(
          AccountId, MiningSpeedBoostConfigurationTokenMiningIndex, MiningSpeedBoostEligibilityTokenMiningIndex,
          MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility, MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage
          // MiningSpeedBoostEligibilityTokenMiningDateAudited, MiningSpeedBoostEligibilityTokenMiningAuditorAccountID
        ),
        /// A mining_speed_boosts_eligibility_token_mining is assigned to an mining_speed_boosts_configuration_token_mining.
        /// (owner of mining_speed_boosts_token_mining, mining_speed_boosts_eligibility_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
		    AssignedTokenMiningEligibilityToConfiguration(AccountId, MiningSpeedBoostEligibilityTokenMiningIndex, MiningSpeedBoostConfigurationTokenMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostEligibilityTokenMining {
        /// Stores all the mining_speed_boosts_eligibility_token_minings, key is the mining_speed_boosts_eligibility_token_mining id / index
        pub MiningSpeedBoostEligibilityTokenMinings get(fn mining_speed_boosts_eligibility_token_mining): map hasher(blake2_256) T::MiningSpeedBoostEligibilityTokenMiningIndex => Option<MiningSpeedBoostEligibilityTokenMining>;

        /// Stores the total number of mining_speed_boosts_eligibility_token_minings. i.e. the next mining_speed_boosts_eligibility_token_mining index
        pub MiningSpeedBoostEligibilityTokenMiningCount get(fn mining_speed_boosts_eligibility_token_mining_count): T::MiningSpeedBoostEligibilityTokenMiningIndex;

        /// Stores mining_speed_boosts_eligibility_token_mining owner
        pub MiningSpeedBoostEligibilityTokenMiningOwners get(fn mining_speed_boosts_eligibility_token_mining_owner): map hasher(blake2_256) T::MiningSpeedBoostEligibilityTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_eligibility_token_mining_result
        pub MiningSpeedBoostEligibilityTokenMiningEligibilityResults get(fn mining_speed_boosts_eligibility_token_mining_eligibility_results): map hasher(blake2_256) (T::MiningSpeedBoostConfigurationTokenMiningIndex, T::MiningSpeedBoostEligibilityTokenMiningIndex) =>
            Option<MiningSpeedBoostEligibilityTokenMiningEligibilityResult<
                T::MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility,
                T::MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage,
                // T::MiningSpeedBoostEligibilityTokenMiningDateAudited,
                // T::MiningSpeedBoostEligibilityTokenMiningAuditorAccountID,
            >>;

        /// Get mining_speed_boosts_configuration_token_mining_id belonging to a mining_speed_boosts_eligibility_token_mining_id
        pub TokenMiningEligibilityConfiguration get(fn token_mining_resulturation): map hasher(blake2_256) T::MiningSpeedBoostEligibilityTokenMiningIndex => Option<T::MiningSpeedBoostConfigurationTokenMiningIndex>;

        /// Get mining_speed_boosts_eligibility_token_mining_id's belonging to a mining_speed_boosts_configuration_token_mining_id
        pub TokenMiningConfigurationEligibilities get(fn token_mining_configuration_eligibilities): map hasher(blake2_256) T::MiningSpeedBoostConfigurationTokenMiningIndex => Option<Vec<T::MiningSpeedBoostEligibilityTokenMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_eligibility_token_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_eligibility_token_mining_id = Self::next_mining_speed_boosts_eligibility_token_mining_id()?;

            // Geneeligibility a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_eligibility_token_mining
            let mining_speed_boosts_eligibility_token_mining = MiningSpeedBoostEligibilityTokenMining(unique_id);
            Self::insert_mining_speed_boosts_eligibility_token_mining(&sender, mining_speed_boosts_eligibility_token_mining_id, mining_speed_boosts_eligibility_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_eligibility_token_mining_id));
        }

        /// Transfer a mining_speed_boosts_eligibility_token_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_eligibility_token_mining_owner(mining_speed_boosts_eligibility_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_eligibility_token_mining");

            Self::update_owner(&to, mining_speed_boosts_eligibility_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_eligibility_token_mining_id));
        }

        // FIXME - implement this and fix the type errors and uncomment it in the integration tests
        // /// Calculate mining_speed_boosts_eligibility_token_mining_result
        // pub fn calculate_mining_speed_boosts_eligibility_token_mining_result(
        //     origin,
        //     mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        //     mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
        // ) {
        //     let sender = ensure_signed(origin)?; 
            
        //     // Ensure that the mining_speed_boosts_eligibility_token_mining_id whose config we want to change actually exists
        //     let is_mining_speed_boosts_eligibility_token_mining = Self::exists_mining_speed_boosts_eligibility_token_mining(mining_speed_boosts_eligibility_token_mining_id).is_ok();
        //     ensure!(is_mining_speed_boosts_eligibility_token_mining, "MiningSpeedBoostEligibilityTokenMining does not exist");

        //     // Ensure that the caller is owner of the mining_speed_boosts_eligibility_token_mining_result they are trying to change
        //     ensure!(Self::mining_speed_boosts_eligibility_token_mining_owner(mining_speed_boosts_eligibility_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_eligibility_token_mining_result");

        //     let DEFAULT_RATE_CONFIG = 0; 
        //     let mut eligibility_token_mining_calculated_eligibility = 0.into();
        //     let mut part_eligibility_token_mining_calculated_eligibility = 0.into();
        //     let mut eligibility_token_mining_token_locked_percentage = 0.into();
        //     let mut token_token_max_token = 0.into();

        //     let mut current_token_type;
        //     let mut current_token_locked_amount;
        //     // Get the config associated with the given configuration_token_mining
        //     if let Some(configuration_token_mining_config) = <mining_speed_boosts_configuration_token_mining::Module<T>>::mining_speed_boosts_configuration_token_mining_token_configs(mining_speed_boosts_configuration_token_mining_id) {
        //       if let token_type = configuration_token_mining_config.token_type {
        //         if token_type != "".to_string() {
        //           current_token_type = token_type.clone();
  
        //           if let token_locked_amount = configuration_token_mining_config.token_locked_amount {
        //             if token_locked_amount != 0 {
        //               current_token_locked_amount = token_locked_amount;
  
        //               // Get list of all sampling_token_mining_ids that correspond to the given mining_speed_boosts_configuration_token_mining_id
        //               // of type MiningSpeedBoostSamplingTokenMiningIndex
        //               let sampling_token_mining_ids = <mining_speed_boosts_sampling_token_mining::Module<T>>
        //                 ::token_mining_configuration_samplings(mining_speed_boosts_configuration_token_mining_id);
      
        //               let mut sample_count = 0;
        //               let mut current_sample_tokens_locked = 0;
        //               let mut current_token_mining_rate = 0;
        //               let mut current_token_max_tokens = 0;
        //               let mut total = 0;
        //               // Iteratve through all the associated samples
        //               for (index, sampling_token_mining_id) in sampling_token_mining_ids.iter().enumerate() {
        //                 // Retrieve the current corresponding sampling_token_mining_config
        //                 // of type MiningSpeedBoostSamplingTokenMiningSamplingConfig
        //                 if let Some(current_sampling_token_mining_config) = <mining_speed_boosts_sampling_token_mining::Module<T>>::mining_speed_boosts_samplings_token_mining_samplings_configs(
        //                   (mining_speed_boosts_configuration_token_mining_id, sampling_token_mining_id)
        //                 ) {
        //                   if let tokens_locked = current_sampling_token_mining_config.token_sample_tokens_locked {
        //                     sample_count += 1;
  
        //                     if tokens_locked == 0 {
        //                       debug::info!("Mining rate sample has nothing locked. Skipping to next sampling.");
        //                       continue;
        //                     }
        //                     current_sample_tokens_locked = tokens_locked;
  
        //                     if let Some(token_mining_rates_config) = <mining_speed_boosts_rates_token_mining::Module<T>>::mining_speed_boosts_rates_token_mining_rates_configs(DEFAULT_RATE_CONFIG) {
                              
        //                       if current_token_type == "MXC".to_string() {
        //                         current_token_mining_rate = token_mining_rates_config.token_token_mxc;
        //                       } else if current_token_type == "IOTA".to_string() {
        //                         current_token_mining_rate = token_mining_rates_config.token_token_iota;
        //                       } else if current_token_type == "DOT".to_string() {
        //                         current_token_mining_rate = token_mining_rates_config.token_token_dot;
        //                       }
        //                       current_token_max_tokens = token_mining_rates_config.token_token_max_token;
        //                       eligibility_token_mining_token_locked_percentage = current_token_mining_rate * (current_sample_tokens_locked / current_token_locked_amount);
                              
        //                       part_eligibility_token_mining_calculated_eligibility = part_eligibility_token_mining_calculated_eligibility + eligibility_token_mining_token_locked_percentage * current_token_max_tokens;
        //                     } else {
        //                       debug::info!("Mining rate config missing");
        //                       // break;
        //                       return Err(DispatchError::Other("Mining rate config missing"));
        //                     }              
        //                   }
        //                 }
        //               }
        //               eligibility_token_mining_calculated_eligibility = part_eligibility_token_mining_calculated_eligibility / sample_count;
        //               debug::info!("Calculate eligibilty based on average {:#?}", eligibility_token_mining_calculated_eligibility);
        //             }
        //           }
        //         }
        //       }
        //     }

        //     // Check if a mining_speed_boosts_eligibility_token_mining_result already exists with the given mining_speed_boosts_eligibility_token_mining_id
        //     // to determine whether to insert new or mutate existing.
        //     if Self::has_value_for_mining_speed_boosts_eligibility_token_mining_result_index(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id).is_ok() {
        //         debug::info!("Mutating values");
        //         <MiningSpeedBoostEligibilityTokenMiningEligibilityResults<T>>::mutate((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id), |mining_speed_boosts_eligibility_token_mining_result| {
        //             if let Some(_mining_speed_boosts_eligibility_token_mining_result) = mining_speed_boosts_eligibility_token_mining_result {
        //                 // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
        //                 _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_calculated_eligibility = eligibility_token_mining_calculated_eligibility.clone();
        //                 _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_token_locked_percentage = eligibility_token_mining_token_locked_percentage.clone();
        //                 // _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_date_audited = eligibility_token_mining_date_audited.clone();
        //                 // _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_auditor_account_id = eligibility_token_mining_auditor_account_id.clone();
        //             }
        //         });
        //         debug::info!("Checking mutated values");
        //         let fetched_mining_speed_boosts_eligibility_token_mining_result = <MiningSpeedBoostEligibilityTokenMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id));
        //         if let Some(_mining_speed_boosts_eligibility_token_mining_result) = fetched_mining_speed_boosts_eligibility_token_mining_result {
        //             debug::info!("Latest field eligibility_token_mining_calculated_eligibility {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_calculated_eligibility);
        //             debug::info!("Latest field eligibility_token_mining_token_locked_percentage {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_token_locked_percentage);
        //             // debug::info!("Latest field eligibility_token_mining_date_audited {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_date_audited);
        //             // debug::info!("Latest field eligibility_token_mining_auditor_account_id {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_auditor_account_id);
        //         }
        //     } else {
        //         debug::info!("Inserting values");

        //         // Create a new mining mining_speed_boosts_eligibility_token_mining_result instance with the input params
        //         let mining_speed_boosts_eligibility_token_mining_result_instance = MiningSpeedBoostEligibilityTokenMiningEligibilityResult {
        //             // Since each parameter passed into the function is optional (i.e. `Option`)
        //             // we will assign a default value if a parameter value is not provided.
        //             eligibility_token_mining_calculated_eligibility: eligibility_token_mining_calculated_eligibility.clone(),
        //             eligibility_token_mining_token_locked_percentage: eligibility_token_mining_token_locked_percentage.clone(),
        //             // eligibility_token_mining_date_audited: eligibility_token_mining_date_audited.clone(),
        //             // eligibility_token_mining_auditor_account_id: eligibility_token_mining_auditor_account_id.clone(),
        //         };

        //         <MiningSpeedBoostEligibilityTokenMiningEligibilityResults<T>>::insert(
        //             (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id),
        //             &mining_speed_boosts_eligibility_token_mining_result_instance
        //         );

        //         debug::info!("Checking inserted values");
        //         let fetched_mining_speed_boosts_eligibility_token_mining_result = <MiningSpeedBoostEligibilityTokenMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id));
        //         if let Some(_mining_speed_boosts_eligibility_token_mining_result) = fetched_mining_speed_boosts_eligibility_token_mining_result {
        //             debug::info!("Inserted field eligibility_token_mining_calculated_eligibility {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_calculated_eligibility);
        //             debug::info!("Inserted field eligibility_token_mining_token_locked_percentage {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_token_locked_percentage);
        //             // debug::info!("Inserted field eligibility_token_mining_date_audited {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_date_audited);
        //             // debug::info!("Inserted field eligibility_token_mining_auditor_account_id {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_auditor_account_id);
        //         }
        //     }

        //     Self::deposit_event(RawEvent::MiningSpeedBoostEligibilityTokenMiningEligibilityResultSet(
        //       sender,
        //       mining_speed_boosts_configuration_token_mining_id,
        //       mining_speed_boosts_eligibility_token_mining_id,
        //       eligibility_token_mining_calculated_eligibility,
        //       eligibility_token_mining_token_locked_percentage,
        //       // eligibility_token_mining_date_audited,
        //       // eligibility_token_mining_auditor_account_id
        //     ));
        // }

        /// Set mining_speed_boosts_eligibility_token_mining_result
        pub fn set_mining_speed_boosts_eligibility_token_mining_eligibility_result(
            origin,
            mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
            mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
            _eligibility_token_mining_calculated_eligibility: Option<T::MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility>,
            _eligibility_token_mining_token_locked_percentage: Option<T::MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage>,
            // _eligibility_token_mining_date_audited: Option<T::MiningSpeedBoostEligibilityTokenMiningDateAudited>,
            // _eligibility_token_mining_auditor_account_id: Option<T::MiningSpeedBoostEligibilityTokenMiningAuditorAccountID>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_eligibility_token_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_eligibility_token_mining = Self::exists_mining_speed_boosts_eligibility_token_mining(mining_speed_boosts_eligibility_token_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_eligibility_token_mining, "MiningSpeedBoostEligibilityTokenMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_eligibility_token_mining_result they are trying to change
            ensure!(Self::mining_speed_boosts_eligibility_token_mining_owner(mining_speed_boosts_eligibility_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_eligibility_token_mining_result");

            // TODO - adjust default eligibilitys
            let eligibility_token_mining_calculated_eligibility = match _eligibility_token_mining_calculated_eligibility.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let eligibility_token_mining_token_locked_percentage = match _eligibility_token_mining_token_locked_percentage {
                Some(value) => value,
                None => 1.into() // Default
            };
            // let eligibility_token_mining_date_audited = match _eligibility_token_mining_date_audited {
            //   Some(value) => value,
            //   None => 1.into() // Default
            // };
            // let eligibility_token_mining_auditor_account_id = match _eligibility_token_mining_auditor_account_id {
            //   Some(value) => value,
            //   None => 1.into() // Default
            // };

            // Check if a mining_speed_boosts_eligibility_token_mining_result already exists with the given mining_speed_boosts_eligibility_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_eligibility_token_mining_result_index(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostEligibilityTokenMiningEligibilityResults<T>>::mutate((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id), |mining_speed_boosts_eligibility_token_mining_result| {
                    if let Some(_mining_speed_boosts_eligibility_token_mining_result) = mining_speed_boosts_eligibility_token_mining_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_calculated_eligibility = eligibility_token_mining_calculated_eligibility.clone();
                        _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_token_locked_percentage = eligibility_token_mining_token_locked_percentage.clone();
                        // _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_date_audited = eligibility_token_mining_date_audited.clone();
                        // _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_auditor_account_id = eligibility_token_mining_auditor_account_id.clone();
                    }
                });

                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_eligibility_token_mining_result = <MiningSpeedBoostEligibilityTokenMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id));
                if let Some(_mining_speed_boosts_eligibility_token_mining_result) = fetched_mining_speed_boosts_eligibility_token_mining_result {
                    debug::info!("Latest field eligibility_token_mining_calculated_eligibility {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_calculated_eligibility);
                    debug::info!("Latest field eligibility_token_mining_token_locked_percentage {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_token_locked_percentage);
                    // debug::info!("Latest field eligibility_token_mining_date_audited {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_date_audited);
                    // debug::info!("Latest field eligibility_token_mining_auditor_account_id {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_auditor_account_id);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_eligibility_token_mining_result instance with the input params
                let mining_speed_boosts_eligibility_token_mining_result_instance = MiningSpeedBoostEligibilityTokenMiningEligibilityResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    eligibility_token_mining_calculated_eligibility: eligibility_token_mining_calculated_eligibility.clone(),
                    eligibility_token_mining_token_locked_percentage: eligibility_token_mining_token_locked_percentage.clone(),
                    // eligibility_token_mining_date_audited: eligibility_token_mining_date_audited.clone(),
                    // eligibility_token_mining_auditor_account_id: eligibility_token_mining_auditor_account_id.clone(),
                };

                <MiningSpeedBoostEligibilityTokenMiningEligibilityResults<T>>::insert(
                    (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id),
                    &mining_speed_boosts_eligibility_token_mining_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_eligibility_token_mining_result = <MiningSpeedBoostEligibilityTokenMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id));
                if let Some(_mining_speed_boosts_eligibility_token_mining_result) = fetched_mining_speed_boosts_eligibility_token_mining_result {
                    debug::info!("Inserted field eligibility_token_mining_calculated_eligibility {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_calculated_eligibility);
                    debug::info!("Inserted field eligibility_token_mining_token_locked_percentage {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_token_locked_percentage);
                    // debug::info!("Inserted field eligibility_token_mining_date_audited {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_date_audited);
                    // debug::info!("Inserted field eligibility_token_mining_auditor_account_id {:#?}", _mining_speed_boosts_eligibility_token_mining_result.eligibility_token_mining_auditor_account_id);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostEligibilityTokenMiningEligibilityResultSet(
                sender,
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_eligibility_token_mining_id,
                eligibility_token_mining_calculated_eligibility,
                eligibility_token_mining_token_locked_percentage,
                // eligibility_token_mining_date_audited,
                // eligibility_token_mining_auditor_account_id
            ));
        }

        pub fn assign_eligibility_to_configuration(
          origin,
          mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
          mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token_mining = <mining_speed_boosts_configuration_token_mining::Module<T>>
                ::exists_mining_speed_boosts_configuration_token_mining(mining_speed_boosts_configuration_token_mining_id).is_ok();
            ensure!(is_configuration_token_mining, "configuration_token_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the eligibility to
            ensure!(
                <mining_speed_boosts_configuration_token_mining::Module<T>>::is_mining_speed_boosts_configuration_token_mining_owner(mining_speed_boosts_configuration_token_mining_id, sender.clone()).is_ok(),
                "Only the configuration_token_mining owner can assign itself a eligibility"
            );

            Self::associate_token_eligibility_with_configuration(mining_speed_boosts_eligibility_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
                .expect("Unable to associate eligibility with configuration");

            // Ensure that the given mining_speed_boosts_eligibility_token_mining_id already exists
            let token_eligibility = Self::mining_speed_boosts_eligibility_token_mining(mining_speed_boosts_eligibility_token_mining_id);
            ensure!(token_eligibility.is_some(), "Invalid mining_speed_boosts_eligibility_token_mining_id");

            // // Ensure that the eligibility is not already owned by a different configuration
            // // Unassign the eligibility from any existing configuration since it may only be owned by one configuration
            // <TokenMiningEligibilityConfiguration<T>>::remove(mining_speed_boosts_eligibility_token_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenMiningEligibilityConfiguration<T>>::insert(mining_speed_boosts_eligibility_token_mining_id, mining_speed_boosts_configuration_token_mining_id);

            Self::deposit_event(RawEvent::AssignedTokenMiningEligibilityToConfiguration(sender, mining_speed_boosts_eligibility_token_mining_id, mining_speed_boosts_configuration_token_mining_id));
		    }
    }
}

impl<T: Trait> Module<T> {
	pub fn is_mining_speed_boosts_eligibility_token_mining_owner(mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex, sender: T::AccountId) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_eligibility_token_mining_owner(&mining_speed_boosts_eligibility_token_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostEligibilityTokenMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_eligibility_token_mining(mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex) -> Result<MiningSpeedBoostEligibilityTokenMining, DispatchError> {
        match Self::mining_speed_boosts_eligibility_token_mining(mining_speed_boosts_eligibility_token_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostEligibilityTokenMining does not exist"))
        }
    }

    pub fn exists_mining_speed_boosts_eligibility_token_mining_result(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_eligibility_token_mining_eligibility_results(
          (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id)
        ) {
            Some(value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostEligibilityTokenMiningEligibilityResult does not exist"))
        }
    }

    pub fn has_value_for_mining_speed_boosts_eligibility_token_mining_result_index(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_speed_boosts_eligibility_token_mining_result has a value that is defined");
        let fetched_mining_speed_boosts_eligibility_token_mining_result = <MiningSpeedBoostEligibilityTokenMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id));
        if let Some(value) = fetched_mining_speed_boosts_eligibility_token_mining_result {
            debug::info!("Found value for mining_speed_boosts_eligibility_token_mining_result");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_eligibility_token_mining_result");
        Err(DispatchError::Other("No value for mining_speed_boosts_eligibility_token_mining_result"))
    }

    /// Only push the eligibility id onto the end of the vector if it does not already exist
    pub fn associate_token_eligibility_with_configuration(
        mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex
    ) -> Result<(), DispatchError>
    {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given eligibility id
        if let Some(configuration_eligibilities) = Self::token_mining_configuration_eligibilities(mining_speed_boosts_configuration_token_mining_id) {
            debug::info!("Configuration id key {:?} exists with value {:?}", mining_speed_boosts_configuration_token_mining_id, configuration_eligibilities);
            let not_configuration_contains_eligibility = !configuration_eligibilities.contains(&mining_speed_boosts_eligibility_token_mining_id);
            ensure!(not_configuration_contains_eligibility, "Configuration already contains the given eligibility id");
            debug::info!("Configuration id key exists but its vector value does not contain the given eligibility id");
            <TokenMiningConfigurationEligibilities<T>>::mutate(mining_speed_boosts_configuration_token_mining_id, |v| {
                if let Some(value) = v {
                    value.push(mining_speed_boosts_eligibility_token_mining_id);
                }
            });
            debug::info!("Associated eligibility {:?} with configuration {:?}", mining_speed_boosts_eligibility_token_mining_id, mining_speed_boosts_configuration_token_mining_id);
            Ok(())
        } else {
            debug::info!("Configuration id key does not yet exist. Creating the configuration key {:?} and appending the eligibility id {:?} to its vector value", mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id);
            <TokenMiningConfigurationEligibilities<T>>::insert(mining_speed_boosts_configuration_token_mining_id, &vec![mining_speed_boosts_eligibility_token_mining_id]);
            Ok(())
        }
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <system::Module<T>>::extrinsic_index(),
            <system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_mining_speed_boosts_eligibility_token_mining_id() -> Result<T::MiningSpeedBoostEligibilityTokenMiningIndex, DispatchError> {
        let mining_speed_boosts_eligibility_token_mining_id = Self::mining_speed_boosts_eligibility_token_mining_count();
        if mining_speed_boosts_eligibility_token_mining_id == <T::MiningSpeedBoostEligibilityTokenMiningIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningSpeedBoostEligibilityTokenMining count overflow"));
        }
        Ok(mining_speed_boosts_eligibility_token_mining_id)
    }

    fn insert_mining_speed_boosts_eligibility_token_mining(owner: &T::AccountId, mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex, mining_speed_boosts_eligibility_token_mining: MiningSpeedBoostEligibilityTokenMining) {
        // Create and store mining mining_speed_boosts_eligibility_token_mining
        <MiningSpeedBoostEligibilityTokenMinings<T>>::insert(mining_speed_boosts_eligibility_token_mining_id, mining_speed_boosts_eligibility_token_mining);
        <MiningSpeedBoostEligibilityTokenMiningCount<T>>::put(mining_speed_boosts_eligibility_token_mining_id + One::one());
        <MiningSpeedBoostEligibilityTokenMiningOwners<T>>::insert(mining_speed_boosts_eligibility_token_mining_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex) {
        <MiningSpeedBoostEligibilityTokenMiningOwners<T>>::insert(mining_speed_boosts_eligibility_token_mining_id, to);
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use sp_core::H256;
    use frame_support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
    use sp_runtime::{
      traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ();
        type CreationFee = ();
    }
    impl transaction_payment::Trait for Test {
        type Currency = Balances;
        type OnTransactionPayment = ();
        type TransactionBaseFee = ();
        type TransactionByteFee = ();
        type WeightToFee = ();
        type FeeMultiplierUpdate = ();
    }
    // FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
    impl roaming_operators::Trait for Test {
        type Event = ();
        type Currency = Balances;
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl mining_speed_boosts_rates_token_mining::Trait for Test {
      type Event = ();
      type MiningSpeedBoostRatesTokenMiningIndex = u64;
      type MiningSpeedBoostRatesTokenMiningTokenMXC = u32;
      type MiningSpeedBoostRatesTokenMiningTokenIOTA = u32;
      type MiningSpeedBoostRatesTokenMiningTokenDOT = u32;
      type MiningSpeedBoostRatesTokenMiningMaxToken = u32;
      type MiningSpeedBoostRatesTokenMiningMaxLoyalty = u32;
    }
    impl mining_speed_boosts_sampling_token_mining::Trait for Test {
      type Event = ();
      type MiningSpeedBoostSamplingTokenMiningIndex = u64;
      type MiningSpeedBoostSamplingTokenMiningSampleDate = u64;
      type MiningSpeedBoostSamplingTokenMiningSampleTokensLocked = u64;
    }
    impl mining_speed_boosts_configuration_token_mining::Trait for Test {
      type Event = ();
      // FIXME - restore when stop temporarily using roaming-operators
      // type Currency = Balances;
      // type Randomness = RandomnessCollectiveFlip;
      type MiningSpeedBoostConfigurationTokenMiningIndex = u64;
      // Mining Speed Boost Token Mining Config
      // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
      type MiningSpeedBoostConfigurationTokenMiningTokenType = Vec<u8>;
      // type MiningSpeedBoostConfigurationTokenMiningTokenType = MiningSpeedBoostConfigurationTokenMiningTokenTypes;
      type MiningSpeedBoostConfigurationTokenMiningTokenLockedAmount = u64;
      type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod = u32;
      type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate = u64;
      type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate = u64;
    }
    impl Trait for Test {
        type Event = ();
        type MiningSpeedBoostEligibilityTokenMiningIndex = u64;
        type MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility = u64;
        type MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage = u32;
        // type MiningSpeedBoostEligibilityTokenMiningDateAudited = u64;
        // type MiningSpeedBoostEligibilityTokenMiningAuditorAccountID = u64;
    }
    //type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type MiningSpeedBoostEligibilityTokenMiningTestModule = Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }
}
