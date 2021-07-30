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
use mining_rates_token;
use mining_sampling_token;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config:
    frame_system::Config
    + roaming_operators::Config
    + mining_rates_token::Config
    + mining_setting_token::Config
    + mining_sampling_token::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningEligibilityTokenIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningEligibilityTokenCalculatedEligibility: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningEligibilityTokenLockedPercentage: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningEligibilityTokenAuditorAccountID: Parameter + Member + AtLeast32Bit +
    // Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningEligibilityToken(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningEligibilityTokenResult<U, V> {
    pub token_calculated_eligibility: U,
    pub token_locked_percentage: V,
    /* pub token_block_audited: W,
     * pub token_auditor_account_id: X, */
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::MiningEligibilityTokenIndex,
        <T as Config>::MiningEligibilityTokenCalculatedEligibility,
        <T as Config>::MiningEligibilityTokenLockedPercentage,
        // <T as Config>::MiningEligibilityTokenAuditorAccountID,
        <T as mining_setting_token::Config>::MiningSettingTokenIndex,
        // <T as frame_system::Config>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_eligibility_token is created. (owner, mining_eligibility_token_id)
        Created(AccountId, MiningEligibilityTokenIndex),
        /// A mining_eligibility_token is transferred. (from, to, mining_eligibility_token_id)
        Transferred(AccountId, AccountId, MiningEligibilityTokenIndex),
        // MiningEligibilityTokenResultSet(
        //   AccountId, MiningSettingTokenIndex, MiningEligibilityTokenIndex,
        //   MiningEligibilityTokenCalculatedEligibility, MiningEligibilityTokenLockedPercentage,
        //   BlockNumber, MiningEligibilityTokenAuditorAccountID
        // ),
        MiningEligibilityTokenResultSet(
          AccountId, MiningSettingTokenIndex, MiningEligibilityTokenIndex,
          MiningEligibilityTokenCalculatedEligibility, MiningEligibilityTokenLockedPercentage
          // BlockNumber, MiningEligibilityTokenAuditorAccountID
        ),
        /// A mining_eligibility_token is assigned to an mining_setting_token.
        /// (owner of mining_token, mining_eligibility_token_id, mining_setting_token_id)
        AssignedTokenEligibilityToConfiguration(AccountId, MiningEligibilityTokenIndex, MiningSettingTokenIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningEligibilityToken {
        /// Stores all the mining_eligibility_tokens, key is the mining_eligibility_token id / index
        pub MiningEligibilityTokens get(fn mining_eligibility_token): map hasher(opaque_blake2_256) T::MiningEligibilityTokenIndex => Option<MiningEligibilityToken>;

        /// Stores the total number of mining_eligibility_tokens. i.e. the next mining_eligibility_token index
        pub MiningEligibilityTokenCount get(fn mining_eligibility_token_count): T::MiningEligibilityTokenIndex;

        /// Stores mining_eligibility_token owner
        pub MiningEligibilityTokenOwners get(fn mining_eligibility_token_owner): map hasher(opaque_blake2_256) T::MiningEligibilityTokenIndex => Option<T::AccountId>;

        /// Stores mining_eligibility_token_result
        pub MiningEligibilityTokenResults get(fn mining_eligibility_token_eligibility_results): map hasher(opaque_blake2_256) (T::MiningSettingTokenIndex, T::MiningEligibilityTokenIndex) =>
            Option<MiningEligibilityTokenResult<
                T::MiningEligibilityTokenCalculatedEligibility,
                T::MiningEligibilityTokenLockedPercentage,
                // T::BlockNumber,
                // T::MiningEligibilityTokenAuditorAccountID,
            >>;

        /// Get mining_setting_token_id belonging to a mining_eligibility_token_id
        pub TokenEligibilityConfiguration get(fn token_resulturation): map hasher(opaque_blake2_256) T::MiningEligibilityTokenIndex => Option<T::MiningSettingTokenIndex>;

        /// Get mining_eligibility_token_id's belonging to a mining_setting_token_id
        pub TokenSettingEligibilities get(fn token_setting_eligibilities): map hasher(opaque_blake2_256) T::MiningSettingTokenIndex => Option<Vec<T::MiningEligibilityTokenIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_eligibility_token
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_eligibility_token_id = Self::next_mining_eligibility_token_id()?;

            // Geneeligibility a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_eligibility_token
            let mining_eligibility_token = MiningEligibilityToken(unique_id);
            Self::insert_mining_eligibility_token(&sender, mining_eligibility_token_id, mining_eligibility_token);

            Self::deposit_event(RawEvent::Created(sender, mining_eligibility_token_id));
        }

        /// Transfer a mining_eligibility_token to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_eligibility_token_id: T::MiningEligibilityTokenIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_eligibility_token_owner(mining_eligibility_token_id) == Some(sender.clone()), "Only owner can transfer mining mining_eligibility_token");

            Self::update_owner(&to, mining_eligibility_token_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_eligibility_token_id));
        }

        // FIXME - implement this and fix the type errors and uncomment it in the integration tests
        // /// Calculate mining_eligibility_token_result
        // pub fn calculate_mining_eligibility_token_result(
        //     origin,
        //     mining_setting_token_id: T::MiningSettingTokenIndex,
        //     mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
        // ) {
        //     let sender = ensure_signed(origin)?;

        //     // Ensure that the mining_eligibility_token_id whose config we want to change actually exists
        //     let is_mining_eligibility_token = Self::exists_mining_eligibility_token(mining_eligibility_token_id).is_ok();
        //     ensure!(is_mining_eligibility_token, "MiningEligibilityToken does not exist");

        //     // Ensure that the caller is owner of the mining_eligibility_token_result they are trying to change
        //     ensure!(Self::mining_eligibility_token_owner(mining_eligibility_token_id) == Some(sender.clone()), "Only owner can set mining_eligibility_token_result");

        //     let DEFAULT_RATE_CONFIG = 0;
        //     let mut token_calculated_eligibility = 0u32.into();
        //     let mut part_token_calculated_eligibility = 0u32.into();
        //     let mut token_locked_percentage = 0u32.into();
        //     let mut token_token_max_token = 0u32.into();

        //     let mut current_token_type;
        //     let mut current_token_lock_amount;
        //     // Get the config associated with the given configuration_token
        //     if let Some(configuration_token_setting) = <mining_setting_token::Module<T>>::mining_setting_token_token_settings(mining_setting_token_id) {
        //       if let token_type = configuration_token_setting.token_type {
        //         if token_type != "".to_string() {
        //           current_token_type = token_type.clone();

        //           if let token_lock_amount = configuration_token_setting.token_lock_amount {
        //             if token_lock_amount != 0 {
        //               current_token_lock_amount = token_lock_amount;

        //               // Get list of all sampling_token_ids that correspond to the given mining_setting_token_id
        //               // of type MiningSamplingTokenIndex
        //               let sampling_token_ids = <mining_sampling_token::Module<T>>
        //                 ::token_setting_samplings(mining_setting_token_id);

        //               let mut sample_count = 0;
        //               let mut current_sample_tokens_locked = 0;
        //               let mut current_token_rate = 0;
        //               let mut current_token_max_tokens = 0;
        //               let mut total = 0;
        //               // Iteratve through all the associated samples
        //               for (index, sampling_token_id) in sampling_token_ids.iter().enumerate() {
        //                 // Retrieve the current corresponding sampling_token_setting
        //                 // of type MiningSamplingTokenSetting
        //                 if let Some(current_sampling_token_setting) = <mining_sampling_token::Module<T>>::mining_samplings_token_samplings_configs(
        //                   (mining_setting_token_id, sampling_token_id)
        //                 ) {
        //                   if let tokens_locked = current_sampling_token_setting.token_sample_locked_amount {
        //                     sample_count += 1;

        //                     if tokens_locked == 0 {
        //                       info!("Mining rate sample has nothing locked. Skipping to next sampling.");
        //                       continue;
        //                     }
        //                     current_sample_tokens_locked = tokens_locked;

        //                     if let Some(token_rates_config) = <mining_rates_token::Module<T>>::mining_rates_token_rates_configs(DEFAULT_RATE_CONFIG) {

        //                       if current_token_type == "MXC".to_string() {
        //                         current_token_rate = token_rates_config.token_token_mxc;
        //                       } else if current_token_type == "IOTA".to_string() {
        //                         current_token_rate = token_rates_config.token_token_iota;
        //                       } else if current_token_type == "DOT".to_string() {
        //                         current_token_rate = token_rates_config.token_token_dot;
        //                       }
        //                       current_token_max_tokens = token_rates_config.token_token_max_token;
        //                       token_locked_percentage = current_token_rate * (current_sample_tokens_locked / current_token_lock_amount);

        //                       part_token_calculated_eligibility = part_token_calculated_eligibility + token_locked_percentage * current_token_max_tokens;
        //                     } else {
        //                       warn!("Mining rate config missing");
        //                       // break;
        //                       return Err(DispatchError::Other("Mining rate config missing"));
        //                     }
        //                   }
        //                 }
        //               }
        //               token_calculated_eligibility = part_token_calculated_eligibility / sample_count;
        //               info!("Calculate eligibilty based on average {:#?}", token_calculated_eligibility);
        //             }
        //           }
        //         }
        //       }
        //     }

        //     // Check if a mining_eligibility_token_result already exists with the given mining_eligibility_token_id
        //     // to determine whether to insert new or mutate existing.
        //     if Self::has_value_for_mining_eligibility_token_result_index(mining_setting_token_id, mining_eligibility_token_id).is_ok() {
        //         info!("Mutating values");
        //         <MiningEligibilityTokenResults<T>>::mutate((mining_setting_token_id, mining_eligibility_token_id), |mining_eligibility_token_result| {
        //             if let Some(_mining_eligibility_token_result) = mining_eligibility_token_result {
        //                 // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
        //                 _mining_eligibility_token_result.token_calculated_eligibility = token_calculated_eligibility.clone();
        //                 _mining_eligibility_token_result.token_locked_percentage = token_locked_percentage.clone();
        //                 // _mining_eligibility_token_result.token_block_audited = token_block_audited.clone();
        //                 // _mining_eligibility_token_result.token_auditor_account_id = token_auditor_account_id.clone();
        //             }
        //         });
        //         info!("Checking mutated values");
        //         let fetched_mining_eligibility_token_result = <MiningEligibilityTokenResults<T>>::get((mining_setting_token_id, mining_eligibility_token_id));
        //         if let Some(_mining_eligibility_token_result) = fetched_mining_eligibility_token_result {
        //             info!("Latest field token_calculated_eligibility {:#?}", _mining_eligibility_token_result.token_calculated_eligibility);
        //             info!("Latest field token_locked_percentage {:#?}", _mining_eligibility_token_result.token_locked_percentage);
        //             // info!("Latest field token_block_audited {:#?}", _mining_eligibility_token_result.token_block_audited);
        //             // info!("Latest field token_auditor_account_id {:#?}", _mining_eligibility_token_result.token_auditor_account_id);
        //         }
        //     } else {
        //         info!("Inserting values");

        //         // Create a new mining mining_eligibility_token_result instance with the input params
        //         let mining_eligibility_token_result_instance = MiningEligibilityTokenResult {
        //             // Since each parameter passed into the function is optional (i.e. `Option`)
        //             // we will assign a default value if a parameter value is not provided.
        //             token_calculated_eligibility: token_calculated_eligibility.clone(),
        //             token_locked_percentage: token_locked_percentage.clone(),
        //             // token_block_audited: token_block_audited.clone(),
        //             // token_auditor_account_id: token_auditor_account_id.clone(),
        //         };

        //         <MiningEligibilityTokenResults<T>>::insert(
        //             (mining_setting_token_id, mining_eligibility_token_id),
        //             &mining_eligibility_token_result_instance
        //         );

        //         info!("Checking inserted values");
        //         let fetched_mining_eligibility_token_result = <MiningEligibilityTokenResults<T>>::get((mining_setting_token_id, mining_eligibility_token_id));
        //         if let Some(_mining_eligibility_token_result) = fetched_mining_eligibility_token_result {
        //             info!("Inserted field token_calculated_eligibility {:#?}", _mining_eligibility_token_result.token_calculated_eligibility);
        //             info!("Inserted field token_locked_percentage {:#?}", _mining_eligibility_token_result.token_locked_percentage);
        //             // info!("Inserted field token_block_audited {:#?}", _mining_eligibility_token_result.token_block_audited);
        //             // info!("Inserted field token_auditor_account_id {:#?}", _mining_eligibility_token_result.token_auditor_account_id);
        //         }
        //     }

        //     Self::deposit_event(RawEvent::MiningEligibilityTokenResultSet(
        //       sender,
        //       mining_setting_token_id,
        //       mining_eligibility_token_id,
        //       token_calculated_eligibility,
        //       token_locked_percentage,
        //       // token_block_audited,
        //       // token_auditor_account_id
        //     ));
        // }

        /// Set mining_eligibility_token_result
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_eligibility_token_eligibility_result(
            origin,
            mining_setting_token_id: T::MiningSettingTokenIndex,
            mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
            _token_calculated_eligibility: Option<T::MiningEligibilityTokenCalculatedEligibility>,
            _token_locked_percentage: Option<T::MiningEligibilityTokenLockedPercentage>,
            // _token_block_audited: Option<T::BlockNumber>,
            // _token_auditor_account_id: Option<T::MiningEligibilityTokenAuditorAccountID>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_eligibility_token_id whose config we want to change actually exists
            let is_mining_eligibility_token = Self::exists_mining_eligibility_token(mining_eligibility_token_id).is_ok();
            ensure!(is_mining_eligibility_token, "MiningEligibilityToken does not exist");

            // Ensure that the caller is owner of the mining_eligibility_token_result they are trying to change
            ensure!(Self::mining_eligibility_token_owner(mining_eligibility_token_id) == Some(sender.clone()), "Only owner can set mining_eligibility_token_result");

            // TODO - adjust default eligibilitys
            let token_calculated_eligibility = match _token_calculated_eligibility.clone() {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            let token_locked_percentage = match _token_locked_percentage {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            // let token_block_audited = match _token_block_audited {
            //   Some(value) => value,
            //   None => 1u32.into() // Default
            // };
            // let token_auditor_account_id = match _token_auditor_account_id {
            //   Some(value) => value,
            //   None => 1u32.into() // Default
            // };

            // Check if a mining_eligibility_token_result already exists with the given mining_eligibility_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_eligibility_token_result_index(mining_setting_token_id, mining_eligibility_token_id).is_ok() {
                info!("Mutating values");
                <MiningEligibilityTokenResults<T>>::mutate((mining_setting_token_id, mining_eligibility_token_id), |mining_eligibility_token_result| {
                    if let Some(_mining_eligibility_token_result) = mining_eligibility_token_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_eligibility_token_result.token_calculated_eligibility = token_calculated_eligibility.clone();
                        _mining_eligibility_token_result.token_locked_percentage = token_locked_percentage.clone();
                        // _mining_eligibility_token_result.token_block_audited = token_block_audited.clone();
                        // _mining_eligibility_token_result.token_auditor_account_id = token_auditor_account_id.clone();
                    }
                });

                info!("Checking mutated values");
                let fetched_mining_eligibility_token_result = <MiningEligibilityTokenResults<T>>::get((mining_setting_token_id, mining_eligibility_token_id));
                if let Some(_mining_eligibility_token_result) = fetched_mining_eligibility_token_result {
                    info!("Latest field token_calculated_eligibility {:#?}", _mining_eligibility_token_result.token_calculated_eligibility);
                    info!("Latest field token_locked_percentage {:#?}", _mining_eligibility_token_result.token_locked_percentage);
                    // info!("Latest field token_block_audited {:#?}", _mining_eligibility_token_result.token_block_audited);
                    // info!("Latest field token_auditor_account_id {:#?}", _mining_eligibility_token_result.token_auditor_account_id);
                }
            } else {
                info!("Inserting values");

                // Create a new mining mining_eligibility_token_result instance with the input params
                let mining_eligibility_token_result_instance = MiningEligibilityTokenResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_calculated_eligibility: token_calculated_eligibility.clone(),
                    token_locked_percentage: token_locked_percentage.clone(),
                    // token_block_audited: token_block_audited.clone(),
                    // token_auditor_account_id: token_auditor_account_id.clone(),
                };

                <MiningEligibilityTokenResults<T>>::insert(
                    (mining_setting_token_id, mining_eligibility_token_id),
                    &mining_eligibility_token_result_instance
                );

                info!("Checking inserted values");
                let fetched_mining_eligibility_token_result = <MiningEligibilityTokenResults<T>>::get((mining_setting_token_id, mining_eligibility_token_id));
                if let Some(_mining_eligibility_token_result) = fetched_mining_eligibility_token_result {
                    info!("Inserted field token_calculated_eligibility {:#?}", _mining_eligibility_token_result.token_calculated_eligibility);
                    info!("Inserted field token_locked_percentage {:#?}", _mining_eligibility_token_result.token_locked_percentage);
                    // info!("Inserted field token_block_audited {:#?}", _mining_eligibility_token_result.token_block_audited);
                    // info!("Inserted field token_auditor_account_id {:#?}", _mining_eligibility_token_result.token_auditor_account_id);
                }
            }

            Self::deposit_event(RawEvent::MiningEligibilityTokenResultSet(
                sender,
                mining_setting_token_id,
                mining_eligibility_token_id,
                token_calculated_eligibility,
                token_locked_percentage,
                // token_block_audited,
                // token_auditor_account_id
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_eligibility_to_configuration(
          origin,
          mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
          mining_setting_token_id: T::MiningSettingTokenIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token = <mining_setting_token::Module<T>>
                ::exists_mining_setting_token(mining_setting_token_id).is_ok();
            ensure!(is_configuration_token, "configuration_token does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the eligibility to
            ensure!(
                <mining_setting_token::Module<T>>::is_mining_setting_token_owner(mining_setting_token_id, sender.clone()).is_ok(),
                "Only the configuration_token owner can assign itself a eligibility"
            );

            Self::associate_token_eligibility_with_configuration(mining_eligibility_token_id, mining_setting_token_id)
                .expect("Unable to associate eligibility with configuration");

            // Ensure that the given mining_eligibility_token_id already exists
            let token_eligibility = Self::mining_eligibility_token(mining_eligibility_token_id);
            ensure!(token_eligibility.is_some(), "Invalid mining_eligibility_token_id");

            // // Ensure that the eligibility is not already owned by a different configuration
            // // Unassign the eligibility from any existing configuration since it may only be owned by one configuration
            // <TokenEligibilityConfiguration<T>>::remove(mining_eligibility_token_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenEligibilityConfiguration<T>>::insert(mining_eligibility_token_id, mining_setting_token_id);

            Self::deposit_event(RawEvent::AssignedTokenEligibilityToConfiguration(sender, mining_eligibility_token_id, mining_setting_token_id));
            }
    }
}

impl<T: Config> Module<T> {
    pub fn is_mining_eligibility_token_owner(
        mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_eligibility_token_owner(&mining_eligibility_token_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningEligibilityToken"
        );
        Ok(())
    }

    pub fn exists_mining_eligibility_token(
        mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
    ) -> Result<MiningEligibilityToken, DispatchError> {
        match Self::mining_eligibility_token(mining_eligibility_token_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningEligibilityToken does not exist")),
        }
    }

    pub fn exists_mining_eligibility_token_result(
        mining_setting_token_id: T::MiningSettingTokenIndex,
        mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_eligibility_token_eligibility_results((mining_setting_token_id, mining_eligibility_token_id))
        {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningEligibilityTokenResult does not exist")),
        }
    }

    pub fn has_value_for_mining_eligibility_token_result_index(
        mining_setting_token_id: T::MiningSettingTokenIndex,
        mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if mining_eligibility_token_result has a value that is defined");
        let fetched_mining_eligibility_token_result =
            <MiningEligibilityTokenResults<T>>::get((mining_setting_token_id, mining_eligibility_token_id));
        if let Some(_value) = fetched_mining_eligibility_token_result {
            info!("Found value for mining_eligibility_token_result");
            return Ok(());
        }
        warn!("No value for mining_eligibility_token_result");
        Err(DispatchError::Other("No value for mining_eligibility_token_result"))
    }

    /// Only push the eligibility id onto the end of the vector if it does not already exist
    pub fn associate_token_eligibility_with_configuration(
        mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
        mining_setting_token_id: T::MiningSettingTokenIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given eligibility id
        if let Some(configuration_eligibilities) = Self::token_setting_eligibilities(mining_setting_token_id) {
            info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_setting_token_id,
                configuration_eligibilities
            );
            let not_configuration_contains_eligibility =
                !configuration_eligibilities.contains(&mining_eligibility_token_id);
            ensure!(not_configuration_contains_eligibility, "Configuration already contains the given eligibility id");
            info!("Configuration id key exists but its vector value does not contain the given eligibility id");
            <TokenSettingEligibilities<T>>::mutate(mining_setting_token_id, |v| {
                if let Some(value) = v {
                    value.push(mining_eligibility_token_id);
                }
            });
            info!(
                "Associated eligibility {:?} with configuration {:?}",
                mining_eligibility_token_id,
                mining_setting_token_id
            );
            Ok(())
        } else {
            info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 eligibility id {:?} to its vector value",
                mining_setting_token_id,
                mining_eligibility_token_id
            );
            <TokenSettingEligibilities<T>>::insert(mining_setting_token_id, &vec![mining_eligibility_token_id]);
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

    fn next_mining_eligibility_token_id() -> Result<T::MiningEligibilityTokenIndex, DispatchError> {
        let mining_eligibility_token_id = Self::mining_eligibility_token_count();
        if mining_eligibility_token_id == <T::MiningEligibilityTokenIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningEligibilityToken count overflow"));
        }
        Ok(mining_eligibility_token_id)
    }

    fn insert_mining_eligibility_token(
        owner: &T::AccountId,
        mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
        mining_eligibility_token: MiningEligibilityToken,
    ) {
        // Create and store mining mining_eligibility_token
        <MiningEligibilityTokens<T>>::insert(mining_eligibility_token_id, mining_eligibility_token);
        <MiningEligibilityTokenCount<T>>::put(mining_eligibility_token_id + One::one());
        <MiningEligibilityTokenOwners<T>>::insert(mining_eligibility_token_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_eligibility_token_id: T::MiningEligibilityTokenIndex) {
        <MiningEligibilityTokenOwners<T>>::insert(mining_eligibility_token_id, to);
    }
}
