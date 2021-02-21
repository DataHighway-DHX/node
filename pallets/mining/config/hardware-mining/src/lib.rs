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
pub trait Trait: frame_system::Trait + roaming_operators::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningConfigHardwareMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // Mining Speed Boost Hardware Mining Config
    type MiningConfigHardwareMiningHardwareSecure: Parameter + Member + Default + Copy; // bool
    type MiningConfigHardwareMiningHardwareType: Parameter + Member + Default;
    type MiningConfigHardwareMiningHardwareID: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    type MiningConfigHardwareMiningHardwareDevEUI: Parameter
        + Member
        + AtLeast32Bit
        + Bounded
        + Default
        + Copy;
    // // Mining Speed Boost Reward
    // type MiningClaimAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    // type MiningClaimDateRedeemed: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningConfigHardwareMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningConfigHardwareMiningHardwareConfig<U, V, W, X, Y, Z> {
    pub hardware_secure: U,
    pub hardware_type: V,
    pub hardware_id: W,
    pub hardware_dev_eui: X,
    pub hardware_lock_start_block: Y,
    pub hardware_lock_interval_blocks: Z,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningConfigHardwareMiningIndex,
        <T as Trait>::MiningConfigHardwareMiningHardwareSecure,
        <T as Trait>::MiningConfigHardwareMiningHardwareType,
        <T as Trait>::MiningConfigHardwareMiningHardwareID,
        <T as Trait>::MiningConfigHardwareMiningHardwareDevEUI,
        <T as frame_system::Trait>::BlockNumber,
        // Balance = BalanceOf<T>,
    {
        /// A mining_config_hardware_mining is created. (owner, mining_config_hardware_mining_id)
        Created(AccountId, MiningConfigHardwareMiningIndex),
        /// A mining_config_hardware_mining is transferred. (from, to, mining_config_hardware_mining_id)
        Transferred(AccountId, AccountId, MiningConfigHardwareMiningIndex),
        MiningConfigHardwareMiningHardwareConfigSet(
          AccountId, MiningConfigHardwareMiningIndex, MiningConfigHardwareMiningHardwareSecure,
          MiningConfigHardwareMiningHardwareType, MiningConfigHardwareMiningHardwareID,
          MiningConfigHardwareMiningHardwareDevEUI, BlockNumber, BlockNumber
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningConfigHardwareMining {
        /// Stores all the mining_config_hardware_minings, key is the mining_config_hardware_mining id / index
        pub MiningConfigHardwareMinings get(fn mining_config_hardware_mining): map hasher(opaque_blake2_256) T::MiningConfigHardwareMiningIndex => Option<MiningConfigHardwareMining>;

        /// Stores the total number of mining_config_hardware_minings. i.e. the next mining_config_hardware_mining index
        pub MiningConfigHardwareMiningCount get(fn mining_config_hardware_mining_count): T::MiningConfigHardwareMiningIndex;

        /// Stores mining_config_hardware_mining owner
        pub MiningConfigHardwareMiningOwners get(fn mining_config_hardware_mining_owner): map hasher(opaque_blake2_256) T::MiningConfigHardwareMiningIndex => Option<T::AccountId>;

        /// Stores mining_config_hardware_mining_hardware_config
        pub MiningConfigHardwareMiningHardwareConfigs get(fn mining_config_hardware_mining_hardware_configs): map hasher(opaque_blake2_256) T::MiningConfigHardwareMiningIndex =>
            Option<MiningConfigHardwareMiningHardwareConfig<T::MiningConfigHardwareMiningHardwareSecure, T::MiningConfigHardwareMiningHardwareType,
                T::MiningConfigHardwareMiningHardwareID, T::MiningConfigHardwareMiningHardwareDevEUI, T::BlockNumber,
                T::BlockNumber>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_config_hardware_mining
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_config_hardware_mining_id = Self::next_mining_config_hardware_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_config_hardware_mining
            let mining_config_hardware_mining = MiningConfigHardwareMining(unique_id);
            Self::insert_mining_config_hardware_mining(&sender, mining_config_hardware_mining_id, mining_config_hardware_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_config_hardware_mining_id));
        }

        /// Transfer a mining_config_hardware_mining to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_config_hardware_mining_owner(mining_config_hardware_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_config_hardware_mining");

            Self::update_owner(&to, mining_config_hardware_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_config_hardware_mining_id));
        }

        /// Set mining_config_hardware_mining_hardware_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_config_hardware_mining_hardware_config(
            origin,
            mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
            _hardware_secure: Option<T::MiningConfigHardwareMiningHardwareSecure>,
            _hardware_type: Option<T::MiningConfigHardwareMiningHardwareType>,
            _hardware_id: Option<T::MiningConfigHardwareMiningHardwareID>,
            _hardware_dev_eui: Option<T::MiningConfigHardwareMiningHardwareDevEUI>,
            _hardware_lock_start_block: Option<T::BlockNumber>,
            _hardware_lock_interval_blocks: Option<T::BlockNumber>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_config_hardware_mining_id whose config we want to change actually exists
            let is_mining_config_hardware_mining = Self::exists_mining_config_hardware_mining(mining_config_hardware_mining_id).is_ok();
            ensure!(is_mining_config_hardware_mining, "MiningConfigHardwareMining does not exist");

            // Ensure that the caller is owner of the mining_config_hardware_mining_hardware_config they are trying to change
            ensure!(Self::mining_config_hardware_mining_owner(mining_config_hardware_mining_id) == Some(sender.clone()), "Only owner can set mining_config_hardware_mining_hardware_config");

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

            // Check if a mining_config_hardware_mining_hardware_config already exists with the given mining_config_hardware_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_config_hardware_mining_hardware_config_index(mining_config_hardware_mining_id).is_ok() {
                debug::info!("Mutating values");
                // TODO
                <MiningConfigHardwareMiningHardwareConfigs<T>>::mutate(mining_config_hardware_mining_id, |mining_config_hardware_mining_hardware_config| {
                    if let Some(_mining_config_hardware_mining_hardware_config) = mining_config_hardware_mining_hardware_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_config_hardware_mining_hardware_config.hardware_secure = hardware_secure.clone();
                        _mining_config_hardware_mining_hardware_config.hardware_type = hardware_type.clone();
                        _mining_config_hardware_mining_hardware_config.hardware_id = hardware_id.clone();
                        _mining_config_hardware_mining_hardware_config.hardware_dev_eui = hardware_dev_eui.clone();
                        _mining_config_hardware_mining_hardware_config.hardware_lock_start_block = hardware_lock_start_block.clone();
                        _mining_config_hardware_mining_hardware_config.hardware_lock_interval_blocks = hardware_lock_interval_blocks.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_config_hardware_mining_hardware_config = <MiningConfigHardwareMiningHardwareConfigs<T>>::get(mining_config_hardware_mining_id);
                if let Some(_mining_config_hardware_mining_hardware_config) = fetched_mining_config_hardware_mining_hardware_config {
                    debug::info!("Latest field hardware_secure {:#?}", _mining_config_hardware_mining_hardware_config.hardware_secure);
                    debug::info!("Latest field hardware_type {:#?}", _mining_config_hardware_mining_hardware_config.hardware_type);
                    debug::info!("Latest field hardware_id {:#?}", _mining_config_hardware_mining_hardware_config.hardware_id);
                    debug::info!("Latest field hardware_dev_eui {:#?}", _mining_config_hardware_mining_hardware_config.hardware_dev_eui);
                    debug::info!("Latest field hardware_lock_start_block {:#?}", _mining_config_hardware_mining_hardware_config.hardware_lock_start_block);
                    debug::info!("Latest field hardware_lock_interval_blocks {:#?}", _mining_config_hardware_mining_hardware_config.hardware_lock_interval_blocks);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_config_hardware_mining_hardware_config instance with the input params
                let mining_config_hardware_mining_hardware_config_instance = MiningConfigHardwareMiningHardwareConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hardware_secure: hardware_secure.clone(),
                    hardware_type: hardware_type.clone(),
                    hardware_id: hardware_id.clone(),
                    hardware_dev_eui: hardware_dev_eui.clone(),
                    hardware_lock_start_block: hardware_lock_start_block.clone(),
                    hardware_lock_interval_blocks: hardware_lock_interval_blocks.clone(),
                };

                <MiningConfigHardwareMiningHardwareConfigs<T>>::insert(
                    mining_config_hardware_mining_id,
                    &mining_config_hardware_mining_hardware_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_config_hardware_mining_hardware_config = <MiningConfigHardwareMiningHardwareConfigs<T>>::get(mining_config_hardware_mining_id);
                if let Some(_mining_config_hardware_mining_hardware_config) = fetched_mining_config_hardware_mining_hardware_config {
                    debug::info!("Inserted field hardware_secure {:#?}", _mining_config_hardware_mining_hardware_config.hardware_secure);
                    debug::info!("Inserted field hardware_type {:#?}", _mining_config_hardware_mining_hardware_config.hardware_type);
                    debug::info!("Inserted field hardware_id {:#?}", _mining_config_hardware_mining_hardware_config.hardware_id);
                    debug::info!("Inserted field hardware_dev_eui {:#?}", _mining_config_hardware_mining_hardware_config.hardware_dev_eui);
                    debug::info!("Inserted field hardware_lock_start_block {:#?}", _mining_config_hardware_mining_hardware_config.hardware_lock_start_block);
                    debug::info!("Inserted field hardware_lock_interval_blocks {:#?}", _mining_config_hardware_mining_hardware_config.hardware_lock_interval_blocks);
                }
            }

            Self::deposit_event(RawEvent::MiningConfigHardwareMiningHardwareConfigSet(
                sender,
                mining_config_hardware_mining_id,
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

impl<T: Trait> Module<T> {
    pub fn is_mining_config_hardware_mining_owner(
        mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_config_hardware_mining_owner(
                &mining_config_hardware_mining_id
            )
            .map(|owner| owner == sender)
            .unwrap_or(false),
            "Sender is not owner of Mining"
        );
        Ok(())
    }

    pub fn exists_mining_config_hardware_mining(
        mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
    ) -> Result<MiningConfigHardwareMining, DispatchError> {
        match Self::mining_config_hardware_mining(
            mining_config_hardware_mining_id,
        ) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningConfigHardwareMining does not exist")),
        }
    }

    pub fn exists_mining_config_hardware_mining_hardware_config(
        mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_config_hardware_mining_hardware_configs(
            mining_config_hardware_mining_id,
        ) {
            Some(_value) => Ok(()),
            None => {
                Err(DispatchError::Other("MiningConfigHardwareMiningHardwareConfig does not exist"))
            }
        }
    }

    pub fn has_value_for_mining_config_hardware_mining_hardware_config_index(
        mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_config_hardware_mining_hardware_config has a value that is defined"
        );
        let fetched_mining_config_hardware_mining_hardware_config =
            <MiningConfigHardwareMiningHardwareConfigs<T>>::get(
                mining_config_hardware_mining_id,
            );
        if let Some(_value) = fetched_mining_config_hardware_mining_hardware_config {
            debug::info!("Found value for mining_config_hardware_mining_hardware_config");
            return Ok(());
        }
        debug::info!("No value for mining_config_hardware_mining_hardware_config");
        Err(DispatchError::Other("No value for mining_config_hardware_mining_hardware_config"))
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

    fn next_mining_config_hardware_mining_id()
    -> Result<T::MiningConfigHardwareMiningIndex, DispatchError> {
        let mining_config_hardware_mining_id =
            Self::mining_config_hardware_mining_count();
        if mining_config_hardware_mining_id ==
            <T::MiningConfigHardwareMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningConfigHardwareMining count overflow"));
        }
        Ok(mining_config_hardware_mining_id)
    }

    fn insert_mining_config_hardware_mining(
        owner: &T::AccountId,
        mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
        mining_config_hardware_mining: MiningConfigHardwareMining,
    ) {
        // Create and store mining mining_config_hardware_mining
        <MiningConfigHardwareMinings<T>>::insert(
            mining_config_hardware_mining_id,
            mining_config_hardware_mining,
        );
        <MiningConfigHardwareMiningCount<T>>::put(
            mining_config_hardware_mining_id + One::one(),
        );
        <MiningConfigHardwareMiningOwners<T>>::insert(
            mining_config_hardware_mining_id,
            owner.clone(),
        );
    }

    fn update_owner(
        to: &T::AccountId,
        mining_config_hardware_mining_id: T::MiningConfigHardwareMiningIndex,
    ) {
        <MiningConfigHardwareMiningOwners<T>>::insert(
            mining_config_hardware_mining_id,
            to,
        );
    }
}
