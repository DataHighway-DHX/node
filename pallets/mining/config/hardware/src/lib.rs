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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Config: frame_system::Config + roaming_operators::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type MiningConfigHardwareIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // Mining Speed Boost Hardware Mining Config
    type MiningConfigHardwareSecure: Parameter + Member + Default + Copy; // bool
    type MiningConfigHardwareType: Parameter + Member + Default;
    type MiningConfigHardwareID: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningConfigHardwareDevEUI: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // // Mining Speed Boost Reward
    // type MiningClaimAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningClaimDateRedeemed: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Config>::Currency as Currency<<T as
// frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningConfigHardware(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningConfigHardwareConfig<U, V, W, X, Y, Z> {
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
        <T as Trait>::MiningConfigHardwareIndex,
        <T as Trait>::MiningConfigHardwareSecure,
        <T as Trait>::MiningConfigHardwareType,
        <T as Trait>::MiningConfigHardwareID,
        <T as Trait>::MiningConfigHardwareDevEUI,
        <T as frame_system::Config>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_config_hardware is created. (owner, mining_config_hardware_id)
        Created(AccountId, MiningConfigHardwareIndex),
        /// A mining_config_hardware is transferred. (from, to, mining_config_hardware_id)
        Transferred(AccountId, AccountId, MiningConfigHardwareIndex),
        MiningConfigHardwareConfigSet(
          AccountId, MiningConfigHardwareIndex, MiningConfigHardwareSecure,
          MiningConfigHardwareType, MiningConfigHardwareID,
          MiningConfigHardwareDevEUI, BlockNumber, BlockNumber
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningConfigHardware {
        /// Stores all the mining_config_hardwares, key is the mining_config_hardware id / index
        pub MiningConfigHardwares get(fn mining_config_hardware): map hasher(opaque_blake2_256) T::MiningConfigHardwareIndex => Option<MiningConfigHardware>;

        /// Stores the total number of mining_config_hardwares. i.e. the next mining_config_hardware index
        pub MiningConfigHardwareCount get(fn mining_config_hardware_count): T::MiningConfigHardwareIndex;

        /// Stores mining_config_hardware owner
        pub MiningConfigHardwareOwners get(fn mining_config_hardware_owner): map hasher(opaque_blake2_256) T::MiningConfigHardwareIndex => Option<T::AccountId>;

        /// Stores mining_config_hardware_hardware_config
        pub MiningConfigHardwareConfigs get(fn mining_config_hardware_hardware_configs): map hasher(opaque_blake2_256) T::MiningConfigHardwareIndex =>
            Option<MiningConfigHardwareConfig<T::MiningConfigHardwareSecure, T::MiningConfigHardwareType,
                T::MiningConfigHardwareID, T::MiningConfigHardwareDevEUI, T::BlockNumber,
                T::BlockNumber>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_config_hardware
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_config_hardware_id = Self::next_mining_config_hardware_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_config_hardware
            let mining_config_hardware = MiningConfigHardware(unique_id);
            Self::insert_mining_config_hardware(&sender, mining_config_hardware_id, mining_config_hardware);

            Self::deposit_event(RawEvent::Created(sender, mining_config_hardware_id));
        }

        /// Transfer a mining_config_hardware to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_config_hardware_id: T::MiningConfigHardwareIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_config_hardware_owner(mining_config_hardware_id) == Some(sender.clone()), "Only owner can transfer mining mining_config_hardware");

            Self::update_owner(&to, mining_config_hardware_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_config_hardware_id));
        }

        /// Set mining_config_hardware_hardware_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_config_hardware_hardware_config(
            origin,
            mining_config_hardware_id: T::MiningConfigHardwareIndex,
            _hardware_secure: Option<T::MiningConfigHardwareSecure>,
            _hardware_type: Option<T::MiningConfigHardwareType>,
            _hardware_id: Option<T::MiningConfigHardwareID>,
            _hardware_dev_eui: Option<T::MiningConfigHardwareDevEUI>,
            _hardware_lock_start_block: Option<T::BlockNumber>,
            _hardware_lock_interval_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_config_hardware_id whose config we want to change actually exists
            let is_mining_config_hardware = Self::exists_mining_config_hardware(mining_config_hardware_id).is_ok();
            ensure!(is_mining_config_hardware, "MiningConfigHardware does not exist");

            // Ensure that the caller is owner of the mining_config_hardware_hardware_config they are trying to change
            ensure!(Self::mining_config_hardware_owner(mining_config_hardware_id) == Some(sender.clone()), "Only owner can set mining_config_hardware_hardware_config");

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
            let hardware_lock_start_block = match _hardware_lock_start_block {
                Some(value) => value,
                None => Default::default() // Default
            };
            let hardware_lock_interval_blocks = match _hardware_lock_interval_blocks {
                Some(value) => value,
                None => Default::default() // Default
            };

            // Check if a mining_config_hardware_hardware_config already exists with the given mining_config_hardware_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_config_hardware_hardware_config_index(mining_config_hardware_id).is_ok() {
                debug::info!("Mutating values");
                // TODO
                <MiningConfigHardwareConfigs<T>>::mutate(mining_config_hardware_id, |mining_config_hardware_hardware_config| {
                    if let Some(_mining_config_hardware_hardware_config) = mining_config_hardware_hardware_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_config_hardware_hardware_config.hardware_secure = hardware_secure.clone();
                        _mining_config_hardware_hardware_config.hardware_type = hardware_type.clone();
                        _mining_config_hardware_hardware_config.hardware_id = hardware_id.clone();
                        _mining_config_hardware_hardware_config.hardware_dev_eui = hardware_dev_eui.clone();
                        _mining_config_hardware_hardware_config.hardware_lock_start_block = hardware_lock_start_block.clone();
                        _mining_config_hardware_hardware_config.hardware_lock_interval_blocks = hardware_lock_interval_blocks.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_config_hardware_hardware_config = <MiningConfigHardwareConfigs<T>>::get(mining_config_hardware_id);
                if let Some(_mining_config_hardware_hardware_config) = fetched_mining_config_hardware_hardware_config {
                    debug::info!("Latest field hardware_secure {:#?}", _mining_config_hardware_hardware_config.hardware_secure);
                    debug::info!("Latest field hardware_type {:#?}", _mining_config_hardware_hardware_config.hardware_type);
                    debug::info!("Latest field hardware_id {:#?}", _mining_config_hardware_hardware_config.hardware_id);
                    debug::info!("Latest field hardware_dev_eui {:#?}", _mining_config_hardware_hardware_config.hardware_dev_eui);
                    debug::info!("Latest field hardware_lock_start_block {:#?}", _mining_config_hardware_hardware_config.hardware_lock_start_block);
                    debug::info!("Latest field hardware_lock_interval_blocks {:#?}", _mining_config_hardware_hardware_config.hardware_lock_interval_blocks);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_config_hardware_hardware_config instance with the input params
                let mining_config_hardware_hardware_config_instance = MiningConfigHardwareConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_secure: hardware_secure.clone(),
                    hardware_type: hardware_type.clone(),
                    hardware_id: hardware_id.clone(),
                    hardware_dev_eui: hardware_dev_eui.clone(),
                    hardware_lock_start_block: hardware_lock_start_block.clone(),
                    hardware_lock_interval_blocks: hardware_lock_interval_blocks.clone(),
                };

                <MiningConfigHardwareConfigs<T>>::insert(
                    mining_config_hardware_id,
                    &mining_config_hardware_hardware_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_config_hardware_hardware_config = <MiningConfigHardwareConfigs<T>>::get(mining_config_hardware_id);
                if let Some(_mining_config_hardware_hardware_config) = fetched_mining_config_hardware_hardware_config {
                    debug::info!("Inserted field hardware_secure {:#?}", _mining_config_hardware_hardware_config.hardware_secure);
                    debug::info!("Inserted field hardware_type {:#?}", _mining_config_hardware_hardware_config.hardware_type);
                    debug::info!("Inserted field hardware_id {:#?}", _mining_config_hardware_hardware_config.hardware_id);
                    debug::info!("Inserted field hardware_dev_eui {:#?}", _mining_config_hardware_hardware_config.hardware_dev_eui);
                    debug::info!("Inserted field hardware_lock_start_block {:#?}", _mining_config_hardware_hardware_config.hardware_lock_start_block);
                    debug::info!("Inserted field hardware_lock_interval_blocks {:#?}", _mining_config_hardware_hardware_config.hardware_lock_interval_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningConfigHardwareConfigSet(
                sender,
                mining_config_hardware_id,
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
    pub fn is_mining_config_hardware_owner(
        mining_config_hardware_id: T::MiningConfigHardwareIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_config_hardware_owner(&mining_config_hardware_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of Mining"
        );
        Ok(())
    }

    pub fn exists_mining_config_hardware(
        mining_config_hardware_id: T::MiningConfigHardwareIndex,
    ) -> Result<MiningConfigHardware, DispatchError> {
        match Self::mining_config_hardware(mining_config_hardware_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningConfigHardware does not exist")),
        }
    }

    pub fn exists_mining_config_hardware_hardware_config(
        mining_config_hardware_id: T::MiningConfigHardwareIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_config_hardware_hardware_configs(mining_config_hardware_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningConfigHardwareConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_config_hardware_hardware_config_index(
        mining_config_hardware_id: T::MiningConfigHardwareIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_config_hardware_hardware_config has a value that is defined");
        let fetched_mining_config_hardware_hardware_config =
            <MiningConfigHardwareConfigs<T>>::get(mining_config_hardware_id);
        if let Some(_value) = fetched_mining_config_hardware_hardware_config {
            debug::info!("Found value for mining_config_hardware_hardware_config");
            return Ok(());
        }
        debug::info!("No value for mining_config_hardware_hardware_config");
        Err(DispatchError::Other("No value for mining_config_hardware_hardware_config"))
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

    fn next_mining_config_hardware_id() -> Result<T::MiningConfigHardwareIndex, DispatchError> {
        let mining_config_hardware_id = Self::mining_config_hardware_count();
        if mining_config_hardware_id == <T::MiningConfigHardwareIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningConfigHardware count overflow"));
        }
        Ok(mining_config_hardware_id)
    }

    fn insert_mining_config_hardware(
        owner: &T::AccountId,
        mining_config_hardware_id: T::MiningConfigHardwareIndex,
        mining_config_hardware: MiningConfigHardware,
    ) {
        // Create and store mining mining_config_hardware
        <MiningConfigHardwares<T>>::insert(mining_config_hardware_id, mining_config_hardware);
        <MiningConfigHardwareCount<T>>::put(mining_config_hardware_id + One::one());
        <MiningConfigHardwareOwners<T>>::insert(mining_config_hardware_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_config_hardware_id: T::MiningConfigHardwareIndex) {
        <MiningConfigHardwareOwners<T>>::insert(mining_config_hardware_id, to);
    }
}
