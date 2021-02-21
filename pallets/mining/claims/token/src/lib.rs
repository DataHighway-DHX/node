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
use mining_config_token;
use mining_eligibility_token;
use mining_rates_token;
use mining_sampling_token;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + roaming_operators::Trait
    + mining_config_token::Trait
    + mining_eligibility_token::Trait
    + mining_rates_token::Trait
    + mining_sampling_token::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningClaimsTokenIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningClaimsTokenClaimAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningClaimsToken(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningClaimsTokenClaimResult<U, V> {
    pub token_claim_amount: U,
    pub token_claim_block_redeemed: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningClaimsTokenIndex,
        <T as Trait>::MiningClaimsTokenClaimAmount,
        <T as mining_config_token::Trait>::MiningConfigTokenIndex,
        <T as frame_system::Trait>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_claims_token is created. (owner, mining_claims_token_id)
        Created(AccountId, MiningClaimsTokenIndex),
        /// A mining_claims_token is transferred. (from, to, mining_claims_token_id)
        Transferred(AccountId, AccountId, MiningClaimsTokenIndex),
        MiningClaimsTokenClaimResultSet(
            AccountId, MiningConfigTokenIndex, MiningClaimsTokenIndex,
            MiningClaimsTokenClaimAmount, BlockNumber
        ),
        /// A mining_claims_token is assigned to an mining_token.
        /// (owner of mining_token, mining_claims_token_id, mining_config_token_id)
        AssignedTokenClaimToConfiguration(AccountId, MiningClaimsTokenIndex, MiningConfigTokenIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningClaimsToken {
        /// Stores all the mining_claims_tokens, key is the mining_claims_token id / index
        pub MiningClaimsTokens get(fn mining_claims_token): map hasher(opaque_blake2_256) T::MiningClaimsTokenIndex => Option<MiningClaimsToken>;

        /// Stores the total number of mining_claims_tokens. i.e. the next mining_claims_token index
        pub MiningClaimsTokenCount get(fn mining_claims_token_count): T::MiningClaimsTokenIndex;

        /// Stores mining_claims_token owner
        pub MiningClaimsTokenOwners get(fn mining_claims_token_owner): map hasher(opaque_blake2_256) T::MiningClaimsTokenIndex => Option<T::AccountId>;

        /// Stores mining_claims_token_claims_result
        pub MiningClaimsTokenClaimResults get(fn mining_claims_token_claims_results): map hasher(opaque_blake2_256) (T::MiningConfigTokenIndex, T::MiningClaimsTokenIndex) =>
            Option<MiningClaimsTokenClaimResult<
                T::MiningClaimsTokenClaimAmount,
                T::BlockNumber
            >>;

        /// Get mining_config_token_id belonging to a mining_claims_token_id
        pub TokenClaimConfiguration get(fn token_claim_configuration): map hasher(opaque_blake2_256) T::MiningClaimsTokenIndex => Option<T::MiningConfigTokenIndex>;

        /// Get mining_claims_token_id's belonging to a mining_config_token_id
        pub TokenConfigClaims get(fn token_config_claims): map hasher(opaque_blake2_256) T::MiningConfigTokenIndex => Option<Vec<T::MiningClaimsTokenIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_claims_token
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_claims_token_id = Self::next_mining_claims_token_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_claims_token
            let mining_claims_token = MiningClaimsToken(unique_id);
            Self::insert_mining_claims_token(&sender, mining_claims_token_id, mining_claims_token);

            Self::deposit_event(RawEvent::Created(sender, mining_claims_token_id));
        }

        /// Transfer a mining_claims_token to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_claims_token_id: T::MiningClaimsTokenIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_claims_token_owner(mining_claims_token_id) == Some(sender.clone()), "Only owner can transfer mining mining_claims_token");

            Self::update_owner(&to, mining_claims_token_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_claims_token_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn claim(
            origin,
            mining_config_token_id: T::MiningConfigTokenIndex,
            mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
            mining_claims_token_id: T::MiningClaimsTokenIndex,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_claims_token_id whose config we want to change actually exists
            let is_mining_claims_token = Self::exists_mining_claims_token(mining_claims_token_id).is_ok();
            ensure!(is_mining_claims_token, "MiningClaimsToken does not exist");

            // Ensure that the caller is owner of the mining_claims_token_claims_result they are trying to change
            ensure!(Self::mining_claims_token_owner(mining_claims_token_id) == Some(sender.clone()), "Only owner can set mining_claims_token_claims_result");

            // Check that only allow the owner of the configuration that the claim belongs to call this extrinsic
            // and claim their eligibility
            ensure!(
              <mining_config_token::Module<T>>::is_mining_config_token_owner(
                mining_config_token_id, sender.clone()
              ).is_ok(),
              "Only the configuration_token owner can claim their associated eligibility"
            );

            // Check that the extrinsic call is made after the end date defined in the provided configuration

            // FIXME
            // let current_block = <frame_system::Module<T>>::block_number();
            // // Get the config associated with the given configuration_token
            // if let Some(configuration_token_config) = <mining_config_token::Module<T>>::mining_config_token_token_configs(mining_config_token_id) {
            //   if let _token_lock_interval_blocks = configuration_token_config.token_lock_interval_blocks {
            //     ensure!(current_block > _token_lock_interval_blocks, "Claim may not be made until after the end of the lock interval");
            // } else {
            //     return Err(DispatchError::Other("Cannot find token_config end block associated with the claim"));
            //   }
            // } else {
            //   return Err(DispatchError::Other("Cannot find token_config associated with the claim"));
            // }

            // Check that the provided eligibility amount has not already been claimed
            // i.e. there should only be a single claim instance for each configuration and eligibility in the MVP
            if let Some(token_config_claims) = Self::token_config_claims(mining_config_token_id) {
              ensure!(token_config_claims.len() == 1, "Cannot have zero or more than one claim associated with configuration/eligibility");
            } else {
              return Err(DispatchError::Other("Cannot find configuration_claims associated with the claim"));
            }

            // Record the claim associated with their configuration/eligibility
            let token_claim_amount: T::MiningClaimsTokenClaimAmount = 0.into();
            let token_claim_block_redeemed: T::BlockNumber = <frame_system::Module<T>>::block_number();
            if let Some(eligibility_token) = <mining_eligibility_token::Module<T>>::mining_eligibility_token_eligibility_results((mining_config_token_id, mining_eligibility_token_id)) {
              if let token_calculated_eligibility = eligibility_token.token_calculated_eligibility {
                ensure!(token_calculated_eligibility > 0.into(), "Calculated eligibility is zero. Nothing to claim.");
                // FIXME - unable to figure out how to cast here!
                // token_claim_amount = (token_calculated_eligibility as T::MiningClaimsTokenClaimAmount).clone();
              } else {
                return Err(DispatchError::Other("Cannot find token_eligibility calculated_eligibility associated with the claim"));
              }
            } else {
              return Err(DispatchError::Other("Cannot find token_eligibility associated with the claim"));
            }

            // Check if a mining_claims_token_claims_result already exists with the given mining_claims_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_claims_token_claims_result_index(mining_config_token_id, mining_claims_token_id).is_ok() {
                debug::info!("Mutating values");
                <MiningClaimsTokenClaimResults<T>>::mutate((mining_config_token_id, mining_claims_token_id), |mining_claims_token_claims_result| {
                    if let Some(_mining_claims_token_claims_result) = mining_claims_token_claims_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_claims_token_claims_result.token_claim_amount = token_claim_amount.clone();
                        _mining_claims_token_claims_result.token_claim_block_redeemed = token_claim_block_redeemed.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_claims_token_claims_result = <MiningClaimsTokenClaimResults<T>>::get((mining_config_token_id, mining_claims_token_id));
                if let Some(_mining_claims_token_claims_result) = fetched_mining_claims_token_claims_result {
                    debug::info!("Latest field token_claim_amount {:#?}", _mining_claims_token_claims_result.token_claim_amount);
                    debug::info!("Latest field token_claim_block_redeemed {:#?}", _mining_claims_token_claims_result.token_claim_block_redeemed);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_claims_token_claims_result instance with the input params
                let mining_claims_token_claims_result_instance = MiningClaimsTokenClaimResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_claim_amount: token_claim_amount.clone(),
                    token_claim_block_redeemed: token_claim_block_redeemed.clone(),
                };

                <MiningClaimsTokenClaimResults<T>>::insert(
                    (mining_config_token_id, mining_claims_token_id),
                    &mining_claims_token_claims_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_claims_token_claims_result = <MiningClaimsTokenClaimResults<T>>::get((mining_config_token_id, mining_claims_token_id));
                if let Some(_mining_claims_token_claims_result) = fetched_mining_claims_token_claims_result {
                    debug::info!("Inserted field token_claim_amount {:#?}", _mining_claims_token_claims_result.token_claim_amount);
                    debug::info!("Inserted field token_claim_block_redeemed {:#?}", _mining_claims_token_claims_result.token_claim_block_redeemed);
                }
            }

            // Self::deposit_event(RawEvent::MiningClaimsTokenClaimResultSet(
            //     sender,
            //     mining_config_token_id,
            //     mining_claims_token_id,
            //     token_claim_amount,
            //     token_claim_block_redeemed,
            // ));

            // After the claim is stored, then if the user wins a proportion of the block reward
            // through validating or nominating, then we will multiply that reward by their
            // claimed eligibility to determine what mining speed bonus they should be given.
        }

        /// Set mining_claims_token_claims_result
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_claims_token_claims_result(
            origin,
            mining_config_token_id: T::MiningConfigTokenIndex,
            mining_eligibility_token_id: T::MiningEligibilityTokenIndex,
            mining_claims_token_id: T::MiningClaimsTokenIndex,
            _token_claim_amount: Option<T::MiningClaimsTokenClaimAmount>,
            _token_claim_block_redeemed: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_claims_token_id whose config we want to change actually exists
            let is_mining_claims_token = Self::exists_mining_claims_token(mining_claims_token_id).is_ok();
            ensure!(is_mining_claims_token, "MiningClaimsToken does not exist");

            // Ensure that the caller is owner of the mining_claims_token_claims_result they are trying to change
            ensure!(Self::mining_claims_token_owner(mining_claims_token_id) == Some(sender.clone()), "Only owner can set mining_claims_token_claims_result");

            // TODO - adjust defaults
            let token_claim_amount = match _token_claim_amount.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_claim_block_redeemed = match _token_claim_block_redeemed {
                Some(value) => value,
                None => <frame_system::Module<T>>::block_number()
            };

            // Check if a mining_claims_token_claims_result already exists with the given mining_claims_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_claims_token_claims_result_index(mining_config_token_id, mining_claims_token_id).is_ok() {
                debug::info!("Mutating values");
                <MiningClaimsTokenClaimResults<T>>::mutate((mining_config_token_id, mining_claims_token_id), |mining_claims_token_claims_result| {
                    if let Some(_mining_claims_token_claims_result) = mining_claims_token_claims_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_claims_token_claims_result.token_claim_amount = token_claim_amount.clone();
                        _mining_claims_token_claims_result.token_claim_block_redeemed = token_claim_block_redeemed.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_claims_token_claims_result = <MiningClaimsTokenClaimResults<T>>::get((mining_config_token_id, mining_claims_token_id));
                if let Some(_mining_claims_token_claims_result) = fetched_mining_claims_token_claims_result {
                    debug::info!("Latest field token_claim_amount {:#?}", _mining_claims_token_claims_result.token_claim_amount);
                    debug::info!("Latest field token_claim_block_redeemed {:#?}", _mining_claims_token_claims_result.token_claim_block_redeemed);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_claims_token_claims_result instance with the input params
                let mining_claims_token_claims_result_instance = MiningClaimsTokenClaimResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_claim_amount: token_claim_amount.clone(),
                    token_claim_block_redeemed: token_claim_block_redeemed.clone(),
                };

                <MiningClaimsTokenClaimResults<T>>::insert(
                    (mining_config_token_id, mining_claims_token_id),
                    &mining_claims_token_claims_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_claims_token_claims_result = <MiningClaimsTokenClaimResults<T>>::get((mining_config_token_id, mining_claims_token_id));
                if let Some(_mining_claims_token_claims_result) = fetched_mining_claims_token_claims_result {
                    debug::info!("Inserted field token_claim_amount {:#?}", _mining_claims_token_claims_result.token_claim_amount);
                    debug::info!("Inserted field token_claim_block_redeemed {:#?}", _mining_claims_token_claims_result.token_claim_block_redeemed);
                }
            }

            Self::deposit_event(RawEvent::MiningClaimsTokenClaimResultSet(
                sender,
                mining_config_token_id,
                mining_claims_token_id,
                token_claim_amount,
                token_claim_block_redeemed,
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_claim_to_configuration(
          origin,
          mining_claims_token_id: T::MiningClaimsTokenIndex,
          mining_config_token_id: T::MiningConfigTokenIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_token = <mining_config_token::Module<T>>
                ::exists_mining_config_token(mining_config_token_id).is_ok();
            ensure!(is_configuration_token, "configuration_token does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the claim to
            ensure!(
                <mining_config_token::Module<T>>::is_mining_config_token_owner(mining_config_token_id, sender.clone()).is_ok(),
                "Only the configuration_token owner can assign itself a claim"
            );

            Self::associate_token_claim_with_configuration(mining_claims_token_id, mining_config_token_id)
                .expect("Unable to associate claim with configuration");

            // Ensure that the given mining_claims_token_id already exists
            let token_claim = Self::mining_claims_token(mining_claims_token_id);
            ensure!(token_claim.is_some(), "Invalid mining_claims_token_id");

            // // Ensure that the claim is not already owned by a different configuration
            // // Unassign the claim from any existing configuration since it may only be owned by one configuration
            // <TokenClaimConfiguration<T>>::remove(mining_claims_token_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <TokenClaimConfiguration<T>>::insert(mining_claims_token_id, mining_config_token_id);

            Self::deposit_event(RawEvent::AssignedTokenClaimToConfiguration(sender, mining_claims_token_id, mining_config_token_id));
            }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_claims_token_owner(
        mining_claims_token_id: T::MiningClaimsTokenIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_claims_token_owner(&mining_claims_token_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of MiningClaimsToken"
        );
        Ok(())
    }

    pub fn exists_mining_claims_token(
        mining_claims_token_id: T::MiningClaimsTokenIndex,
    ) -> Result<MiningClaimsToken, DispatchError> {
        match Self::mining_claims_token(mining_claims_token_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningClaimsToken does not exist")),
        }
    }

    pub fn exists_mining_claims_token_claims_result(
        mining_config_token_id: T::MiningConfigTokenIndex,
        mining_claims_token_id: T::MiningClaimsTokenIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_claims_token_claims_results((mining_config_token_id, mining_claims_token_id)) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningClaimsTokenClaimResult does not exist")),
        }
    }

    pub fn has_value_for_mining_claims_token_claims_result_index(
        mining_config_token_id: T::MiningConfigTokenIndex,
        mining_claims_token_id: T::MiningClaimsTokenIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_claims_token_claims_result has a value that is defined");
        let fetched_mining_claims_token_claims_result =
            <MiningClaimsTokenClaimResults<T>>::get((mining_config_token_id, mining_claims_token_id));
        if let Some(_value) = fetched_mining_claims_token_claims_result {
            debug::info!("Found value for mining_claims_token_claims_result");
            return Ok(());
        }
        debug::info!("No value for mining_claims_token_claims_result");
        Err(DispatchError::Other("No value for mining_claims_token_claims_result"))
    }

    /// Only push the claim id onto the end of the vector if it does not already exist
    pub fn associate_token_claim_with_configuration(
        mining_claims_token_id: T::MiningClaimsTokenIndex,
        mining_config_token_id: T::MiningConfigTokenIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given claim id
        if let Some(configuration_claims) = Self::token_config_claims(mining_config_token_id) {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_config_token_id,
                configuration_claims
            );
            let not_configuration_contains_claim = !configuration_claims.contains(&mining_claims_token_id);
            ensure!(not_configuration_contains_claim, "Configuration already contains the given claim id");
            debug::info!("Configuration id key exists but its vector value does not contain the given claim id");
            <TokenConfigClaims<T>>::mutate(mining_config_token_id, |v| {
                if let Some(value) = v {
                    value.push(mining_claims_token_id);
                }
            });
            debug::info!(
                "Associated claim {:?} with configuration {:?}",
                mining_claims_token_id,
                mining_config_token_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the claim \
                 id {:?} to its vector value",
                mining_config_token_id,
                mining_claims_token_id
            );
            <TokenConfigClaims<T>>::insert(mining_config_token_id, &vec![mining_claims_token_id]);
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

    fn next_mining_claims_token_id() -> Result<T::MiningClaimsTokenIndex, DispatchError> {
        let mining_claims_token_id = Self::mining_claims_token_count();
        if mining_claims_token_id == <T::MiningClaimsTokenIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningClaimsToken count overflow"));
        }
        Ok(mining_claims_token_id)
    }

    fn insert_mining_claims_token(
        owner: &T::AccountId,
        mining_claims_token_id: T::MiningClaimsTokenIndex,
        mining_claims_token: MiningClaimsToken,
    ) {
        // Create and store mining mining_claims_token
        <MiningClaimsTokens<T>>::insert(mining_claims_token_id, mining_claims_token);
        <MiningClaimsTokenCount<T>>::put(mining_claims_token_id + One::one());
        <MiningClaimsTokenOwners<T>>::insert(mining_claims_token_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_claims_token_id: T::MiningClaimsTokenIndex) {
        <MiningClaimsTokenOwners<T>>::insert(mining_claims_token_id, to);
    }
}
