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
use frame_system::{self as system, ensure_signed};
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
    type MiningSpeedBoostRatesTokenMiningIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesTokenMiningTokenMXC: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesTokenMiningTokenIOTA: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesTokenMiningTokenDOT: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesTokenMiningMaxToken: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningSpeedBoostRatesTokenMiningMaxLoyalty: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostRatesTokenMining(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostRatesTokenMiningRatesConfig<U, V, W, X, Y> {
    pub token_token_mxc: U,
    pub token_token_iota: V,
    pub token_token_dot: W,
    pub token_max_token: X,
    pub token_max_loyalty: Y,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostRatesTokenMiningIndex,
        <T as Trait>::MiningSpeedBoostRatesTokenMiningTokenMXC,
        <T as Trait>::MiningSpeedBoostRatesTokenMiningTokenIOTA,
        <T as Trait>::MiningSpeedBoostRatesTokenMiningTokenDOT,
        <T as Trait>::MiningSpeedBoostRatesTokenMiningMaxToken,
        <T as Trait>::MiningSpeedBoostRatesTokenMiningMaxLoyalty,
        // Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_rates_token_mining is created. (owner, mining_speed_boosts_rates_token_mining_id)
        Created(AccountId, MiningSpeedBoostRatesTokenMiningIndex),
        /// A mining_speed_boosts_rates_token_mining is transferred. (from, to, mining_speed_boosts_rates_token_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostRatesTokenMiningIndex),
        MiningSpeedBoostRatesTokenMiningRatesConfigSet(
            AccountId, MiningSpeedBoostRatesTokenMiningIndex, MiningSpeedBoostRatesTokenMiningTokenMXC,
            MiningSpeedBoostRatesTokenMiningTokenIOTA, MiningSpeedBoostRatesTokenMiningTokenDOT,
            MiningSpeedBoostRatesTokenMiningMaxToken, MiningSpeedBoostRatesTokenMiningMaxLoyalty
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostRatesTokenMining {
        /// Stores all the mining_speed_boosts_rates_token_minings, key is the mining_speed_boosts_rates_token_mining id / index
        pub MiningSpeedBoostRatesTokenMinings get(fn mining_speed_boosts_rates_token_mining): map hasher(blake2_256) T::MiningSpeedBoostRatesTokenMiningIndex => Option<MiningSpeedBoostRatesTokenMining>;

        /// Stores the total number of mining_speed_boosts_rates_token_minings. i.e. the next mining_speed_boosts_rates_token_mining index
        pub MiningSpeedBoostRatesTokenMiningCount get(fn mining_speed_boosts_rates_token_mining_count): T::MiningSpeedBoostRatesTokenMiningIndex;

        /// Stores mining_speed_boosts_rates_token_mining owner
        pub MiningSpeedBoostRatesTokenMiningOwners get(fn mining_speed_boosts_rates_token_mining_owner): map hasher(blake2_256) T::MiningSpeedBoostRatesTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_rates_token_mining_rates_config
        pub MiningSpeedBoostRatesTokenMiningRatesConfigs get(fn mining_speed_boosts_rates_token_mining_rates_configs): map hasher(blake2_256) T::MiningSpeedBoostRatesTokenMiningIndex =>
            Option<MiningSpeedBoostRatesTokenMiningRatesConfig<T::MiningSpeedBoostRatesTokenMiningTokenMXC, T::MiningSpeedBoostRatesTokenMiningTokenIOTA,
            T::MiningSpeedBoostRatesTokenMiningTokenDOT, T::MiningSpeedBoostRatesTokenMiningMaxToken, T::MiningSpeedBoostRatesTokenMiningMaxLoyalty>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_rates_token_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_rates_token_mining_id = Self::next_mining_speed_boosts_rates_token_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_rates_token_mining
            let mining_speed_boosts_rates_token_mining = MiningSpeedBoostRatesTokenMining(unique_id);
            Self::insert_mining_speed_boosts_rates_token_mining(&sender, mining_speed_boosts_rates_token_mining_id, mining_speed_boosts_rates_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_rates_token_mining_id));
        }

        /// Transfer a mining_speed_boosts_rates_token_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_rates_token_mining_id: T::MiningSpeedBoostRatesTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_rates_token_mining_owner(mining_speed_boosts_rates_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_rates_token_mining");

            Self::update_owner(&to, mining_speed_boosts_rates_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_rates_token_mining_id));
        }

        /// Set mining_speed_boosts_rates_token_mining_rates_config
        pub fn set_mining_speed_boosts_rates_token_mining_rates_config(
            origin,
            mining_speed_boosts_rates_token_mining_id: T::MiningSpeedBoostRatesTokenMiningIndex,
            _token_token_mxc: Option<T::MiningSpeedBoostRatesTokenMiningTokenMXC>,
            _token_token_iota: Option<T::MiningSpeedBoostRatesTokenMiningTokenIOTA>,
            _token_token_dot: Option<T::MiningSpeedBoostRatesTokenMiningTokenDOT>,
            _token_max_token: Option<T::MiningSpeedBoostRatesTokenMiningMaxToken>,
            _token_max_loyalty: Option<T::MiningSpeedBoostRatesTokenMiningMaxLoyalty>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_rates_token_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_rates_token_mining = Self::exists_mining_speed_boosts_rates_token_mining(mining_speed_boosts_rates_token_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_rates_token_mining, "MiningSpeedBoostRatesTokenMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_rates_token_mining_rates_config they are trying to change
            ensure!(Self::mining_speed_boosts_rates_token_mining_owner(mining_speed_boosts_rates_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_rates_token_mining_rates_config");

            // TODO - adjust default rates
            let token_token_mxc = match _token_token_mxc.clone() {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_token_iota = match _token_token_iota {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_token_dot = match _token_token_dot {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_max_token = match _token_max_token {
                Some(value) => value,
                None => 1.into() // Default
            };
            let token_max_loyalty = match _token_max_loyalty {
                Some(value) => value,
                None => 1.into() // Default
            };

            // FIXME - how to use float and overcome error:
            //  the trait `std::str::FromStr` is not implemented for `<T as Trait>::MiningSpeedBoostRatesTokenMiningMaxToken
            // if token_token_mxc > "1.2".parse().unwrap() || token_token_iota > "1.2".parse().unwrap() || token_token_dot > "1.2".parse().unwrap() || token_max_token > "1.6".parse().unwrap() || token_max_loyalty > "1.2".parse().unwrap() {
            //   debug::info!("Token rate cannot be this large");

            //   return Ok(());
            // }

            // Check if a mining_speed_boosts_rates_token_mining_rates_config already exists with the given mining_speed_boosts_rates_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_rates_token_mining_rates_config_index(mining_speed_boosts_rates_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostRatesTokenMiningRatesConfigs<T>>::mutate(mining_speed_boosts_rates_token_mining_id, |mining_speed_boosts_rates_token_mining_rates_config| {
                    if let Some(_mining_speed_boosts_rates_token_mining_rates_config) = mining_speed_boosts_rates_token_mining_rates_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_rates_token_mining_rates_config.token_token_mxc = token_token_mxc.clone();
                        _mining_speed_boosts_rates_token_mining_rates_config.token_token_iota = token_token_iota.clone();
                        _mining_speed_boosts_rates_token_mining_rates_config.token_token_dot = token_token_dot.clone();
                        _mining_speed_boosts_rates_token_mining_rates_config.token_max_token = token_max_token.clone();
                        _mining_speed_boosts_rates_token_mining_rates_config.token_max_loyalty = token_max_loyalty.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_rates_token_mining_rates_config = <MiningSpeedBoostRatesTokenMiningRatesConfigs<T>>::get(mining_speed_boosts_rates_token_mining_id);
                if let Some(_mining_speed_boosts_rates_token_mining_rates_config) = fetched_mining_speed_boosts_rates_token_mining_rates_config {
                    debug::info!("Latest field token_token_mxc {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_token_mxc);
                    debug::info!("Latest field token_token_iota {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_token_iota);
                    debug::info!("Latest field token_token_dot {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_token_dot);
                    debug::info!("Latest field token_max_token {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_max_token);
                    debug::info!("Latest field token_max_loyalty {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_max_loyalty);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_rates_token_mining_rates_config instance with the input params
                let mining_speed_boosts_rates_token_mining_rates_config_instance = MiningSpeedBoostRatesTokenMiningRatesConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_token_mxc: token_token_mxc.clone(),
                    token_token_iota: token_token_iota.clone(),
                    token_token_dot: token_token_dot.clone(),
                    token_max_token: token_max_token.clone(),
                    token_max_loyalty: token_max_loyalty.clone(),
                };

                <MiningSpeedBoostRatesTokenMiningRatesConfigs<T>>::insert(
                    mining_speed_boosts_rates_token_mining_id,
                    &mining_speed_boosts_rates_token_mining_rates_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_rates_token_mining_rates_config = <MiningSpeedBoostRatesTokenMiningRatesConfigs<T>>::get(mining_speed_boosts_rates_token_mining_id);
                if let Some(_mining_speed_boosts_rates_token_mining_rates_config) = fetched_mining_speed_boosts_rates_token_mining_rates_config {
                    debug::info!("Inserted field token_token_mxc {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_token_mxc);
                    debug::info!("Inserted field token_token_iota {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_token_iota);
                    debug::info!("Inserted field token_token_dot {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_token_dot);
                    debug::info!("Inserted field token_max_token {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_max_token);
                    debug::info!("Inserted field token_max_loyalty {:#?}", _mining_speed_boosts_rates_token_mining_rates_config.token_max_loyalty);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostRatesTokenMiningRatesConfigSet(
                sender,
                mining_speed_boosts_rates_token_mining_id,
                token_token_mxc,
                token_token_iota,
                token_token_dot,
                token_max_token,
                token_max_loyalty,
            ));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_mining_speed_boosts_rates_token_mining_owner(
        mining_speed_boosts_rates_token_mining_id: T::MiningSpeedBoostRatesTokenMiningIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_speed_boosts_rates_token_mining_owner(&mining_speed_boosts_rates_token_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoostRatesTokenMining"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_rates_token_mining(
        mining_speed_boosts_rates_token_mining_id: T::MiningSpeedBoostRatesTokenMiningIndex,
    ) -> Result<MiningSpeedBoostRatesTokenMining, DispatchError> {
        match Self::mining_speed_boosts_rates_token_mining(mining_speed_boosts_rates_token_mining_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningSpeedBoostRatesTokenMining does not exist")),
        }
    }

    pub fn exists_mining_speed_boosts_rates_token_mining_rates_config(
        mining_speed_boosts_rates_token_mining_id: T::MiningSpeedBoostRatesTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_speed_boosts_rates_token_mining_rates_configs(mining_speed_boosts_rates_token_mining_id) {
            Some(value) => Ok(()),
            None => Err(DispatchError::Other("MiningSpeedBoostRatesTokenMiningRatesConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_speed_boosts_rates_token_mining_rates_config_index(
        mining_speed_boosts_rates_token_mining_id: T::MiningSpeedBoostRatesTokenMiningIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_speed_boosts_rates_token_mining_rates_config has a value that is defined");
        let fetched_mining_speed_boosts_rates_token_mining_rates_config =
            <MiningSpeedBoostRatesTokenMiningRatesConfigs<T>>::get(mining_speed_boosts_rates_token_mining_id);
        if let Some(value) = fetched_mining_speed_boosts_rates_token_mining_rates_config {
            debug::info!("Found value for mining_speed_boosts_rates_token_mining_rates_config");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_rates_token_mining_rates_config");
        Err(DispatchError::Other("No value for mining_speed_boosts_rates_token_mining_rates_config"))
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

    fn next_mining_speed_boosts_rates_token_mining_id()
    -> Result<T::MiningSpeedBoostRatesTokenMiningIndex, DispatchError> {
        let mining_speed_boosts_rates_token_mining_id = Self::mining_speed_boosts_rates_token_mining_count();
        if mining_speed_boosts_rates_token_mining_id ==
            <T::MiningSpeedBoostRatesTokenMiningIndex as Bounded>::max_value()
        {
            return Err(DispatchError::Other("MiningSpeedBoostRatesTokenMining count overflow"));
        }
        Ok(mining_speed_boosts_rates_token_mining_id)
    }

    fn insert_mining_speed_boosts_rates_token_mining(
        owner: &T::AccountId,
        mining_speed_boosts_rates_token_mining_id: T::MiningSpeedBoostRatesTokenMiningIndex,
        mining_speed_boosts_rates_token_mining: MiningSpeedBoostRatesTokenMining,
    ) {
        // Create and store mining mining_speed_boosts_rates_token_mining
        <MiningSpeedBoostRatesTokenMinings<T>>::insert(
            mining_speed_boosts_rates_token_mining_id,
            mining_speed_boosts_rates_token_mining,
        );
        <MiningSpeedBoostRatesTokenMiningCount<T>>::put(mining_speed_boosts_rates_token_mining_id + One::one());
        <MiningSpeedBoostRatesTokenMiningOwners<T>>::insert(mining_speed_boosts_rates_token_mining_id, owner.clone());
    }

    fn update_owner(
        to: &T::AccountId,
        mining_speed_boosts_rates_token_mining_id: T::MiningSpeedBoostRatesTokenMiningIndex,
    ) {
        <MiningSpeedBoostRatesTokenMiningOwners<T>>::insert(mining_speed_boosts_rates_token_mining_id, to);
    }
}
