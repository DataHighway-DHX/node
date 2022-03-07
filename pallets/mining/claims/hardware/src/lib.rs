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
use scale_info::TypeInfo;
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
use mining_setting_hardware;
use mining_eligibility_hardware;
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
    + mining_setting_hardware::Config
    + mining_eligibility_hardware::Config
    + mining_rates_hardware::Config
    + mining_sampling_hardware::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningClaimsHardwareIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningClaimsHardwareClaimAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningClaimsHardware(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct MiningClaimsHardwareClaimResult<U, V> {
    pub hardware_claim_amount: U,
    pub hardware_claim_block_redeemed: V,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::MiningClaimsHardwareIndex,
        <T as Config>::MiningClaimsHardwareClaimAmount,
        <T as mining_setting_hardware::Config>::MiningSettingHardwareIndex,
        <T as frame_system::Config>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_claims_hardware is created. (owner, mining_claims_hardware_id)
        Created(AccountId, MiningClaimsHardwareIndex),
        /// A mining_claims_hardware is transferred. (from, to, mining_claims_hardware_id)
        Transferred(AccountId, AccountId, MiningClaimsHardwareIndex),
        MiningClaimsHardwareClaimResultSet(
            AccountId, MiningSettingHardwareIndex, MiningClaimsHardwareIndex,
            MiningClaimsHardwareClaimAmount, BlockNumber
        ),
        /// A mining_claims_hardware is assigned to an mining_hardware.
        /// (owner of mining_hardware, mining_claims_hardware_id, mining_setting_hardware_id)
        AssignedHardwareClaimToConfiguration(AccountId, MiningClaimsHardwareIndex, MiningSettingHardwareIndex),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningClaimsHardware {
        /// Stores all the mining_claims_hardwares, key is the mining_claims_hardware id / index
        pub MiningClaimsHardwares get(fn mining_claims_hardware): map hasher(opaque_blake2_256) T::MiningClaimsHardwareIndex => Option<MiningClaimsHardware>;

        /// Stores the total number of mining_claims_hardwares. i.e. the next mining_claims_hardware index
        pub MiningClaimsHardwareCount get(fn mining_claims_hardware_count): T::MiningClaimsHardwareIndex;

        /// Stores mining_claims_hardware owner
        pub MiningClaimsHardwareOwners get(fn mining_claims_hardware_owner): map hasher(opaque_blake2_256) T::MiningClaimsHardwareIndex => Option<T::AccountId>;

        /// Stores mining_claims_hardware_claims_result
        pub MiningClaimsHardwareClaimResults get(fn mining_claims_hardware_claims_results): map hasher(opaque_blake2_256) (T::MiningSettingHardwareIndex, T::MiningClaimsHardwareIndex) =>
            Option<MiningClaimsHardwareClaimResult<
                T::MiningClaimsHardwareClaimAmount,
                T::BlockNumber
            >>;

        /// Get mining_setting_hardware_id belonging to a mining_claims_hardware_id
        pub HardwareClaimConfiguration get(fn hardware_claim_configuration): map hasher(opaque_blake2_256) T::MiningClaimsHardwareIndex => Option<T::MiningSettingHardwareIndex>;

        /// Get mining_claims_hardware_id's belonging to a mining_setting_hardware_id
        pub HardwareSettingClaims get(fn hardware_config_claims): map hasher(opaque_blake2_256) T::MiningSettingHardwareIndex => Option<Vec<T::MiningClaimsHardwareIndex>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_claims_hardware
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_claims_hardware_id = Self::next_mining_claims_hardware_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_claims_hardware
            let mining_claims_hardware = MiningClaimsHardware(unique_id);
            Self::insert_mining_claims_hardware(&sender, mining_claims_hardware_id, mining_claims_hardware);

            Self::deposit_event(RawEvent::Created(sender, mining_claims_hardware_id));
        }

        /// Transfer a mining_claims_hardware to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_claims_hardware_id: T::MiningClaimsHardwareIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_claims_hardware_owner(mining_claims_hardware_id) == Some(sender.clone()), "Only owner can transfer mining mining_claims_hardware");

            Self::update_owner(&to, mining_claims_hardware_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_claims_hardware_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn claim(
            origin,
            mining_setting_hardware_id: T::MiningSettingHardwareIndex,
            mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
            mining_claims_hardware_id: T::MiningClaimsHardwareIndex,
        ) {
            let _sender = ensure_signed(origin)?;

            // TODO - implement similar to claims/token when it is working and uncomment the integration tests
            return Err(DispatchError::Other("Not implemented"));
        }

        /// Set mining_claims_hardware_claims_result
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_claims_hardware_claims_result(
            origin,
            mining_setting_hardware_id: T::MiningSettingHardwareIndex,
            mining_eligibility_hardware_id: T::MiningEligibilityHardwareIndex,
            mining_claims_hardware_id: T::MiningClaimsHardwareIndex,
            _hardware_claim_amount: Option<T::MiningClaimsHardwareClaimAmount>,
            // FIXME - the date should be generated without the extrinsic itself, not passed as an argument like this
            _hardware_claim_block_redeemed: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_claims_hardware_id whose config we want to change actually exists
            let is_mining_claims_hardware = Self::exists_mining_claims_hardware(mining_claims_hardware_id).is_ok();
            ensure!(is_mining_claims_hardware, "MiningClaimsHardware does not exist");

            // Ensure that the caller is owner of the mining_claims_hardware_claims_result they are trying to change
            ensure!(Self::mining_claims_hardware_owner(mining_claims_hardware_id) == Some(sender.clone()), "Only owner can set mining_claims_hardware_claims_result");

            // TODO - adjust defaults
            let hardware_claim_amount = match _hardware_claim_amount.clone() {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            let hardware_claim_block_redeemed = match _hardware_claim_block_redeemed {
                Some(value) => value,
                None => 1u32.into() // Default
            };

            // Check if a mining_claims_hardware_claims_result already exists with the given mining_claims_hardware_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_claims_hardware_claims_result_index(mining_setting_hardware_id, mining_claims_hardware_id).is_ok() {
                info!("Mutating values");
                <MiningClaimsHardwareClaimResults<T>>::mutate((mining_setting_hardware_id, mining_claims_hardware_id), |mining_claims_hardware_claims_result| {
                    if let Some(_mining_claims_hardware_claims_result) = mining_claims_hardware_claims_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_claims_hardware_claims_result.hardware_claim_amount = hardware_claim_amount.clone();
                        _mining_claims_hardware_claims_result.hardware_claim_block_redeemed = hardware_claim_block_redeemed.clone();
                    }
                });
                info!("Checking mutated values");
                let fetched_mining_claims_hardware_claims_result = <MiningClaimsHardwareClaimResults<T>>::get((mining_setting_hardware_id, mining_claims_hardware_id));
                if let Some(_mining_claims_hardware_claims_result) = fetched_mining_claims_hardware_claims_result {
                    info!("Latest field hardware_claim_amount {:#?}", _mining_claims_hardware_claims_result.hardware_claim_amount);
                    info!("Latest field hardware_claim_block_redeemed {:#?}", _mining_claims_hardware_claims_result.hardware_claim_block_redeemed);
                }
            } else {
                info!("Inserting values");

                // Create a new mining mining_claims_hardware_claims_result instance with the input params
                let mining_claims_hardware_claims_result_instance = MiningClaimsHardwareClaimResult {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_claim_amount: hardware_claim_amount.clone(),
                    hardware_claim_block_redeemed: hardware_claim_block_redeemed.clone(),
                };

                <MiningClaimsHardwareClaimResults<T>>::insert(
                    (mining_setting_hardware_id, mining_claims_hardware_id),
                    &mining_claims_hardware_claims_result_instance
                );

                info!("Checking inserted values");
                let fetched_mining_claims_hardware_claims_result = <MiningClaimsHardwareClaimResults<T>>::get((mining_setting_hardware_id, mining_claims_hardware_id));
                if let Some(_mining_claims_hardware_claims_result) = fetched_mining_claims_hardware_claims_result {
                    info!("Inserted field hardware_claim_amount {:#?}", _mining_claims_hardware_claims_result.hardware_claim_amount);
                    info!("Inserted field hardware_claim_block_redeemed {:#?}", _mining_claims_hardware_claims_result.hardware_claim_block_redeemed);
                }
            }

            Self::deposit_event(RawEvent::MiningClaimsHardwareClaimResultSet(
                sender,
                mining_setting_hardware_id,
                mining_claims_hardware_id,
                hardware_claim_amount,
                hardware_claim_block_redeemed,
            ));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn assign_claim_to_configuration(
          origin,
          mining_claims_hardware_id: T::MiningClaimsHardwareIndex,
          mining_setting_hardware_id: T::MiningSettingHardwareIndex
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the given configuration id already exists
            let is_configuration_hardware = <mining_setting_hardware::Module<T>>
                ::exists_mining_setting_hardware(mining_setting_hardware_id).is_ok();
            ensure!(is_configuration_hardware, "configuration_hardware does not exist");

            // Ensure that caller of the function is the owner of the configuration id to assign the claim to
            ensure!(
                <mining_setting_hardware::Module<T>>::is_mining_setting_hardware_owner(mining_setting_hardware_id, sender.clone()).is_ok(),
                "Only the configuration_hardware owner can assign itself a claim"
            );

            Self::associate_hardware_claim_with_configuration(mining_claims_hardware_id, mining_setting_hardware_id)
                .expect("Unable to associate claim with configuration");

            // Ensure that the given mining_claims_hardware_id already exists
            let hardware_claim = Self::mining_claims_hardware(mining_claims_hardware_id);
            ensure!(hardware_claim.is_some(), "Invalid mining_claims_hardware_id");

            // // Ensure that the claim is not already owned by a different configuration
            // // Unassign the claim from any existing configuration since it may only be owned by one configuration
            // <HardwareClaimConfiguration<T>>::remove(mining_claims_hardware_id);

            // Assign the network owner to the given operator (even if already belongs to them)
            <HardwareClaimConfiguration<T>>::insert(mining_claims_hardware_id, mining_setting_hardware_id);

            Self::deposit_event(RawEvent::AssignedHardwareClaimToConfiguration(sender, mining_claims_hardware_id, mining_setting_hardware_id));
            }
    }
}

impl<T: Config> Module<T> {
    pub fn is_mining_claims_hardware_owner(
        mining_claims_hardware_id: T::MiningClaimsHardwareIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_claims_hardware_owner(&mining_claims_hardware_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningClaimsHardware"
        );
        Ok(())
    }

    pub fn exists_mining_claims_hardware(
        mining_claims_hardware_id: T::MiningClaimsHardwareIndex,
    ) -> Result<MiningClaimsHardware, DispatchError> {
        match Self::mining_claims_hardware(mining_claims_hardware_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningClaimsHardware does not exist")),
        }
    }

    pub fn exists_mining_claims_hardware_claims_result(
        mining_setting_hardware_id: T::MiningSettingHardwareIndex,
        mining_claims_hardware_id: T::MiningClaimsHardwareIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_claims_hardware_claims_results((mining_setting_hardware_id, mining_claims_hardware_id)) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningClaimsHardwareClaimResult does not exist")),
        }
    }

    pub fn has_value_for_mining_claims_hardware_claims_result_index(
        mining_setting_hardware_id: T::MiningSettingHardwareIndex,
        mining_claims_hardware_id: T::MiningClaimsHardwareIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if mining_claims_hardware_claims_result has a value that is defined");
        let fetched_mining_claims_hardware_claims_result =
            <MiningClaimsHardwareClaimResults<T>>::get((mining_setting_hardware_id, mining_claims_hardware_id));
        if let Some(_value) = fetched_mining_claims_hardware_claims_result {
            info!("Found value for mining_claims_hardware_claims_result");
            return Ok(());
        }
        warn!("No value for mining_claims_hardware_claims_result");
        Err(DispatchError::Other("No value for mining_claims_hardware_claims_result"))
    }

    /// Only push the claim id onto the end of the vector if it does not already exist
    pub fn associate_hardware_claim_with_configuration(
        mining_claims_hardware_id: T::MiningClaimsHardwareIndex,
        mining_setting_hardware_id: T::MiningSettingHardwareIndex,
    ) -> Result<(), DispatchError> {
        // Early exit with error since do not want to append if the given configuration id already exists as a key,
        // and where its corresponding value is a vector that already contains the given claim id
        if let Some(configuration_claims) = Self::hardware_config_claims(mining_setting_hardware_id) {
            info!(
                "Configuration id key {:?} exists with value {:?}",
                mining_setting_hardware_id,
                configuration_claims
            );
            let not_configuration_contains_claim = !configuration_claims.contains(&mining_claims_hardware_id);
            ensure!(not_configuration_contains_claim, "Configuration already contains the given claim id");
            info!("Configuration id key exists but its vector value does not contain the given claim id");
            <HardwareSettingClaims<T>>::mutate(mining_setting_hardware_id, |v| {
                if let Some(value) = v {
                    value.push(mining_claims_hardware_id);
                }
            });
            info!(
                "Associated claim {:?} with configuration {:?}",
                mining_claims_hardware_id,
                mining_setting_hardware_id
            );
            Ok(())
        } else {
            info!(
                "Configuration id key does not yet exist. Creating the configuration key {:?} and appending the claim \
                 id {:?} to its vector value",
                mining_setting_hardware_id,
                mining_claims_hardware_id
            );
            <HardwareSettingClaims<T>>::insert(mining_setting_hardware_id, &vec![mining_claims_hardware_id]);
            Ok(())
        }
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <frame_system::Pallet<T>>::extrinsic_index(),
            <frame_system::Pallet<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_mining_claims_hardware_id() -> Result<T::MiningClaimsHardwareIndex, DispatchError> {
        let mining_claims_hardware_id = Self::mining_claims_hardware_count();
        if mining_claims_hardware_id == <T::MiningClaimsHardwareIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningClaimsHardware count overflow"));
        }
        Ok(mining_claims_hardware_id)
    }

    fn insert_mining_claims_hardware(
        owner: &T::AccountId,
        mining_claims_hardware_id: T::MiningClaimsHardwareIndex,
        mining_claims_hardware: MiningClaimsHardware,
    ) {
        // Create and store mining mining_claims_hardware
        <MiningClaimsHardwares<T>>::insert(mining_claims_hardware_id, mining_claims_hardware);
        <MiningClaimsHardwareCount<T>>::put(mining_claims_hardware_id + One::one());
        <MiningClaimsHardwareOwners<T>>::insert(mining_claims_hardware_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_claims_hardware_id: T::MiningClaimsHardwareIndex) {
        <MiningClaimsHardwareOwners<T>>::insert(mining_claims_hardware_id, to);
    }
}
