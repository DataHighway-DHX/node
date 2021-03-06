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
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ExchangeRate(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct ExchangeRateConfig<H, D, I, F, P> {
    pub hbtc: H,
    pub dot: D,
    pub iota: I,
    pub fil: F,
    pub decimals_after_point: P,
}

pub trait Trait: frame_system::Trait + roaming_operators::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type ExchangeRateIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type HBTCRate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type DOTRate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type IOTARate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type FILRate: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type DecimalsAfterPoint: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::ExchangeRateIndex,
        <T as Trait>::HBTCRate,
        <T as Trait>::DOTRate,
        <T as Trait>::IOTARate,
        <T as Trait>::FILRate,
        <T as Trait>::DecimalsAfterPoint,
    {
        /// A exchange_rate is created. (owner, exchange_rate_index)
        Created(AccountId, ExchangeRateIndex),
        /// A exchange_rate is transferred. (from, to, exchange_rate_index)
        Transferred(AccountId, AccountId, ExchangeRateIndex),
        ConfigSet(
            AccountId, ExchangeRateIndex, HBTCRate,
            DOTRate, IOTARate,
            FILRate, DecimalsAfterPoint
        ),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as ExchangeRate {
        pub ExchangeRates get(fn exchange_rates): map hasher(opaque_blake2_256) T::ExchangeRateIndex => Option<ExchangeRate>;
        pub ExchangeRateOwners get(fn exchange_rate_owner): map hasher(opaque_blake2_256) T::ExchangeRateIndex => Option<T::AccountId>;
        pub ExchangeRateCount get(fn exchange_rate_count): T::ExchangeRateIndex;
        pub ExchangeRateConfigs get(fn exchange_rate_configs): map hasher(opaque_blake2_256) T::ExchangeRateIndex =>
            Option<ExchangeRateConfig<T::HBTCRate, T::DOTRate, T::IOTARate, T::FILRate, T::DecimalsAfterPoint>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().writes(3)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let exchange_rate_id = Self::next_exchange_rate_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            let exchange_rate = ExchangeRate(unique_id);

            <ExchangeRates<T>>::insert(
                exchange_rate_id,
                exchange_rate,
            );
            <ExchangeRateCount<T>>::put(exchange_rate_id + One::one());
            <ExchangeRateOwners<T>>::insert(exchange_rate_id, &sender.clone());

            Self::deposit_event(RawEvent::Created(sender, exchange_rate_id));
        }

        /// Transfer a exchange_rate to new owner
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn transfer(origin, to: T::AccountId, exchange_rate_id: T::ExchangeRateIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::exchange_rate_owner(exchange_rate_id) == Some(sender.clone()), "Only owner can transfer exchange_rate");

            Self::update_owner(&to, exchange_rate_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, exchange_rate_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_config(
            origin,
            exchange_rate_id: T::ExchangeRateIndex,
            hbtc_rate: Option<T::HBTCRate>,
            dot_rate: Option<T::DOTRate>,
            iota_rate: Option<T::IOTARate>,
            fil_rate: Option<T::FILRate>,
            decimals_after_point: Option<T::DecimalsAfterPoint>
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the exchange_rate_id whose config we want to change actually exists
            let is_exchange_rate = Self::exists_exchange_rates(exchange_rate_id).is_ok();
            ensure!(is_exchange_rate, "ExchangeRates does not exist");

            // Ensure that the caller is owner of the exchange_rate they are trying to change
            ensure!(Self::exchange_rate_owner(exchange_rate_id) == Some(sender.clone()), "Only owner can set exchange_rate_config");

            let out_hbtc_rate = match hbtc_rate.clone() {
                Some(value) => value,
                None => 200000u32.into() // Default
            };

            let out_dot_rate = match dot_rate.clone() {
                Some(value) => value,
                None => 100u32.into() // Default
            };

            let out_iota_rate = match iota_rate.clone() {
                Some(value) => value,
                None => 5u32.into() // Default
            };

            let out_fil_rate = match fil_rate.clone() {
                Some(value) => value,
                None => 200u32.into() // Default
            };

            let out_decimals_after_point = match decimals_after_point.clone() {
                Some(value) => value,
                None => 2u32.into() // Default
            };

            // Check if a exchange_rate_config already exists with the given exchange_rate_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_exchange_rates_index(exchange_rate_id).is_ok() {
                debug::info!("Mutating values");
                <ExchangeRateConfigs<T>>::mutate(exchange_rate_id, |exchange_rate_config| {
                    if let Some(_exchange_rate_config) = exchange_rate_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _exchange_rate_config.hbtc = out_hbtc_rate.clone();
                        _exchange_rate_config.dot = out_dot_rate.clone();
                        _exchange_rate_config.iota = out_iota_rate.clone();
                        _exchange_rate_config.fil = out_fil_rate.clone();
                        _exchange_rate_config.decimals_after_point = out_decimals_after_point.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_exchange_rate_config = <ExchangeRateConfigs<T>>::get(exchange_rate_id);
                if let Some(_exchange_rate_config) = fetched_exchange_rate_config {
                    debug::info!("Latest field hbtc {:#?}", _exchange_rate_config.hbtc);
                    debug::info!("Latest field dot {:#?}", _exchange_rate_config.dot);
                    debug::info!("Latest field iota {:#?}", _exchange_rate_config.iota);
                    debug::info!("Latest field fil {:#?}", _exchange_rate_config.fil);
                    debug::info!("Latest field decimals_after_point {:#?}", _exchange_rate_config.decimals_after_point);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining exchange_rate_config instance with the input params
                let exchange_rate_config = ExchangeRateConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    hbtc: out_hbtc_rate.clone(),
                    dot: out_dot_rate.clone(),
                    iota: out_iota_rate.clone(),
                    fil: out_fil_rate.clone(),
                    decimals_after_point: out_decimals_after_point.clone(),
                };

                <ExchangeRateConfigs<T>>::insert(
                    exchange_rate_id,
                    &exchange_rate_config
                );

                debug::info!("Checking inserted values");
                let fetched_exchange_rate_config = <ExchangeRateConfigs<T>>::get(exchange_rate_id);
                if let Some(_exchange_rate_config) = fetched_exchange_rate_config {
                    debug::info!("Latest field hbtc {:#?}", _exchange_rate_config.hbtc);
                    debug::info!("Latest field dot {:#?}", _exchange_rate_config.dot);
                    debug::info!("Latest field iota {:#?}", _exchange_rate_config.iota);
                    debug::info!("Latest field fil {:#?}", _exchange_rate_config.fil);
                    debug::info!("Latest field decimals_after_point {:#?}", _exchange_rate_config.decimals_after_point);
                }
            }

            Self::deposit_event(RawEvent::ConfigSet(
                sender,
                exchange_rate_id,
                out_hbtc_rate,
                out_dot_rate,
                out_iota_rate,
                out_fil_rate,
                out_decimals_after_point,
            ));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_exchange_rate_owner(
        exchange_rate_id: T::ExchangeRateIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::exchange_rate_owner(&exchange_rate_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of ExchangeRate"
        );
        Ok(())
    }

    pub fn exists_exchange_rates(exchange_rate_id: T::ExchangeRateIndex) -> Result<ExchangeRate, DispatchError> {
        match Self::exchange_rates(exchange_rate_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("ExchangeRates does not exist")),
        }
    }

    pub fn has_value_for_exchange_rates_index(exchange_rate_id: T::ExchangeRateIndex) -> Result<(), DispatchError> {
        debug::info!("Checking if exchange_rate_config has a value that is defined");
        let fetched_exchange_rate_config = <ExchangeRateConfigs<T>>::get(exchange_rate_id);
        if let Some(_value) = fetched_exchange_rate_config {
            debug::info!("Found value for exchange_rate_config");
            return Ok(());
        }
        debug::info!("No value for exchange_rate_config");
        Err(DispatchError::Other("No value for exchange_rate_config"))
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            <T as roaming_operators::Trait>::Randomness::random(&[0]),
            sender,
            <frame_system::Module<T>>::extrinsic_index(),
            <frame_system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_exchange_rate_id() -> Result<T::ExchangeRateIndex, DispatchError> {
        let exchange_rate_id = Self::exchange_rate_count();
        if exchange_rate_id == <T::ExchangeRateIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("ExchangeRate count overflow"));
        }
        Ok(exchange_rate_id)
    }

    fn update_owner(to: &T::AccountId, exchange_rate_id: T::ExchangeRateIndex) {
        <ExchangeRateOwners<T>>::insert(exchange_rate_id, to);
    }
}
