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
use mining_speed_boosts_rates_token_mining;
use mining_speed_boosts_sampling_token_mining;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + roaming_operators::Trait
    + mining_speed_boosts_configuration_token_mining::Trait
    + mining_speed_boosts_eligibility_token_mining::Trait
    + mining_speed_boosts_rates_token_mining::Trait
    + mining_speed_boosts_sampling_token_mining::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningSpeedBoostLodgementsTokenMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostLodgementsTokenMiningLodgementAmount: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningSpeedBoostLodgementsTokenMiningLodgementDateRedeemed: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostLodgementsTokenMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostLodgementsTokenMiningLodgementResult<U, V> {
    pub token_claim_amount: U,
    pub token_claim_date_redeemed: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostLodgementsTokenMiningIndex,
        <T as Trait>::MiningSpeedBoostLodgementsTokenMiningLodgementAmount,
        <T as Trait>::MiningSpeedBoostLodgementsTokenMiningLodgementDateRedeemed,
        <T as mining_speed_boosts_configuration_token_mining::Trait>::MiningSpeedBoostConfigurationTokenMiningIndex,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_lodgements_token_mining is created. (owner, mining_speed_boosts_lodgements_token_mining_id)
        Created(AccountId, MiningSpeedBoostLodgementsTokenMiningIndex),
        /// A mining_speed_boosts_lodgements_token_mining is transferred. (from, to, mining_speed_boosts_lodgements_token_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostLodgementsTokenMiningIndex),
        MiningSpeedBoostLodgementsTokenMiningLodgementResultSet(
            AccountId, MiningSpeedBoostConfigurationTokenMiningIndex, MiningSpeedBoostLodgementsTokenMiningIndex,
            MiningSpeedBoostLodgementsTokenMiningLodgementAmount, MiningSpeedBoostLodgementsTokenMiningLodgementDateRedeemed
        ),
        /// A mining_speed_boosts_lodgements_token_mining is assigned to an mining_speed_boosts_token_mining.
        /// (owner of mining_speed_boosts_token_mining, mining_speed_boosts_lodgements_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
            AssignedTokenMiningLodgementToConfiguration(AccountId, MiningSpeedBoostLodgementsTokenMiningIndex, MiningSpeedBoostConfigurationTokenMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostLodgementsTokenMining {
        /// Stores all the mining_speed_boosts_lodgements_token_minings, key is the mining_speed_boosts_lodgements_token_mining id / index
        pub MiningSpeedBoostLodgementsTokenMinings get(fn mining_speed_boosts_lodgements_token_mining): map hasher(blake2_256) T::MiningSpeedBoostLodgementsTokenMiningIndex => Option<MiningSpeedBoostLodgementsTokenMining>;

        /// Stores the total number of mining_speed_boosts_lodgements_token_minings. i.e. the next mining_speed_boosts_lodgements_token_mining index
        pub MiningSpeedBoostLodgementsTokenMiningCount get(fn mining_speed_boosts_lodgements_token_mining_count): T::MiningSpeedBoostLodgementsTokenMiningIndex;

        /// Stores mining_speed_boosts_lodgements_token_mining owner
        pub MiningSpeedBoostLodgementsTokenMiningOwners get(fn mining_speed_boosts_lodgements_token_mining_owner): map hasher(blake2_256) T::MiningSpeedBoostLodgementsTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_lodgements_token_mining_lodgements_result
        pub MiningSpeedBoostLodgementsTokenMiningLodgementResults get(fn mining_speed_boosts_lodgements_token_mining_lodgements_results): map hasher(blake2_256) (T::MiningSpeedBoostConfigurationTokenMiningIndex, T::MiningSpeedBoostLodgementsTokenMiningIndex) =>
            Option<MiningSpeedBoostLodgementsTokenMiningLodgementResult<
                T::MiningSpeedBoostLodgementsTokenMiningLodgementAmount,
                T::MiningSpeedBoostLodgementsTokenMiningLodgementDateRedeemed
            >>;

        /// Get mining_speed_boosts_configuration_token_mining_id belonging to a mining_speed_boosts_lodgements_token_mining_id
        pub TokenMiningLodgementConfiguration get(fn token_mining_claim_configuration): map hasher(blake2_256) T::MiningSpeedBoostLodgementsTokenMiningIndex => Option<T::MiningSpeedBoostConfigurationTokenMiningIndex>;

        /// Get mining_speed_boosts_lodgements_token_mining_id's belonging to a mining_speed_boosts_configuration_token_mining_id
        pub TokenMiningConfigurationLodgements get(fn token_mining_configuration_lodgements): map hasher(blake2_256) T::MiningSpeedBoostConfigurationTokenMiningIndex => Option<Vec<T::MiningSpeedBoostLodgementsTokenMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_lodgements_token_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_lodgements_token_mining_id = Self::next_mining_speed_boosts_lodgements_token_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_lodgements_token_mining
            let mining_speed_boosts_lodgements_token_mining = MiningSpeedBoostLodgementsTokenMining(unique_id);
            Self::insert_mining_speed_boosts_lodgements_token_mining(&sender, mining_speed_boosts_lodgements_token_mining_id, mining_speed_boosts_lodgements_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_lodgements_token_mining_id));
        }

        /// Transfer a mining_speed_boosts_lodgements_token_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_lodgements_token_mining_owner(mining_speed_boosts_lodgements_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_lodgements_token_mining");

            Self::update_owner(&to, mining_speed_boosts_lodgements_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_lodgements_token_mining_id));
        }

        pub fn claim(
            origin,
            mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
            mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
            mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_lodgements_token_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_lodgements_token_mining = Self::exists_mining_speed_boosts_lodgements_token_mining(mining_speed_boosts_lodgements_token_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_lodgements_token_mining, "MiningSpeedBoostLodgementsTokenMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_lodgements_token_mining_lodgements_result they are trying to change
            ensure!(Self::mining_speed_boosts_lodgements_token_mining_owner(mining_speed_boosts_lodgements_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_lodgements_token_mining_lodgements_result");

            // Check that only allow the owner of the configuration that the claim belongs to call this extrinsic
            // and claim their eligibility
            ensure!(
              <mining_speed_boosts_configuration_token_mining::Module<T>>::is_mining_speed_boosts_configuration_token_mining_owner(
                mining_speed_boosts_configuration_token_mining_id, sender.clone()
              ).is_ok(),
              "Only the configuration_token_mining owner can claim their associated eligibility"
            );

            // Check that the extrinsic call is made after the end date defined in the provided configuration

            // FIXME - add system time now
            let TIME_NOW = 123.into();
            // Get the config associated with the given configuration_token_mining
            if let Some(configuration_token_mining_config) = <mining_speed_boosts_configuration_token_mining::Module<T>>::mining_speed_boosts_configuration_token_mining_token_configs(mining_speed_boosts_configuration_token_mining_id) {
              if let token_lock_period_end_date = configuration_token_mining_config.token_lock_period_end_date {
                // FIXME - get this to work when add system time
                // ensure!(TIME_NOW > token_lock_period_end_date, "Lodgement may not be made until after the end date of the lock period");
              } else {
                return Err(DispatchError::Other("Cannot find token_mining_config end_date associated with the claim"));
              }
            } else {
              return Err(DispatchError::Other("Cannot find token_mining_config associated with the claim"));
            }

            // Check that the provided eligibility amount has not already been claimed
            // i.e. there should only be a single claim instance for each configuration and eligibility in the MVP
            if let Some(token_mining_configuration_lodgements) = Self::token_mining_configuration_lodgements(mining_speed_boosts_configuration_token_mining_id) {
              ensure!(token_mining_configuration_lodgements.len() == 1, "Cannot have zero or more than one claim associated with configuration/eligibility");
            } else {
              return Err(DispatchError::Other("Cannot find configuration_lodgements associated with the claim"));
            }

            // Record the claim associated with their configuration/eligibility
            let token_claim_amount: T::MiningSpeedBoostLodgementsTokenMiningLodgementAmount = 0.into();
            let token_claim_date_redeemed: T::MiningSpeedBoostLodgementsTokenMiningLodgementDateRedeemed = TIME_NOW;
            if let Some(eligibility_token_mining) = <mining_speed_boosts_eligibility_token_mining::Module<T>>::mining_speed_boosts_eligibility_token_mining_eligibility_results((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id)) {
              if let token_mining_calculated_eligibility = eligibility_token_mining.eligibility_token_mining_calculated_eligibility {
                ensure!(token_mining_calculated_eligibility > 0.into(), "Calculated eligibility is zero. Nothing to claim.");
                // FIXME - unable to figure out how to cast here!
                // token_claim_amount = (token_mining_calculated_eligibility as T::MiningSpeedBoostLodgementsTokenMiningLodgementAmount).clone();
              } else {
                return Err(DispatchError::Other("Cannot find token_mining_eligibility calculated_eligibility associated with the claim"));
              }
            } else {
              return Err(DispatchError::Other("Cannot find token_mining_eligibility associated with the claim"));
            }

            // Check if a mining_speed_boosts_lodgements_token_mining_lodgements_result already exists with the given mining_speed_boosts_lodgements_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_lodgements_token_mining_lodgements_result_index(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostLodgementsTokenMiningLodgementResults<T>>::mutate((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id), |mining_speed_boosts_lodgements_token_mining_lodgements_result| {
                    if let Some(_mining_speed_boosts_lodgements_token_mining_lodgements_result) = mining_speed_boosts_lodgements_token_mining_lodgements_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_amount = token_claim_amount.clone();
                        _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_date_redeemed = token_claim_date_redeemed.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result = <MiningSpeedBoostLodgementsTokenMiningLodgementResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id));
                if let Some(_mining_speed_boosts_lodgements_token_mining_lodgements_result) = fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result {
                    debug::info!("Latest field token_claim_amount {:#?}", _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_amount);
                    debug::info!("Latest field token_claim_date_redeemed {:#?}", _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_date_redeemed);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_lodgements_token_mining_lodgements_result instance with the input params
                let mining_speed_boosts_lodgements_token_mining_lodgements_result_instance = MiningSpeedBoostLodgementsTokenMiningLodgementResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_claim_amount: token_claim_amount.clone(),
                    token_claim_date_redeemed: token_claim_date_redeemed.clone(),
                };

                <MiningSpeedBoostLodgementsTokenMiningLodgementResults<T>>::insert(
                    (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id),
                    &mining_speed_boosts_lodgements_token_mining_lodgements_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result = <MiningSpeedBoostLodgementsTokenMiningLodgementResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id));
                if let Some(_mining_speed_boosts_lodgements_token_mining_lodgements_result) = fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result {
                    debug::info!("Inserted field token_claim_amount {:#?}", _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_amount);
                    debug::info!("Inserted field token_claim_date_redeemed {:#?}", _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_date_redeemed);
                }
            }

            // Self::deposit_event(RawEvent::MiningSpeedBoostLodgementsTokenMiningLodgementResultSet(
            //     sender,
            //     mining_speed_boosts_configuration_token_mining_id,
            //     mining_speed_boosts_lodgements_token_mining_id,
            //     token_claim_amount,
            //     token_claim_date_redeemed,
            // ));

            // After the claim is stored, then if the user wins a proportion of the block reward
            // through validating or nominating, then we will multiply that reward by their
            // claimed eligibility to determine what mining speed bonus they should be given.
        }

        /// Set mining_speed_boosts_lodgements_token_mining_lodgements_result
        pub fn set_mining_speed_boosts_lodgements_token_mining_lodgements_result(
            origin,
            mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
            mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
            mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
            _token_claim_amount: Option<T::MiningSpeedBoostLodgementsTokenMiningLodgementAmount>,
            _token_claim_date_redeemed: Option<T::MiningSpeedBoostLodgementsTokenMiningLodgementDateRedeemed>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_lodgements_token_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_lodgements_token_mining = Self::exists_mining_speed_boosts_lodgements_token_mining(mining_speed_boosts_lodgements_token_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_lodgements_token_mining, "MiningSpeedBoostLodgementsTokenMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_lodgements_token_mining_lodgements_result they are trying to change
            ensure!(Self::mining_speed_boosts_lodgements_token_mining_owner(mining_speed_boosts_lodgements_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_lodgements_token_mining_lodgements_result");

            // TODO - adjust defaults
            let token_claim_amount = match _token_claim_amount.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_claim_date_redeemed = match _token_claim_date_redeemed {
                Some(value) => value,
                None => 1.into() // Default
            };

            // Check if a mining_speed_boosts_lodgements_token_mining_lodgements_result already exists with the given mining_speed_boosts_lodgements_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_lodgements_token_mining_lodgements_result_index(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostLodgementsTokenMiningLodgementResults<T>>::mutate((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id), |mining_speed_boosts_lodgements_token_mining_lodgements_result| {
                    if let Some(_mining_speed_boosts_lodgements_token_mining_lodgements_result) = mining_speed_boosts_lodgements_token_mining_lodgements_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_amount = token_claim_amount.clone();
                        _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_date_redeemed = token_claim_date_redeemed.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result = <MiningSpeedBoostLodgementsTokenMiningLodgementResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id));
                if let Some(_mining_speed_boosts_lodgements_token_mining_lodgements_result) = fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result {
                    debug::info!("Latest field token_claim_amount {:#?}", _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_amount);
                    debug::info!("Latest field token_claim_date_redeemed {:#?}", _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_date_redeemed);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_lodgements_token_mining_lodgements_result instance with the input params
                let mining_speed_boosts_lodgements_token_mining_lodgements_result_instance = MiningSpeedBoostLodgementsTokenMiningLodgementResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_claim_amount: token_claim_amount.clone(),
                    token_claim_date_redeemed: token_claim_date_redeemed.clone(),
                };

                <MiningSpeedBoostLodgementsTokenMiningLodgementResults<T>>::insert(
                    (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id),
                    &mining_speed_boosts_lodgements_token_mining_lodgements_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result = <MiningSpeedBoostLodgementsTokenMiningLodgementResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_lodgements_token_mining_id));
                if let Some(_mining_speed_boosts_lodgements_token_mining_lodgements_result) = fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result {
                    debug::info!("Inserted field token_claim_amount {:#?}", _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_amount);
                    debug::info!("Inserted field token_claim_date_redeemed {:#?}", _mining_speed_boosts_lodgements_token_mining_lodgements_result.token_claim_date_redeemed);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostLodgementsTokenMiningLodgementResultSet(
                sender,
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_lodgements_token_mining_id,
                token_claim_amount,
                token_claim_date_redeemed,
            ));
        }

        pub fn assign_claim_to_configuration(
          origin,
          mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
          mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token_mining = <mining_speed_boosts_configuration_token_mining::Module<T>>
                ::exists_mining_speed_boosts_configuration_token_mining(mining_speed_boosts_configuration_token_mining_id).is_ok();
            ensure!(is_configuration_token_mining, "configuration_token_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the claim to
            ensure!(
                <mining_speed_boosts_configuration_token_mining::Module<T>>::is_mining_speed_boosts_configuration_token_mining_owner(mining_speed_boosts_configuration_token_mining_id, sender.clone()).is_ok(),
                "Only the configuration_token_mining owner can assign itself a claim"
            );

            Self::associate_token_claim_with_configuration(mining_speed_boosts_lodgements_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
                .expect("Unable to associate claim with configuration");

            // Ensure that the given mining_speed_boosts_lodgements_token_mining_id already exists
            let token_claim = Self::mining_speed_boosts_lodgements_token_mining(mining_speed_boosts_lodgements_token_mining_id);
            ensure!(token_claim.is_some(), "Invalid mining_speed_boosts_lodgements_token_mining_id");

            // // Ensure that the claim is not already owned by a different configuration
            // // Unassign the claim from any existing configuration since it may only be owned by one configuration
            // <TokenMiningLodgementConfiguration<T>>::remove(mining_speed_boosts_lodgements_token_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenMiningLodgementConfiguration<T>>::insert(mining_speed_boosts_lodgements_token_mining_id, mining_speed_boosts_configuration_token_mining_id);

            Self::deposit_event(RawEvent::AssignedTokenMiningLodgementToConfiguration(sender, mining_speed_boosts_lodgements_token_mining_id, mining_speed_boosts_configuration_token_mining_id));
            }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_speed_boosts_lodgements_token_mining_owner(
        mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_lodgements_token_mining_owner(&mining_speed_boosts_lodgements_token_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostLodgementsTokenMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_lodgements_token_mining(
        mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
    ) -> Result<MiningSpeedBoostLodgementsTokenMining, DispatchError> {
        match Self::mining_speed_boosts_lodgements_token_mining(mining_speed_boosts_lodgements_token_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostLodgementsTokenMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_lodgements_token_mining_lodgements_result(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_lodgements_token_mining_lodgements_results((
            mining_speed_boosts_configuration_token_mining_id,
            mining_speed_boosts_lodgements_token_mining_id,
        )) {
            Some(value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostLodgementsTokenMiningLodgementResult does not exist")),
        }
    }

    pub fn has_value_for_mining_speed_boosts_lodgements_token_mining_lodgements_result_index(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_speed_boosts_lodgements_token_mining_lodgements_result has a value that is defined"
        );
        let fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result =
            <MiningSpeedBoostLodgementsTokenMiningLodgementResults<T>>::get((
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_lodgements_token_mining_id,
            ));
        if let Some(value) = fetched_mining_speed_boosts_lodgements_token_mining_lodgements_result {
            debug::info!("Found value for mining_speed_boosts_lodgements_token_mining_lodgements_result");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_lodgements_token_mining_lodgements_result");
        Err(DispatchError::Other("No value for mining_speed_boosts_lodgements_token_mining_lodgements_result"))
    }

    /// Only push the claim id onto the end of the vector if it does not already exist
    pub fn associate_token_claim_with_configuration(
        mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given claim id
        if let Some(configuration_lodgements) =
            Self::token_mining_configuration_lodgements(mining_speed_boosts_configuration_token_mining_id)
        {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_speed_boosts_configuration_token_mining_id,
                configuration_lodgements
            );
            let not_configuration_contains_claim =
                !configuration_lodgements.contains(&mining_speed_boosts_lodgements_token_mining_id);
            ensure!(not_configuration_contains_claim, "Configuration already contains the given claim id");
            debug::info!("Configuration id key exists but its vector value does not contain the given claim id");
            <TokenMiningConfigurationLodgements<T>>::mutate(mining_speed_boosts_configuration_token_mining_id, |v| {
                if let Some(value) = v {
                    value.push(mining_speed_boosts_lodgements_token_mining_id);
                }
            });
            debug::info!(
                "Associated claim {:?} with configuration {:?}",
                mining_speed_boosts_lodgements_token_mining_id,
                mining_speed_boosts_configuration_token_mining_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the claim \
                 id {:?} to its vector value",
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_lodgements_token_mining_id
            );
            <TokenMiningConfigurationLodgements<T>>::insert(
                mining_speed_boosts_configuration_token_mining_id,
                &vec![mining_speed_boosts_lodgements_token_mining_id],
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

    fn next_mining_speed_boosts_lodgements_token_mining_id()
    -> Result<T::MiningSpeedBoostLodgementsTokenMiningIndex, DispatchError> {
        let mining_speed_boosts_lodgements_token_mining_id = Self::mining_speed_boosts_lodgements_token_mining_count();
        if mining_speed_boosts_lodgements_token_mining_id ==
            <T::MiningSpeedBoostLodgementsTokenMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostLodgementsTokenMining count overflow"));
        }
        Ok(mining_speed_boosts_lodgements_token_mining_id)
    }

    fn insert_mining_speed_boosts_lodgements_token_mining(
        owner: &T::AccountId,
        mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
        mining_speed_boosts_lodgements_token_mining: MiningSpeedBoostLodgementsTokenMining,
    ) {
        // Create and store mining mining_speed_boosts_lodgements_token_mining
        <MiningSpeedBoostLodgementsTokenMinings<T>>::insert(
            mining_speed_boosts_lodgements_token_mining_id,
            mining_speed_boosts_lodgements_token_mining,
        );
        <MiningSpeedBoostLodgementsTokenMiningCount<T>>::put(
            mining_speed_boosts_lodgements_token_mining_id + One::one(),
        );
        <MiningSpeedBoostLodgementsTokenMiningOwners<T>>::insert(
            mining_speed_boosts_lodgements_token_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_lodgements_token_mining_id: T::MiningSpeedBoostLodgementsTokenMiningIndex,
    ) {
        <MiningSpeedBoostLodgementsTokenMiningOwners<T>>::insert(mining_speed_boosts_lodgements_token_mining_id, to);
    }
}
