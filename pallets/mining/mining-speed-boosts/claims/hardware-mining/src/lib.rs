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
use mining_speed_boosts_configuration_hardware_mining;
use mining_speed_boosts_eligibility_hardware_mining;
use mining_speed_boosts_rates_hardware_mining;
use mining_speed_boosts_sampling_hardware_mining;
use roaming_operators;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait:
    system::Trait
    + roaming_operators::Trait
    + mining_speed_boosts_configuration_hardware_mining::Trait
    + mining_speed_boosts_eligibility_hardware_mining::Trait
    + mining_speed_boosts_rates_hardware_mining::Trait
    + mining_speed_boosts_sampling_hardware_mining::Trait
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MiningSpeedBoostClaimsHardwareMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostClaimsHardwareMiningClaimAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostClaimsHardwareMiningClaimDateRedeemed: Parameter
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
pub struct MiningSpeedBoostClaimsHardwareMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostClaimsHardwareMiningClaimResult<U, V> {
    pub hardware_claim_amount: U,
    pub hardware_claim_date_redeemed: V,
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostClaimsHardwareMiningIndex,
        <T as Trait>::MiningSpeedBoostClaimsHardwareMiningClaimAmount,
        <T as Trait>::MiningSpeedBoostClaimsHardwareMiningClaimDateRedeemed,
        <T as mining_speed_boosts_configuration_hardware_mining::Trait>::MiningSpeedBoostConfigurationHardwareMiningIndex,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_claims_hardware_mining is created. (owner, mining_speed_boosts_claims_hardware_mining_id)
        Created(AccountId, MiningSpeedBoostClaimsHardwareMiningIndex),
        /// A mining_speed_boosts_claims_hardware_mining is transferred. (from, to, mining_speed_boosts_claims_hardware_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostClaimsHardwareMiningIndex),
        MiningSpeedBoostClaimsHardwareMiningClaimResultSet(
            AccountId, MiningSpeedBoostConfigurationHardwareMiningIndex, MiningSpeedBoostClaimsHardwareMiningIndex,
            MiningSpeedBoostClaimsHardwareMiningClaimAmount, MiningSpeedBoostClaimsHardwareMiningClaimDateRedeemed
        ),
        /// A mining_speed_boosts_claims_hardware_mining is assigned to an mining_speed_boosts_hardware_mining.
        /// (owner of mining_speed_boosts_hardware_mining, mining_speed_boosts_claims_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id)
            AssignedHardwareMiningClaimToConfiguration(AccountId, MiningSpeedBoostClaimsHardwareMiningIndex, MiningSpeedBoostConfigurationHardwareMiningIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostClaimsHardwareMining {
        /// Stores all the mining_speed_boosts_claims_hardware_minings, key is the mining_speed_boosts_claims_hardware_mining id / index
        pub MiningSpeedBoostClaimsHardwareMinings get(fn mining_speed_boosts_claims_hardware_mining): map hasher(blake2_256) T::MiningSpeedBoostClaimsHardwareMiningIndex => Option<MiningSpeedBoostClaimsHardwareMining>;

        /// Stores the total number of mining_speed_boosts_claims_hardware_minings. i.e. the next mining_speed_boosts_claims_hardware_mining index
        pub MiningSpeedBoostClaimsHardwareMiningCount get(fn mining_speed_boosts_claims_hardware_mining_count): T::MiningSpeedBoostClaimsHardwareMiningIndex;

        /// Stores mining_speed_boosts_claims_hardware_mining owner
        pub MiningSpeedBoostClaimsHardwareMiningOwners get(fn mining_speed_boosts_claims_hardware_mining_owner): map hasher(blake2_256) T::MiningSpeedBoostClaimsHardwareMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_claims_hardware_mining_claims_result
        pub MiningSpeedBoostClaimsHardwareMiningClaimResults get(fn mining_speed_boosts_claims_hardware_mining_claims_results): map hasher(blake2_256) (T::MiningSpeedBoostConfigurationHardwareMiningIndex, T::MiningSpeedBoostClaimsHardwareMiningIndex) =>
            Option<MiningSpeedBoostClaimsHardwareMiningClaimResult<
                T::MiningSpeedBoostClaimsHardwareMiningClaimAmount,
                T::MiningSpeedBoostClaimsHardwareMiningClaimDateRedeemed
            >>;

        /// Get mining_speed_boosts_configuration_hardware_mining_id belonging to a mining_speed_boosts_claims_hardware_mining_id
        pub HardwareMiningClaimConfiguration get(fn hardware_mining_claim_configuration): map hasher(blake2_256) T::MiningSpeedBoostClaimsHardwareMiningIndex => Option<T::MiningSpeedBoostConfigurationHardwareMiningIndex>;

        /// Get mining_speed_boosts_claims_hardware_mining_id's belonging to a mining_speed_boosts_configuration_hardware_mining_id
        pub HardwareMiningConfigurationClaims get(fn hardware_mining_configuration_claims): map hasher(blake2_256) T::MiningSpeedBoostConfigurationHardwareMiningIndex => Option<Vec<T::MiningSpeedBoostClaimsHardwareMiningIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_claims_hardware_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_claims_hardware_mining_id = Self::next_mining_speed_boosts_claims_hardware_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_claims_hardware_mining
            let mining_speed_boosts_claims_hardware_mining = MiningSpeedBoostClaimsHardwareMining(unique_id);
            Self::insert_mining_speed_boosts_claims_hardware_mining(&sender, mining_speed_boosts_claims_hardware_mining_id, mining_speed_boosts_claims_hardware_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_claims_hardware_mining_id));
        }

        /// Transfer a mining_speed_boosts_claims_hardware_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_claims_hardware_mining_owner(mining_speed_boosts_claims_hardware_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_claims_hardware_mining");

            Self::update_owner(&to, mining_speed_boosts_claims_hardware_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_claims_hardware_mining_id));
        }

        pub fn claim(
            origin,
            mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
            mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex,
            mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
        ) {
            let sender = ensure_signed(origin)?;

            // TODO - implement similar to claims/token-mining when it is working and uncomment the integration tests
            return Err(DispatchError::Other("Not implemented"));
        }

        /// Set mining_speed_boosts_claims_hardware_mining_claims_result
        pub fn set_mining_speed_boosts_claims_hardware_mining_claims_result(
            origin,
            mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
            mining_speed_boosts_eligibility_hardware_mining_id: T::MiningSpeedBoostEligibilityHardwareMiningIndex,
            mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
            _hardware_claim_amount: Option<T::MiningSpeedBoostClaimsHardwareMiningClaimAmount>,
            // FIXME - the date should be generated without the extrinsic itself, not passed as an argument like this
            _hardware_claim_date_redeemed: Option<T::MiningSpeedBoostClaimsHardwareMiningClaimDateRedeemed>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_claims_hardware_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_claims_hardware_mining = Self::exists_mining_speed_boosts_claims_hardware_mining(mining_speed_boosts_claims_hardware_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_claims_hardware_mining, "MiningSpeedBoostClaimsHardwareMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_claims_hardware_mining_claims_result they are trying to change
            ensure!(Self::mining_speed_boosts_claims_hardware_mining_owner(mining_speed_boosts_claims_hardware_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_claims_hardware_mining_claims_result");

            // TODO - adjust defaults
            let hardware_claim_amount = match _hardware_claim_amount.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let hardware_claim_date_redeemed = match _hardware_claim_date_redeemed {
                Some(value) => value,
                None => 1.into() // Default
            };

            // Check if a mining_speed_boosts_claims_hardware_mining_claims_result already exists with the given mining_speed_boosts_claims_hardware_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_claims_hardware_mining_claims_result_index(mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_claims_hardware_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostClaimsHardwareMiningClaimResults<T>>::mutate((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_claims_hardware_mining_id), |mining_speed_boosts_claims_hardware_mining_claims_result| {
                    if let Some(_mining_speed_boosts_claims_hardware_mining_claims_result) = mining_speed_boosts_claims_hardware_mining_claims_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_claims_hardware_mining_claims_result.hardware_claim_amount = hardware_claim_amount.clone();
                        _mining_speed_boosts_claims_hardware_mining_claims_result.hardware_claim_date_redeemed = hardware_claim_date_redeemed.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_claims_hardware_mining_claims_result = <MiningSpeedBoostClaimsHardwareMiningClaimResults<T>>::get((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_claims_hardware_mining_id));
                if let Some(_mining_speed_boosts_claims_hardware_mining_claims_result) = fetched_mining_speed_boosts_claims_hardware_mining_claims_result {
                    debug::info!("Latest field hardware_claim_amount {:#?}", _mining_speed_boosts_claims_hardware_mining_claims_result.hardware_claim_amount);
                    debug::info!("Latest field hardware_claim_date_redeemed {:#?}", _mining_speed_boosts_claims_hardware_mining_claims_result.hardware_claim_date_redeemed);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_claims_hardware_mining_claims_result instance with the input params
                let mining_speed_boosts_claims_hardware_mining_claims_result_instance = MiningSpeedBoostClaimsHardwareMiningClaimResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_claim_amount: hardware_claim_amount.clone(),
                    hardware_claim_date_redeemed: hardware_claim_date_redeemed.clone(),
                };

                <MiningSpeedBoostClaimsHardwareMiningClaimResults<T>>::insert(
                    (mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_claims_hardware_mining_id),
                    &mining_speed_boosts_claims_hardware_mining_claims_result_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_claims_hardware_mining_claims_result = <MiningSpeedBoostClaimsHardwareMiningClaimResults<T>>::get((mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_claims_hardware_mining_id));
                if let Some(_mining_speed_boosts_claims_hardware_mining_claims_result) = fetched_mining_speed_boosts_claims_hardware_mining_claims_result {
                    debug::info!("Inserted field hardware_claim_amount {:#?}", _mining_speed_boosts_claims_hardware_mining_claims_result.hardware_claim_amount);
                    debug::info!("Inserted field hardware_claim_date_redeemed {:#?}", _mining_speed_boosts_claims_hardware_mining_claims_result.hardware_claim_date_redeemed);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostClaimsHardwareMiningClaimResultSet(
                sender,
                mining_speed_boosts_configuration_hardware_mining_id,
                mining_speed_boosts_claims_hardware_mining_id,
                hardware_claim_amount,
                hardware_claim_date_redeemed,
            ));
        }

        pub fn assign_claim_to_configuration(
          origin,
          mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
          mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_hardware_mining = <mining_speed_boosts_configuration_hardware_mining::Module<T>>
                ::exists_mining_speed_boosts_configuration_hardware_mining(mining_speed_boosts_configuration_hardware_mining_id).is_ok();
            ensure!(is_configuration_hardware_mining, "configuration_hardware_mining does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the claim to
            ensure!(
                <mining_speed_boosts_configuration_hardware_mining::Module<T>>::is_mining_speed_boosts_configuration_hardware_mining_owner(mining_speed_boosts_configuration_hardware_mining_id, sender.clone()).is_ok(),
                "Only the configuration_hardware_mining owner can assign itself a claim"
            );

            Self::associate_hardware_claim_with_configuration(mining_speed_boosts_claims_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id)
                .expect("Unable to associate claim with configuration");

            // Ensure that the given mining_speed_boosts_claims_hardware_mining_id already exists
            let hardware_claim = Self::mining_speed_boosts_claims_hardware_mining(mining_speed_boosts_claims_hardware_mining_id);
            ensure!(hardware_claim.is_some(), "Invalid mining_speed_boosts_claims_hardware_mining_id");

            // // Ensure that the claim is not already owned by a different configuration
            // // Unassign the claim from any existing configuration since it may only be owned by one configuration
            // <HardwareMiningClaimConfiguration<T>>::remove(mining_speed_boosts_claims_hardware_mining_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <HardwareMiningClaimConfiguration<T>>::insert(mining_speed_boosts_claims_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id);

            Self::deposit_event(RawEvent::AssignedHardwareMiningClaimToConfiguration(sender, mining_speed_boosts_claims_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining_id));
            }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_speed_boosts_claims_hardware_mining_owner(
        mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_claims_hardware_mining_owner(&mining_speed_boosts_claims_hardware_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostClaimsHardwareMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_claims_hardware_mining(
        mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
    ) -> Result<MiningSpeedBoostClaimsHardwareMining, DispatchError> {
        match Self::mining_speed_boosts_claims_hardware_mining(mining_speed_boosts_claims_hardware_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostClaimsHardwareMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_claims_hardware_mining_claims_result(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
        mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_claims_hardware_mining_claims_results((
            mining_speed_boosts_configuration_hardware_mining_id,
            mining_speed_boosts_claims_hardware_mining_id,
        )) {
            Some(value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostClaimsHardwareMiningClaimResult does not exist")),
        }
    }

    pub fn has_value_for_mining_speed_boosts_claims_hardware_mining_claims_result_index(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
        mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_speed_boosts_claims_hardware_mining_claims_result has a value that is defined"
        );
        let fetched_mining_speed_boosts_claims_hardware_mining_claims_result =
            <MiningSpeedBoostClaimsHardwareMiningClaimResults<T>>::get((
                mining_speed_boosts_configuration_hardware_mining_id,
                mining_speed_boosts_claims_hardware_mining_id,
            ));
        if let Some(value) = fetched_mining_speed_boosts_claims_hardware_mining_claims_result {
            debug::info!("Found value for mining_speed_boosts_claims_hardware_mining_claims_result");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_claims_hardware_mining_claims_result");
        Err(DispatchError::Other("No value for mining_speed_boosts_claims_hardware_mining_claims_result"))
    }

    /// Only push the claim id onto the end of the vector if it does not already exist
    pub fn associate_hardware_claim_with_configuration(
        mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given claim id
        if let Some(configuration_claims) =
            Self::hardware_mining_configuration_claims(mining_speed_boosts_configuration_hardware_mining_id)
        {
            debug::info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_speed_boosts_configuration_hardware_mining_id,
                configuration_claims
            );
            let not_configuration_contains_claim =
                !configuration_claims.contains(&mining_speed_boosts_claims_hardware_mining_id);
            ensure!(not_configuration_contains_claim, "Configuration already contains the given claim id");
            debug::info!("Configuration id key exists but its vector value does not contain the given claim id");
            <HardwareMiningConfigurationClaims<T>>::mutate(mining_speed_boosts_configuration_hardware_mining_id, |v| {
                if let Some(value) = v {
                    value.push(mining_speed_boosts_claims_hardware_mining_id);
                }
            });
            debug::info!(
                "Associated claim {:?} with configuration {:?}",
                mining_speed_boosts_claims_hardware_mining_id,
                mining_speed_boosts_configuration_hardware_mining_id
            );
            Ok(())
        } else {
            debug::info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the claim \
                 id {:?} to its vector value",
                mining_speed_boosts_configuration_hardware_mining_id,
                mining_speed_boosts_claims_hardware_mining_id
            );
            <HardwareMiningConfigurationClaims<T>>::insert(
                mining_speed_boosts_configuration_hardware_mining_id,
                &vec![mining_speed_boosts_claims_hardware_mining_id],
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

    fn next_mining_speed_boosts_claims_hardware_mining_id()
    -> Result<T::MiningSpeedBoostClaimsHardwareMiningIndex, DispatchError> {
        let mining_speed_boosts_claims_hardware_mining_id = Self::mining_speed_boosts_claims_hardware_mining_count();
        if mining_speed_boosts_claims_hardware_mining_id ==
            <T::MiningSpeedBoostClaimsHardwareMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostClaimsHardwareMining count overflow"));
        }
        Ok(mining_speed_boosts_claims_hardware_mining_id)
    }

    fn insert_mining_speed_boosts_claims_hardware_mining(
        owner: &T::AccountId,
        mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
        mining_speed_boosts_claims_hardware_mining: MiningSpeedBoostClaimsHardwareMining,
    ) {
        // Create and store mining mining_speed_boosts_claims_hardware_mining
        <MiningSpeedBoostClaimsHardwareMinings<T>>::insert(
            mining_speed_boosts_claims_hardware_mining_id,
            mining_speed_boosts_claims_hardware_mining,
        );
        <MiningSpeedBoostClaimsHardwareMiningCount<T>>::put(mining_speed_boosts_claims_hardware_mining_id + One::one());
        <MiningSpeedBoostClaimsHardwareMiningOwners<T>>::insert(
            mining_speed_boosts_claims_hardware_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_claims_hardware_mining_id: T::MiningSpeedBoostClaimsHardwareMiningIndex,
    ) {
        <MiningSpeedBoostClaimsHardwareMiningOwners<T>>::insert(mining_speed_boosts_claims_hardware_mining_id, to);
    }
}
