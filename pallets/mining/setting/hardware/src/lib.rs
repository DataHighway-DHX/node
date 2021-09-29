#![cfg_attr(not(feature = "std"), no_std)]

use codec::{
    Decode,
    Encode,
};
use frame_support::{
    log,
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
    type MiningSettingHardwareIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // Mining Speed Boost Hardware Mining Config
    type MiningSettingHardwareSecure: Parameter + Member + Default + Copy; // bool
    type MiningSettingHardwareType: Parameter + Member + Default;
    type MiningSettingHardwareID: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSettingHardwareDevEUI: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // // Mining Speed Boost Reward
    // type MiningClaimAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningClaimDateRedeemed: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSettingHardware(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSettingHardwareSetting<U, V, W, X, Y, Z> {
    pub hardware_secure: U,
    pub hardware_type: V,
    pub hardware_id: W,
    pub hardware_dev_eui: X,
    pub hardware_lock_start_block: Y,
    pub hardware_lock_interval_blocks: Z,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
        <T as Config>::MiningSettingHardwareIndex,
        <T as Config>::MiningSettingHardwareSecure,
        <T as Config>::MiningSettingHardwareType,
        <T as Config>::MiningSettingHardwareID,
        <T as Config>::MiningSettingHardwareDevEUI,
        <T as frame_system::Config>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_setting_hardware is created. (owner, mining_setting_hardware_id)
        Created(AccountId, MiningSettingHardwareIndex),
        /// A mining_setting_hardware is transferred. (from, to, mining_setting_hardware_id)
        Transferred(AccountId, AccountId, MiningSettingHardwareIndex),
        MiningSettingHardwareSettingSet(
          AccountId, MiningSettingHardwareIndex, MiningSettingHardwareSecure,
          MiningSettingHardwareType, MiningSettingHardwareID,
          MiningSettingHardwareDevEUI, BlockNumber, BlockNumber
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningSettingHardware {
        /// Stores all the mining_setting_hardwares, key is the mining_setting_hardware id / index
        pub MiningSettingHardwares get(fn mining_setting_hardware): map hasher(opaque_blake2_256) T::MiningSettingHardwareIndex => Option<MiningSettingHardware>;

        /// Stores the total number of mining_setting_hardwares. i.e. the next mining_setting_hardware index
        pub MiningSettingHardwareCount get(fn mining_setting_hardware_count): T::MiningSettingHardwareIndex;

        /// Stores mining_setting_hardware owner
        pub MiningSettingHardwareOwners get(fn mining_setting_hardware_owner): map hasher(opaque_blake2_256) T::MiningSettingHardwareIndex => Option<T::AccountId>;

        /// Stores mining_setting_hardware_hardware_config
        pub MiningSettingHardwareSettings get(fn mining_setting_hardware_hardware_configs): map hasher(opaque_blake2_256) T::MiningSettingHardwareIndex =>
            Option<MiningSettingHardwareSetting<T::MiningSettingHardwareSecure, T::MiningSettingHardwareType,
                T::MiningSettingHardwareID, T::MiningSettingHardwareDevEUI, T::BlockNumber,
                T::BlockNumber>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_setting_hardware
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_setting_hardware_id = Self::next_mining_setting_hardware_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_setting_hardware
            let mining_setting_hardware = MiningSettingHardware(unique_id);
            Self::insert_mining_setting_hardware(&sender, mining_setting_hardware_id, mining_setting_hardware);

            Self::deposit_event(RawEvent::Created(sender, mining_setting_hardware_id));
        }

        /// Transfer a mining_setting_hardware to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_setting_hardware_id: T::MiningSettingHardwareIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_setting_hardware_owner(mining_setting_hardware_id) == Some(sender.clone()), "Only owner can transfer mining mining_setting_hardware");

            Self::update_owner(&to, mining_setting_hardware_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_setting_hardware_id));
        }

        /// Set mining_setting_hardware_hardware_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_setting_hardware_hardware_config(
            origin,
            mining_setting_hardware_id: T::MiningSettingHardwareIndex,
            _hardware_secure: Option<T::MiningSettingHardwareSecure>,
            _hardware_type: Option<T::MiningSettingHardwareType>,
            _hardware_id: Option<T::MiningSettingHardwareID>,
            _hardware_dev_eui: Option<T::MiningSettingHardwareDevEUI>,
            _hardware_lock_start_block: Option<T::BlockNumber>,
            _hardware_lock_interval_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_setting_hardware_id whose config we want to change actually exists
            let is_mining_setting_hardware = Self::exists_mining_setting_hardware(mining_setting_hardware_id).is_ok();
            ensure!(is_mining_setting_hardware, "MiningSettingHardware does not exist");

            // Ensure that the caller is owner of the mining_setting_hardware_hardware_config they are trying to change
            ensure!(Self::mining_setting_hardware_owner(mining_setting_hardware_id) == Some(sender.clone()), "Only owner can set mining_setting_hardware_hardware_config");

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
                None => 3u32.into() // Default
            };
            let hardware_dev_eui = match _hardware_dev_eui {
                Some(value) => value,
                None => Default::default() // Default
            };
            let hardware_lock_start_block = match _hardware_lock_start_block {
                Some(value) => value,
                None => Default::default() // Default
            };
            let hardware_lock_interval_blocks = match _hardware_lock_interval_blocks {
                Some(value) => value,
                None => Default::default() // Default
            };

            // Check if a mining_setting_hardware_hardware_config already exists with the given mining_setting_hardware_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_setting_hardware_hardware_config_index(mining_setting_hardware_id).is_ok() {
                log::info!("Mutating values");
                // TODO
                <MiningSettingHardwareSettings<T>>::mutate(mining_setting_hardware_id, |mining_setting_hardware_hardware_config| {
                    if let Some(_mining_setting_hardware_hardware_config) = mining_setting_hardware_hardware_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_setting_hardware_hardware_config.hardware_secure = hardware_secure.clone();
                        _mining_setting_hardware_hardware_config.hardware_type = hardware_type.clone();
                        _mining_setting_hardware_hardware_config.hardware_id = hardware_id.clone();
                        _mining_setting_hardware_hardware_config.hardware_dev_eui = hardware_dev_eui.clone();
                        _mining_setting_hardware_hardware_config.hardware_lock_start_block = hardware_lock_start_block.clone();
                        _mining_setting_hardware_hardware_config.hardware_lock_interval_blocks = hardware_lock_interval_blocks.clone();
                    }
                });
                log::info!("Checking mutated values");
                let fetched_mining_setting_hardware_hardware_config = <MiningSettingHardwareSettings<T>>::get(mining_setting_hardware_id);
                if let Some(_mining_setting_hardware_hardware_config) = fetched_mining_setting_hardware_hardware_config {
                    log::info!("Latest field hardware_secure {:#?}", _mining_setting_hardware_hardware_config.hardware_secure);
                    log::info!("Latest field hardware_type {:#?}", _mining_setting_hardware_hardware_config.hardware_type);
                    log::info!("Latest field hardware_id {:#?}", _mining_setting_hardware_hardware_config.hardware_id);
                    log::info!("Latest field hardware_dev_eui {:#?}", _mining_setting_hardware_hardware_config.hardware_dev_eui);
                    log::info!("Latest field hardware_lock_start_block {:#?}", _mining_setting_hardware_hardware_config.hardware_lock_start_block);
                    log::info!("Latest field hardware_lock_interval_blocks {:#?}", _mining_setting_hardware_hardware_config.hardware_lock_interval_blocks);
                }
            } else {
                log::info!("Inserting values");

                // Create a new mining mining_setting_hardware_hardware_config instance with the input params
                let mining_setting_hardware_hardware_config_instance = MiningSettingHardwareSetting {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_secure: hardware_secure.clone(),
                    hardware_type: hardware_type.clone(),
                    hardware_id: hardware_id.clone(),
                    hardware_dev_eui: hardware_dev_eui.clone(),
                    hardware_lock_start_block: hardware_lock_start_block.clone(),
                    hardware_lock_interval_blocks: hardware_lock_interval_blocks.clone(),
                };

                <MiningSettingHardwareSettings<T>>::insert(
                    mining_setting_hardware_id,
                    &mining_setting_hardware_hardware_config_instance
                );

                log::info!("Checking inserted values");
                let fetched_mining_setting_hardware_hardware_config = <MiningSettingHardwareSettings<T>>::get(mining_setting_hardware_id);
                if let Some(_mining_setting_hardware_hardware_config) = fetched_mining_setting_hardware_hardware_config {
                    log::info!("Inserted field hardware_secure {:#?}", _mining_setting_hardware_hardware_config.hardware_secure);
                    log::info!("Inserted field hardware_type {:#?}", _mining_setting_hardware_hardware_config.hardware_type);
                    log::info!("Inserted field hardware_id {:#?}", _mining_setting_hardware_hardware_config.hardware_id);
                    log::info!("Inserted field hardware_dev_eui {:#?}", _mining_setting_hardware_hardware_config.hardware_dev_eui);
                    log::info!("Inserted field hardware_lock_start_block {:#?}", _mining_setting_hardware_hardware_config.hardware_lock_start_block);
                    log::info!("Inserted field hardware_lock_interval_blocks {:#?}", _mining_setting_hardware_hardware_config.hardware_lock_interval_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningSettingHardwareSettingSet(
                sender,
                mining_setting_hardware_id,
                hardware_secure,
                hardware_type,
                hardware_id,
                hardware_dev_eui,
                hardware_lock_start_block,
                hardware_lock_interval_blocks,
            ));
        }
    }
}

impl<T: Config> Module<T> {
    pub fn is_mining_setting_hardware_owner(
        mining_setting_hardware_id: T::MiningSettingHardwareIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_setting_hardware_owner(&mining_setting_hardware_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of Mining"
        );
        Ok(())
    }

    pub fn exists_mining_setting_hardware(
        mining_setting_hardware_id: T::MiningSettingHardwareIndex,
    ) -> Result<MiningSettingHardware, DispatchError> {
        match Self::mining_setting_hardware(mining_setting_hardware_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSettingHardware does not exist")),
        }
    }

    pub fn exists_mining_setting_hardware_hardware_config(
        mining_setting_hardware_id: T::MiningSettingHardwareIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_setting_hardware_hardware_configs(mining_setting_hardware_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningSettingHardwareSetting does not exist")),
        }
    }

    pub fn has_value_for_mining_setting_hardware_hardware_config_index(
        mining_setting_hardware_id: T::MiningSettingHardwareIndex,
    ) -> Result<(), DispatchError> {
        log::info!("Checking if mining_setting_hardware_hardware_config has a value that is defined");
        let fetched_mining_setting_hardware_hardware_config =
            <MiningSettingHardwareSettings<T>>::get(mining_setting_hardware_id);
        if let Some(_value) = fetched_mining_setting_hardware_hardware_config {
            log::info!("Found value for mining_setting_hardware_hardware_config");
            return Ok(());
        }
        log::info!("No value for mining_setting_hardware_hardware_config");
        Err(DispatchError::Other("No value for mining_setting_hardware_hardware_config"))
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

    fn next_mining_setting_hardware_id() -> Result<T::MiningSettingHardwareIndex, DispatchError> {
        let mining_setting_hardware_id = Self::mining_setting_hardware_count();
        if mining_setting_hardware_id == <T::MiningSettingHardwareIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningSettingHardware count overflow"));
        }
        Ok(mining_setting_hardware_id)
    }

    fn insert_mining_setting_hardware(
        owner: &T::AccountId,
        mining_setting_hardware_id: T::MiningSettingHardwareIndex,
        mining_setting_hardware: MiningSettingHardware,
    ) {
        // Create and store mining mining_setting_hardware
        <MiningSettingHardwares<T>>::insert(mining_setting_hardware_id, mining_setting_hardware);
        <MiningSettingHardwareCount<T>>::put(mining_setting_hardware_id + One::one());
        <MiningSettingHardwareOwners<T>>::insert(mining_setting_hardware_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_setting_hardware_id: T::MiningSettingHardwareIndex) {
        <MiningSettingHardwareOwners<T>>::insert(mining_setting_hardware_id, to);
    }
}
