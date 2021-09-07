#![cfg_attr(not(feature = "std"), no_std)]

use log::{warn, info};
use chrono::{
    NaiveDate,
    NaiveDateTime,
    Duration,
};
use codec::{
    Decode,
    Encode,
};
use frame_support::{
    decl_error,
    decl_event,
    decl_module,
    decl_storage,
    ensure,
    traits::{
        Currency,
        ExistenceRequirement,
        Get,
        Randomness,
    },
    Parameter,
};
use frame_system::{
    ensure_signed,
    ensure_root,
};
// use module_primitives::{
//     constants::time::MILLISECS_PER_BLOCK,
//     types::*,
// };
use sp_io::hashing::blake2_128;
use sp_runtime::{
    print,
    traits::{
        AtLeast32Bit,
        Bounded,
        CheckedAdd,
        Member,
        One,
        Printable,
    },
    DispatchError,
};
use sp_std::{
    convert::{
        TryFrom,
        TryInto,
    },
    prelude::*, // Imports Vec
};

pub use mining_rewards;

/// The module's configuration trait.
pub trait Config:
    frame_system::Config
    + pallet_balances::Config
    + pallet_timestamp::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
}

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

decl_event!(
    pub enum Event<T> where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        Created(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NoneValue,
        /// Some math operation overflowed
        Overflow,
    }
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningRewards {
        pub BondedDHXForAccountForDate get(fn bonded_dhx_of_account_for_date) config(): map hasher(opaque_blake2_256) Date => Option<Vec<BondedData<T>>>;

        pub MiningRewardsDHXForDate get(fn mining_rewards_dhx_for_date) config(): map hasher(opaque_blake2_256) Date => BalanceOf<T>;

        pub MiningRewardsDHXCurrent get(fn mining_rewards_dhx_current) config(): u128;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
        fn deposit_event() = default;

		// // `on_initialize` is executed at the beginning of the block before any extrinsic are
		// // dispatched.
		// //
		// // This function must return the weight consumed by `on_initialize` and `on_finalize`.
		// fn on_initialize(_n: T::BlockNumber) -> Weight {
		// 	// Anything that needs to be done at the start of the block.

		// 	// In the genesis config we set the default value of StorageValue `MiningRewardsDHXCurrent`
		// 	// to 5000 UNIT tokens, which would represent the total rewards to be distributed
		// 	// in a year. Governance may choose to change that during the year or in subsequent years.
		// 	//
		// 	// At the start of each block after genesis, we check the current timestamp
		// 	// (e.g. 27th August 2021 @ ~7am is 1630049371000), where milliseconds/day is 86400000,
		// 	// and then determine the timestamp at the start of that day (e.g. 27th August 2021 @ 12am
		// 	// is 1630022400000, since we want the timestamp of the start of the day to represent that
		// 	// day, as we will store the rewards in UNIT tokens that are available for that day
		// 	// as a value under that key using the `MiningRewardsDHXForDate` StorageMap if it does
		// 	// not already exist (e.g. if it was not yet generated automatically in any blocks earlier
		// 	// on that day, and not yet added manually by a user calling the `set_mining_rewards_dhx_for_date`
		// 	// extrinsic dispatchable function).
		// 	//
		// 	// Remaining rewards available for a given day that are stored under a key that is the
		// 	// timestamp of that day may be modified by calling `reduce_remaining_mining_rewards_dhx_for_date`.

		// 	// Check if current date is in storage, otherwise add it.
		// 	let current_date = <pallet_timestamp::Module<T>>::get();

		// 	let requested_date_as_u64;
		// 	let u64_in_millis = Self::convert_moment_to_u64_in_milliseconds(current_date.clone());
		// 	match u64_in_millis {
		// 		Err(_e) => {
		// 			log::error!("Unable to convert Moment to u64 in millis for current_date");
		// 			return 0;
		// 		},
		// 		Ok(ref x) => {
		// 			requested_date_as_u64 = x;
		// 		}
		// 	}
		// 	log::info!("requested_date_as_u64: {:?}", requested_date_as_u64.clone());

		// 	let requested_date_millis;
		// 	let start_of_date = Self::convert_u64_in_milliseconds_to_start_of_date(requested_date_as_u64.clone());
		// 	match start_of_date {
		// 		Err(_e) => {
		// 			log::error!("Unable to convert u64 millis to start of date for current_date");
		// 			return 0;
		// 		},
		// 		Ok(ref x) => {
		// 			requested_date_millis = x;
		// 		}
		// 	}

		// 	// https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
		// 	let contains_key = <MiningRewardsDHXForDate<T>>::contains_key(&requested_date_millis);
		// 	if contains_key == false {
		// 		let mining_rewards_dhx_current_u128;
		// 		let dhx_to_try = <MiningRewardsDHXCurrent<T>>::get();
		// 		if let Some(_mining_rewards_dhx_current_u128) = dhx_to_try {
		// 			mining_rewards_dhx_current_u128 = _mining_rewards_dhx_current_u128;
		// 		} else {
		// 			log::error!("Unable to convert Moment to i64 for requested_date");
		// 			return 0;
		// 		}

		// 		let mining_rewards;
		// 		let _mining_rewards = Self::convert_u128_to_balance(mining_rewards_dhx_current_u128.clone());
		// 		match _mining_rewards {
		// 			Err(_e) => {
		// 				log::error!("Unable to convert u128 to balance for mining_rewards");
		// 				return 0;
		// 			},
		// 			Ok(ref x) => {
		// 				mining_rewards = x;
		// 			}
		// 		}

		// 		// Update storage. Use MiningRewardsDHXCurrent as fallback incase not previously set prior to block
		// 		<MiningRewardsDHXForDate<T>>::insert(requested_date_millis.clone(), &mining_rewards);
		// 		log::info!("on_initialize");
		// 		log::info!("requested_date_millis: {:?}", requested_date_millis.clone());
		// 		log::info!("mining_rewards: {:?}", &mining_rewards);
		// 	}

		// 	return 0;
		// }

		// // `on_finalize` is executed at the end of block after all extrinsic are dispatched.
		// fn on_finalize(_n: T::BlockNumber) {
		// 	// Perform necessary data/state clean up here.
		// }

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
                  <MiningRewardsDHXForDate<T>>::insert(requested_date_millis.clone(), &bonded_data);
                  log::info!("account_id: {:?}", &account_id);
                  log::info!("bonded_data: {:?}", &bonded_data);

                  // Emit an event.
                  // TODO

                  // Return a successful DispatchResultWithPostInfo
                  Ok(())
          }

          // customised by governance at any time
  #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
  pub fn set_mining_rewards_dhx_current(origin: OriginFor<T>, mining_rewards: BalanceOf<T>) -> DispatchResult {
      let _who = ensure_signed(origin)?;

      let mining_rewards_as_u128 = Self::convert_balance_to_u128(mining_rewards.clone())?;

      // Update storage
      <MiningRewardsDHXCurrent<T>>::put(&mining_rewards_as_u128);
      log::info!("mining_rewards: {:?}", &mining_rewards_as_u128);

      // Emit an event.
      // TODO

      // Return a successful DispatchResultWithPostInfo
      Ok(())
  }

  // customised by governance at any time. this function allows us to change it each year
    // https://docs.google.com/spreadsheets/d/1W2AzOH9Cs9oCR8UYfYCbpmd9X7hp-USbYXL7AuwMY_Q/edit#gid=970997021
  #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
  pub fn set_mining_rewards_dhx_for_date(origin: OriginFor<T>, mining_rewards: BalanceOf<T>, timestamp: u64) -> DispatchResult {
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
      <MiningRewardsDHXForDate<T>>::insert(requested_date_millis.clone(), &mining_rewards);
      log::info!("mining_rewards: {:?}", &mining_rewards);

      // Emit an event.
      // TODO

      // Return a successful DispatchResultWithPostInfo
      Ok(())
  }

  #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
  // TODO - change from `reduce_` to `change_`, and provide another parameter `change: u8`, whose value
  // maybe 0 or 1 to represent that we want to make a corresponding decrease or increase to the remaining
  // dhx rewards allowance for a given date.
  pub fn reduce_remaining_mining_rewards_dhx_for_date(origin: OriginFor<T>, daily_rewards: BalanceOf<T>, timestamp: u64) -> DispatchResult {
      let _who = ensure_signed(origin)?;

      let requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(timestamp.clone())?;

      // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
      ensure!(<MiningRewardsDHXForDate<T>>::contains_key(&requested_date_millis), DispatchError::Other("Date key must exist to reduce allowance."));

      let existing_allowance_to_try = <MiningRewardsDHXForDate<T>>::get(&requested_date_millis);

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

      let new_remaining_allowance_as_u128 = existing_allowance_as_u128 - daily_rewards_as_u128;
      let new_remaining_allowance_as_balance = Self::convert_u128_to_balance(new_remaining_allowance_as_u128.clone())?;

      // Update storage
      <MiningRewardsDHXForDate<T>>::mutate(
          &requested_date_millis,
          |allowance| {
              if let Some(_allowance) = allowance {
                  *_allowance = new_remaining_allowance_as_balance.clone();
              }
              log::info!("Reduced mining_rewards_dhx_for_date at Date: {:?}", &requested_date_millis);
          },
      );

      // Emit an event.
      // TODO

      // Return a successful DispatchResultWithPostInfo
      Ok(())
  }

        // // Toggle premine status to enable or disable daily reward limits in `is_supernode_claim_reasonable`
        // #[weight = 10_000 + T::DbWeight::get().writes(1)]
        // pub fn set_is_premine(
        //     origin,
        //     _is_premine: bool,
        // ) -> Result<(), DispatchError> {
        //     let sender = ensure_root(origin)?;

        //     IsPremine::put(_is_premine.clone());

        //     // Self::deposit_event(RawEvent::IsPremining(
        //     //     _is_premine.clone(),
        //     //     sender.clone(),
        //     // ));

        //     Ok(())
        // }

    }
}

impl<T: Config> Module<T> {

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

    // /// Create a new mining mining_eligibility_proxy
    // pub fn create(sender: T::AccountId) -> Result<T::MiningEligibilityProxyIndex, DispatchError> {

    // }
}
