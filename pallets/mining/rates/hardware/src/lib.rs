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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config: frame_system::Config + roaming_operators::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningRatesHardwareIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningRatesHardwareSecure: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningRatesHardwareInsecure: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningRatesHardwareMaxHardware: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningRatesHardwareCategory1MaxTokenBonusPerGateway: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningRatesHardwareCategory2MaxTokenBonusPerGateway: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningRatesHardwareCategory3MaxTokenBonusPerGateway: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningRatesHardware(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningRatesHardwareSetting<U, V, W, X, Y, Z> {
    pub hardware_hardware_secure: U,
    pub hardware_hardware_insecure: V,
    pub hardware_max_hardware: W,
    pub hardware_category_1_max_token_bonus_per_gateway: X,
    pub hardware_category_2_max_token_bonus_per_gateway: Y,
    pub hardware_category_3_max_token_bonus_per_gateway: Z,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::MiningRatesHardwareIndex,
        <T as Config>::MiningRatesHardwareSecure,
        <T as Config>::MiningRatesHardwareInsecure,
        <T as Config>::MiningRatesHardwareMaxHardware,
        <T as Config>::MiningRatesHardwareCategory1MaxTokenBonusPerGateway,
        <T as Config>::MiningRatesHardwareCategory2MaxTokenBonusPerGateway,
        <T as Config>::MiningRatesHardwareCategory3MaxTokenBonusPerGateway,
        // Balance = BalanceOf<T>,
    {
        /// A mining_rates_hardware is created. (owner, mining_rates_hardware_id)
        Created(AccountId, MiningRatesHardwareIndex),
        /// A mining_rates_hardware is transferred. (from, to, mining_rates_hardware_id)
        Transferred(AccountId, AccountId, MiningRatesHardwareIndex),
        MiningRatesHardwareSettingSet(
            AccountId, MiningRatesHardwareIndex, MiningRatesHardwareSecure,
            MiningRatesHardwareInsecure, MiningRatesHardwareMaxHardware,
            MiningRatesHardwareCategory1MaxTokenBonusPerGateway,
            MiningRatesHardwareCategory2MaxTokenBonusPerGateway,
            MiningRatesHardwareCategory3MaxTokenBonusPerGateway
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningRatesHardware {
        /// Stores all the mining_rates_hardwares, key is the mining_rates_hardware id / index
        pub MiningRatesHardwares get(fn mining_rates_hardware): map hasher(opaque_blake2_256) T::MiningRatesHardwareIndex => Option<MiningRatesHardware>;

        /// Stores the total number of mining_rates_hardwares. i.e. the next mining_rates_hardware index
        pub MiningRatesHardwareCount get(fn mining_rates_hardware_count): T::MiningRatesHardwareIndex;

        /// Stores mining_rates_hardware owner
        pub MiningRatesHardwareOwners get(fn mining_rates_hardware_owner): map hasher(opaque_blake2_256) T::MiningRatesHardwareIndex => Option<T::AccountId>;

        /// Stores mining_rates_hardware_rates_config
        pub MiningRatesHardwareSettings get(fn mining_rates_hardware_rates_configs): map hasher(opaque_blake2_256) T::MiningRatesHardwareIndex =>
            Option<MiningRatesHardwareSetting<T::MiningRatesHardwareSecure,
            T::MiningRatesHardwareInsecure, T::MiningRatesHardwareMaxHardware,
            T::MiningRatesHardwareCategory1MaxTokenBonusPerGateway,
            T::MiningRatesHardwareCategory2MaxTokenBonusPerGateway,
            T::MiningRatesHardwareCategory3MaxTokenBonusPerGateway>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_rates_hardware
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_rates_hardware_id = Self::next_mining_rates_hardware_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_rates_hardware
            let mining_rates_hardware = MiningRatesHardware(unique_id);
            Self::insert_mining_rates_hardware(&sender, mining_rates_hardware_id, mining_rates_hardware);

            Self::deposit_event(RawEvent::Created(sender, mining_rates_hardware_id));
        }

        /// Transfer a mining_rates_hardware to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_rates_hardware_id: T::MiningRatesHardwareIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_rates_hardware_owner(mining_rates_hardware_id) == Some(sender.clone()), "Only owner can transfer mining mining_rates_hardware");

            Self::update_owner(&to, mining_rates_hardware_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_rates_hardware_id));
        }

        /// Set mining_rates_hardware_rates_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_rates_hardware_rates_config(
            origin,
            mining_rates_hardware_id: T::MiningRatesHardwareIndex,
            _hardware_hardware_secure: Option<T::MiningRatesHardwareSecure>,
            _hardware_hardware_insecure: Option<T::MiningRatesHardwareInsecure>,
            _hardware_max_hardware: Option<T::MiningRatesHardwareMaxHardware>,
            _hardware_category_1_max_token_bonus_per_gateway: Option<T::MiningRatesHardwareCategory1MaxTokenBonusPerGateway>,
            _hardware_category_2_max_token_bonus_per_gateway: Option<T::MiningRatesHardwareCategory2MaxTokenBonusPerGateway>,
            _hardware_category_3_max_token_bonus_per_gateway: Option<T::MiningRatesHardwareCategory3MaxTokenBonusPerGateway>
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_rates_hardware_id whose config we want to change actually exists
            let is_mining_rates_hardware = Self::exists_mining_rates_hardware(mining_rates_hardware_id).is_ok();
            ensure!(is_mining_rates_hardware, "MiningRatesHardware does not exist");

            // Ensure that the caller is owner of the mining_rates_hardware_rates_config they are trying to change
            ensure!(Self::mining_rates_hardware_owner(mining_rates_hardware_id) == Some(sender.clone()), "Only owner can set mining_rates_hardware_rates_config");

            // TODO - adjust default rates
            let hardware_hardware_secure = match _hardware_hardware_secure.clone() {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            let hardware_hardware_insecure = match _hardware_hardware_insecure {
                Some(value) => value,
                None => 1u32.into() // Default
            };
            let hardware_max_hardware = match _hardware_max_hardware {
              Some(value) => value,
              None => 1u32.into() // Default
            };
            let hardware_category_1_max_token_bonus_per_gateway = match _hardware_category_1_max_token_bonus_per_gateway.clone() {
                Some(value) => value,
                None => 1000000u32.into() // Default
            };
            let hardware_category_2_max_token_bonus_per_gateway = match _hardware_category_2_max_token_bonus_per_gateway {
                Some(value) => value,
                None => 500000u32.into() // Default
            };
            let hardware_category_3_max_token_bonus_per_gateway = match _hardware_category_3_max_token_bonus_per_gateway {
                Some(value) => value,
                None => 250000u32.into() // Default
            };

            // Check if a mining_rates_hardware_rates_config already exists with the given mining_rates_hardware_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_rates_hardware_rates_config_index(mining_rates_hardware_id).is_ok() {
                info!("Mutating values");
                <MiningRatesHardwareSettings<T>>::mutate(mining_rates_hardware_id, |mining_rates_hardware_rates_config| {
                    if let Some(_mining_rates_hardware_rates_config) = mining_rates_hardware_rates_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_rates_hardware_rates_config.hardware_hardware_secure = hardware_hardware_secure.clone();
                        _mining_rates_hardware_rates_config.hardware_hardware_insecure = hardware_hardware_insecure.clone();
                        _mining_rates_hardware_rates_config.hardware_max_hardware = hardware_max_hardware.clone();
                        _mining_rates_hardware_rates_config.hardware_category_1_max_token_bonus_per_gateway = hardware_category_1_max_token_bonus_per_gateway.clone();
                        _mining_rates_hardware_rates_config.hardware_category_2_max_token_bonus_per_gateway = hardware_category_2_max_token_bonus_per_gateway.clone();
                        _mining_rates_hardware_rates_config.hardware_category_3_max_token_bonus_per_gateway = hardware_category_3_max_token_bonus_per_gateway.clone();
                    }
                });
                info!("Checking mutated values");
                let fetched_mining_rates_hardware_rates_config = <MiningRatesHardwareSettings<T>>::get(mining_rates_hardware_id);
                if let Some(_mining_rates_hardware_rates_config) = fetched_mining_rates_hardware_rates_config {
                    info!("Latest field hardware_hardware_secure {:#?}", _mining_rates_hardware_rates_config.hardware_hardware_secure);
                    info!("Latest field hardware_hardware_insecure {:#?}", _mining_rates_hardware_rates_config.hardware_hardware_insecure);
                    info!("Latest field hardware_max_hardware {:#?}", _mining_rates_hardware_rates_config.hardware_max_hardware);
                    info!("Latest field hardware_category_1_max_token_bonus_per_gateway {:#?}", _mining_rates_hardware_rates_config.hardware_category_1_max_token_bonus_per_gateway);
                    info!("Latest field hardware_category_2_max_token_bonus_per_gateway {:#?}", _mining_rates_hardware_rates_config.hardware_category_2_max_token_bonus_per_gateway);
                    info!("Latest field hardware_category_3_max_token_bonus_per_gateway {:#?}", _mining_rates_hardware_rates_config.hardware_category_3_max_token_bonus_per_gateway);
                }
            } else {
                info!("Inserting values");

                // Create a new mining mining_rates_hardware_rates_config instance with the input params
                let mining_rates_hardware_rates_config_instance = MiningRatesHardwareSetting {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_hardware_secure: hardware_hardware_secure.clone(),
                    hardware_hardware_insecure: hardware_hardware_insecure.clone(),
                    hardware_max_hardware: hardware_max_hardware.clone(),
                    hardware_category_1_max_token_bonus_per_gateway: hardware_category_1_max_token_bonus_per_gateway.clone(),
                    hardware_category_2_max_token_bonus_per_gateway: hardware_category_2_max_token_bonus_per_gateway.clone(),
                    hardware_category_3_max_token_bonus_per_gateway: hardware_category_3_max_token_bonus_per_gateway.clone(),
                };

                <MiningRatesHardwareSettings<T>>::insert(
                    mining_rates_hardware_id,
                    &mining_rates_hardware_rates_config_instance
                );

                info!("Checking inserted values");
                let fetched_mining_rates_hardware_rates_config = <MiningRatesHardwareSettings<T>>::get(mining_rates_hardware_id);
                if let Some(_mining_rates_hardware_rates_config) = fetched_mining_rates_hardware_rates_config {
                    info!("Inserted field hardware_hardware_secure {:#?}", _mining_rates_hardware_rates_config.hardware_hardware_secure);
                    info!("Inserted field hardware_hardware_insecure {:#?}", _mining_rates_hardware_rates_config.hardware_hardware_insecure);
                    info!("Inserted field hardware_max_hardware {:#?}", _mining_rates_hardware_rates_config.hardware_max_hardware);
                    info!("Inserted field hardware_category_1_max_token_bonus_per_gateway {:#?}", _mining_rates_hardware_rates_config.hardware_category_1_max_token_bonus_per_gateway);
                    info!("Inserted field hardware_category_2_max_token_bonus_per_gateway {:#?}", _mining_rates_hardware_rates_config.hardware_category_2_max_token_bonus_per_gateway);
                    info!("Inserted field hardware_category_3_max_token_bonus_per_gateway {:#?}", _mining_rates_hardware_rates_config.hardware_category_3_max_token_bonus_per_gateway);
                }
            }

            Self::deposit_event(RawEvent::MiningRatesHardwareSettingSet(
                sender,
                mining_rates_hardware_id,
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

impl<T: Config> Module<T> {
    pub fn is_mining_rates_hardware_owner(
        mining_rates_hardware_id: T::MiningRatesHardwareIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_rates_hardware_owner(&mining_rates_hardware_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of MiningRatesHardware"
        );
        Ok(())
    }

    pub fn exists_mining_rates_hardware(
        mining_rates_hardware_id: T::MiningRatesHardwareIndex,
    ) -> Result<MiningRatesHardware, DispatchError> {
        match Self::mining_rates_hardware(mining_rates_hardware_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningRatesHardware does not exist")),
        }
    }

    pub fn exists_mining_rates_hardware_rates_config(
        mining_rates_hardware_id: T::MiningRatesHardwareIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_rates_hardware_rates_configs(mining_rates_hardware_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningRatesHardwareSetting does not exist")),
        }
    }

    pub fn has_value_for_mining_rates_hardware_rates_config_index(
        mining_rates_hardware_id: T::MiningRatesHardwareIndex,
    ) -> Result<(), DispatchError> {
        info!("Checking if mining_rates_hardware_rates_config has a value that is defined");
        let fetched_mining_rates_hardware_rates_config = <MiningRatesHardwareSettings<T>>::get(mining_rates_hardware_id);
        if let Some(_value) = fetched_mining_rates_hardware_rates_config {
            info!("Found value for mining_rates_hardware_rates_config");
            return Ok(());
        }
        warn!("No value for mining_rates_hardware_rates_config");
        Err(DispatchError::Other("No value for mining_rates_hardware_rates_config"))
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

    fn next_mining_rates_hardware_id() -> Result<T::MiningRatesHardwareIndex, DispatchError> {
        let mining_rates_hardware_id = Self::mining_rates_hardware_count();
        if mining_rates_hardware_id == <T::MiningRatesHardwareIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningRatesHardware count overflow"));
        }
        Ok(mining_rates_hardware_id)
    }

    fn insert_mining_rates_hardware(
        owner: &T::AccountId,
        mining_rates_hardware_id: T::MiningRatesHardwareIndex,
        mining_rates_hardware: MiningRatesHardware,
    ) {
        // Create and store mining mining_rates_hardware
        <MiningRatesHardwares<T>>::insert(mining_rates_hardware_id, mining_rates_hardware);
        <MiningRatesHardwareCount<T>>::put(mining_rates_hardware_id + One::one());
        <MiningRatesHardwareOwners<T>>::insert(mining_rates_hardware_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_rates_hardware_id: T::MiningRatesHardwareIndex) {
        <MiningRatesHardwareOwners<T>>::insert(mining_rates_hardware_id, to);
    }
}
