#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_io::hashing::{blake2_128};
use sp_runtime::traits::{Bounded, Member, One, SimpleArithmetic};
use frame_support::traits::{Currency, ExistenceRequirement, Randomness};
/// A runtime module for managing non-fungible tokens
use frame_support::{decl_event, decl_module, decl_storage, ensure, Parameter, debug};
use system::ensure_signed;
use sp_std::prelude::*; // Imports Vec

// FIXME - remove this, only used this approach since do not know how to use BalanceOf using only mining-speed-boosts runtime module
use roaming_operators;
use mining_speed_boosts_rates_hardware_mining;
use mining_speed_boosts_configuration_hardware_mining;
use mining_speed_boosts_sampling_hardware_mining;

/// The module's eligibilitys trait.
pub trait Trait: system::Trait +
    roaming_operators::Trait +
    mining_speed_boosts_rates_hardware_mining::Trait +
    mining_speed_boosts_configuration_hardware_mining::Trait +
    mining_speed_boosts_sampling_hardware_mining::Trait
  {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MiningSpeedBoostEligibilityHardwareMiningIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    type MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    type MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    // type MiningSpeedBoostEligibilityHardwareMiningDateAudited: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    // type MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
  }

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostEligibilityHardwareMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostEligibilityHardwareMiningEligibilityResult<U, V> {
    pub eligibility_hardware_mining_calculated_eligibility: U,
    pub eligibility_hardware_mining_hardware_uptime_percentage: V,
    // pub eligibility_hardware_mining_date_audited: W,
    // pub eligibility_hardware_mining_auditor_account_id: X,
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostEligibilityHardwareMiningIndex,
        <T as Trait>::MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility,
        <T as Trait>::MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage,
        // <T as Trait>::MiningSpeedBoostEligibilityHardwareMiningDateAudited,
        // <T as Trait>::MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID,
        <T as mining_speed_boosts_configuration_hardware_mining::Trait>::MiningSpeedBoostConfigurationHardwareMiningIndex,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_eligibility_hardware_mining is created. (owner, mining_speed_boosts_eligibility_hardware_mining_id)
        Created(AccountId, MiningSpeedBoostEligibilityHardwareMiningIndex),
        /// A mining_speed_boosts_eligibility_hardware_mining is transferred. (from, to, mining_speed_boosts_eligibility_hardware_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostEligibilityHardwareMiningIndex),
        // MiningSpeedBoostEligibilityHardwareMiningEligibilityResultSet(
        //   AccountId, MiningSpeedBoostConfigurationHardwareMiningIndex, MiningSpeedBoostEligibilityHardwareMiningIndex,
        //   MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility, MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage,
        //   MiningSpeedBoostEligibilityHardwareMiningDateAudited, MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID
        // ),
        MiningSpeedBoostEligibilityHardwareMiningEligibilityResultSet(
          AccountId, MiningSpeedBoostConfigurationHardwareMiningIndex, MiningSpeedBoostEligibilityHardwareMiningIndex,
          MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility, MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage
          // MiningSpeedBoostEligibilityHardwareMiningDateAudited, MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID
        ),
        /// A mining_speed_boosts_eligibility_hardware_mining is assigned to an mining_speed_boosts_configuration_hardware_mining.
        /// (owner of mining_speed_boosts_hardware_mining, mining_speed_boosts_eligibility_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id)
		    AssignedHardwareMiningEligibilityToConfiguration(AccountId, MiningSpeedBoostEligibilityHardwareMiningIndex, MiningSpeedBoostConfigurationHardwareMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostEligibilityHardwareMining {
        /// Stores all the mining_speed_boosts_eligibility_hardware_minings, key is the mining_speed_boosts_eligibility_hardware_mining id / index
        pub MiningSpeedBoostEligibilityHardwareMinings get(fn mining_speed_boosts_eligibility_hardware_mining): map hasher(blake2_256) T::MiningSpeedBoostEligibilityHardwareMiningIndex => Option<MiningSpeedBoostEligibilityHardwareMining>;

        /// Stores the total number of mining_speed_boosts_eligibility_hardware_minings. i.e. the next mining_speed_boosts_eligibility_hardware_mining index
        pub MiningSpeedBoostEligibilityHardwareMiningCount get(fn mining_speed_boosts_eligibility_hardware_mining_count): T::MiningSpeedBoostEligibilityHardwareMiningIndex;

        /// Stores mining_speed_boosts_eligibility_hardware_mining owner
        pub MiningSpeedBoostEligibilityHardwareMiningOwners get(fn mining_speed_boosts_eligibility_hardware_mining_owner): map hasher(blake2_256) T::MiningSpeedBoostEligibilityHardwareMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_eligibility_hardware_mining_result
        pub MiningSpeedBoostEligibilityHardwareMiningEligibilityResults get(fn mining_speed_boosts_eligibility_hardware_mining_eligibility_results): map (T::MiningSpeedBoostConfigurationHardwareMiningIndex, T::MiningSpeedBoostEligibilityHardwareMiningIndex) =>
            Option<MiningSpeedBoostEligibilityHardwareMiningEligibilityResult<
                T::MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility,
                T::MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage,
                // T::MiningSpeedBoostEligibilityHardwareMiningDateAudited,
                // T::MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID,
            >>;

        /// Get mining_speed_boosts_configuration_hardware_mining_id belonging to a mining_speed_boosts_eligibility_hardware_mining_id
        pub HardwareMiningEligibilityConfiguration get(fn hardware_mining_resulturation): map hasher(blake2_256) T::MiningSpeedBoostEligibilityHardwareMiningIndex => Option<T::MiningSpeedBoostConfigurationHardwareMiningIndex>;

        /// Get mining_speed_boosts_eligibility_hardware_mining_id's belonging to a mining_speed_boosts_configuration_hardware_mining_id
        pub HardwareMiningConfigurationEligibilities get(fn hardware_mining_configuration_eligibilities): map hasher(blake2_256) T::MiningSpeedBoostConfigurationHardwareMiningIndex => Option<Vec<T::MiningSpeedBoostEligibilityHardwareMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_eligibility_hardware_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_eligibility_hardware_mining_id = Self::next_mining_speed_boosts_eligibility_hardware_mining_id()?;

            // Geneeligibility a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_eligibility_hardware_mining
            let mining_speed_boosts_eligibility_hardware_mining = MiningSpeedBoostEligibilityHardwareMining(unique_id);
            Self::insert_mining_speed_boosts_eligibility_hardware_mining(&sender, mining_speed_boosts_eligibility_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_eligibility_hardware_mining_id));
        }

        /// Transfer a mining_speed_boosts_eligibility_hardware_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_eligibility_hardware_mining_owner(mining_speed_boosts_eligibility_hardware_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_eligibility_hardware_mining");

            Self::update_owner(&to, mining_speed_boosts_eligibility_hardware_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_eligibility_hardware_mining_id));
        }

        // FIXME - implement this and fix the type errors and uncomment it in the integration tests
        // /// Calculate mining_speed_boosts_eligibility_hardware_mining_result
        // pub fn calculate_mining_speed_boosts_eligibility_hardware_mining_result(
        //     origin,
        //     mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
        //     mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex,
        // ) {
        //     let sender = ensure_signed(origin)?; 
            
        //     // Ensure that the mining_speed_boosts_eligibility_hardware_mining_id whose config we want to change actually exists
        //     let is_mining_speed_boosts_eligibility_hardware_mining = Self::exists_mining_speed_boosts_eligibility_hardware_mining(mining_speed_boosts_eligibility_hardware_mining_id).is_ok();
        //     ensure!(is_mining_speed_boosts_eligibility_hardware_mining, "MiningSpeedBoostEligibilityHardwareMining does not exist");

        //     // Ensure that the caller is owner of the mining_speed_boosts_eligibility_hardware_mining_result they are trying to change
        //     ensure!(Self::mining_speed_boosts_eligibility_hardware_mining_owner(mining_speed_boosts_eligibility_hardware_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_eligibility_hardware_mining_result");

        //     let DEFAULT_RATE_CONFIG = 0; 
        //     let mut eligibility_hardware_mining_calculated_eligibility = 0.into();
        //     let mut part_eligibility_hardware_mining_calculated_eligibility = 0.into();
        //     let mut eligibility_hardware_mining_hardware_uptime_percentage = 0.into();
        //     let mut token_token_max_token = 0.into();

        //     let mut current_token_type;
        //     let mut current_hardware_uptime_amount;
        //     // Get the config associated with the given configuration_hardware_mining
        //     if let Some(configuration_hardware_mining_config) = <mining_speed_boosts_configuration_hardware_mining::Module<T>>::mining_speed_boosts_configuration_hardware_mining_token_configs(mining_speed_boosts_configuration_hardware_mining_id) {
        //       if let token_type = configuration_hardware_mining_config.token_type {
        //         if token_type != "".to_string() {
        //           current_token_type = token_type.clone();
  
        //           if let hardware_uptime_amount = configuration_hardware_mining_config.hardware_uptime_amount {
        //             if hardware_uptime_amount != 0 {
        //               current_hardware_uptime_amount = hardware_uptime_amount;
  
        //               // Get list of all sampling_hardware_mining_ids that correspond to the given mining_speed_boosts_configuration_hardware_mining_id
        //               // of type MiningSpeedBoostSamplingHardwareMiningIndex
        //               let sampling_hardware_mining_ids = <mining_speed_boosts_sampling_hardware_mining::Module<T>>
        //                 ::hardware_mining_configuration_samplings(mining_speed_boosts_configuration_hardware_mining_id);
      
        //               let mut sample_count = 0;
        //               let mut current_sample_tokens_locked = 0;
        //               let mut current_hardware_mining_rate = 0;
        //               let mut current_token_max_tokens = 0;
        //               let mut total = 0;
        //               // Iteratve through all the associated samples
        //               for (index, sampling_hardware_mining_id) in sampling_hardware_mining_ids.iter().enumerate() {
        //                 // Retrieve the current corresponding sampling_hardware_mining_config
        //                 // of type MiningSpeedBoostSamplingHardwareMiningSamplingConfig
        //                 if let Some(current_sampling_hardware_mining_config) = <mining_speed_boosts_sampling_hardware_mining::Module<T>>::mining_speed_boosts_samplings_hardware_mining_samplings_configs(
        //                   (mining_speed_boosts_configuration_hardware_mining_id, sampling_hardware_mining_id)
        //                 ) {
        //                   if let tokens_locked = current_sampling_hardware_mining_config.token_sample_tokens_locked {
        //                     sample_count += 1;
  
        //                     if tokens_locked == 0 {
        //                       debug::info!("Mining rate sample has nothing locked. Skipping to next sampling.");
        //                       continue;
        //                     }
        //                     current_sample_tokens_locked = tokens_locked;
  
        //                     if let Some(hardware_mining_rates_config) = <mining_speed_boosts_rates_hardware_mining::Module<T>>::mining_speed_boosts_rates_hardware_mining_rates_configs(DEFAULT_RATE_CONFIG) {
                              
        //                       if current_token_type == "MXC".to_string() {
        //                         current_hardware_mining_rate = hardware_mining_rates_config.token_token_mxc;
        //                       } else if current_token_type == "IOTA".to_string() {
        //                         current_hardware_mining_rate = hardware_mining_rates_config.token_token_iota;
        //                       } else if current_token_type == "DOT".to_string() {
        //                         current_hardware_mining_rate = hardware_mining_rates_config.token_token_dot;
        //                       }
        //                       current_token_max_tokens = hardware_mining_rates_config.token_token_max_token;
        //                       eligibility_hardware_mining_hardware_uptime_percentage = current_hardware_mining_rate * (current_sample_tokens_locked / current_hardware_uptime_amount);
                              
        //                       part_eligibility_hardware_mining_calculated_eligibility = part_eligibility_hardware_mining_calculated_eligibility + eligibility_hardware_mining_hardware_uptime_percentage * current_token_max_tokens;
        //                     } else {
        //                       debug::info!("Mining rate config missing");
        //                       break;
        //                       return Err("Mining rate config missing");
        //                     }                
        //                   }
        //                 }
        //               }
        //               eligibility_hardware_mining_calculated_eligibility = part_eligibility_hardware_mining_calculated_eligibility / sample_count;
        //               debug::info!("Calculate eligibilty based on average {:#?}", eligibility_hardware_mining_calculated_eligibility);
        //             }
        //           }
        //         }
        //       }
        //     }

        //     // Check if a mining_speed_boosts_eligibility_hardware_mining_result already exists with the given mining_speed_boosts_eligibility_hardware_mining_id
        //     // to determine whether to insert new or mutate existing.
        //     if Self::has_value_for_mining_speed_boosts_eligibility_hardware_mining_result_index(mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id).is_ok() {
        //         debug::info!("Mutating values");
        //         <MiningSpeedBoostEligibilityHardwareMiningEligibilityResults<T>>::mutate((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id), |mining_speed_boosts_eligibility_hardware_mining_result| {
        //             if let Some(_mining_speed_boosts_eligibility_hardware_mining_result) = mining_speed_boosts_eligibility_hardware_mining_result {
        //                 // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
        //                 _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_calculated_eligibility = eligibility_hardware_mining_calculated_eligibility.clone();
        //                 _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_hardware_uptime_percentage = eligibility_hardware_mining_hardware_uptime_percentage.clone();
        //                 // _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_date_audited = eligibility_hardware_mining_date_audited.clone();
        //                 // _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_auditor_account_id = eligibility_hardware_mining_auditor_account_id.clone();
        //             }
        //         });
        //         debug::info!("Checking mutated values");
        //         let fetched_mining_speed_boosts_eligibility_hardware_mining_result = <MiningSpeedBoostEligibilityHardwareMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id));
        //         if let Some(_mining_speed_boosts_eligibility_hardware_mining_result) = fetched_mining_speed_boosts_eligibility_hardware_mining_result {
        //             debug::info!("Latest field eligibility_hardware_mining_calculated_eligibility {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_calculated_eligibility);
        //             debug::info!("Latest field eligibility_hardware_mining_hardware_uptime_percentage {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_hardware_uptime_percentage);
        //             // debug::info!("Latest field eligibility_hardware_mining_date_audited {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_date_audited);
        //             // debug::info!("Latest field eligibility_hardware_mining_auditor_account_id {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_auditor_account_id);
        //         }
        //     } else {
        //         debug::info!("Inserting values");

        //         // Create a new mining mining_speed_boosts_eligibility_hardware_mining_result instance with the input params
        //         let mining_speed_boosts_eligibility_hardware_mining_result_instance = MiningSpeedBoostEligibilityHardwareMiningEligibilityResult {
        //             // Since each parameter passed into the function is optional (i.e. `Option`)
        //             // we will assign a default value if a parameter value is not provided.
        //             eligibility_hardware_mining_calculated_eligibility: eligibility_hardware_mining_calculated_eligibility.clone(),
        //             eligibility_hardware_mining_hardware_uptime_percentage: eligibility_hardware_mining_hardware_uptime_percentage.clone(),
        //             // eligibility_hardware_mining_date_audited: eligibility_hardware_mining_date_audited.clone(),
        //             // eligibility_hardware_mining_auditor_account_id: eligibility_hardware_mining_auditor_account_id.clone(),
        //         };

        //         <MiningSpeedBoostEligibilityHardwareMiningEligibilityResults<T>>::insert(
        //             (mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id),
        //             &mining_speed_boosts_eligibility_hardware_mining_result_instance
        //         );

        //         debug::info!("Checking inserted values");
        //         let fetched_mining_speed_boosts_eligibility_hardware_mining_result = <MiningSpeedBoostEligibilityHardwareMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id));
        //         if let Some(_mining_speed_boosts_eligibility_hardware_mining_result) = fetched_mining_speed_boosts_eligibility_hardware_mining_result {
        //             debug::info!("Inserted field eligibility_hardware_mining_calculated_eligibility {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_calculated_eligibility);
        //             debug::info!("Inserted field eligibility_hardware_mining_hardware_uptime_percentage {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_hardware_uptime_percentage);
        //             // debug::info!("Inserted field eligibility_hardware_mining_date_audited {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_date_audited);
        //             // debug::info!("Inserted field eligibility_hardware_mining_auditor_account_id {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_auditor_account_id);
        //         }
        //     }

        //     Self::deposit_event(RawEvent::MiningSpeedBoostEligibilityHardwareMiningEligibilityResultSet(
        //       sender,
        //       mining_speed_boosts_configuration_hardware_mining_id,
        //       mining_speed_boosts_eligibility_hardware_mining_id,
        //       eligibility_hardware_mining_calculated_eligibility,
        //       eligibility_hardware_mining_hardware_uptime_percentage,
        //       // eligibility_hardware_mining_date_audited,
        //       // eligibility_hardware_mining_auditor_account_id
        //     ));
        // }

        /// Set mining_speed_boosts_eligibility_hardware_mining_result
        pub fn set_mining_speed_boosts_eligibility_hardware_mining_eligibility_result(
            origin,
            mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
            mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex,
            _eligibility_hardware_mining_calculated_eligibility: Option<T::MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility>,
            _eligibility_hardware_mining_hardware_uptime_percentage: Option<T::MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage>,
            // _eligibility_hardware_mining_date_audited: Option<T::MiningSpeedBoostEligibilityHardwareMiningDateAudited>,
            // _eligibility_hardware_mining_auditor_account_id: Option<T::MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_eligibility_hardware_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_eligibility_hardware_mining = Self::exists_mining_speed_boosts_eligibility_hardware_mining(mining_speed_boosts_eligibility_hardware_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_eligibility_hardware_mining, "MiningSpeedBoostEligibilityHardwareMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_eligibility_hardware_mining_result they are trying to change
            ensure!(Self::mining_speed_boosts_eligibility_hardware_mining_owner(mining_speed_boosts_eligibility_hardware_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_eligibility_hardware_mining_result");

            // TODO - adjust default eligibilitys
            let eligibility_hardware_mining_calculated_eligibility = match _eligibility_hardware_mining_calculated_eligibility.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let eligibility_hardware_mining_hardware_uptime_percentage = match _eligibility_hardware_mining_hardware_uptime_percentage {
                Some(value) => value,
                None => 1.into() // Default
            };
            // let eligibility_hardware_mining_date_audited = match _eligibility_hardware_mining_date_audited {
            //   Some(value) => value,
            //   None => 1.into() // Default
            // };
            // let eligibility_hardware_mining_auditor_account_id = match _eligibility_hardware_mining_auditor_account_id {
            //   Some(value) => value,
            //   None => 1.into() // Default
            // };

            // Check if a mining_speed_boosts_eligibility_hardware_mining_result already exists with the given mining_speed_boosts_eligibility_hardware_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_eligibility_hardware_mining_result_index(mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostEligibilityHardwareMiningEligibilityResults<T>>::mutate((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id), |mining_speed_boosts_eligibility_hardware_mining_result| {
                    if let Some(_mining_speed_boosts_eligibility_hardware_mining_result) = mining_speed_boosts_eligibility_hardware_mining_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_calculated_eligibility = eligibility_hardware_mining_calculated_eligibility.clone();
                        _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_hardware_uptime_percentage = eligibility_hardware_mining_hardware_uptime_percentage.clone();
                        // _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_date_audited = eligibility_hardware_mining_date_audited.clone();
                        // _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_auditor_account_id = eligibility_hardware_mining_auditor_account_id.clone();
                    }
                });

                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_eligibility_hardware_mining_result = <MiningSpeedBoostEligibilityHardwareMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id));
                if let Some(_mining_speed_boosts_eligibility_hardware_mining_result) = fetched_mining_speed_boosts_eligibility_hardware_mining_result {
                    debug::info!("Latest field eligibility_hardware_mining_calculated_eligibility {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_calculated_eligibility);
                    debug::info!("Latest field eligibility_hardware_mining_hardware_uptime_percentage {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_hardware_uptime_percentage);
                    // debug::info!("Latest field eligibility_hardware_mining_date_audited {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_date_audited);
                    // debug::info!("Latest field eligibility_hardware_mining_auditor_account_id {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_auditor_account_id);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_eligibility_hardware_mining_result instance with the input params
                let mining_speed_boosts_eligibility_hardware_mining_result_instance = MiningSpeedBoostEligibilityHardwareMiningEligibilityResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    eligibility_hardware_mining_calculated_eligibility: eligibility_hardware_mining_calculated_eligibility.clone(),
                    eligibility_hardware_mining_hardware_uptime_percentage: eligibility_hardware_mining_hardware_uptime_percentage.clone(),
                    // eligibility_hardware_mining_date_audited: eligibility_hardware_mining_date_audited.clone(),
                    // eligibility_hardware_mining_auditor_account_id: eligibility_hardware_mining_auditor_account_id.clone(),
                };

                <MiningSpeedBoostEligibilityHardwareMiningEligibilityResults<T>>::insert(
                    (mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id),
                    &mining_speed_boosts_eligibility_hardware_mining_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_eligibility_hardware_mining_result = <MiningSpeedBoostEligibilityHardwareMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id));
                if let Some(_mining_speed_boosts_eligibility_hardware_mining_result) = fetched_mining_speed_boosts_eligibility_hardware_mining_result {
                    debug::info!("Inserted field eligibility_hardware_mining_calculated_eligibility {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_calculated_eligibility);
                    debug::info!("Inserted field eligibility_hardware_mining_hardware_uptime_percentage {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_hardware_uptime_percentage);
                    // debug::info!("Inserted field eligibility_hardware_mining_date_audited {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_date_audited);
                    // debug::info!("Inserted field eligibility_hardware_mining_auditor_account_id {:#?}", _mining_speed_boosts_eligibility_hardware_mining_result.eligibility_hardware_mining_auditor_account_id);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostEligibilityHardwareMiningEligibilityResultSet(
                sender,
                mining_speed_boosts_configuration_hardware_mining_id,
                mining_speed_boosts_eligibility_hardware_mining_id,
                eligibility_hardware_mining_calculated_eligibility,
                eligibility_hardware_mining_hardware_uptime_percentage,
                // eligibility_hardware_mining_date_audited,
                // eligibility_hardware_mining_auditor_account_id
            ));
        }

        pub fn assign_eligibility_to_configuration(
          origin,
          mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex,
          mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_hardware_mining = <mining_speed_boosts_configuration_hardware_mining::Module<T>>
                ::exists_mining_speed_boosts_configuration_hardware_mining(mining_speed_boosts_configuration_hardware_mining_id).is_ok();
            ensure!(is_configuration_hardware_mining, "configuration_hardware_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the eligibility to
            ensure!(
                <mining_speed_boosts_configuration_hardware_mining::Module<T>>::is_mining_speed_boosts_configuration_hardware_mining_owner(mining_speed_boosts_configuration_hardware_mining_id, sender.clone()).is_ok(),
                "Only the configuration_hardware_mining owner can assign itself a eligibility"
            );

            Self::associate_token_eligibility_with_configuration(mining_speed_boosts_eligibility_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id)
                .expect("Unable to associate eligibility with configuration");

            // Ensure that the given mining_speed_boosts_eligibility_hardware_mining_id already exists
            let token_eligibility = Self::mining_speed_boosts_eligibility_hardware_mining(mining_speed_boosts_eligibility_hardware_mining_id);
            ensure!(token_eligibility.is_some(), "Invalid mining_speed_boosts_eligibility_hardware_mining_id");

            // // Ensure that the eligibility is not already owned by a different configuration
            // // Unassign the eligibility from any existing configuration since it may only be owned by one configuration
            // <HardwareMiningEligibilityConfiguration<T>>::remove(mining_speed_boosts_eligibility_hardware_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <HardwareMiningEligibilityConfiguration<T>>::insert(mining_speed_boosts_eligibility_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id);

            Self::deposit_event(RawEvent::AssignedHardwareMiningEligibilityToConfiguration(sender, mining_speed_boosts_eligibility_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id));
		    }
    }
}

impl<T: Trait> Module<T> {
	pub fn is_mining_speed_boosts_eligibility_hardware_mining_owner(mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex, sender: T::AccountId) -> Result<(), &'static str> {
        ensure!(
            Self::mining_speed_boosts_eligibility_hardware_mining_owner(&mining_speed_boosts_eligibility_hardware_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostEligibilityHardwareMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_eligibility_hardware_mining(mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex) -> Result<MiningSpeedBoostEligibilityHardwareMining, &'static str> {
        match Self::mining_speed_boosts_eligibility_hardware_mining(mining_speed_boosts_eligibility_hardware_mining_id) {
            Some(value) => Ok(value),
            None => Err("MiningSpeedBoostEligibilityHardwareMining does not exist")
        }
    }

    pub fn exists_mining_speed_boosts_eligibility_hardware_mining_result(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
        mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex
    ) -> Result<(), &'static str> {
        match Self::mining_speed_boosts_eligibility_hardware_mining_eligibility_results(
          (mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id)
        ) {
            Some(value) => Ok(()),
            None => Err("MiningSpeedBoostEligibilityHardwareMiningEligibilityResult does not exist")
        }
    }

    pub fn has_value_for_mining_speed_boosts_eligibility_hardware_mining_result_index(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
        mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex
    ) -> Result<(), &'static str> {
        debug::info!("Checking if mining_speed_boosts_eligibility_hardware_mining_result has a value that is defined");
        let fetched_mining_speed_boosts_eligibility_hardware_mining_result = <MiningSpeedBoostEligibilityHardwareMiningEligibilityResults<T>>::get((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id));
        if let Some(value) = fetched_mining_speed_boosts_eligibility_hardware_mining_result {
            debug::info!("Found value for mining_speed_boosts_eligibility_hardware_mining_result");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_eligibility_hardware_mining_result");
        Err("No value for mining_speed_boosts_eligibility_hardware_mining_result")
    }

    /// Only push the eligibility id onto the end of the vector if it does not already exist
    pub fn associate_token_eligibility_with_configuration(
        mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex,
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex
    ) -> Result<(), &'static str>
    {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given eligibility id
        if let Some(configuration_eligibilities) = Self::hardware_mining_configuration_eligibilities(mining_speed_boosts_configuration_hardware_mining_id) {
            debug::info!("Configuration id key {:?} exists with value {:?}", mining_speed_boosts_configuration_hardware_mining_id, configuration_eligibilities);
            let not_configuration_contains_eligibility = !configuration_eligibilities.contains(&mining_speed_boosts_eligibility_hardware_mining_id);
            ensure!(not_configuration_contains_eligibility, "Configuration already contains the given eligibility id");
            debug::info!("Configuration id key exists but its vector value does not contain the given eligibility id");
            <HardwareMiningConfigurationEligibilities<T>>::mutate(mining_speed_boosts_configuration_hardware_mining_id, |v| {
                if let Some(value) = v {
                    value.push(mining_speed_boosts_eligibility_hardware_mining_id);
                }
            });
            debug::info!("Associated eligibility {:?} with configuration {:?}", mining_speed_boosts_eligibility_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id);
            Ok(())
        } else {
            debug::info!("Configuration id key does not yet exist. Creating the configuration key {:?} and appending the eligibility id {:?} to its vector value", mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining_id);
            <HardwareMiningConfigurationEligibilities<T>>::insert(mining_speed_boosts_configuration_hardware_mining_id, &vec![mining_speed_boosts_eligibility_hardware_mining_id]);
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

    fn next_mining_speed_boosts_eligibility_hardware_mining_id() -> Result<T::MiningSpeedBoostEligibilityHardwareMiningIndex, &'static str> {
        let mining_speed_boosts_eligibility_hardware_mining_id = Self::mining_speed_boosts_eligibility_hardware_mining_count();
        if mining_speed_boosts_eligibility_hardware_mining_id == <T::MiningSpeedBoostEligibilityHardwareMiningIndex as Bounded>::max_value() {
            return Err("MiningSpeedBoostEligibilityHardwareMining count overflow");
        }
        Ok(mining_speed_boosts_eligibility_hardware_mining_id)
    }

    fn insert_mining_speed_boosts_eligibility_hardware_mining(owner: &T::AccountId, mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex, mining_speed_boosts_eligibility_hardware_mining: MiningSpeedBoostEligibilityHardwareMining) {
        // Create and store mining mining_speed_boosts_eligibility_hardware_mining
        <MiningSpeedBoostEligibilityHardwareMinings<T>>::insert(mining_speed_boosts_eligibility_hardware_mining_id, mining_speed_boosts_eligibility_hardware_mining);
        <MiningSpeedBoostEligibilityHardwareMiningCount<T>>::put(mining_speed_boosts_eligibility_hardware_mining_id + One::one());
        <MiningSpeedBoostEligibilityHardwareMiningOwners<T>>::insert(mining_speed_boosts_eligibility_hardware_mining_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex) {
        <MiningSpeedBoostEligibilityHardwareMiningOwners<T>>::insert(mining_speed_boosts_eligibility_hardware_mining_id, to);
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
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ();
        type TransferFee = ();
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
    impl mining_speed_boosts_rates_hardware_mining::Trait for Test {
      type Event = ();
      type MiningSpeedBoostRatesHardwareMiningIndex = u64;
      // Mining Speed Boost Rate
      type MiningSpeedBoostRatesHardwareMiningHardwareSecure = u32;
      type MiningSpeedBoostRatesHardwareMiningHardwareInsecure = u32;
      // Mining Speed Boost Max Rates
      type MiningSpeedBoostRatesHardwareMiningMaxHardware = u32;
    }
    impl mining_speed_boosts_sampling_hardware_mining::Trait for Test {
      type Event = ();
      type MiningSpeedBoostSamplingHardwareMiningIndex = u64;
      type MiningSpeedBoostSamplingHardwareMiningSampleDate = u64;
      type MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline = u64;
    }
    impl mining_speed_boosts_configuration_hardware_mining::Trait for Test {
      type Event = ();
      // FIXME - restore when stop temporarily using roaming-operators
      // type Currency = Balances;
      // type Randomness = RandomnessCollectiveFlip;
      type MiningSpeedBoostConfigurationHardwareMiningIndex = u64;
      // Mining Speed Boost Hardware Mining Config
      type MiningSpeedBoostConfigurationHardwareMiningHardwareSecure = bool;
      // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
      type MiningSpeedBoostConfigurationHardwareMiningHardwareType = Vec<u8>;
      // type MiningSpeedBoostConfigurationHardwareMiningHardwareType = MiningSpeedBoostConfigurationHardwareMiningHardwareTypes;
      type MiningSpeedBoostConfigurationHardwareMiningHardwareID = u64;
      type MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI = u64;
      type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate = u64;
      type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate = u64;
    }
    impl Trait for Test {
        type Event = ();
        type MiningSpeedBoostEligibilityHardwareMiningIndex = u64;
        type MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility = u64;
        type MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage = u32;
        // type MiningSpeedBoostEligibilityHardwareMiningDateAudited = u64;
        // type MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID = u64;
    }
    //type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type MiningSpeedBoostEligibilityHardwareMiningTestModule = Module<Test>;
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
