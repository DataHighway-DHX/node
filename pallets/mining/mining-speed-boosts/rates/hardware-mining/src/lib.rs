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
    traits::Get,
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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + roaming_operators::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningSpeedBoostRatesHardwareMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesHardwareMiningHardwareSecure: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesHardwareMiningHardwareInsecure: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningSpeedBoostRatesHardwareMiningMaxHardware: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesHardwareMiningCategory1MaxTokenBonusPerGateway: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningSpeedBoostRatesHardwareMiningCategory2MaxTokenBonusPerGateway: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningSpeedBoostRatesHardwareMiningCategory3MaxTokenBonusPerGateway: Parameter
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
pub struct MiningSpeedBoostRatesHardwareMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostRatesHardwareMiningRatesConfig<U, V, W, X, Y, Z> {
    pub hardware_hardware_secure: U,
    pub hardware_hardware_insecure: V,
    pub hardware_max_hardware: W,
    pub hardware_category_1_max_token_bonus_per_gateway: X,
    pub hardware_category_2_max_token_bonus_per_gateway: Y,
    pub hardware_category_3_max_token_bonus_per_gateway: Z,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningIndex,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningHardwareSecure,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningHardwareInsecure,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningMaxHardware,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningCategory1MaxTokenBonusPerGateway,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningCategory2MaxTokenBonusPerGateway,
        <T as Trait>::MiningSpeedBoostRatesHardwareMiningCategory3MaxTokenBonusPerGateway,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_rates_hardware_mining is created. (owner, mining_speed_boosts_rates_hardware_mining_id)
        Created(AccountId, MiningSpeedBoostRatesHardwareMiningIndex),
        /// A mining_speed_boosts_rates_hardware_mining is transferred. (from, to, mining_speed_boosts_rates_hardware_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostRatesHardwareMiningIndex),
        MiningSpeedBoostRatesHardwareMiningRatesConfigSet(
            AccountId, MiningSpeedBoostRatesHardwareMiningIndex, MiningSpeedBoostRatesHardwareMiningHardwareSecure,
            MiningSpeedBoostRatesHardwareMiningHardwareInsecure, MiningSpeedBoostRatesHardwareMiningMaxHardware,
            MiningSpeedBoostRatesHardwareMiningCategory1MaxTokenBonusPerGateway,
            MiningSpeedBoostRatesHardwareMiningCategory2MaxTokenBonusPerGateway,
            MiningSpeedBoostRatesHardwareMiningCategory3MaxTokenBonusPerGateway
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostRatesHardwareMining {
        /// Stores all the mining_speed_boosts_rates_hardware_minings, key is the mining_speed_boosts_rates_hardware_mining id / index
        pub MiningSpeedBoostRatesHardwareMinings get(fn mining_speed_boosts_rates_hardware_mining): map hasher(opaque_blake2_256) T::MiningSpeedBoostRatesHardwareMiningIndex => Option<MiningSpeedBoostRatesHardwareMining>;

        /// Stores the total number of mining_speed_boosts_rates_hardware_minings. i.e. the next mining_speed_boosts_rates_hardware_mining index
        pub MiningSpeedBoostRatesHardwareMiningCount get(fn mining_speed_boosts_rates_hardware_mining_count): T::MiningSpeedBoostRatesHardwareMiningIndex;

        /// Stores mining_speed_boosts_rates_hardware_mining owner
        pub MiningSpeedBoostRatesHardwareMiningOwners get(fn mining_speed_boosts_rates_hardware_mining_owner): map hasher(opaque_blake2_256) T::MiningSpeedBoostRatesHardwareMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_rates_hardware_mining_rates_config
        pub MiningSpeedBoostRatesHardwareMiningRatesConfigs get(fn mining_speed_boosts_rates_hardware_mining_rates_configs): map hasher(opaque_blake2_256) T::MiningSpeedBoostRatesHardwareMiningIndex =>
            Option<MiningSpeedBoostRatesHardwareMiningRatesConfig<T::MiningSpeedBoostRatesHardwareMiningHardwareSecure,
            T::MiningSpeedBoostRatesHardwareMiningHardwareInsecure, T::MiningSpeedBoostRatesHardwareMiningMaxHardware,
            T::MiningSpeedBoostRatesHardwareMiningCategory1MaxTokenBonusPerGateway,
            T::MiningSpeedBoostRatesHardwareMiningCategory2MaxTokenBonusPerGateway,
            T::MiningSpeedBoostRatesHardwareMiningCategory3MaxTokenBonusPerGateway>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_rates_hardware_mining
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_rates_hardware_mining_id = Self::next_mining_speed_boosts_rates_hardware_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_rates_hardware_mining
            let mining_speed_boosts_rates_hardware_mining = MiningSpeedBoostRatesHardwareMining(unique_id);
            Self::insert_mining_speed_boosts_rates_hardware_mining(&sender, mining_speed_boosts_rates_hardware_mining_id, mining_speed_boosts_rates_hardware_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_rates_hardware_mining_id));
        }

        /// Transfer a mining_speed_boosts_rates_hardware_mining to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_rates_hardware_mining_owner(mining_speed_boosts_rates_hardware_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_rates_hardware_mining");

            Self::update_owner(&to, mining_speed_boosts_rates_hardware_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_rates_hardware_mining_id));
        }

        /// Set mining_speed_boosts_rates_hardware_mining_rates_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_speed_boosts_rates_hardware_mining_rates_config(
            origin,
            mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
            _hardware_hardware_secure: Option<T::MiningSpeedBoostRatesHardwareMiningHardwareSecure>,
            _hardware_hardware_insecure: Option<T::MiningSpeedBoostRatesHardwareMiningHardwareInsecure>,
            _hardware_max_hardware: Option<T::MiningSpeedBoostRatesHardwareMiningMaxHardware>,
            _hardware_category_1_max_token_bonus_per_gateway: Option<T::MiningSpeedBoostRatesHardwareMiningCategory1MaxTokenBonusPerGateway>,
            _hardware_category_2_max_token_bonus_per_gateway: Option<T::MiningSpeedBoostRatesHardwareMiningCategory2MaxTokenBonusPerGateway>,
            _hardware_category_3_max_token_bonus_per_gateway: Option<T::MiningSpeedBoostRatesHardwareMiningCategory3MaxTokenBonusPerGateway>
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_rates_hardware_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_rates_hardware_mining = Self::exists_mining_speed_boosts_rates_hardware_mining(mining_speed_boosts_rates_hardware_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_rates_hardware_mining, "MiningSpeedBoostRatesHardwareMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_rates_hardware_mining_rates_config they are trying to change
            ensure!(Self::mining_speed_boosts_rates_hardware_mining_owner(mining_speed_boosts_rates_hardware_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_rates_hardware_mining_rates_config");

            // TODO - adjust default rates
            let hardware_hardware_secure = match _hardware_hardware_secure.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let hardware_hardware_insecure = match _hardware_hardware_insecure {
                Some(value) => value,
                None => 1.into() // Default
            };
            let hardware_max_hardware = match _hardware_max_hardware {
              Some(value) => value,
              None => 1.into() // Default
            };
            let hardware_category_1_max_token_bonus_per_gateway = match _hardware_category_1_max_token_bonus_per_gateway.clone() {
                Some(value) => value,
                None => 1000000.into() // Default
            };
            let hardware_category_2_max_token_bonus_per_gateway = match _hardware_category_2_max_token_bonus_per_gateway {
                Some(value) => value,
                None => 500000.into() // Default
            };
            let hardware_category_3_max_token_bonus_per_gateway = match _hardware_category_3_max_token_bonus_per_gateway {
                Some(value) => value,
                None => 250000.into() // Default
            };

            // Check if a mining_speed_boosts_rates_hardware_mining_rates_config already exists with the given mining_speed_boosts_rates_hardware_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_rates_hardware_mining_rates_config_index(mining_speed_boosts_rates_hardware_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::mutate(mining_speed_boosts_rates_hardware_mining_id, |mining_speed_boosts_rates_hardware_mining_rates_config| {
                    if let Some(_mining_speed_boosts_rates_hardware_mining_rates_config) = mining_speed_boosts_rates_hardware_mining_rates_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_secure = hardware_hardware_secure.clone();
                        _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_insecure = hardware_hardware_insecure.clone();
                        _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_max_hardware = hardware_max_hardware.clone();
                        _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_category_1_max_token_bonus_per_gateway = hardware_category_1_max_token_bonus_per_gateway.clone();
                        _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_category_2_max_token_bonus_per_gateway = hardware_category_2_max_token_bonus_per_gateway.clone();
                        _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_category_3_max_token_bonus_per_gateway = hardware_category_3_max_token_bonus_per_gateway.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_rates_hardware_mining_rates_config = <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::get(mining_speed_boosts_rates_hardware_mining_id);
                if let Some(_mining_speed_boosts_rates_hardware_mining_rates_config) = fetched_mining_speed_boosts_rates_hardware_mining_rates_config {
                    debug::info!("Latest field hardware_hardware_secure {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_secure);
                    debug::info!("Latest field hardware_hardware_insecure {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_insecure);
                    debug::info!("Latest field hardware_max_hardware {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_max_hardware);
                    debug::info!("Latest field hardware_category_1_max_token_bonus_per_gateway {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_category_1_max_token_bonus_per_gateway);
                    debug::info!("Latest field hardware_category_2_max_token_bonus_per_gateway {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_category_2_max_token_bonus_per_gateway);
                    debug::info!("Latest field hardware_category_3_max_token_bonus_per_gateway {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_category_3_max_token_bonus_per_gateway);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_rates_hardware_mining_rates_config instance with the input params
                let mining_speed_boosts_rates_hardware_mining_rates_config_instance = MiningSpeedBoostRatesHardwareMiningRatesConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_hardware_secure: hardware_hardware_secure.clone(),
                    hardware_hardware_insecure: hardware_hardware_insecure.clone(),
                    hardware_max_hardware: hardware_max_hardware.clone(),
                    hardware_category_1_max_token_bonus_per_gateway: hardware_category_1_max_token_bonus_per_gateway.clone(),
                    hardware_category_2_max_token_bonus_per_gateway: hardware_category_2_max_token_bonus_per_gateway.clone(),
                    hardware_category_3_max_token_bonus_per_gateway: hardware_category_3_max_token_bonus_per_gateway.clone(),
                };

                <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::insert(
                    mining_speed_boosts_rates_hardware_mining_id,
                    &mining_speed_boosts_rates_hardware_mining_rates_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_rates_hardware_mining_rates_config = <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::get(mining_speed_boosts_rates_hardware_mining_id);
                if let Some(_mining_speed_boosts_rates_hardware_mining_rates_config) = fetched_mining_speed_boosts_rates_hardware_mining_rates_config {
                    debug::info!("Inserted field hardware_hardware_secure {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_secure);
                    debug::info!("Inserted field hardware_hardware_insecure {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_hardware_insecure);
                    debug::info!("Inserted field hardware_max_hardware {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_max_hardware);
                    debug::info!("Inserted field hardware_category_1_max_token_bonus_per_gateway {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_category_1_max_token_bonus_per_gateway);
                    debug::info!("Inserted field hardware_category_2_max_token_bonus_per_gateway {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_category_2_max_token_bonus_per_gateway);
                    debug::info!("Inserted field hardware_category_3_max_token_bonus_per_gateway {:#?}", _mining_speed_boosts_rates_hardware_mining_rates_config.hardware_category_3_max_token_bonus_per_gateway);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostRatesHardwareMiningRatesConfigSet(
                sender,
                mining_speed_boosts_rates_hardware_mining_id,
                hardware_hardware_secure,
                hardware_hardware_insecure,
                hardware_max_hardware,
                hardware_category_1_max_token_bonus_per_gateway,
                hardware_category_2_max_token_bonus_per_gateway,
                hardware_category_3_max_token_bonus_per_gateway,
            ));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_speed_boosts_rates_hardware_mining_owner(
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_rates_hardware_mining_owner(&mining_speed_boosts_rates_hardware_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostRatesHardwareMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_rates_hardware_mining(
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
    ) -> Result<MiningSpeedBoostRatesHardwareMining, DispatchError> {
        match Self::mining_speed_boosts_rates_hardware_mining(mining_speed_boosts_rates_hardware_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostRatesHardwareMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_rates_hardware_mining_rates_config(
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_rates_hardware_mining_rates_configs(
            mining_speed_boosts_rates_hardware_mining_id,
        ) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostRatesHardwareMiningRatesConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_speed_boosts_rates_hardware_mining_rates_config_index(
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_speed_boosts_rates_hardware_mining_rates_config has a value that is defined");
        let fetched_mining_speed_boosts_rates_hardware_mining_rates_config =
            <MiningSpeedBoostRatesHardwareMiningRatesConfigs<T>>::get(mining_speed_boosts_rates_hardware_mining_id);
        if let Some(_value) = fetched_mining_speed_boosts_rates_hardware_mining_rates_config {
            debug::info!("Found value for mining_speed_boosts_rates_hardware_mining_rates_config");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_rates_hardware_mining_rates_config");
        Err(DispatchError::Other("No value for mining_speed_boosts_rates_hardware_mining_rates_config"))
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

    fn next_mining_speed_boosts_rates_hardware_mining_id()
    -> Result<T::MiningSpeedBoostRatesHardwareMiningIndex, DispatchError> {
        let mining_speed_boosts_rates_hardware_mining_id = Self::mining_speed_boosts_rates_hardware_mining_count();
        if mining_speed_boosts_rates_hardware_mining_id ==
            <T::MiningSpeedBoostRatesHardwareMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostRatesHardwareMining count overflow"));
        }
        Ok(mining_speed_boosts_rates_hardware_mining_id)
    }

    fn insert_mining_speed_boosts_rates_hardware_mining(
        owner: &T::AccountId,
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
        mining_speed_boosts_rates_hardware_mining: MiningSpeedBoostRatesHardwareMining,
    ) {
        // Create and store mining mining_speed_boosts_rates_hardware_mining
        <MiningSpeedBoostRatesHardwareMinings<T>>::insert(
            mining_speed_boosts_rates_hardware_mining_id,
            mining_speed_boosts_rates_hardware_mining,
        );
        <MiningSpeedBoostRatesHardwareMiningCount<T>>::put(mining_speed_boosts_rates_hardware_mining_id + One::one());
        <MiningSpeedBoostRatesHardwareMiningOwners<T>>::insert(
            mining_speed_boosts_rates_hardware_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_rates_hardware_mining_id: T::MiningSpeedBoostRatesHardwareMiningIndex,
    ) {
        <MiningSpeedBoostRatesHardwareMiningOwners<T>>::insert(mining_speed_boosts_rates_hardware_mining_id, to);
    }
}
