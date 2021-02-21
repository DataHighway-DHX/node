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
use mining_config_token_mining;
use mining_eligibility_token_mining;
use mining_rates_token_mining;
use mining_sampling_token_mining;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + roaming_operators::Trait
    + mining_config_token_mining::Trait
    + mining_eligibility_token_mining::Trait
    + mining_rates_token_mining::Trait
    + mining_sampling_token_mining::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningClaimsTokenMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningClaimsTokenMiningClaimAmount: Parameter
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
pub struct MiningClaimsTokenMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningClaimsTokenMiningClaimResult<U, V> {
    pub token_claim_amount: U,
    pub token_claim_block_redeemed: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningClaimsTokenMiningIndex,
        <T as Trait>::MiningClaimsTokenMiningClaimAmount,
        <T as mining_config_token_mining::Trait>::MiningConfigTokenMiningIndex,
        <T as frame_system::Trait>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_claims_token_mining is created. (owner, mining_claims_token_mining_id)
        Created(AccountId, MiningClaimsTokenMiningIndex),
        /// A mining_claims_token_mining is transferred. (from, to, mining_claims_token_mining_id)
        Transferred(AccountId, AccountId, MiningClaimsTokenMiningIndex),
        MiningClaimsTokenMiningClaimResultSet(
            AccountId, MiningConfigTokenMiningIndex, MiningClaimsTokenMiningIndex,
            MiningClaimsTokenMiningClaimAmount, BlockNumber
        ),
        /// A mining_claims_token_mining is assigned to an mining_token_mining.
        /// (owner of mining_token_mining, mining_claims_token_mining_id, mining_config_token_mining_id)
        AssignedTokenMiningClaimToConfiguration(AccountId, MiningClaimsTokenMiningIndex, MiningConfigTokenMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningClaimsTokenMining {
        /// Stores all the mining_claims_token_minings, key is the mining_claims_token_mining id / index
        pub MiningClaimsTokenMinings get(fn mining_claims_token_mining): map hasher(opaque_blake2_256) T::MiningClaimsTokenMiningIndex => Option<MiningClaimsTokenMining>;

        /// Stores the total number of mining_claims_token_minings. i.e. the next mining_claims_token_mining index
        pub MiningClaimsTokenMiningCount get(fn mining_claims_token_mining_count): T::MiningClaimsTokenMiningIndex;

        /// Stores mining_claims_token_mining owner
        pub MiningClaimsTokenMiningOwners get(fn mining_claims_token_mining_owner): map hasher(opaque_blake2_256) T::MiningClaimsTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_claims_token_mining_claims_result
        pub MiningClaimsTokenMiningClaimResults get(fn mining_claims_token_mining_claims_results): map hasher(opaque_blake2_256) (T::MiningConfigTokenMiningIndex, T::MiningClaimsTokenMiningIndex) =>
            Option<MiningClaimsTokenMiningClaimResult<
                T::MiningClaimsTokenMiningClaimAmount,
                T::BlockNumber
            >>;

        /// Get mining_config_token_mining_id belonging to a mining_claims_token_mining_id
        pub TokenMiningClaimConfiguration get(fn token_mining_claim_configuration): map hasher(opaque_blake2_256) T::MiningClaimsTokenMiningIndex => Option<T::MiningConfigTokenMiningIndex>;

        /// Get mining_claims_token_mining_id's belonging to a mining_config_token_mining_id
        pub TokenMiningConfigClaims get(fn token_mining_config_claims): map hasher(opaque_blake2_256) T::MiningConfigTokenMiningIndex => Option<Vec<T::MiningClaimsTokenMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_claims_token_mining
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_claims_token_mining_id = Self::next_mining_claims_token_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_claims_token_mining
            let mining_claims_token_mining = MiningClaimsTokenMining(unique_id);
            Self::insert_mining_claims_token_mining(&sender, mining_claims_token_mining_id, mining_claims_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_claims_token_mining_id));
        }

        /// Transfer a mining_claims_token_mining to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_claims_token_mining_owner(mining_claims_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_claims_token_mining");

            Self::update_owner(&to, mining_claims_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_claims_token_mining_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn claim(
            origin,
            mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
            mining_eligibility_token_mining_id: T::MiningEligibilityTokenMiningIndex,
            mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_claims_token_mining_id whose config we want to change actually exists
            let is_mining_claims_token_mining = Self::exists_mining_claims_token_mining(mining_claims_token_mining_id).is_ok();
            ensure!(is_mining_claims_token_mining, "MiningClaimsTokenMining does not exist");

            // Ensure that the caller is owner of the mining_claims_token_mining_claims_result they are trying to change
            ensure!(Self::mining_claims_token_mining_owner(mining_claims_token_mining_id) == Some(sender.clone()), "Only owner can set mining_claims_token_mining_claims_result");

            // Check that only allow the owner of the configuration that the claim belongs to call this extrinsic
            // and claim their eligibility
            ensure!(
              <mining_config_token_mining::Module<T>>::is_mining_config_token_mining_owner(
                mining_config_token_mining_id, sender.clone()
              ).is_ok(),
              "Only the configuration_token_mining owner can claim their associated eligibility"
            );

            // Check that the extrinsic call is made after the end date defined in the provided configuration

            // FIXME
            // let current_block = <frame_system::Module<T>>::block_number();
            // // Get the config associated with the given configuration_token_mining
            // if let Some(configuration_token_mining_config) = <mining_config_token_mining::Module<T>>::mining_config_token_mining_token_configs(mining_config_token_mining_id) {
            //   if let _token_lock_interval_blocks = configuration_token_mining_config.token_lock_interval_blocks {
            //     ensure!(current_block > _token_lock_interval_blocks, "Claim may not be made until after the end of the lock interval");
            // } else {
            //     return Err(DispatchError::Other("Cannot find token_mining_config end block associated with the claim"));
            //   }
            // } else {
            //   return Err(DispatchError::Other("Cannot find token_mining_config associated with the claim"));
            // }

            // Check that the provided eligibility amount has not already been claimed
            // i.e. there should only be a single claim instance for each configuration and eligibility in the MVP
            if let Some(token_mining_config_claims) = Self::token_mining_config_claims(mining_config_token_mining_id) {
              ensure!(token_mining_config_claims.len() == 1, "Cannot have zero or more than one claim associated with configuration/eligibility");
            } else {
              return Err(DispatchError::Other("Cannot find configuration_claims associated with the claim"));
            }

            // Record the claim associated with their configuration/eligibility
            let token_claim_amount: T::MiningClaimsTokenMiningClaimAmount = 0.into();
            let token_claim_block_redeemed: T::BlockNumber = <frame_system::Module<T>>::block_number();
            if let Some(eligibility_token_mining) = <mining_eligibility_token_mining::Module<T>>::mining_eligibility_token_mining_eligibility_results((mining_config_token_mining_id, mining_eligibility_token_mining_id)) {
              if let token_mining_calculated_eligibility = eligibility_token_mining.token_calculated_eligibility {
                ensure!(token_mining_calculated_eligibility > 0.into(), "Calculated eligibility is zero. Nothing to claim.");
                // FIXME - unable to figure out how to cast here!
                // token_claim_amount = (token_mining_calculated_eligibility as T::MiningClaimsTokenMiningClaimAmount).clone();
              } else {
                return Err(DispatchError::Other("Cannot find token_mining_eligibility calculated_eligibility associated with the claim"));
              }
            } else {
              return Err(DispatchError::Other("Cannot find token_mining_eligibility associated with the claim"));
            }

            // Check if a mining_claims_token_mining_claims_result already exists with the given mining_claims_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_claims_token_mining_claims_result_index(mining_config_token_mining_id, mining_claims_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningClaimsTokenMiningClaimResults<T>>::mutate((mining_config_token_mining_id, mining_claims_token_mining_id), |mining_claims_token_mining_claims_result| {
                    if let Some(_mining_claims_token_mining_claims_result) = mining_claims_token_mining_claims_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_claims_token_mining_claims_result.token_claim_amount = token_claim_amount.clone();
                        _mining_claims_token_mining_claims_result.token_claim_block_redeemed = token_claim_block_redeemed.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_claims_token_mining_claims_result = <MiningClaimsTokenMiningClaimResults<T>>::get((mining_config_token_mining_id, mining_claims_token_mining_id));
                if let Some(_mining_claims_token_mining_claims_result) = fetched_mining_claims_token_mining_claims_result {
                    debug::info!("Latest field token_claim_amount {:#?}", _mining_claims_token_mining_claims_result.token_claim_amount);
                    debug::info!("Latest field token_claim_block_redeemed {:#?}", _mining_claims_token_mining_claims_result.token_claim_block_redeemed);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_claims_token_mining_claims_result instance with the input params
                let mining_claims_token_mining_claims_result_instance = MiningClaimsTokenMiningClaimResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_claim_amount: token_claim_amount.clone(),
                    token_claim_block_redeemed: token_claim_block_redeemed.clone(),
                };

                <MiningClaimsTokenMiningClaimResults<T>>::insert(
                    (mining_config_token_mining_id, mining_claims_token_mining_id),
                    &mining_claims_token_mining_claims_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_claims_token_mining_claims_result = <MiningClaimsTokenMiningClaimResults<T>>::get((mining_config_token_mining_id, mining_claims_token_mining_id));
                if let Some(_mining_claims_token_mining_claims_result) = fetched_mining_claims_token_mining_claims_result {
                    debug::info!("Inserted field token_claim_amount {:#?}", _mining_claims_token_mining_claims_result.token_claim_amount);
                    debug::info!("Inserted field token_claim_block_redeemed {:#?}", _mining_claims_token_mining_claims_result.token_claim_block_redeemed);
                }
            }

            // Self::deposit_event(RawEvent::MiningClaimsTokenMiningClaimResultSet(
            //     sender,
            //     mining_config_token_mining_id,
            //     mining_claims_token_mining_id,
            //     token_claim_amount,
            //     token_claim_block_redeemed,
            // ));

            // After the claim is stored, then if the user wins a proportion of the block reward
            // through validating or nominating, then we will multiply that reward by their
            // claimed eligibility to determine what mining speed bonus they should be given.
        }

        /// Set mining_claims_token_mining_claims_result
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_claims_token_mining_claims_result(
            origin,
            mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
            mining_eligibility_token_mining_id: T::MiningEligibilityTokenMiningIndex,
            mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
            _token_claim_amount: Option<T::MiningClaimsTokenMiningClaimAmount>,
            _token_claim_block_redeemed: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_claims_token_mining_id whose config we want to change actually exists
            let is_mining_claims_token_mining = Self::exists_mining_claims_token_mining(mining_claims_token_mining_id).is_ok();
            ensure!(is_mining_claims_token_mining, "MiningClaimsTokenMining does not exist");

            // Ensure that the caller is owner of the mining_claims_token_mining_claims_result they are trying to change
            ensure!(Self::mining_claims_token_mining_owner(mining_claims_token_mining_id) == Some(sender.clone()), "Only owner can set mining_claims_token_mining_claims_result");

            // TODO - adjust defaults
            let token_claim_amount = match _token_claim_amount.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_claim_block_redeemed = match _token_claim_block_redeemed {
                Some(value) => value,
                None => <frame_system::Module<T>>::block_number()
            };

            // Check if a mining_claims_token_mining_claims_result already exists with the given mining_claims_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_claims_token_mining_claims_result_index(mining_config_token_mining_id, mining_claims_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningClaimsTokenMiningClaimResults<T>>::mutate((mining_config_token_mining_id, mining_claims_token_mining_id), |mining_claims_token_mining_claims_result| {
                    if let Some(_mining_claims_token_mining_claims_result) = mining_claims_token_mining_claims_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_claims_token_mining_claims_result.token_claim_amount = token_claim_amount.clone();
                        _mining_claims_token_mining_claims_result.token_claim_block_redeemed = token_claim_block_redeemed.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_claims_token_mining_claims_result = <MiningClaimsTokenMiningClaimResults<T>>::get((mining_config_token_mining_id, mining_claims_token_mining_id));
                if let Some(_mining_claims_token_mining_claims_result) = fetched_mining_claims_token_mining_claims_result {
                    debug::info!("Latest field token_claim_amount {:#?}", _mining_claims_token_mining_claims_result.token_claim_amount);
                    debug::info!("Latest field token_claim_block_redeemed {:#?}", _mining_claims_token_mining_claims_result.token_claim_block_redeemed);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_claims_token_mining_claims_result instance with the input params
                let mining_claims_token_mining_claims_result_instance = MiningClaimsTokenMiningClaimResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_claim_amount: token_claim_amount.clone(),
                    token_claim_block_redeemed: token_claim_block_redeemed.clone(),
                };

                <MiningClaimsTokenMiningClaimResults<T>>::insert(
                    (mining_config_token_mining_id, mining_claims_token_mining_id),
                    &mining_claims_token_mining_claims_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_claims_token_mining_claims_result = <MiningClaimsTokenMiningClaimResults<T>>::get((mining_config_token_mining_id, mining_claims_token_mining_id));
                if let Some(_mining_claims_token_mining_claims_result) = fetched_mining_claims_token_mining_claims_result {
                    debug::info!("Inserted field token_claim_amount {:#?}", _mining_claims_token_mining_claims_result.token_claim_amount);
                    debug::info!("Inserted field token_claim_block_redeemed {:#?}", _mining_claims_token_mining_claims_result.token_claim_block_redeemed);
                }
            }

            Self::deposit_event(RawEvent::MiningClaimsTokenMiningClaimResultSet(
                sender,
                mining_config_token_mining_id,
                mining_claims_token_mining_id,
                token_claim_amount,
                token_claim_block_redeemed,
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_claim_to_configuration(
          origin,
          mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
          mining_config_token_mining_id: T::MiningConfigTokenMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token_mining = <mining_config_token_mining::Module<T>>
                ::exists_mining_config_token_mining(mining_config_token_mining_id).is_ok();
            ensure!(is_configuration_token_mining, "configuration_token_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the claim to
            ensure!(
                <mining_config_token_mining::Module<T>>::is_mining_config_token_mining_owner(mining_config_token_mining_id, sender.clone()).is_ok(),
                "Only the configuration_token_mining owner can assign itself a claim"
            );

            Self::associate_token_claim_with_configuration(mining_claims_token_mining_id, mining_config_token_mining_id)
                .expect("Unable to associate claim with configuration");

            // Ensure that the given mining_claims_token_mining_id already exists
            let token_claim = Self::mining_claims_token_mining(mining_claims_token_mining_id);
            ensure!(token_claim.is_some(), "Invalid mining_claims_token_mining_id");

            // // Ensure that the claim is not already owned by a different configuration
            // // Unassign the claim from any existing configuration since it may only be owned by one configuration
            // <TokenMiningClaimConfiguration<T>>::remove(mining_claims_token_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenMiningClaimConfiguration<T>>::insert(mining_claims_token_mining_id, mining_config_token_mining_id);

            Self::deposit_event(RawEvent::AssignedTokenMiningClaimToConfiguration(sender, mining_claims_token_mining_id, mining_config_token_mining_id));
            }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_claims_token_mining_owner(
        mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_claims_token_mining_owner(&mining_claims_token_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningClaimsTokenMining"
        );
        Ok(())
    }

    pub fn exists_mining_claims_token_mining(
        mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
    ) -> Result<MiningClaimsTokenMining, DispatchError> {
        match Self::mining_claims_token_mining(mining_claims_token_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningClaimsTokenMining does not exist")),
        }
    }

    pub fn exists_mining_claims_token_mining_claims_result(
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
        mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_claims_token_mining_claims_results((
            mining_config_token_mining_id,
            mining_claims_token_mining_id,
        )) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningClaimsTokenMiningClaimResult does not exist")),
        }
    }

    pub fn has_value_for_mining_claims_token_mining_claims_result_index(
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
        mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_claims_token_mining_claims_result has a value that is defined"
        );
        let fetched_mining_claims_token_mining_claims_result =
            <MiningClaimsTokenMiningClaimResults<T>>::get((
                mining_config_token_mining_id,
                mining_claims_token_mining_id,
            ));
        if let Some(_value) = fetched_mining_claims_token_mining_claims_result {
            debug::info!("Found value for mining_claims_token_mining_claims_result");
            return Ok(());
        }
        debug::info!("No value for mining_claims_token_mining_claims_result");
        Err(DispatchError::Other("No value for mining_claims_token_mining_claims_result"))
    }

    /// Only push the claim id onto the end of the vector if it does not already exist
    pub fn associate_token_claim_with_configuration(
        mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
        mining_config_token_mining_id: T::MiningConfigTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given claim id
        if let Some(configuration_claims) =
            Self::token_mining_config_claims(mining_config_token_mining_id)
        {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_config_token_mining_id,
                configuration_claims
            );
            let not_configuration_contains_claim =
                !configuration_claims.contains(&mining_claims_token_mining_id);
            ensure!(not_configuration_contains_claim, "Configuration already contains the given claim id");
            debug::info!("Configuration id key exists but its vector value does not contain the given claim id");
            <TokenMiningConfigClaims<T>>::mutate(mining_config_token_mining_id, |v| {
                if let Some(value) = v {
                    value.push(mining_claims_token_mining_id);
                }
            });
            debug::info!(
                "Associated claim {:?} with configuration {:?}",
                mining_claims_token_mining_id,
                mining_config_token_mining_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the claim \
                 id {:?} to its vector value",
                mining_config_token_mining_id,
                mining_claims_token_mining_id
            );
            <TokenMiningConfigClaims<T>>::insert(
                mining_config_token_mining_id,
                &vec![mining_claims_token_mining_id],
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

    fn next_mining_claims_token_mining_id()
    -> Result<T::MiningClaimsTokenMiningIndex, DispatchError> {
        let mining_claims_token_mining_id = Self::mining_claims_token_mining_count();
        if mining_claims_token_mining_id ==
            <T::MiningClaimsTokenMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningClaimsTokenMining count overflow"));
        }
        Ok(mining_claims_token_mining_id)
    }

    fn insert_mining_claims_token_mining(
        owner: &T::AccountId,
        mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
        mining_claims_token_mining: MiningClaimsTokenMining,
    ) {
        // Create and store mining mining_claims_token_mining
        <MiningClaimsTokenMinings<T>>::insert(
            mining_claims_token_mining_id,
            mining_claims_token_mining,
        );
        <MiningClaimsTokenMiningCount<T>>::put(
            mining_claims_token_mining_id + One::one(),
        );
        <MiningClaimsTokenMiningOwners<T>>::insert(
            mining_claims_token_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_claims_token_mining_id: T::MiningClaimsTokenMiningIndex,
    ) {
        <MiningClaimsTokenMiningOwners<T>>::insert(mining_claims_token_mining_id, to);
    }
}
