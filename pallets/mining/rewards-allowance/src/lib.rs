#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use chrono::{
        NaiveDateTime,
    };
    use codec::{
        Decode,
        Encode,
    };
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*,
        traits::{
            Currency,
        }
    };
    use frame_system::pallet_prelude::*;
    use sp_std::{
        convert::{
            TryFrom,
            TryInto,
        },
        prelude::*, // Imports Vec
    };

    // type BalanceOf<T> = <T as pallet_balances::Config>::Balance;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type Date = i64;

    #[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "std", derive())]
    pub struct BondedDHXForAccountData<U, V, W> {
        pub account_id: U,
        pub bonded_dhx_current: V,
        pub requestor_account_id: W,
    }

    type BondedData<T> = BondedDHXForAccountData<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as frame_system::Config>::AccountId,
    >;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config
        + pallet_balances::Config
        + pallet_timestamp::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage
    // Learn more about declaring storage items:
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
    #[pallet::storage]
    #[pallet::getter(fn bonded_dhx_of_account_for_date)]
    pub(super) type BondedDHXForAccountForDate<T: Config> = StorageMap<_, Blake2_128Concat,
        Date,
        BondedData<T>
    >;

    #[pallet::storage]
    #[pallet::getter(fn rewards_allowance_dhx_for_date)]
    pub(super) type RewardsAllowanceDHXForDate<T: Config> = StorageMap<_, Blake2_128Concat,
        Date,
        BalanceOf<T>
    >;

    #[pallet::storage]
    #[pallet::getter(fn rewards_allowance_dhx_current)]
    pub(super) type RewardsAllowanceDHXCurrent<T: Config> = StorageValue<_, u128>;

    // The genesis config type.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub rewards_allowance_dhx_for_date: Vec<(Date, BalanceOf<T>)>,
        pub rewards_allowance_dhx_current: u128,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                rewards_allowance_dhx_for_date: Default::default(),
                // 5000 UNIT, where UNIT token has 18 decimal places
                rewards_allowance_dhx_current: 5_000_000_000_000_000_000_000u128,
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (a, b) in &self.rewards_allowance_dhx_for_date {
                <RewardsAllowanceDHXForDate<T>>::insert(a, b);
            }
            <RewardsAllowanceDHXCurrent<T>>::put(&self.rewards_allowance_dhx_current);
        }
    }

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::metadata(
        T::AccountId = "AccountId",
        BondedData<T> = "BondedData",
        BalanceOf<T> = "BalanceOf",
        T::AccountId = "Date"
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Storage of the bonded DHX of an account on a specific date by a requesting origin account.
        /// \[date, amount_dhx_bonded, account_dhx_bonded, sender\]
        SetBondedDHXOfAccountForDateStored(Date, BondedData<T>, T::AccountId, T::AccountId),

        /// Storage of the default reward allowance in DHX by an origin account.
        /// \[amount_dhx, sender\]
        SetRewardsAllowanceDHXCurrentStored(u128, T::AccountId),

        /// Storage of a new reward allowance in DHX for a specific date by an origin account.
        /// \[date, amount_dhx, sender\]
        SetRewardsAllowanceDHXForDateStored(Date, BalanceOf<T>, T::AccountId),

        /// Change the stored reward allowance in DHX for a specific date by an origin account, and
        /// where change is 0 for an decrease or any other value like 1 for an increase to the remaining
        /// rewards allowance.
        /// \[date, reduction_amount_dhx, sender, change\]
        ChangedRewardsAllowanceDHXForDateStored(Date, BalanceOf<T>, T::AccountId, u8),
    }

    // Errors inform users that something went wrong should be descriptive and have helpful documentation
    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        /// Preimage already noted
		DuplicatePreimage,
        StorageOverflow,
        StorageUnderflow,
    }

    // Pallet implements [`Hooks`] trait to define some logic to execute in some context.
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // `on_initialize` is executed at the beginning of the block before any extrinsic are
        // dispatched.
        //
        // This function must return the weight consumed by `on_initialize` and `on_finalize`.
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            // Anything that needs to be done at the start of the block.

            // In the genesis config we set the default value of StorageValue `RewardsAllowanceDHXCurrent`
            // to 5000 UNIT tokens, which would represent the total rewards to be distributed
            // in a year. Governance may choose to change that during the year or in subsequent years.
            //
            // At the start of each block after genesis, we check the current timestamp
            // (e.g. 27th August 2021 @ ~7am is 1630049371000), where milliseconds/day is 86400000,
            // and then determine the timestamp at the start of that day (e.g. 27th August 2021 @ 12am
            // is 1630022400000, since we want the timestamp of the start of the day to represent that
            // day, as we will store the rewards in UNIT tokens that are available for that day
            // as a value under that key using the `RewardsAllowanceDHXForDate` StorageMap if it does
            // not already exist (e.g. if it was not yet generated automatically in any blocks earlier
            // on that day, and not yet added manually by a user calling the `set_rewards_allowance_dhx_for_date`
            // extrinsic dispatchable function).
            //
            // Remaining rewards available for a given day that are stored under a key that is the
            // timestamp of that day may be modified by calling `reduce_remaining_rewards_allowance_dhx_for_date`.

            // Check if current date is in storage, otherwise add it.
            let current_date = <pallet_timestamp::Pallet<T>>::get();

            let requested_date_as_u64;
            let u64_in_millis = Self::convert_moment_to_u64_in_milliseconds(current_date.clone());
            match u64_in_millis {
                Err(_e) => {
                    log::error!("Unable to convert Moment to u64 in millis for current_date");
                    return 0;
                },
                Ok(ref x) => {
                    requested_date_as_u64 = x;
                }
            }
            log::info!("requested_date_as_u64: {:?}", requested_date_as_u64.clone());

            let requested_date_millis;
            let start_of_date = Self::convert_u64_in_milliseconds_to_start_of_date(requested_date_as_u64.clone());
            match start_of_date {
                Err(_e) => {
                    log::error!("Unable to convert u64 millis to start of date for current_date");
                    return 0;
                },
                Ok(ref x) => {
                    requested_date_millis = x;
                }
            }

            // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
            let contains_key = <RewardsAllowanceDHXForDate<T>>::contains_key(&requested_date_millis);
            if contains_key == false {
                let rewards_allowance_dhx_current_u128;
                let dhx_to_try = <RewardsAllowanceDHXCurrent<T>>::get();
                if let Some(_rewards_allowance_dhx_current_u128) = dhx_to_try {
                    rewards_allowance_dhx_current_u128 = _rewards_allowance_dhx_current_u128;
                } else {
                    log::error!("Unable to convert Moment to i64 for requested_date");
                    return 0;
                }

                let rewards_allowance;
                let _rewards_allowance = Self::convert_u128_to_balance(rewards_allowance_dhx_current_u128.clone());
                match _rewards_allowance {
                    Err(_e) => {
                        log::error!("Unable to convert u128 to balance for rewards_allowance");
                        return 0;
                    },
                    Ok(ref x) => {
                        rewards_allowance = x;
                    }
                }

                // Update storage. Use RewardsAllowanceDHXCurrent as fallback incase not previously set prior to block
                <RewardsAllowanceDHXForDate<T>>::insert(requested_date_millis.clone(), &rewards_allowance);
                log::info!("on_initialize");
                log::info!("requested_date_millis: {:?}", requested_date_millis.clone());
                log::info!("rewards_allowance: {:?}", &rewards_allowance);
            }

            return 0;
        }

        // `on_finalize` is executed at the end of block after all extrinsic are dispatched.
        fn on_finalize(_n: T::BlockNumber) {
            // Perform necessary data/state clean up here.
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // customised by governance at any time. this function allows us to change it each year
        // https://docs.google.com/spreadsheets/d/1W2AzOH9Cs9oCR8UYfYCbpmd9X7hp-USbYXL7AuwMY_Q/edit#gid=970997021
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_bonded_dhx_of_account_for_date(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Note: we DO need the following as we're using the current timestamp, rather than a function parameter.
            let timestamp: <T as pallet_timestamp::Config>::Moment = <pallet_timestamp::Pallet<T>>::get();
            let requested_date_as_u64 = Self::convert_moment_to_u64_in_milliseconds(timestamp.clone())?;
            log::info!("requested_date_as_u64: {:?}", requested_date_as_u64.clone());

            // convert the requested date/time to the start of that day date/time to signify that date for lookup
            // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
            let requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(requested_date_as_u64.clone())?;

            // TODO - fetch from democracy or elections
            let bonded_dhx_current_u128 = 1000u128;

            let bonded_dhx_current;
            let _bonded_dhx_current = Self::convert_u128_to_balance(bonded_dhx_current_u128.clone());
            match _bonded_dhx_current {
                Err(_e) => {
                    log::error!("Unable to convert u128 to balance for bonded_dhx_current");
                    return Err(DispatchError::Other("Unable to convert u128 to balance for bonded_dhx_current"));
                },
                Ok(ref x) => {
                    bonded_dhx_current = x;
                }
            }

            let bonded_data: BondedData<T> = BondedDHXForAccountData {
                account_id: account_id.clone(),
                bonded_dhx_current: bonded_dhx_current.clone(),
                requestor_account_id: _who.clone(),
            };

            // Update storage. Override the default that may have been set in on_initialize
            <BondedDHXForAccountForDate<T>>::insert(requested_date_millis.clone(), &bonded_data);
            log::info!("account_id: {:?}", &account_id);
            log::info!("bonded_data: {:?}", &bonded_data);

            // Emit an event.
            Self::deposit_event(Event::SetBondedDHXOfAccountForDateStored(
                requested_date_millis.clone(),
                bonded_data.clone(),
                account_id.clone(),
                _who.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // customised by governance at any time
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_allowance_dhx_current(origin: OriginFor<T>, rewards_allowance: BalanceOf<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let rewards_allowance_as_u128 = Self::convert_balance_to_u128(rewards_allowance.clone())?;

            // Update storage
            <RewardsAllowanceDHXCurrent<T>>::put(&rewards_allowance_as_u128);
            log::info!("rewards_allowance: {:?}", &rewards_allowance_as_u128);

            // Emit an event.
            Self::deposit_event(Event::SetRewardsAllowanceDHXCurrentStored(
                rewards_allowance_as_u128.clone(),
                _who.clone()
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // customised by governance at any time. this function allows us to change it each year
        // https://docs.google.com/spreadsheets/d/1W2AzOH9Cs9oCR8UYfYCbpmd9X7hp-USbYXL7AuwMY_Q/edit#gid=970997021
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_allowance_dhx_for_date(origin: OriginFor<T>, rewards_allowance: BalanceOf<T>, timestamp: u64) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Note: we do not need the following as we're not using the current timestamp, rather the function parameter.
            // let current_date = <pallet_timestamp::Pallet<T>>::get();
            // let requested_date_as_u64 = Self::convert_moment_to_u64_in_milliseconds(timestamp.clone())?;
            // log::info!("requested_date_as_u64: {:?}", requested_date_as_u64.clone());

            // Note: to get current timestamp `<pallet_timestamp::Pallet<T>>::get()`
            // convert the requested date/time to the start of that day date/time to signify that date for lookup
            // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
            let requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(timestamp.clone())?;

            // Update storage. Override the default that may have been set in on_initialize
            <RewardsAllowanceDHXForDate<T>>::insert(requested_date_millis.clone(), &rewards_allowance);
            log::info!("rewards_allowance: {:?}", &rewards_allowance);

            // Emit an event.
            Self::deposit_event(Event::SetRewardsAllowanceDHXForDateStored(
                requested_date_millis.clone(),
                rewards_allowance.clone(),
                _who.clone()
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // parameter `change: u8` value may be 0 or 1 (or any other value) to represent that we want to make a
        // corresponding decrease or increase to the remaining dhx rewards allowance for a given date.
        pub fn change_remaining_rewards_allowance_dhx_for_date(origin: OriginFor<T>, daily_rewards: BalanceOf<T>, timestamp: u64, change: u8) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(timestamp.clone())?;

            // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
            ensure!(<RewardsAllowanceDHXForDate<T>>::contains_key(&requested_date_millis), DispatchError::Other("Date key must exist to reduce allowance."));

            let existing_allowance_to_try = <RewardsAllowanceDHXForDate<T>>::get(&requested_date_millis);

            // Validate inputs so the daily_rewards is less or equal to the existing_allowance
            let existing_allowance_as_u128;
            if let Some(_existing_allowance_to_try) = existing_allowance_to_try.clone() {
                existing_allowance_as_u128 = Self::convert_balance_to_u128(_existing_allowance_to_try.clone())?;
                log::info!("existing_allowance_as_u128: {:?}", existing_allowance_as_u128.clone());
            } else {
                return Err(DispatchError::Other("Unable to retrieve balance from value provided"));
            }

            let daily_rewards_as_u128;
            daily_rewards_as_u128 = Self::convert_balance_to_u128(daily_rewards.clone())?;
            log::info!("daily_rewards_as_u128: {:?}", daily_rewards_as_u128.clone());

            ensure!(daily_rewards_as_u128 > 0u128, DispatchError::Other("Daily rewards must be greater than zero"));
            ensure!(existing_allowance_as_u128 >= daily_rewards_as_u128, DispatchError::Other("Daily rewards cannot exceed current remaining allowance"));

            let new_remaining_allowance_as_balance;
            if change == 0 {
                // Decrementing the value will error in the event of underflow.
                let new_remaining_allowance_as_u128 = existing_allowance_as_u128.checked_sub(daily_rewards_as_u128).ok_or(Error::<T>::StorageUnderflow)?;
                new_remaining_allowance_as_balance = Self::convert_u128_to_balance(new_remaining_allowance_as_u128.clone())?;
                log::info!("Decreasing rewards_allowance_dhx_for_date at Date: {:?}", &requested_date_millis);
            } else {
                // Incrementing the value will error in the event of overflow.
                let new_remaining_allowance_as_u128 = existing_allowance_as_u128.checked_add(daily_rewards_as_u128).ok_or(Error::<T>::StorageOverflow)?;
                new_remaining_allowance_as_balance = Self::convert_u128_to_balance(new_remaining_allowance_as_u128.clone())?;
                log::info!("Increasing rewards_allowance_dhx_for_date at Date: {:?}", &requested_date_millis);
            }

            // Update storage
            <RewardsAllowanceDHXForDate<T>>::mutate(
                &requested_date_millis,
                |allowance| {
                    if let Some(_allowance) = allowance {
                        *_allowance = new_remaining_allowance_as_balance.clone();
                    }
                },
            );

            // Emit an event.
            Self::deposit_event(Event::ChangedRewardsAllowanceDHXForDateStored(
                requested_date_millis.clone(),
                new_remaining_allowance_as_balance.clone(),
                _who.clone(),
                change.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }
    }

    // Private functions

    impl<T: Config> Pallet<T> {
        fn convert_moment_to_u64_in_milliseconds(date: T::Moment) -> Result<u64, DispatchError> {
            let date_as_u64_millis;
            if let Some(_date_as_u64) = TryInto::<u64>::try_into(date).ok() {
                date_as_u64_millis = _date_as_u64;
            } else {
                return Err(DispatchError::Other("Unable to convert Moment to i64 for date"));
            }
            return Ok(date_as_u64_millis);
        }

        fn convert_u64_in_milliseconds_to_start_of_date(date_as_u64_millis: u64) -> Result<Date, DispatchError> {
            let date_as_u64_secs = date_as_u64_millis.clone() / 1000u64;
            // https://docs.rs/chrono/0.4.6/chrono/naive/struct.NaiveDateTime.html#method.from_timestamp
            let date = NaiveDateTime::from_timestamp(i64::try_from(date_as_u64_secs).unwrap(), 0).date();
            log::info!("date_as_u64_secs: {:?}", date_as_u64_secs.clone());

            let date_start_millis = date.and_hms(0, 0, 0).timestamp() * 1000;
            log::info!("date_start_millis: {:?}", date_start_millis.clone());
            log::info!("Timestamp requested Date: {:?}", date);
            return Ok(date_start_millis);
        }

        fn convert_balance_to_u128(balance: BalanceOf<T>) -> Result<u128, DispatchError> {
            let balance_as_u128;

            if let Some(_balance_as_u128) = TryInto::<u128>::try_into(balance).ok() {
                balance_as_u128 = _balance_as_u128;
            } else {
                return Err(DispatchError::Other("Unable to convert Balance to u128 for balance"));
            }
            log::info!("balance_as_u128: {:?}", balance_as_u128.clone());

            return Ok(balance_as_u128);
        }

        fn convert_u128_to_balance(balance_as_u128: u128) -> Result<BalanceOf<T>, DispatchError> {
            let balance;

            if let Some(_balance) = TryInto::<BalanceOf<T>>::try_into(balance_as_u128).ok() {
                balance = _balance;
            } else {
                return Err(DispatchError::Other("Unable to convert u128 to Balance for balance"));
            }
            log::info!("balance: {:?}", balance.clone());

            return Ok(balance);
        }
    }
}
