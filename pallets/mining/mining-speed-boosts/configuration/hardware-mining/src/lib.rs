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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait: system::Trait + roaming_operators::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MiningSpeedBoostConfigurationHardwareMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // Mining Speed Boost Hardware Mining Config
    type MiningSpeedBoostConfigurationHardwareMiningHardwareSecure: Parameter + Member + Default + Copy; // bool
    type MiningSpeedBoostConfigurationHardwareMiningHardwareType: Parameter + Member + Default;
    type MiningSpeedBoostConfigurationHardwareMiningHardwareID: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    // // Mining Speed Boost Reward
    // type MiningSpeedBoostClaimAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningSpeedBoostClaimDateRedeemed: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostConfigurationHardwareMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostConfigurationHardwareMiningHardwareConfig<U, V, W, X, Y, Z> {
    pub hardware_secure: U,
    pub hardware_type: V,
    pub hardware_id: W,
    pub hardware_dev_eui: X,
    pub hardware_lock_period_start_date: Y,
    pub hardware_lock_period_end_date: Z,
}

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostConfigurationHardwareMiningIndex,
        <T as Trait>::MiningSpeedBoostConfigurationHardwareMiningHardwareSecure,
        <T as Trait>::MiningSpeedBoostConfigurationHardwareMiningHardwareType,
        <T as Trait>::MiningSpeedBoostConfigurationHardwareMiningHardwareID,
        <T as Trait>::MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI,
        <T as Trait>::MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate,
        <T as Trait>::MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_configuration_hardware_mining is created. (owner, mining_speed_boosts_configuration_hardware_mining_id)
        Created(AccountId, MiningSpeedBoostConfigurationHardwareMiningIndex),
        /// A mining_speed_boosts_configuration_hardware_mining is transferred. (from, to, mining_speed_boosts_configuration_hardware_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostConfigurationHardwareMiningIndex),
        MiningSpeedBoostConfigurationHardwareMiningHardwareConfigSet(
          AccountId, MiningSpeedBoostConfigurationHardwareMiningIndex, MiningSpeedBoostConfigurationHardwareMiningHardwareSecure,
          MiningSpeedBoostConfigurationHardwareMiningHardwareType, MiningSpeedBoostConfigurationHardwareMiningHardwareID,
          MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI, MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate,
          MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostConfigurationHardwareMining {
        /// Stores all the mining_speed_boosts_configuration_hardware_minings, key is the mining_speed_boosts_configuration_hardware_mining id / index
        pub MiningSpeedBoostConfigurationHardwareMinings get(fn mining_speed_boosts_configuration_hardware_mining): map hasher(blake2_256) T::MiningSpeedBoostConfigurationHardwareMiningIndex => Option<MiningSpeedBoostConfigurationHardwareMining>;

        /// Stores the total number of mining_speed_boosts_configuration_hardware_minings. i.e. the next mining_speed_boosts_configuration_hardware_mining index
        pub MiningSpeedBoostConfigurationHardwareMiningCount get(fn mining_speed_boosts_configuration_hardware_mining_count): T::MiningSpeedBoostConfigurationHardwareMiningIndex;

        /// Stores mining_speed_boosts_configuration_hardware_mining owner
        pub MiningSpeedBoostConfigurationHardwareMiningOwners get(fn mining_speed_boosts_configuration_hardware_mining_owner): map hasher(blake2_256) T::MiningSpeedBoostConfigurationHardwareMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_configuration_hardware_mining_hardware_config
        pub MiningSpeedBoostConfigurationHardwareMiningHardwareConfigs get(fn mining_speed_boosts_configuration_hardware_mining_hardware_configs): map hasher(blake2_256) T::MiningSpeedBoostConfigurationHardwareMiningIndex =>
            Option<MiningSpeedBoostConfigurationHardwareMiningHardwareConfig<T::MiningSpeedBoostConfigurationHardwareMiningHardwareSecure, T::MiningSpeedBoostConfigurationHardwareMiningHardwareType,
                T::MiningSpeedBoostConfigurationHardwareMiningHardwareID, T::MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI, T::MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate,
                T::MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_configuration_hardware_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_configuration_hardware_mining_id = Self::next_mining_speed_boosts_configuration_hardware_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_configuration_hardware_mining
            let mining_speed_boosts_configuration_hardware_mining = MiningSpeedBoostConfigurationHardwareMining(unique_id);
            Self::insert_mining_speed_boosts_configuration_hardware_mining(&sender, mining_speed_boosts_configuration_hardware_mining_id, mining_speed_boosts_configuration_hardware_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_configuration_hardware_mining_id));
        }

        /// Transfer a mining_speed_boosts_configuration_hardware_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_configuration_hardware_mining_owner(mining_speed_boosts_configuration_hardware_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_configuration_hardware_mining");

            Self::update_owner(&to, mining_speed_boosts_configuration_hardware_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_configuration_hardware_mining_id));
        }

        /// Set mining_speed_boosts_configuration_hardware_mining_hardware_config
        pub fn set_mining_speed_boosts_configuration_hardware_mining_hardware_config(
            origin,
            mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
            _hardware_secure: Option<T::MiningSpeedBoostConfigurationHardwareMiningHardwareSecure>,
            _hardware_type: Option<T::MiningSpeedBoostConfigurationHardwareMiningHardwareType>,
            _hardware_id: Option<T::MiningSpeedBoostConfigurationHardwareMiningHardwareID>,
            _hardware_dev_eui: Option<T::MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI>,
            _hardware_lock_period_start_date: Option<T::MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate>,
            _hardware_lock_period_end_date: Option<T::MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_configuration_hardware_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_configuration_hardware_mining = Self::exists_mining_speed_boosts_configuration_hardware_mining(mining_speed_boosts_configuration_hardware_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_configuration_hardware_mining, "MiningSpeedBoostConfigurationHardwareMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_configuration_hardware_mining_hardware_config they are trying to change
            ensure!(Self::mining_speed_boosts_configuration_hardware_mining_owner(mining_speed_boosts_configuration_hardware_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_configuration_hardware_mining_hardware_config");

            let hardware_secure = match _hardware_secure.clone() {
                Some(value) => value,
                None => Default::default() // Default
            };
            let hardware_type = match _hardware_type {
                Some(value) => value,
                // FIXME - get this fallback to work!
                // None => b"gateway".to_vec() // Default
                None => Default::default() // Default
            };
            let hardware_id = match _hardware_id {
                Some(value) => value,
                None => 3.into() // Default
            };
            let hardware_dev_eui = match _hardware_dev_eui {
                Some(value) => value,
                None => Default::default() // Default
            };
            let hardware_lock_period_start_date = match _hardware_lock_period_start_date {
                Some(value) => value,
                None => Default::default() // Default
            };
            let hardware_lock_period_end_date = match _hardware_lock_period_end_date {
                Some(value) => value,
                None => Default::default() // Default
            };

            // Check if a mining_speed_boosts_configuration_hardware_mining_hardware_config already exists with the given mining_speed_boosts_configuration_hardware_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_configuration_hardware_mining_hardware_config_index(mining_speed_boosts_configuration_hardware_mining_id).is_ok() {
                debug::info!("Mutating values");
                // TODO
                <MiningSpeedBoostConfigurationHardwareMiningHardwareConfigs<T>>::mutate(mining_speed_boosts_configuration_hardware_mining_id, |mining_speed_boosts_configuration_hardware_mining_hardware_config| {
                    if let Some(_mining_speed_boosts_configuration_hardware_mining_hardware_config) = mining_speed_boosts_configuration_hardware_mining_hardware_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_secure = hardware_secure.clone();
                        _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_type = hardware_type.clone();
                        _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_id = hardware_id.clone();
                        _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_dev_eui = hardware_dev_eui.clone();
                        _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_lock_period_start_date = hardware_lock_period_start_date.clone();
                        _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_lock_period_end_date = hardware_lock_period_end_date.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_configuration_hardware_mining_hardware_config = <MiningSpeedBoostConfigurationHardwareMiningHardwareConfigs<T>>::get(mining_speed_boosts_configuration_hardware_mining_id);
                if let Some(_mining_speed_boosts_configuration_hardware_mining_hardware_config) = fetched_mining_speed_boosts_configuration_hardware_mining_hardware_config {
                    debug::info!("Latest field hardware_secure {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_secure);
                    debug::info!("Latest field hardware_type {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_type);
                    debug::info!("Latest field hardware_id {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_id);
                    debug::info!("Latest field hardware_dev_eui {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_dev_eui);
                    debug::info!("Latest field hardware_lock_period_start_date {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_lock_period_start_date);
                    debug::info!("Latest field hardware_lock_period_end_date {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_lock_period_end_date);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_configuration_hardware_mining_hardware_config instance with the input params
                let mining_speed_boosts_configuration_hardware_mining_hardware_config_instance = MiningSpeedBoostConfigurationHardwareMiningHardwareConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_secure: hardware_secure.clone(),
                    hardware_type: hardware_type.clone(),
                    hardware_id: hardware_id.clone(),
                    hardware_dev_eui: hardware_dev_eui.clone(),
                    hardware_lock_period_start_date: hardware_lock_period_start_date.clone(),
                    hardware_lock_period_end_date: hardware_lock_period_end_date.clone(),
                };

                <MiningSpeedBoostConfigurationHardwareMiningHardwareConfigs<T>>::insert(
                    mining_speed_boosts_configuration_hardware_mining_id,
                    &mining_speed_boosts_configuration_hardware_mining_hardware_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_configuration_hardware_mining_hardware_config = <MiningSpeedBoostConfigurationHardwareMiningHardwareConfigs<T>>::get(mining_speed_boosts_configuration_hardware_mining_id);
                if let Some(_mining_speed_boosts_configuration_hardware_mining_hardware_config) = fetched_mining_speed_boosts_configuration_hardware_mining_hardware_config {
                    debug::info!("Inserted field hardware_secure {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_secure);
                    debug::info!("Inserted field hardware_type {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_type);
                    debug::info!("Inserted field hardware_id {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_id);
                    debug::info!("Inserted field hardware_dev_eui {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_dev_eui);
                    debug::info!("Inserted field hardware_lock_period_start_date {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_lock_period_start_date);
                    debug::info!("Inserted field hardware_lock_period_end_date {:#?}", _mining_speed_boosts_configuration_hardware_mining_hardware_config.hardware_lock_period_end_date);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostConfigurationHardwareMiningHardwareConfigSet(
                sender,
                mining_speed_boosts_configuration_hardware_mining_id,
                hardware_secure,
                hardware_type,
                hardware_id,
                hardware_dev_eui,
                hardware_lock_period_start_date,
                hardware_lock_period_end_date,
            ));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_speed_boosts_configuration_hardware_mining_owner(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_configuration_hardware_mining_owner(
                &mining_speed_boosts_configuration_hardware_mining_id
            )
            .map(|owner| owner == sender)
            .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoost"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_configuration_hardware_mining(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
    ) -> Result<MiningSpeedBoostConfigurationHardwareMining, DispatchError> {
        match Self::mining_speed_boosts_configuration_hardware_mining(
            mining_speed_boosts_configuration_hardware_mining_id,
        ) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostConfigurationHardwareMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_configuration_hardware_mining_hardware_config(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_configuration_hardware_mining_hardware_configs(
            mining_speed_boosts_configuration_hardware_mining_id,
        ) {
            Some(value) => Ok(()),
            None => {
                Err(DispatchError::Other("MiningSpeedBoostConfigurationHardwareMiningHardwareConfig does not exist"))
            }
        }
    }

    pub fn has_value_for_mining_speed_boosts_configuration_hardware_mining_hardware_config_index(
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_speed_boosts_configuration_hardware_mining_hardware_config has a value that is defined"
        );
        let fetched_mining_speed_boosts_configuration_hardware_mining_hardware_config =
            <MiningSpeedBoostConfigurationHardwareMiningHardwareConfigs<T>>::get(
                mining_speed_boosts_configuration_hardware_mining_id,
            );
        if let Some(value) = fetched_mining_speed_boosts_configuration_hardware_mining_hardware_config {
            debug::info!("Found value for mining_speed_boosts_configuration_hardware_mining_hardware_config");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_configuration_hardware_mining_hardware_config");
        Err(DispatchError::Other("No value for mining_speed_boosts_configuration_hardware_mining_hardware_config"))
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

    fn next_mining_speed_boosts_configuration_hardware_mining_id()
    -> Result<T::MiningSpeedBoostConfigurationHardwareMiningIndex, DispatchError> {
        let mining_speed_boosts_configuration_hardware_mining_id =
            Self::mining_speed_boosts_configuration_hardware_mining_count();
        if mining_speed_boosts_configuration_hardware_mining_id ==
            <T::MiningSpeedBoostConfigurationHardwareMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostConfigurationHardwareMining count overflow"));
        }
        Ok(mining_speed_boosts_configuration_hardware_mining_id)
    }

    fn insert_mining_speed_boosts_configuration_hardware_mining(
        owner: &T::AccountId,
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
        mining_speed_boosts_configuration_hardware_mining: MiningSpeedBoostConfigurationHardwareMining,
    ) {
        // Create and store mining mining_speed_boosts_configuration_hardware_mining
        <MiningSpeedBoostConfigurationHardwareMinings<T>>::insert(
            mining_speed_boosts_configuration_hardware_mining_id,
            mining_speed_boosts_configuration_hardware_mining,
        );
        <MiningSpeedBoostConfigurationHardwareMiningCount<T>>::put(
            mining_speed_boosts_configuration_hardware_mining_id + One::one(),
        );
        <MiningSpeedBoostConfigurationHardwareMiningOwners<T>>::insert(
            mining_speed_boosts_configuration_hardware_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_configuration_hardware_mining_id: T::MiningSpeedBoostConfigurationHardwareMiningIndex,
    ) {
        <MiningSpeedBoostConfigurationHardwareMiningOwners<T>>::insert(
            mining_speed_boosts_configuration_hardware_mining_id,
            to,
        );
    }
}
