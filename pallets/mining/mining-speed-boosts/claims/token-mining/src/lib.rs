#![cfg_attr(not(feature = "std"), no_std)]

use codec::{
    Decode,
    Encode,
};
use frame_support::traits::{
    Currency,
    ExistenceRequirement,
    Randomness,
};
/// A runtime module for managing non-fungible tokens
use frame_support::{
    debug,
    decl_event,
    decl_module,
    decl_storage,
    ensure,
    Parameter,
};
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
use system::ensure_signed;

// FIXME - remove roaming_operators here, only use this approach since do not know how to use BalanceOf using only
// mining-speed-boosts runtime module
use mining_speed_boosts_configuration_token_mining;
use mining_speed_boosts_eligibility_token_mining;
use mining_speed_boosts_rates_token_mining;
use mining_speed_boosts_sampling_token_mining;
use roaming_operators;

/// The module's trait.
pub trait Trait:
    system::Trait
    + roaming_operators::Trait
    + mining_speed_boosts_configuration_token_mining::Trait
    + mining_speed_boosts_eligibility_token_mining::Trait
    + mining_speed_boosts_rates_token_mining::Trait
    + mining_speed_boosts_sampling_token_mining::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MiningSpeedBoostClaimsTokenMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostClaimsTokenMiningClaimAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostClaimsTokenMiningClaimDateRedeemed: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostClaimsTokenMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostClaimsTokenMiningClaimResult<U, V> {
    pub token_claim_amount: U,
    pub token_claim_date_redeemed: V,
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostClaimsTokenMiningIndex,
        <T as Trait>::MiningSpeedBoostClaimsTokenMiningClaimAmount,
        <T as Trait>::MiningSpeedBoostClaimsTokenMiningClaimDateRedeemed,
        <T as mining_speed_boosts_configuration_token_mining::Trait>::MiningSpeedBoostConfigurationTokenMiningIndex,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_claims_token_mining is created. (owner, mining_speed_boosts_claims_token_mining_id)
        Created(AccountId, MiningSpeedBoostClaimsTokenMiningIndex),
        /// A mining_speed_boosts_claims_token_mining is transferred. (from, to, mining_speed_boosts_claims_token_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostClaimsTokenMiningIndex),
        MiningSpeedBoostClaimsTokenMiningClaimResultSet(
            AccountId, MiningSpeedBoostConfigurationTokenMiningIndex, MiningSpeedBoostClaimsTokenMiningIndex,
            MiningSpeedBoostClaimsTokenMiningClaimAmount, MiningSpeedBoostClaimsTokenMiningClaimDateRedeemed
        ),
        /// A mining_speed_boosts_claims_token_mining is assigned to an mining_speed_boosts_token_mining.
        /// (owner of mining_speed_boosts_token_mining, mining_speed_boosts_claims_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
            AssignedTokenMiningClaimToConfiguration(AccountId, MiningSpeedBoostClaimsTokenMiningIndex, MiningSpeedBoostConfigurationTokenMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostClaimsTokenMining {
        /// Stores all the mining_speed_boosts_claims_token_minings, key is the mining_speed_boosts_claims_token_mining id / index
        pub MiningSpeedBoostClaimsTokenMinings get(fn mining_speed_boosts_claims_token_mining): map hasher(blake2_256) T::MiningSpeedBoostClaimsTokenMiningIndex => Option<MiningSpeedBoostClaimsTokenMining>;

        /// Stores the total number of mining_speed_boosts_claims_token_minings. i.e. the next mining_speed_boosts_claims_token_mining index
        pub MiningSpeedBoostClaimsTokenMiningCount get(fn mining_speed_boosts_claims_token_mining_count): T::MiningSpeedBoostClaimsTokenMiningIndex;

        /// Stores mining_speed_boosts_claims_token_mining owner
        pub MiningSpeedBoostClaimsTokenMiningOwners get(fn mining_speed_boosts_claims_token_mining_owner): map hasher(blake2_256) T::MiningSpeedBoostClaimsTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_claims_token_mining_claims_result
        pub MiningSpeedBoostClaimsTokenMiningClaimResults get(fn mining_speed_boosts_claims_token_mining_claims_results): map hasher(blake2_256) (T::MiningSpeedBoostConfigurationTokenMiningIndex, T::MiningSpeedBoostClaimsTokenMiningIndex) =>
            Option<MiningSpeedBoostClaimsTokenMiningClaimResult<
                T::MiningSpeedBoostClaimsTokenMiningClaimAmount,
                T::MiningSpeedBoostClaimsTokenMiningClaimDateRedeemed
            >>;

        /// Get mining_speed_boosts_configuration_token_mining_id belonging to a mining_speed_boosts_claims_token_mining_id
        pub TokenMiningClaimConfiguration get(fn token_mining_claim_configuration): map hasher(blake2_256) T::MiningSpeedBoostClaimsTokenMiningIndex => Option<T::MiningSpeedBoostConfigurationTokenMiningIndex>;

        /// Get mining_speed_boosts_claims_token_mining_id's belonging to a mining_speed_boosts_configuration_token_mining_id
        pub TokenMiningConfigurationClaims get(fn token_mining_configuration_claims): map hasher(blake2_256) T::MiningSpeedBoostConfigurationTokenMiningIndex => Option<Vec<T::MiningSpeedBoostClaimsTokenMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_claims_token_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_claims_token_mining_id = Self::next_mining_speed_boosts_claims_token_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_claims_token_mining
            let mining_speed_boosts_claims_token_mining = MiningSpeedBoostClaimsTokenMining(unique_id);
            Self::insert_mining_speed_boosts_claims_token_mining(&sender, mining_speed_boosts_claims_token_mining_id, mining_speed_boosts_claims_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_claims_token_mining_id));
        }

        /// Transfer a mining_speed_boosts_claims_token_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_claims_token_mining_owner(mining_speed_boosts_claims_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_claims_token_mining");

            Self::update_owner(&to, mining_speed_boosts_claims_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_claims_token_mining_id));
        }

        pub fn claim(
            origin,
            mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
            mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
            mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_claims_token_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_claims_token_mining = Self::exists_mining_speed_boosts_claims_token_mining(mining_speed_boosts_claims_token_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_claims_token_mining, "MiningSpeedBoostClaimsTokenMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_claims_token_mining_claims_result they are trying to change
            ensure!(Self::mining_speed_boosts_claims_token_mining_owner(mining_speed_boosts_claims_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_claims_token_mining_claims_result");

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
                // ensure!(TIME_NOW > token_lock_period_end_date, "Claim may not be made until after the end date of the lock period");
              } else {
                return Err(DispatchError::Other("Cannot find token_mining_config end_date associated with the claim"));
              }
            } else {
              return Err(DispatchError::Other("Cannot find token_mining_config associated with the claim"));
            }

            // Check that the provided eligibility amount has not already been claimed
            // i.e. there should only be a single claim instance for each configuration and eligibility in the MVP
            if let Some(token_mining_configuration_claims) = Self::token_mining_configuration_claims(mining_speed_boosts_configuration_token_mining_id) {
              ensure!(token_mining_configuration_claims.len() == 1, "Cannot have zero or more than one claim associated with configuration/eligibility");
            } else {
              return Err(DispatchError::Other("Cannot find configuration_claims associated with the claim"));
            }

            // Record the claim associated with their configuration/eligibility
            let token_claim_amount: T::MiningSpeedBoostClaimsTokenMiningClaimAmount = 0.into();
            let token_claim_date_redeemed: T::MiningSpeedBoostClaimsTokenMiningClaimDateRedeemed = TIME_NOW;
            if let Some(eligibility_token_mining) = <mining_speed_boosts_eligibility_token_mining::Module<T>>::mining_speed_boosts_eligibility_token_mining_eligibility_results((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_eligibility_token_mining_id)) {
              if let token_mining_calculated_eligibility = eligibility_token_mining.eligibility_token_mining_calculated_eligibility {
                ensure!(token_mining_calculated_eligibility > 0.into(), "Calculated eligibility is zero. Nothing to claim.");
                // FIXME - unable to figure out how to cast here!
                // token_claim_amount = (token_mining_calculated_eligibility as T::MiningSpeedBoostClaimsTokenMiningClaimAmount).clone();
              } else {
                return Err(DispatchError::Other("Cannot find token_mining_eligibility calculated_eligibility associated with the claim"));
              }
            } else {
              return Err(DispatchError::Other("Cannot find token_mining_eligibility associated with the claim"));
            }

            // Check if a mining_speed_boosts_claims_token_mining_claims_result already exists with the given mining_speed_boosts_claims_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_claims_token_mining_claims_result_index(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostClaimsTokenMiningClaimResults<T>>::mutate((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id), |mining_speed_boosts_claims_token_mining_claims_result| {
                    if let Some(_mining_speed_boosts_claims_token_mining_claims_result) = mining_speed_boosts_claims_token_mining_claims_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_claims_token_mining_claims_result.token_claim_amount = token_claim_amount.clone();
                        _mining_speed_boosts_claims_token_mining_claims_result.token_claim_date_redeemed = token_claim_date_redeemed.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_claims_token_mining_claims_result = <MiningSpeedBoostClaimsTokenMiningClaimResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id));
                if let Some(_mining_speed_boosts_claims_token_mining_claims_result) = fetched_mining_speed_boosts_claims_token_mining_claims_result {
                    debug::info!("Latest field token_claim_amount {:#?}", _mining_speed_boosts_claims_token_mining_claims_result.token_claim_amount);
                    debug::info!("Latest field token_claim_date_redeemed {:#?}", _mining_speed_boosts_claims_token_mining_claims_result.token_claim_date_redeemed);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_claims_token_mining_claims_result instance with the input params
                let mining_speed_boosts_claims_token_mining_claims_result_instance = MiningSpeedBoostClaimsTokenMiningClaimResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_claim_amount: token_claim_amount.clone(),
                    token_claim_date_redeemed: token_claim_date_redeemed.clone(),
                };

                <MiningSpeedBoostClaimsTokenMiningClaimResults<T>>::insert(
                    (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id),
                    &mining_speed_boosts_claims_token_mining_claims_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_claims_token_mining_claims_result = <MiningSpeedBoostClaimsTokenMiningClaimResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id));
                if let Some(_mining_speed_boosts_claims_token_mining_claims_result) = fetched_mining_speed_boosts_claims_token_mining_claims_result {
                    debug::info!("Inserted field token_claim_amount {:#?}", _mining_speed_boosts_claims_token_mining_claims_result.token_claim_amount);
                    debug::info!("Inserted field token_claim_date_redeemed {:#?}", _mining_speed_boosts_claims_token_mining_claims_result.token_claim_date_redeemed);
                }
            }

            // Self::deposit_event(RawEvent::MiningSpeedBoostClaimsTokenMiningClaimResultSet(
            //     sender,
            //     mining_speed_boosts_configuration_token_mining_id,
            //     mining_speed_boosts_claims_token_mining_id,
            //     token_claim_amount,
            //     token_claim_date_redeemed,
            // ));

            // After the claim is stored, then if the user wins a proportion of the block reward
            // through validating or nominating, then we will multiply that reward by their
            // claimed eligibility to determine what mining speed bonus they should be given.
        }

        /// Set mining_speed_boosts_claims_token_mining_claims_result
        pub fn set_mining_speed_boosts_claims_token_mining_claims_result(
            origin,
            mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
            mining_speed_boosts_eligibility_token_mining_id: T::MiningSpeedBoostEligibilityTokenMiningIndex,
            mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
            _token_claim_amount: Option<T::MiningSpeedBoostClaimsTokenMiningClaimAmount>,
            _token_claim_date_redeemed: Option<T::MiningSpeedBoostClaimsTokenMiningClaimDateRedeemed>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_claims_token_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_claims_token_mining = Self::exists_mining_speed_boosts_claims_token_mining(mining_speed_boosts_claims_token_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_claims_token_mining, "MiningSpeedBoostClaimsTokenMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_claims_token_mining_claims_result they are trying to change
            ensure!(Self::mining_speed_boosts_claims_token_mining_owner(mining_speed_boosts_claims_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_claims_token_mining_claims_result");

            // TODO - adjust defaults
            let token_claim_amount = match _token_claim_amount.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_claim_date_redeemed = match _token_claim_date_redeemed {
                Some(value) => value,
                None => 1.into() // Default
            };

            // Check if a mining_speed_boosts_claims_token_mining_claims_result already exists with the given mining_speed_boosts_claims_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_claims_token_mining_claims_result_index(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostClaimsTokenMiningClaimResults<T>>::mutate((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id), |mining_speed_boosts_claims_token_mining_claims_result| {
                    if let Some(_mining_speed_boosts_claims_token_mining_claims_result) = mining_speed_boosts_claims_token_mining_claims_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_claims_token_mining_claims_result.token_claim_amount = token_claim_amount.clone();
                        _mining_speed_boosts_claims_token_mining_claims_result.token_claim_date_redeemed = token_claim_date_redeemed.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_claims_token_mining_claims_result = <MiningSpeedBoostClaimsTokenMiningClaimResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id));
                if let Some(_mining_speed_boosts_claims_token_mining_claims_result) = fetched_mining_speed_boosts_claims_token_mining_claims_result {
                    debug::info!("Latest field token_claim_amount {:#?}", _mining_speed_boosts_claims_token_mining_claims_result.token_claim_amount);
                    debug::info!("Latest field token_claim_date_redeemed {:#?}", _mining_speed_boosts_claims_token_mining_claims_result.token_claim_date_redeemed);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_claims_token_mining_claims_result instance with the input params
                let mining_speed_boosts_claims_token_mining_claims_result_instance = MiningSpeedBoostClaimsTokenMiningClaimResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_claim_amount: token_claim_amount.clone(),
                    token_claim_date_redeemed: token_claim_date_redeemed.clone(),
                };

                <MiningSpeedBoostClaimsTokenMiningClaimResults<T>>::insert(
                    (mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id),
                    &mining_speed_boosts_claims_token_mining_claims_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_claims_token_mining_claims_result = <MiningSpeedBoostClaimsTokenMiningClaimResults<T>>::get((mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_claims_token_mining_id));
                if let Some(_mining_speed_boosts_claims_token_mining_claims_result) = fetched_mining_speed_boosts_claims_token_mining_claims_result {
                    debug::info!("Inserted field token_claim_amount {:#?}", _mining_speed_boosts_claims_token_mining_claims_result.token_claim_amount);
                    debug::info!("Inserted field token_claim_date_redeemed {:#?}", _mining_speed_boosts_claims_token_mining_claims_result.token_claim_date_redeemed);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostClaimsTokenMiningClaimResultSet(
                sender,
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_claims_token_mining_id,
                token_claim_amount,
                token_claim_date_redeemed,
            ));
        }

        pub fn assign_claim_to_configuration(
          origin,
          mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
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

            Self::associate_token_claim_with_configuration(mining_speed_boosts_claims_token_mining_id, mining_speed_boosts_configuration_token_mining_id)
                .expect("Unable to associate claim with configuration");

            // Ensure that the given mining_speed_boosts_claims_token_mining_id already exists
            let token_claim = Self::mining_speed_boosts_claims_token_mining(mining_speed_boosts_claims_token_mining_id);
            ensure!(token_claim.is_some(), "Invalid mining_speed_boosts_claims_token_mining_id");

            // // Ensure that the claim is not already owned by a different configuration
            // // Unassign the claim from any existing configuration since it may only be owned by one configuration
            // <TokenMiningClaimConfiguration<T>>::remove(mining_speed_boosts_claims_token_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenMiningClaimConfiguration<T>>::insert(mining_speed_boosts_claims_token_mining_id, mining_speed_boosts_configuration_token_mining_id);

            Self::deposit_event(RawEvent::AssignedTokenMiningClaimToConfiguration(sender, mining_speed_boosts_claims_token_mining_id, mining_speed_boosts_configuration_token_mining_id));
            }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_speed_boosts_claims_token_mining_owner(
        mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_claims_token_mining_owner(&mining_speed_boosts_claims_token_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostClaimsTokenMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_claims_token_mining(
        mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
    ) -> Result<MiningSpeedBoostClaimsTokenMining, DispatchError> {
        match Self::mining_speed_boosts_claims_token_mining(mining_speed_boosts_claims_token_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostClaimsTokenMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_claims_token_mining_claims_result(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_claims_token_mining_claims_results((
            mining_speed_boosts_configuration_token_mining_id,
            mining_speed_boosts_claims_token_mining_id,
        )) {
            Some(value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostClaimsTokenMiningClaimResult does not exist")),
        }
    }

    pub fn has_value_for_mining_speed_boosts_claims_token_mining_claims_result_index(
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
        mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_speed_boosts_claims_token_mining_claims_result has a value that is defined");
        let fetched_mining_speed_boosts_claims_token_mining_claims_result =
            <MiningSpeedBoostClaimsTokenMiningClaimResults<T>>::get((
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_claims_token_mining_id,
            ));
        if let Some(value) = fetched_mining_speed_boosts_claims_token_mining_claims_result {
            debug::info!("Found value for mining_speed_boosts_claims_token_mining_claims_result");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_claims_token_mining_claims_result");
        Err(DispatchError::Other("No value for mining_speed_boosts_claims_token_mining_claims_result"))
    }

    /// Only push the claim id onto the end of the vector if it does not already exist
    pub fn associate_token_claim_with_configuration(
        mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
        mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given claim id
        if let Some(configuration_claims) =
            Self::token_mining_configuration_claims(mining_speed_boosts_configuration_token_mining_id)
        {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_speed_boosts_configuration_token_mining_id,
                configuration_claims
            );
            let not_configuration_contains_claim =
                !configuration_claims.contains(&mining_speed_boosts_claims_token_mining_id);
            ensure!(not_configuration_contains_claim, "Configuration already contains the given claim id");
            debug::info!("Configuration id key exists but its vector value does not contain the given claim id");
            <TokenMiningConfigurationClaims<T>>::mutate(mining_speed_boosts_configuration_token_mining_id, |v| {
                if let Some(value) = v {
                    value.push(mining_speed_boosts_claims_token_mining_id);
                }
            });
            debug::info!(
                "Associated claim {:?} with configuration {:?}",
                mining_speed_boosts_claims_token_mining_id,
                mining_speed_boosts_configuration_token_mining_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the claim \
                 id {:?} to its vector value",
                mining_speed_boosts_configuration_token_mining_id,
                mining_speed_boosts_claims_token_mining_id
            );
            <TokenMiningConfigurationClaims<T>>::insert(
                mining_speed_boosts_configuration_token_mining_id,
                &vec![mining_speed_boosts_claims_token_mining_id],
            );
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

    fn next_mining_speed_boosts_claims_token_mining_id()
    -> Result<T::MiningSpeedBoostClaimsTokenMiningIndex, DispatchError> {
        let mining_speed_boosts_claims_token_mining_id = Self::mining_speed_boosts_claims_token_mining_count();
        if mining_speed_boosts_claims_token_mining_id ==
            <T::MiningSpeedBoostClaimsTokenMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostClaimsTokenMining count overflow"));
        }
        Ok(mining_speed_boosts_claims_token_mining_id)
    }

    fn insert_mining_speed_boosts_claims_token_mining(
        owner: &T::AccountId,
        mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
        mining_speed_boosts_claims_token_mining: MiningSpeedBoostClaimsTokenMining,
    ) {
        // Create and store mining mining_speed_boosts_claims_token_mining
        <MiningSpeedBoostClaimsTokenMinings<T>>::insert(
            mining_speed_boosts_claims_token_mining_id,
            mining_speed_boosts_claims_token_mining,
        );
        <MiningSpeedBoostClaimsTokenMiningCount<T>>::put(mining_speed_boosts_claims_token_mining_id + One::one());
        <MiningSpeedBoostClaimsTokenMiningOwners<T>>::insert(mining_speed_boosts_claims_token_mining_id, owner.clone());
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_claims_token_mining_id: T::MiningSpeedBoostClaimsTokenMiningIndex,
    ) {
        <MiningSpeedBoostClaimsTokenMiningOwners<T>>::insert(mining_speed_boosts_claims_token_mining_id, to);
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_ok,
        impl_outer_origin,
        parameter_types,
        weights::Weight,
    };
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{
            BlakeTwo256,
            IdentityLookup,
        },
        Perbill,
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
        type AccountData = ();
        type AccountId = u64;
        type AvailableBlockRatio = AvailableBlockRatio;
        type BlockHashCount = BlockHashCount;
        type BlockNumber = u64;
        type Call = ();
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Header = Header;
        type Index = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type MaximumBlockLength = MaximumBlockLength;
        type MaximumBlockWeight = MaximumBlockWeight;
        type ModuleToIndex = ();
        type OnNewAccount = ();
        type OnReapAccount = ();
        type Origin = Origin;
        type Version = ();
    }
    impl balances::Trait for Test {
        type AccountStore = ();
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ();
    }
    impl transaction_payment::Trait for Test {
        type Currency = Balances;
        type FeeMultiplierUpdate = ();
        type OnTransactionPayment = ();
        type TransactionBaseFee = ();
        type TransactionByteFee = ();
        type WeightToFee = ();
    }
    // FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
    impl roaming_operators::Trait for Test {
        type Currency = Balances;
        type Event = ();
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl mining_speed_boosts_configuration_token_mining::Trait for Test {
        type Event = ();
        // FIXME - restore when stop temporarily using roaming-operators
        // type Currency = Balances;
        // type Randomness = RandomnessCollectiveFlip;
        type MiningSpeedBoostConfigurationTokenMiningIndex = u64;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod = u32;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate = u64;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate = u64;
        // type MiningSpeedBoostConfigurationTokenMiningTokenType = MiningSpeedBoostConfigurationTokenMiningTokenTypes;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockedAmount = u64;
        // Mining Speed Boost Token Mining Config
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningSpeedBoostConfigurationTokenMiningTokenType = Vec<u8>;
    }
    impl mining_speed_boosts_eligibility_token_mining::Trait for Test {
        type Event = ();
        type MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility = u64;
        type MiningSpeedBoostEligibilityTokenMiningIndex = u64;
        type MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage = u32;
        // type MiningSpeedBoostEligibilityTokenMiningDateAudited = u64;
        // type MiningSpeedBoostEligibilityTokenMiningAuditorAccountID = u64;
    }
    impl mining_speed_boosts_rates_token_mining::Trait for Test {
        type Event = ();
        type MiningSpeedBoostRatesTokenMiningIndex = u64;
        type MiningSpeedBoostRatesTokenMiningMaxLoyalty = u32;
        // Mining Speed Boost Max Rates
        type MiningSpeedBoostRatesTokenMiningMaxToken = u32;
        type MiningSpeedBoostRatesTokenMiningTokenDOT = u32;
        type MiningSpeedBoostRatesTokenMiningTokenIOTA = u32;
        // Mining Speed Boost Rate
        type MiningSpeedBoostRatesTokenMiningTokenMXC = u32;
    }
    impl mining_speed_boosts_sampling_token_mining::Trait for Test {
        type Event = ();
        type MiningSpeedBoostSamplingTokenMiningIndex = u64;
        type MiningSpeedBoostSamplingTokenMiningSampleDate = u64;
        type MiningSpeedBoostSamplingTokenMiningSampleTokensLocked = u64;
    }
    impl Trait for Test {
        type Event = ();
        type MiningSpeedBoostClaimsTokenMiningClaimAmount = u64;
        type MiningSpeedBoostClaimsTokenMiningClaimDateRedeemed = u64;
        type MiningSpeedBoostClaimsTokenMiningIndex = u64;
    }
    // type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type MiningSpeedBoostClaimsTokenMiningTestModule = Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }
}
