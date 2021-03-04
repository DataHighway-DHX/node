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
use mining_config_hardware;
use mining_rates_hardware;
use mining_sampling_hardware;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config:
    frame_system::Config
    + roaming_operators::Config
    + mining_rates_hardware::Config
    + mining_config_hardware::Config
    + mining_sampling_hardware::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningEligibilityHardwareIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningEligibilityHardwareCalculatedEligibility: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningEligibilityHardwareUptimePercentage: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningEligibilityHardwareAuditorAccountID: Parameter + Member + AtLeast32Bit +
    // Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningEligibilityHardware(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningEligibilityHardwareResult<U, V> {
    pub hardware_calculated_eligibility: U,
    pub hardware_uptime_percentage: V,
    /* pub hardware_block_audited: W,
     * pub hardware_auditor_account_id: X, */
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Trait>::MiningEligibilityHardwareIndex,
        <T as Trait>::MiningEligibilityHardwareCalculatedEligibility,
        <T as Trait>::MiningEligibilityHardwareUptimePercentage,
        // <T as Trait>::MiningEligibilityHardwareAuditorAccountID,
        <T as mining_config_hardware::Config>::MiningConfigHardwareIndex,
        // <T as frame_system::Config>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_eligibility_hardware is created. (owner, mining_eligibility_hardware_id)
        Created(AccountId, MiningEligibilityHardwareIndex),
        /// A mining_eligibility_hardware is transferred. (from, to, mining_eligibility_hardware_id)
        Transferred(AccountId, AccountId, MiningEligibilityHardwareIndex),
        // MiningEligibilityHardwareResultSet(
        //   AccountId, MiningConfigHardwareIndex, MiningEligibilityHardwareIndex,
        //   MiningEligibilityHardwareCalculatedEligibility, MiningEligibilityHardwareUptimePercentage,
        //   BlockNumber, MiningEligibilityHardwareAuditorAccountID
        // ),
        MiningEligibilityHardwareResultSet(
          AccountId, MiningConfigHardwareIndex, MiningEligibilityHardwareIndex,
          MiningEligibilityHardwareCalculatedEligibility, MiningEligibilityHardwareUptimePercentage
          // BlockNumber, MiningEligibilityHardwareAuditorAccountID
        ),
        /// A mining_eligibility_hardware is assigned to an mining_config_hardware.
        /// (owner of mining_hardware, mining_eligibility_hardware_id, mining_config_hardware_id)
        AssignedHardwareEligibilityToConfiguration(AccountId, MiningEligibilityHardwareIndex, MiningConfigHardwareIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningEligibilityHardware {
        /// Stores all the mining_eligibility_hardwares, key is the mining_eligibility_hardware id / index
        pub MiningEligibilityHardwares get(fn mining_eligibility_hardware): map hasher(opaque_blake2_256) T::MiningEligibilityHardwareIndex => Option<MiningEligibilityHardware>;

        /// Stores the total number of mining_eligibility_hardwares. i.e. the next mining_eligibility_hardware index
        pub MiningEligibilityHardwareCount get(fn mining_eligibility_hardware_count): T::MiningEligibilityHardwareIndex;

        /// Stores mining_eligibility_hardware owner
        pub MiningEligibilityHardwareOwners get(fn mining_eligibility_hardware_owner): map hasher(opaque_blake2_256) T::MiningEligibilityHardwareIndex => Option<T::AccountId>;

        /// Stores mining_eligibility_hardware_result
        pub MiningEligibilityHardwareResults get(fn mining_eligibility_hardware_eligibility_results): map hasher(opaque_blake2_256) (T::MiningConfigHardwareIndex, T::MiningEligibilityHardwareIndex) =>
            Option<MiningEligibilityHardwareResult<
                T::MiningEligibilityHardwareCalculatedEligibility,
                T::MiningEligibilityHardwareUptimePercentage,
                // T::BlockNumber,
                // T::MiningEligibilityHardwareAuditorAccountID,
            >>;

        /// Get mining_config_hardware_id belonging to a mining_eligibility_hardware_id
        pub HardwareEligibilityConfiguration get(fn hardware_resulturation): map hasher(opaque_blake2_256) T::MiningEligibilityHardwareIndex => Option<T::MiningConfigHardwareIndex>;

        /// Get mining_eligibility_hardware_id's belonging to a mining_config_hardware_id
        pub HardwareConfigEligibilities get(fn hardware_config_eligibilities): map hasher(opaque_blake2_256) T::MiningConfigHardwareIndex => Option<Vec<T::MiningEligibilityHardwareIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_eligibility_hardware
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_eligibility_hardware_id = Self::next_mining_eligibility_hardware_id()?;

            // Geneeligibility a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_eligibility_hardware
            let mining_eligibility_hardware = MiningEligibilityHardware(unique_id);
            Self::insert_mining_eligibility_hardware(&sender, mining_eligibility_hardware_id, mining_eligibility_hardware);

            Self::deposit_event(RawEvent::Created(sender, mining_eligibility_hardware_id));
        }

        /// Transfer a mining_eligibility_hardware to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_eligibility_hardware_owner(mining_eligibility_hardware_id) == Some(sender.clone()), "Only owner can transfer mining mining_eligibility_hardware");

            Self::update_owner(&to, mining_eligibility_hardware_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_eligibility_hardware_id));
        }

        // FIXME - implement this and fix the type errors and uncomment it in the integration tests
        // /// Calculate mining_eligibility_hardware_result
        // pub fn calculate_mining_eligibility_hardware_result(
        //     origin,
        //     mining_config_hardware_id: T::MiningConfigHardwareIndex,
        //     mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
        // ) {
        //     let sender = ensure_signed(origin)?;

        //     // Ensure that the mining_eligibility_hardware_id whose config we want to change actually exists
        //     let is_mining_eligibility_hardware = Self::exists_mining_eligibility_hardware(mining_eligibility_hardware_id).is_ok();
        //     ensure!(is_mining_eligibility_hardware, "MiningEligibilityHardware does not exist");

        //     // Ensure that the caller is owner of the mining_eligibility_hardware_result they are trying to change
        //     ensure!(Self::mining_eligibility_hardware_owner(mining_eligibility_hardware_id) == Some(sender.clone()), "Only owner can set mining_eligibility_hardware_result");

        //     let DEFAULT_RATE_CONFIG = 0;
        //     let mut hardware_calculated_eligibility = 0.into();
        //     let mut part_hardware_calculated_eligibility = 0.into();
        //     let mut hardware_uptime_percentage = 0.into();
        //     let mut token_token_max_token = 0.into();

        //     let mut current_token_type;
        //     let mut current_hardware_uptime_amount;
        //     // Get the config associated with the given configuration_hardware
        //     if let Some(configuration_hardware_config) = <mining_config_hardware::Module<T>>::mining_config_hardware_token_configs(mining_config_hardware_id) {
        //       if let token_type = configuration_hardware_config.token_type {
        //         if token_type != "".to_string() {
        //           current_token_type = token_type.clone();

        //           if let hardware_uptime_amount = configuration_hardware_config.hardware_uptime_amount {
        //             if hardware_uptime_amount != 0 {
        //               current_hardware_uptime_amount = hardware_uptime_amount;

        //               // Get list of all sampling_hardware_ids that correspond to the given mining_config_hardware_id
        //               // of type MiningSamplingHardwareIndex
        //               let sampling_hardware_ids = <mining_sampling_hardware::Module<T>>
        //                 ::hardware_config_samplings(mining_config_hardware_id);

        //               let mut sample_count = 0;
        //               let mut current_sample_tokens_locked = 0;
        //               let mut current_hardware_rate = 0;
        //               let mut current_token_max_tokens = 0;
        //               let mut total = 0;
        //               // Iteratve through all the associated samples
        //               for (index, sampling_hardware_id) in sampling_hardware_ids.iter().enumerate() {
        //                 // Retrieve the current corresponding sampling_hardware_config
        //                 // of type MiningSamplingHardwareConfig
        //                 if let Some(current_sampling_hardware_config) = <mining_sampling_hardware::Module<T>>::mining_samplings_hardware_samplings_configs(
        //                   (mining_config_hardware_id, sampling_hardware_id)
        //                 ) {
        //                   if let tokens_locked = current_sampling_hardware_config.token_sample_locked_amount {
        //                     sample_count += 1;

        //                     if tokens_locked == 0 {
        //                       debug::info!("Mining rate sample has nothing locked. Skipping to next sampling.");
        //                       continue;
        //                     }
        //                     current_sample_tokens_locked = tokens_locked;

        //                     if let Some(hardware_rates_config) = <mining_rates_hardware::Module<T>>::mining_rates_hardware_rates_configs(DEFAULT_RATE_CONFIG) {

        //                       if current_token_type == "MXC".to_string() {
        //                         current_hardware_rate = hardware_rates_config.token_token_mxc;
        //                       } else if current_token_type == "IOTA".to_string() {
        //                         current_hardware_rate = hardware_rates_config.token_token_iota;
        //                       } else if current_token_type == "DOT".to_string() {
        //                         current_hardware_rate = hardware_rates_config.token_token_dot;
        //                       }
        //                       current_token_max_tokens = hardware_rates_config.token_token_max_token;
        //                       hardware_uptime_percentage = current_hardware_rate * (current_sample_tokens_locked / current_hardware_uptime_amount);

        //                       part_hardware_calculated_eligibility = part_hardware_calculated_eligibility + hardware_uptime_percentage * current_token_max_tokens;
        //                     } else {
        //                       debug::info!("Mining rate config missing");
        //                       break;
        //                       return Err(DispatchError::Other("Mining rate config missing"));
        //                     }
        //                   }
        //                 }
        //               }
        //               hardware_calculated_eligibility = part_hardware_calculated_eligibility / sample_count;
        //               debug::info!("Calculate eligibilty based on average {:#?}", hardware_calculated_eligibility);
        //             }
        //           }
        //         }
        //       }
        //     }

        //     // Check if a mining_eligibility_hardware_result already exists with the given mining_eligibility_hardware_id
        //     // to determine whether to insert new or mutate existing.
        //     if Self::has_value_for_mining_eligibility_hardware_result_index(mining_config_hardware_id, mining_eligibility_hardware_id).is_ok() {
        //         debug::info!("Mutating values");
        //         <MiningEligibilityHardwareResults<T>>::mutate((mining_config_hardware_id, mining_eligibility_hardware_id), |mining_eligibility_hardware_result| {
        //             if let Some(_mining_eligibility_hardware_result) = mining_eligibility_hardware_result {
        //                 // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
        //                 _mining_eligibility_hardware_result.hardware_calculated_eligibility = hardware_calculated_eligibility.clone();
        //                 _mining_eligibility_hardware_result.hardware_uptime_percentage = hardware_uptime_percentage.clone();
        //                 // _mining_eligibility_hardware_result.hardware_block_audited = hardware_block_audited.clone();
        //                 // _mining_eligibility_hardware_result.hardware_auditor_account_id = hardware_auditor_account_id.clone();
        //             }
        //         });
        //         debug::info!("Checking mutated values");
        //         let fetched_mining_eligibility_hardware_result = <MiningEligibilityHardwareResults<T>>::get((mining_config_hardware_id, mining_eligibility_hardware_id));
        //         if let Some(_mining_eligibility_hardware_result) = fetched_mining_eligibility_hardware_result {
        //             debug::info!("Latest field hardware_calculated_eligibility {:#?}", _mining_eligibility_hardware_result.hardware_calculated_eligibility);
        //             debug::info!("Latest field hardware_uptime_percentage {:#?}", _mining_eligibility_hardware_result.hardware_uptime_percentage);
        //             // debug::info!("Latest field hardware_block_audited {:#?}", _mining_eligibility_hardware_result.hardware_block_audited);
        //             // debug::info!("Latest field hardware_auditor_account_id {:#?}", _mining_eligibility_hardware_result.hardware_auditor_account_id);
        //         }
        //     } else {
        //         debug::info!("Inserting values");

        //         // Create a new mining mining_eligibility_hardware_result instance with the input params
        //         let mining_eligibility_hardware_result_instance = MiningEligibilityHardwareResult {
        //             // Since each parameter passed into the function is optional (i.e. `Option`)
        //             // we will assign a default value if a parameter value is not provided.
        //             hardware_calculated_eligibility: hardware_calculated_eligibility.clone(),
        //             hardware_uptime_percentage: hardware_uptime_percentage.clone(),
        //             // hardware_block_audited: hardware_block_audited.clone(),
        //             // hardware_auditor_account_id: hardware_auditor_account_id.clone(),
        //         };

        //         <MiningEligibilityHardwareResults<T>>::insert(
        //             (mining_config_hardware_id, mining_eligibility_hardware_id),
        //             &mining_eligibility_hardware_result_instance
        //         );

        //         debug::info!("Checking inserted values");
        //         let fetched_mining_eligibility_hardware_result = <MiningEligibilityHardwareResults<T>>::get((mining_config_hardware_id, mining_eligibility_hardware_id));
        //         if let Some(_mining_eligibility_hardware_result) = fetched_mining_eligibility_hardware_result {
        //             debug::info!("Inserted field hardware_calculated_eligibility {:#?}", _mining_eligibility_hardware_result.hardware_calculated_eligibility);
        //             debug::info!("Inserted field hardware_uptime_percentage {:#?}", _mining_eligibility_hardware_result.hardware_uptime_percentage);
        //             // debug::info!("Inserted field hardware_block_audited {:#?}", _mining_eligibility_hardware_result.hardware_block_audited);
        //             // debug::info!("Inserted field hardware_auditor_account_id {:#?}", _mining_eligibility_hardware_result.hardware_auditor_account_id);
        //         }
        //     }

        //     Self::deposit_event(RawEvent::MiningEligibilityHardwareResultSet(
        //       sender,
        //       mining_config_hardware_id,
        //       mining_eligibility_hardware_id,
        //       hardware_calculated_eligibility,
        //       hardware_uptime_percentage,
        //       // hardware_block_audited,
        //       // hardware_auditor_account_id
        //     ));
        // }

        /// Set mining_eligibility_hardware_result
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_eligibility_hardware_eligibility_result(
            origin,
            mining_config_hardware_id: T::MiningConfigHardwareIndex,
            mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
            _hardware_calculated_eligibility: Option<T::MiningEligibilityHardwareCalculatedEligibility>,
            _hardware_uptime_percentage: Option<T::MiningEligibilityHardwareUptimePercentage>,
            // _hardware_block_audited: Option<T::BlockNumber>,
            // _hardware_auditor_account_id: Option<T::MiningEligibilityHardwareAuditorAccountID>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_eligibility_hardware_id whose config we want to change actually exists
            let is_mining_eligibility_hardware = Self::exists_mining_eligibility_hardware(mining_eligibility_hardware_id).is_ok();
            ensure!(is_mining_eligibility_hardware, "MiningEligibilityHardware does not exist");

            // Ensure that the caller is owner of the mining_eligibility_hardware_result they are trying to change
            ensure!(Self::mining_eligibility_hardware_owner(mining_eligibility_hardware_id) == Some(sender.clone()), "Only owner can set mining_eligibility_hardware_result");

            // TODO - adjust default eligibilitys
            let hardware_calculated_eligibility = match _hardware_calculated_eligibility.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let hardware_uptime_percentage = match _hardware_uptime_percentage {
                Some(value) => value,
                None => 1.into() // Default
            };
            // let hardware_block_audited = match _hardware_block_audited {
            //   Some(value) => value,
            //   None => 1.into() // Default
            // };
            // let hardware_auditor_account_id = match _hardware_auditor_account_id {
            //   Some(value) => value,
            //   None => 1.into() // Default
            // };

            // Check if a mining_eligibility_hardware_result already exists with the given mining_eligibility_hardware_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_eligibility_hardware_result_index(mining_config_hardware_id, mining_eligibility_hardware_id).is_ok() {
                debug::info!("Mutating values");
                <MiningEligibilityHardwareResults<T>>::mutate((mining_config_hardware_id, mining_eligibility_hardware_id), |mining_eligibility_hardware_result| {
                    if let Some(_mining_eligibility_hardware_result) = mining_eligibility_hardware_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_eligibility_hardware_result.hardware_calculated_eligibility = hardware_calculated_eligibility.clone();
                        _mining_eligibility_hardware_result.hardware_uptime_percentage = hardware_uptime_percentage.clone();
                        // _mining_eligibility_hardware_result.hardware_block_audited = hardware_block_audited.clone();
                        // _mining_eligibility_hardware_result.hardware_auditor_account_id = hardware_auditor_account_id.clone();
                    }
                });

                debug::info!("Checking mutated values");
                let fetched_mining_eligibility_hardware_result = <MiningEligibilityHardwareResults<T>>::get((mining_config_hardware_id, mining_eligibility_hardware_id));
                if let Some(_mining_eligibility_hardware_result) = fetched_mining_eligibility_hardware_result {
                    debug::info!("Latest field hardware_calculated_eligibility {:#?}", _mining_eligibility_hardware_result.hardware_calculated_eligibility);
                    debug::info!("Latest field hardware_uptime_percentage {:#?}", _mining_eligibility_hardware_result.hardware_uptime_percentage);
                    // debug::info!("Latest field hardware_block_audited {:#?}", _mining_eligibility_hardware_result.hardware_block_audited);
                    // debug::info!("Latest field hardware_auditor_account_id {:#?}", _mining_eligibility_hardware_result.hardware_auditor_account_id);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_eligibility_hardware_result instance with the input params
                let mining_eligibility_hardware_result_instance = MiningEligibilityHardwareResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_calculated_eligibility: hardware_calculated_eligibility.clone(),
                    hardware_uptime_percentage: hardware_uptime_percentage.clone(),
                    // hardware_block_audited: hardware_block_audited.clone(),
                    // hardware_auditor_account_id: hardware_auditor_account_id.clone(),
                };

                <MiningEligibilityHardwareResults<T>>::insert(
                    (mining_config_hardware_id, mining_eligibility_hardware_id),
                    &mining_eligibility_hardware_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_eligibility_hardware_result = <MiningEligibilityHardwareResults<T>>::get((mining_config_hardware_id, mining_eligibility_hardware_id));
                if let Some(_mining_eligibility_hardware_result) = fetched_mining_eligibility_hardware_result {
                    debug::info!("Inserted field hardware_calculated_eligibility {:#?}", _mining_eligibility_hardware_result.hardware_calculated_eligibility);
                    debug::info!("Inserted field hardware_uptime_percentage {:#?}", _mining_eligibility_hardware_result.hardware_uptime_percentage);
                    // debug::info!("Inserted field hardware_block_audited {:#?}", _mining_eligibility_hardware_result.hardware_block_audited);
                    // debug::info!("Inserted field hardware_auditor_account_id {:#?}", _mining_eligibility_hardware_result.hardware_auditor_account_id);
                }
            }

            Self::deposit_event(RawEvent::MiningEligibilityHardwareResultSet(
                sender,
                mining_config_hardware_id,
                mining_eligibility_hardware_id,
                hardware_calculated_eligibility,
                hardware_uptime_percentage,
                // hardware_block_audited,
                // hardware_auditor_account_id
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_eligibility_to_configuration(
          origin,
          mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
          mining_config_hardware_id: T::MiningConfigHardwareIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_hardware = <mining_config_hardware::Module<T>>
                ::exists_mining_config_hardware(mining_config_hardware_id).is_ok();
            ensure!(is_configuration_hardware, "configuration_hardware does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the eligibility to
            ensure!(
                <mining_config_hardware::Module<T>>::is_mining_config_hardware_owner(mining_config_hardware_id, sender.clone()).is_ok(),
                "Only the configuration_hardware owner can assign itself a eligibility"
            );

            Self::associate_token_eligibility_with_configuration(mining_eligibility_hardware_id, mining_config_hardware_id)
                .expect("Unable to associate eligibility with configuration");

            // Ensure that the given mining_eligibility_hardware_id already exists
            let token_eligibility = Self::mining_eligibility_hardware(mining_eligibility_hardware_id);
            ensure!(token_eligibility.is_some(), "Invalid mining_eligibility_hardware_id");

            // // Ensure that the eligibility is not already owned by a different configuration
            // // Unassign the eligibility from any existing configuration since it may only be owned by one configuration
            // <HardwareEligibilityConfiguration<T>>::remove(mining_eligibility_hardware_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <HardwareEligibilityConfiguration<T>>::insert(mining_eligibility_hardware_id, mining_config_hardware_id);

            Self::deposit_event(RawEvent::AssignedHardwareEligibilityToConfiguration(sender, mining_eligibility_hardware_id, mining_config_hardware_id));
            }
    }
}

impl<T: Config> Module<T> {
    pub fn is_mining_eligibility_hardware_owner(
        mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_eligibility_hardware_owner(&mining_eligibility_hardware_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningEligibilityHardware"
        );
        Ok(())
    }

    pub fn exists_mining_eligibility_hardware(
        mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
    ) -> Result<MiningEligibilityHardware, DispatchError> {
        match Self::mining_eligibility_hardware(mining_eligibility_hardware_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningEligibilityHardware does not exist")),
        }
    }

    pub fn exists_mining_eligibility_hardware_result(
        mining_config_hardware_id: T::MiningConfigHardwareIndex,
        mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_eligibility_hardware_eligibility_results((
            mining_config_hardware_id,
            mining_eligibility_hardware_id,
        )) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningEligibilityHardwareResult does not exist")),
        }
    }

    pub fn has_value_for_mining_eligibility_hardware_result_index(
        mining_config_hardware_id: T::MiningConfigHardwareIndex,
        mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_eligibility_hardware_result has a value that is defined");
        let fetched_mining_eligibility_hardware_result =
            <MiningEligibilityHardwareResults<T>>::get((mining_config_hardware_id, mining_eligibility_hardware_id));
        if let Some(_value) = fetched_mining_eligibility_hardware_result {
            debug::info!("Found value for mining_eligibility_hardware_result");
            return Ok(());
        }
        debug::info!("No value for mining_eligibility_hardware_result");
        Err(DispatchError::Other("No value for mining_eligibility_hardware_result"))
    }

    /// Only push the eligibility id onto the end of the vector if it does not already exist
    pub fn associate_token_eligibility_with_configuration(
        mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
        mining_config_hardware_id: T::MiningConfigHardwareIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given eligibility id
        if let Some(configuration_eligibilities) = Self::hardware_config_eligibilities(mining_config_hardware_id) {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_config_hardware_id,
                configuration_eligibilities
            );
            let not_configuration_contains_eligibility =
                !configuration_eligibilities.contains(&mining_eligibility_hardware_id);
            ensure!(not_configuration_contains_eligibility, "Configuration already contains the given eligibility id");
            debug::info!("Configuration id key exists but its vector value does not contain the given eligibility id");
            <HardwareConfigEligibilities<T>>::mutate(mining_config_hardware_id, |v| {
                if let Some(value) = v {
                    value.push(mining_eligibility_hardware_id);
                }
            });
            debug::info!(
                "Associated eligibility {:?} with configuration {:?}",
                mining_eligibility_hardware_id,
                mining_config_hardware_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the \
                 eligibility id {:?} to its vector value",
                mining_config_hardware_id,
                mining_eligibility_hardware_id
            );
            <HardwareConfigEligibilities<T>>::insert(mining_config_hardware_id, &vec![mining_eligibility_hardware_id]);
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

    fn next_mining_eligibility_hardware_id() -> Result<T::MiningEligibilityHardwareIndex, DispatchError> {
        let mining_eligibility_hardware_id = Self::mining_eligibility_hardware_count();
        if mining_eligibility_hardware_id == <T::MiningEligibilityHardwareIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningEligibilityHardware count overflow"));
        }
        Ok(mining_eligibility_hardware_id)
    }

    fn insert_mining_eligibility_hardware(
        owner: &T::AccountId,
        mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
        mining_eligibility_hardware: MiningEligibilityHardware,
    ) {
        // Create and store mining mining_eligibility_hardware
        <MiningEligibilityHardwares<T>>::insert(mining_eligibility_hardware_id, mining_eligibility_hardware);
        <MiningEligibilityHardwareCount<T>>::put(mining_eligibility_hardware_id + One::one());
        <MiningEligibilityHardwareOwners<T>>::insert(mining_eligibility_hardware_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex) {
        <MiningEligibilityHardwareOwners<T>>::insert(mining_eligibility_hardware_id, to);
    }
}
