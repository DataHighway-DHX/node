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
pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    type MiningRatesTokenIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy + From<u64> + Into<u64>;
    type MiningRatesTokenTokenMXC: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningRatesTokenTokenIOTA: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningRatesTokenTokenDOT: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningRatesTokenMaxToken: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningRatesTokenMaxLoyalty: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningRatesToken(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningRatesTokenConfig<U, V, W, X, Y> {
    pub token_token_mxc: U,
    pub token_token_iota: V,
    pub token_token_dot: W,
    pub token_max_token: X,
    pub token_max_loyalty: Y,
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningRatesTokenIndex,
        <T as Trait>::MiningRatesTokenTokenMXC,
        <T as Trait>::MiningRatesTokenTokenIOTA,
        <T as Trait>::MiningRatesTokenTokenDOT,
        <T as Trait>::MiningRatesTokenMaxToken,
        <T as Trait>::MiningRatesTokenMaxLoyalty,
        // Balance = BalanceOf<T>,
    {
        /// A mining_rates_token is created. (owner, mining_rates_token_id)
        Created(AccountId, MiningRatesTokenIndex),
        /// A mining_rates_token is transferred. (from, to, mining_rates_token_id)
        Transferred(AccountId, AccountId, MiningRatesTokenIndex),
        MiningRatesTokenConfigSet(
            AccountId, MiningRatesTokenIndex, MiningRatesTokenTokenMXC,
            MiningRatesTokenTokenIOTA, MiningRatesTokenTokenDOT,
            MiningRatesTokenMaxToken, MiningRatesTokenMaxLoyalty
        ),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningRatesToken {
        /// Stores all the mining_rates_tokens, key is the mining_rates_token id / index
        pub MiningRatesTokens get(fn mining_rates_token): map hasher(opaque_blake2_256) T::MiningRatesTokenIndex => Option<MiningRatesToken>;

        /// Stores the total number of mining_rates_tokens. i.e. the next mining_rates_token index
        pub MiningRatesTokenCount get(fn mining_rates_token_count): T::MiningRatesTokenIndex;

        /// Stores mining_rates_token owner
        pub MiningRatesTokenOwners get(fn mining_rates_token_owner): map hasher(opaque_blake2_256) T::MiningRatesTokenIndex => Option<T::AccountId>;

        /// Stores mining_rates_token_rates_config
        pub MiningRatesTokenConfigs get(fn mining_rates_token_rates_configs): map hasher(opaque_blake2_256) T::MiningRatesTokenIndex =>
            Option<MiningRatesTokenConfig<T::MiningRatesTokenTokenMXC, T::MiningRatesTokenTokenIOTA,
            T::MiningRatesTokenTokenDOT, T::MiningRatesTokenMaxToken, T::MiningRatesTokenMaxLoyalty>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_rates_token
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_rates_token_id = Self::next_mining_rates_token_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_rates_token
            let mining_rates_token = MiningRatesToken(unique_id);
            Self::insert_mining_rates_token(&sender, mining_rates_token_id, mining_rates_token);

            Self::deposit_event(RawEvent::Created(sender, mining_rates_token_id));
        }

        /// Transfer a mining_rates_token to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, mining_rates_token_id: T::MiningRatesTokenIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_rates_token_owner(mining_rates_token_id) == Some(sender.clone()), "Only owner can transfer mining mining_rates_token");

            Self::update_owner(&to, mining_rates_token_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_rates_token_id));
        }

        /// Set mining_rates_token_rates_config
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_mining_rates_token_rates_config(
            origin,
            mining_rates_token_id: T::MiningRatesTokenIndex,
            _token_token_mxc: Option<T::MiningRatesTokenTokenMXC>,
            _token_token_iota: Option<T::MiningRatesTokenTokenIOTA>,
            _token_token_dot: Option<T::MiningRatesTokenTokenDOT>,
            _token_max_token: Option<T::MiningRatesTokenMaxToken>,
            _token_max_loyalty: Option<T::MiningRatesTokenMaxLoyalty>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_rates_token_id whose config we want to change actually exists
            let is_mining_rates_token = Self::exists_mining_rates_token(mining_rates_token_id).is_ok();
            ensure!(is_mining_rates_token, "MiningRatesToken does not exist");

            // Ensure that the caller is owner of the mining_rates_token_rates_config they are trying to change
            ensure!(Self::mining_rates_token_owner(mining_rates_token_id) == Some(sender.clone()), "Only owner can set mining_rates_token_rates_config");

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
            //  the trait `std::str::FromStr` is not implemented for `<T as Trait>::MiningRatesTokenMaxToken
            // if token_token_mxc > "1.2".parse().unwrap() || token_token_iota > "1.2".parse().unwrap() || token_token_dot > "1.2".parse().unwrap() || token_max_token > "1.6".parse().unwrap() || token_max_loyalty > "1.2".parse().unwrap() {
            //   debug::info!("Token rate cannot be this large");

            //   return Ok(());
            // }

            // Check if a mining_rates_token_rates_config already exists with the given mining_rates_token_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_rates_token_rates_config_index(mining_rates_token_id).is_ok() {
                debug::info!("Mutating values");
                <MiningRatesTokenConfigs<T>>::mutate(mining_rates_token_id, |mining_rates_token_rates_config| {
                    if let Some(_mining_rates_token_rates_config) = mining_rates_token_rates_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_rates_token_rates_config.token_token_mxc = token_token_mxc.clone();
                        _mining_rates_token_rates_config.token_token_iota = token_token_iota.clone();
                        _mining_rates_token_rates_config.token_token_dot = token_token_dot.clone();
                        _mining_rates_token_rates_config.token_max_token = token_max_token.clone();
                        _mining_rates_token_rates_config.token_max_loyalty = token_max_loyalty.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_rates_token_rates_config = <MiningRatesTokenConfigs<T>>::get(mining_rates_token_id);
                if let Some(_mining_rates_token_rates_config) = fetched_mining_rates_token_rates_config {
                    debug::info!("Latest field token_token_mxc {:#?}", _mining_rates_token_rates_config.token_token_mxc);
                    debug::info!("Latest field token_token_iota {:#?}", _mining_rates_token_rates_config.token_token_iota);
                    debug::info!("Latest field token_token_dot {:#?}", _mining_rates_token_rates_config.token_token_dot);
                    debug::info!("Latest field token_max_token {:#?}", _mining_rates_token_rates_config.token_max_token);
                    debug::info!("Latest field token_max_loyalty {:#?}", _mining_rates_token_rates_config.token_max_loyalty);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_rates_token_rates_config instance with the input params
                let mining_rates_token_rates_config_instance = MiningRatesTokenConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_token_mxc: token_token_mxc.clone(),
                    token_token_iota: token_token_iota.clone(),
                    token_token_dot: token_token_dot.clone(),
                    token_max_token: token_max_token.clone(),
                    token_max_loyalty: token_max_loyalty.clone(),
                };

                <MiningRatesTokenConfigs<T>>::insert(
                    mining_rates_token_id,
                    &mining_rates_token_rates_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_rates_token_rates_config = <MiningRatesTokenConfigs<T>>::get(mining_rates_token_id);
                if let Some(_mining_rates_token_rates_config) = fetched_mining_rates_token_rates_config {
                    debug::info!("Inserted field token_token_mxc {:#?}", _mining_rates_token_rates_config.token_token_mxc);
                    debug::info!("Inserted field token_token_iota {:#?}", _mining_rates_token_rates_config.token_token_iota);
                    debug::info!("Inserted field token_token_dot {:#?}", _mining_rates_token_rates_config.token_token_dot);
                    debug::info!("Inserted field token_max_token {:#?}", _mining_rates_token_rates_config.token_max_token);
                    debug::info!("Inserted field token_max_loyalty {:#?}", _mining_rates_token_rates_config.token_max_loyalty);
                }
            }

            Self::deposit_event(RawEvent::MiningRatesTokenConfigSet(
                sender,
                mining_rates_token_id,
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
    pub fn is_mining_rates_token_owner(
        mining_rates_token_id: T::MiningRatesTokenIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_rates_token_owner(&mining_rates_token_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of MiningRatesToken"
        );
        Ok(())
    }

    pub fn exists_mining_rates_token(
        mining_rates_token_id: T::MiningRatesTokenIndex,
    ) -> Result<MiningRatesToken, DispatchError> {
        match Self::mining_rates_token(mining_rates_token_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningRatesToken does not exist")),
        }
    }

    pub fn exists_mining_rates_token_rates_config(
        mining_rates_token_id: T::MiningRatesTokenIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_rates_token_rates_configs(mining_rates_token_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningRatesTokenConfig does not exist")),
        }
    }

    pub fn has_value_for_mining_rates_token_rates_config_index(
        mining_rates_token_id: T::MiningRatesTokenIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_rates_token_rates_config has a value that is defined");
        let fetched_mining_rates_token_rates_config = <MiningRatesTokenConfigs<T>>::get(mining_rates_token_id);
        if let Some(_value) = fetched_mining_rates_token_rates_config {
            debug::info!("Found value for mining_rates_token_rates_config");
            return Ok(());
        }
        debug::info!("No value for mining_rates_token_rates_config");
        Err(DispatchError::Other("No value for mining_rates_token_rates_config"))
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

    fn next_mining_rates_token_id() -> Result<T::MiningRatesTokenIndex, DispatchError> {
        let mining_rates_token_id = Self::mining_rates_token_count();
        if mining_rates_token_id == <T::MiningRatesTokenIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningRatesToken count overflow"));
        }
        Ok(mining_rates_token_id)
    }

    fn insert_mining_rates_token(
        owner: &T::AccountId,
        mining_rates_token_id: T::MiningRatesTokenIndex,
        mining_rates_token: MiningRatesToken,
    ) {
        // Create and store mining mining_rates_token
        <MiningRatesTokens<T>>::insert(mining_rates_token_id, mining_rates_token);
        <MiningRatesTokenCount<T>>::put(mining_rates_token_id + One::one());
        <MiningRatesTokenOwners<T>>::insert(mining_rates_token_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_rates_token_id: T::MiningRatesTokenIndex) {
        <MiningRatesTokenOwners<T>>::insert(mining_rates_token_id, to);
    }
}
