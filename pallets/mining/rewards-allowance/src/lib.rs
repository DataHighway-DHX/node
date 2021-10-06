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
    use log::{warn, info};
    use chrono::{
        NaiveDateTime,
    };
    use rand::{seq::SliceRandom, Rng};
    use substrate_fixed::{
        types::{
            extra::U3,
            U16F16,
            U32F32,
            U64F64,
        },
        FixedU128,
    };
    use codec::{
        Decode,
        Encode,
    };
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*,
        traits::{
            Currency,
            ExistenceRequirement,
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
    use sp_core::{
        sr25519,
    };
    use sp_runtime::traits::{
        IdentifyAccount,
        One,
        Verify,
    };
    use pallet_balances::{BalanceLock};
    use module_primitives::{
        types::{
            AccountId,
            Balance,
            Signature,
        },
    };

    // type BalanceOf<T> = <T as pallet_balances::Config>::Balance;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type BalanceFromBalancePallet<T> = <T as pallet_balances::Config>::Balance;
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
        + pallet_democracy::Config
        + pallet_balances::Config
        + pallet_timestamp::Config
        + pallet_treasury::Config {
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
    #[pallet::getter(fn rewards_allowance_dhx_for_date_distributed)]
    pub(super) type RewardsAllowanceDHXForDateDistributed<T: Config> = StorageMap<_, Blake2_128Concat,
        Date,
        bool
    >;

    #[pallet::storage]
    #[pallet::getter(fn rewards_allowance_dhx_daily)]
    pub(super) type RewardsAllowanceDHXDaily<T: Config> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_aggregated_dhx_for_all_miners_for_date)]
    pub(super) type RewardsAggregatedDHXForAllMinersForDate<T: Config> = StorageMap<_, Blake2_128Concat,
        Date,
        BalanceOf<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn rewards_accumulated_dhx_for_miner_for_date)]
    pub(super) type RewardsAccumulatedDHXForMinerForDate<T: Config> = StorageMap<_, Blake2_128Concat,
        (
            Date,
            T::AccountId,
        ),
        BalanceOf<T>,
    >;

	/// Those who registered that they want to participate in DHX Mining
	///
	/// TWOX-NOTE: Safe, as increasing integer keys are safe.
    #[pallet::storage]
    #[pallet::getter(fn registered_dhx_miners)]
    pub(super) type RegisteredDHXMiners<T: Config> = StorageValue<_, Vec<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn min_bonded_dhx_daily)]
    pub(super) type MinBondedDHXDaily<T: Config> = StorageValue<_, BalanceOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn cooling_off_period_days)]
    pub(super) type CoolingOffPeriodDays<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn cooling_off_period_days_remaining)]
    pub(super) type CoolingOffPeriodDaysRemaining<T: Config> = StorageMap<_, Blake2_128Concat,
        T::AccountId,
        (
            // date when cooling off period started for a given miner, or the date when we last reduced their cooling off period.
            // we do not reduce their cooling off period days remaining if we've already set this to a date that is the
            // current date for a miner (i.e. only reduce the days remaining once per day per miner)
            Date,
            u32, // days remaining
            // 0: unbonded (i.e. never bonded, or finished cool-down period and no longer bonding)
            // 1: bonded/bonding (i.e. waiting in the cool-down period before start getting rewards or eligible for rewards)
            // 2: unbonding (i.e. if they are bonding less than the threshold whilst getting rewards,
            //   this unbonding starts and they must wait until it finishes, which is when this value
            //   would be set to 0u32, before bonding and then waiting for the cool-down period all over again)
            u32,
        ),
    >;

    // The genesis config type.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub rewards_allowance_dhx_for_date: Vec<(Date, BalanceOf<T>)>,
        pub rewards_allowance_dhx_for_date_distributed: Vec<(Date, bool)>,
        pub rewards_allowance_dhx_daily: u128,
        pub rewards_aggregated_dhx_for_all_miners_for_date: Vec<(Date, BalanceOf<T>)>,
        pub rewards_accumulated_dhx_for_miner_for_date: Vec<((Date, T::AccountId), BalanceOf<T>)>,
        pub registered_dhx_miners: Vec<T::AccountId>,
        pub min_bonded_dhx_daily: BalanceOf<T>,
        pub cooling_off_period_days: u32,
        pub cooling_off_period_days_remaining: Vec<(T::AccountId, (Date, u32, u32))>,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                rewards_allowance_dhx_for_date: Default::default(),
                rewards_allowance_dhx_for_date_distributed: Default::default(),
                // 5000 UNIT, where UNIT token has 18 decimal places
                rewards_allowance_dhx_daily: 5_000_000_000_000_000_000_000u128,
                rewards_aggregated_dhx_for_all_miners_for_date: Default::default(),
                rewards_accumulated_dhx_for_miner_for_date: Default::default(),
                registered_dhx_miners: vec![
                    Default::default(),
                    Default::default(),
                ],
                min_bonded_dhx_daily: Default::default(),
                cooling_off_period_days: Default::default(),
                // Note: this doesn't seem to work, even if it's just `vec![Default::default()]` it doesn't use
                // the defaults in chain_spec.rs, so we set defaults later with `let mut cooling_off_period_days_remaining`
                cooling_off_period_days_remaining: vec![
                    (
                        Default::default(),
                        (
                            Default::default(),
                            Default::default(),
                            Default::default(),
                        ),
                    ),
                ]
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
            for (a, b) in &self.rewards_allowance_dhx_for_date_distributed {
                <RewardsAllowanceDHXForDateDistributed<T>>::insert(a, b);
            }
            <RewardsAllowanceDHXDaily<T>>::put(&self.rewards_allowance_dhx_daily);
            for (a) in &self.registered_dhx_miners {
                <RegisteredDHXMiners<T>>::append(a);
            }
            for (a, b) in &self.rewards_aggregated_dhx_for_all_miners_for_date {
                <RewardsAggregatedDHXForAllMinersForDate<T>>::insert(a, b);
            }
            for ((a, b), c) in &self.rewards_accumulated_dhx_for_miner_for_date {
                <RewardsAccumulatedDHXForMinerForDate<T>>::insert((a, b), c);
            }
            <MinBondedDHXDaily<T>>::put(&self.min_bonded_dhx_daily);
            <CoolingOffPeriodDays<T>>::put(&self.cooling_off_period_days);
            for (a, (b, c, d)) in &self.cooling_off_period_days_remaining {
                <CoolingOffPeriodDaysRemaining<T>>::insert(a, (b, c, d));
            }
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
        /// Storage of a sending account as a registered DHX miner
        /// \[sender]
        SetRegisteredDHXMiner(T::AccountId),

        /// Storage of the default minimum DHX that must be bonded by each registered DHX miner each day
        /// to be eligible for rewards
        /// \[amount_dhx, sender\]
        SetMinBondedDHXDailyStored(u128, T::AccountId),

        /// Storage of the default cooling off period in days
        /// \[cooling_off_period_days\]
        SetCoolingOffPeriodDaysStored(u32),

        /// Storage of the bonded DHX of an account on a specific date by a requesting origin account.
        /// \[date, amount_dhx_bonded, account_dhx_bonded, sender\]
        SetBondedDHXOfAccountForDateStored(Date, BondedData<T>, T::AccountId, T::AccountId),

        /// Storage of the default daily reward allowance in DHX by an origin account.
        /// \[amount_dhx, sender\]
        SetRewardsAllowanceDHXDailyStored(u128, T::AccountId),

        /// Storage of a new reward allowance in DHX for a specific date by an origin account.
        /// \[date, amount_dhx, sender\]
        SetRewardsAllowanceDHXForDateStored(Date, BalanceOf<T>, T::AccountId),

        /// Change the stored reward allowance in DHX for a specific date by an origin account, and
        /// where change is 0 for an decrease or any other value like 1 for an increase to the remaining
        /// rewards allowance.
        /// \[date, reduction_amount_dhx, sender, change\]
        ChangedRewardsAllowanceDHXForDateStored(Date, BalanceOf<T>, T::AccountId, u8),

        /// Transferred a proportion of the daily DHX rewards allowance to a DHX Miner on a given date
        /// \[date, miner_reward, remaining_rewards_allowance_today, miner_account_id\]
        TransferredRewardsAllowanceDHXToMinerForDate(Date, BalanceOf<T>, BalanceOf<T>, T::AccountId),

        /// Exhausted distributing all the daily DHX rewards allowance to DHX Miners on a given date.
        /// Note: There may be some leftover for the day so we record it here
        /// \[date, remaining_rewards_allowance_today\]
        DistributedRewardsAllowanceDHXForDate(Date, BalanceOf<T>),
    }

    // Errors inform users that something went wrong should be descriptive and have helpful documentation
    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        /// Preimage already noted
		DuplicatePreimage,
        /// Proposal does not exist
		ProposalMissing,
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
        // TODO - update with the weight consumed
        fn on_initialize(_n: T::BlockNumber) -> Weight {
            // Anything that needs to be done at the start of the block.
            let timestamp: <T as pallet_timestamp::Config>::Moment = <pallet_timestamp::Pallet<T>>::get();
            log::info!("_n: {:?}", _n.clone());
            log::info!("timestamp: {:?}", timestamp.clone());
            let requested_date_as_u64;
            let _requested_date_as_u64 = Self::convert_moment_to_u64_in_milliseconds(timestamp.clone());
            match _requested_date_as_u64 {
                Err(_e) => {
                    log::error!("Unable to convert Moment to u64 in millis for timestamp");
                    return 0;
                },
                Ok(ref x) => {
                    requested_date_as_u64 = x;
                }
            }
            log::info!("requested_date_as_u64: {:?}", requested_date_as_u64.clone());

            // do not run when block number is 1, which is when timestamp is 0 because this
            // timestamp corresponds to 1970-01-01
            if requested_date_as_u64.clone() == 0u64 {
                return 0;
            }
            let start_of_requested_date_millis;
            let _start_of_requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(requested_date_as_u64.clone());
            match _start_of_requested_date_millis {
                Err(_e) => {
                    log::error!("Unable to convert u64 in milliseconds to start_of_requested_date_millis");
                    return 0;
                },
                Ok(ref x) => {
                    start_of_requested_date_millis = x;
                }
            }
            log::info!("start_of_requested_date_millis: {:?}", start_of_requested_date_millis.clone());

            // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
            let contains_key = <RewardsAllowanceDHXForDate<T>>::contains_key(&start_of_requested_date_millis);
            log::info!("contains_key for date: {:?}, {:?}", start_of_requested_date_millis.clone(), contains_key.clone());

            // add the start_of_requested_date to storage if it doesn't already exist
            if contains_key == false {
                let rewards_allowance_dhx_daily_u128;
                let dhx_to_try = <RewardsAllowanceDHXDaily<T>>::get();
                if let Some(_rewards_allowance_dhx_daily_u128) = dhx_to_try {
                    rewards_allowance_dhx_daily_u128 = _rewards_allowance_dhx_daily_u128;
                } else {
                    log::error!("Unable to convert Moment to i64 for requested_date");
                    return 0;
                }

                let rewards_allowance_dhx_daily;
                let _rewards_allowance_dhx_daily = Self::convert_u128_to_balance(rewards_allowance_dhx_daily_u128.clone());
                match _rewards_allowance_dhx_daily {
                    Err(_e) => {
                        log::error!("Unable to convert u128 to balance for rewards_allowance_dhx_daily");
                        return 0;
                    },
                    Ok(ref x) => {
                        rewards_allowance_dhx_daily = x;
                    }
                }

                // Update storage. Use RewardsAllowanceDHXDaily as fallback incase not previously set prior to block
                <RewardsAllowanceDHXForDate<T>>::insert(start_of_requested_date_millis.clone(), &rewards_allowance_dhx_daily);
                <RewardsAllowanceDHXForDateDistributed<T>>::insert(start_of_requested_date_millis.clone(), false);
                log::info!("on_initialize");
                log::info!("rewards_allowance: {:?}", &rewards_allowance_dhx_daily);
            }

            // we only check accounts that have registered that they want to participate in DHX Mining
            let reg_dhx_miners;
            let reg_dhx_miners_to_try = <RegisteredDHXMiners<T>>::get();
            if let Some(_reg_dhx_miners_to_try) = reg_dhx_miners_to_try {
                reg_dhx_miners = _reg_dhx_miners_to_try;
            } else {
                log::error!("Unable to retrieve any registered DHX Miners");
                return 0;
            }
            if reg_dhx_miners.len() == 0 {
                log::error!("Registered DHX Miners has no elements");
                return 0;
            };
            let mut miner_count = 0;

            for (index, miner) in reg_dhx_miners.iter().enumerate() {
                miner_count += 1;
                log::info!("miner_count {:#?}", miner_count);
                log::info!("miner {:#?}", miner);
                // let locks_until_block_for_account = <pallet_balances::Pallet<T>>::locks(miner.clone());
                // // NOTE - I fixed the following error by using `.into_inner()` after asking the community here and getting a
                // // response in Substrate Builders weekly meeting https://matrix.to/#/!HzySYSaIhtyWrwiwEV:matrix.org/$163243681163543vyfkW:matrix.org?via=matrix.parity.io&via=matrix.org&via=corepaper.org
                // //
                // // `WeakBoundedVec<BalanceLock<<T as pallet_balances::Config>::Balance>,
                // // <T as pallet_balances::Config>::MaxLocks>` cannot be formatted using
                // // `{:?}` because it doesn't implement `core::fmt::Debug`
                // //
                // // https://substrate.dev/rustdocs/latest/frame_support/storage/weak_bounded_vec/struct.WeakBoundedVec.html
                // log::info!("miner locks {:#?}", locks_until_block_for_account.into_inner().clone());
                // let locked: BalanceLock<<T as pallet_balances::Config>::Balance> =
                //     locks_until_block_for_account.into_inner().clone()[0];

                // assume DHX Miner only has one lock for simplicity. retrieve the amount locked
                // TODO - miner may have multiple locks, so we actually want to go through the vector
                // and find the lock(s) we're interested in, and aggregate the total, and later check
                // that it's greater than min_bonded_dhx_daily_u128.
                // default for demonstration incase miner does not have any locks when checking below.
                //
                // Test with 2x registered miners each with values like `25133000000000000000000u128`, which is over
                // half of 5000 DHX daily allowance (of 2500 DHX), but in that case we split the rewards
                // (i.e. 25,133 DHX locked at 10:1 gives 2513 DHX reward)
                let mut locks_first_amount_as_u128 = 25_133_000_000_000_000_000_000u128;

                let locked_vec = <pallet_balances::Pallet<T>>::locks(miner.clone()).into_inner();
                if locked_vec.len() != 0 {
                    let locks_first_amount: <T as pallet_balances::Config>::Balance =
                        <pallet_balances::Pallet<T>>::locks(miner.clone()).into_inner().clone()[0].amount;

                    let _locks_first_amount_as_u128 = Self::convert_balance_to_u128_from_pallet_balance(locks_first_amount.clone());
                    match _locks_first_amount_as_u128.clone() {
                        Err(_e) => {
                            log::error!("Unable to convert balance to u128");
                            return 0;
                        },
                        Ok(x) => {
                            locks_first_amount_as_u128 = x;
                        }
                    }
                    log::info!("locks_first_amount_as_u128: {:?}", locks_first_amount_as_u128.clone());
                }

                // Example output below of vote with 9.9999 tokens on a referendum associated with a proposal
                // that was seconded
                //
                // BalanceLock {
                //     id: [
                //         100,
                //         101,
                //         109,
                //         111,
                //         99,
                //         114,
                //         97,
                //         99,
                //     ],
                //     amount: 9999900000000000000,
                //     reasons: Reasons::Misc,
                // },

                // TODO - refactor to use `convert_balance_to_u128` instead of all the following
                let min_bonded_dhx_daily;
                let min_bonded_dhx_daily_to_try = <MinBondedDHXDaily<T>>::get();
                if let Some(_min_bonded_dhx_daily_to_try) = min_bonded_dhx_daily_to_try {
                    min_bonded_dhx_daily = _min_bonded_dhx_daily_to_try;
                } else {
                    log::error!("Unable to retrieve any min. bonded DHX daily");
                    return 0;
                }

                let min_bonded_dhx_daily_u128;
                if let Some(_min_bonded_dhx_daily_u128) = TryInto::<u128>::try_into(min_bonded_dhx_daily).ok() {
                    min_bonded_dhx_daily_u128 = _min_bonded_dhx_daily_u128;
                } else {
                    log::error!("Unable to convert BalanceOf to u128 for min_bonded_dhx_daily");
                    return 0;
                }
                log::info!("min_bonded_dhx_daily_u128: {:?}", min_bonded_dhx_daily_u128.clone());

                let mut is_bonding_min_dhx = false;
                if locks_first_amount_as_u128 >= min_bonded_dhx_daily_u128 {
                    is_bonding_min_dhx = true;
                }
                log::info!("is_bonding_min_dhx: {:?} {:?}", is_bonding_min_dhx.clone(), miner.clone());

                let cooling_off_period_days;
                let cooling_off_period_days_to_try = <CoolingOffPeriodDays<T>>::get();
                if let Some(_cooling_off_period_days_to_try) = cooling_off_period_days_to_try {
                    cooling_off_period_days = _cooling_off_period_days_to_try;
                } else {
                    log::error!("Unable to retrieve cooling off period days");
                    return 0;
                }

                let mut cooling_off_period_days_remaining = (
                    start_of_requested_date_millis.clone(),
                    7u32,
                    0u32,
                );
                let cooling_off_period_days_remaining_to_try = <CoolingOffPeriodDaysRemaining<T>>::get(miner.clone());
                if let Some(_cooling_off_period_days_remaining_to_try) = cooling_off_period_days_remaining_to_try {
                    // we do not change cooling_off_period_days_remaining.0 to the default value in the chain_spec.rs of 0,
                    // instead we want to use today's date `start_of_requested_date_millis.clone()` by default, as we did above.
                    if _cooling_off_period_days_remaining_to_try.0 != 0 {
                        cooling_off_period_days_remaining.0 = _cooling_off_period_days_remaining_to_try.0;
                    }
                    cooling_off_period_days_remaining.1 = _cooling_off_period_days_remaining_to_try.1;
                    cooling_off_period_days_remaining.2 = _cooling_off_period_days_remaining_to_try.2;
                } else {
                    log::info!("Unable to retrieve cooling off period days remaining for given miner, using default {:?}", miner.clone());
                }
                log::info!("cooling_off_period_days_remaining {:?} {:?} {:?}", start_of_requested_date_millis.clone(), cooling_off_period_days_remaining, miner.clone());
                // if cooling_off_period_days_remaining.2 is 0u32, it means we haven't recognised they that are bonding yet (unbonded),
                // they aren't currently bonding, they haven't started cooling off to start bonding,
                // or have already finished cooling down after bonding.
                // so if we detect they are now bonding above the min. then we should start at max. remaining days
                // before starting to decrement on subsequent blocks
                if
                    cooling_off_period_days_remaining.2 == 0u32 &&
                    is_bonding_min_dhx == true
                {
                    <CoolingOffPeriodDaysRemaining<T>>::insert(
                        miner.clone(),
                        (
                            start_of_requested_date_millis.clone(),
                            cooling_off_period_days.clone(),
                            1u32, // they are bonded again, waiting to start getting rewards
                        ),
                    );
                    log::info!("Added CoolingOffPeriodDaysRemaining for miner {:?} {:?} {:?}", start_of_requested_date_millis.clone(), miner.clone(), cooling_off_period_days.clone());
                // if cooling_off_period_days_remaining.0 is not the start of the current date
                //   (since if they just started bonding and we just set days remaining to 7, or we already decremented
                //   a miner's days remaining for the current date, then we want to wait until the next day until we
                //   decrement another day).
                // if cooling_off_period_days_remaining.1 is Some(above 0), then decrement, but not eligible yet for rewards.
                } else if
                    cooling_off_period_days_remaining.0 != start_of_requested_date_millis.clone() &&
                    cooling_off_period_days_remaining.1 > 0u32 &&
                    is_bonding_min_dhx == true
                {
                    // println!("[reducing_days] block: {:#?}, miner: {:#?}, date_start: {:#?} remain_days: {:#?}", _n, miner_count, start_of_requested_date_millis, cooling_off_period_days_remaining);
                    let old_cooling_off_period_days_remaining = cooling_off_period_days_remaining.1.clone();

                    // we cannot do this because of error: cannot use the `?` operator in a method that returns `()`
                    // let new_cooling_off_period_days_remaining =
                    //     old_cooling_off_period_days_remaining.checked_sub(One::one()).ok_or(Error::<T>::StorageOverflow)?;

                    // Subtract, handling overflow
                    let new_cooling_off_period_days_remaining;
                    let _new_cooling_off_period_days_remaining =
                        old_cooling_off_period_days_remaining.checked_sub(One::one());
                    match _new_cooling_off_period_days_remaining {
                        None => {
                            log::error!("Unable to subtract one from cooling_off_period_days_remaining due to StorageOverflow");
                            return 0;
                        },
                        Some(x) => {
                            new_cooling_off_period_days_remaining = x;
                        }
                    }

                    // Write the new value to storage
                    <CoolingOffPeriodDaysRemaining<T>>::insert(
                        miner.clone(),
                        (
                            start_of_requested_date_millis.clone(),
                            new_cooling_off_period_days_remaining.clone(),
                            1u32, // they are bonded again, waiting to start getting rewards
                        ),
                    );
                    log::info!("Reduced CoolingOffPeriodDaysRemaining for miner {:?} {:?} {:?}", start_of_requested_date_millis.clone(), miner.clone(), new_cooling_off_period_days_remaining.clone());
                // if cooling_off_period_days_remaining.0 is not the start of the current date
                //   (since if we decremented days remaining from 1 to 0 days left for a miner
                //   then we want to wait until the next day before we distribute the rewards to them)
                // if cooling_off_period_days_remaining.1 is Some(0),
                // and if cooling_off_period_days_remaining.2 is 1
                // and then no more cooling off days, but don't decrement,
                // and say they are eligible for reward payments
                } else if
                    cooling_off_period_days_remaining.0 != start_of_requested_date_millis.clone() &&
                    cooling_off_period_days_remaining.1 == 0u32 &&
                    cooling_off_period_days_remaining.2 == 1u32 &&
                    is_bonding_min_dhx == true
                {
                    // println!("[eligible] block: {:#?}, miner: {:#?}, date_start: {:#?} remain_days: {:#?}", _n, miner_count, start_of_requested_date_millis, cooling_off_period_days_remaining);

                    // we need to add that they are eligible for rewards on the current date too
                    <CoolingOffPeriodDaysRemaining<T>>::insert(
                        miner.clone(),
                        (
                            start_of_requested_date_millis.clone(),
                            0u32,
                            1u32,
                        ),
                    );

                    // only accumulate the DHX reward for each registered miner once per day
                    // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
                    if <RewardsAccumulatedDHXForMinerForDate<T>>::contains_key(
                        (
                            start_of_requested_date_millis.clone(),
                            miner.clone(),
                        )
                    ) == true {
                        continue;
                    }

                    let rewards_allowance_dhx_daily;
                    let rewards_allowance_dhx_daily_to_try = <RewardsAllowanceDHXDaily<T>>::get();
                    if let Some(_rewards_allowance_dhx_daily_to_try) = rewards_allowance_dhx_daily_to_try {
                        rewards_allowance_dhx_daily = _rewards_allowance_dhx_daily_to_try;
                    } else {
                        log::error!("Unable to retrieve rewards_allowance_dhx_daily");
                        return 0;
                    }

                    // calculate the daily reward for the miner in DHX based on their bonded DHX.
                    // it should be a proportion taking other eligible miner's who are eligible for
                    // daily rewards into account since we want to split them fairly.
                    //
                    // assuming min_bonded_dhx_daily is 10u128, and they have that minimum of 10 DHX bonded (10u128) for
                    // the locks_first_amount_as_u128 value, then they are eligible for 1 DHX reward
                    //
                    // Divide, handling overflow
                    let mut daily_reward_for_miner_as_u128 = 0u128;
                    // note: this rounds down to the nearest integer
                    let _daily_reward_for_miner_as_u128 = locks_first_amount_as_u128.clone().checked_div(min_bonded_dhx_daily_u128.clone());
                    match _daily_reward_for_miner_as_u128 {
                        None => {
                            log::error!("Unable to divide min_bonded_dhx_daily from locks_first_amount_as_u128 due to StorageOverflow");
                            return 0;
                        },
                        Some(x) => {
                            daily_reward_for_miner_as_u128 = x;
                        }
                    }
                    log::info!("daily_reward_for_miner_as_u128: {:?}", daily_reward_for_miner_as_u128.clone());
                    // println!("[eligible] block: {:#?}, miner: {:#?}, date_start: {:#?} daily_reward_for_miner_as_u128: {:#?}", _n, miner_count, start_of_requested_date_millis, daily_reward_for_miner_as_u128);

                    let daily_reward_for_miner;
                    let _daily_reward_for_miner = Self::convert_u128_to_balance(daily_reward_for_miner_as_u128.clone());
                    match _daily_reward_for_miner {
                        Err(_e) => {
                            log::error!("Unable to convert u128 to balance for daily_reward_for_miner");
                            return 0;
                        },
                        Ok(ref x) => {
                            daily_reward_for_miner = x;
                        }
                    }
                    log::info!("daily_reward_for_miner: {:?}", daily_reward_for_miner.clone());

                    let mut rewards_aggregated_dhx_daily: BalanceOf<T> = 0u32.into(); // initialize
                    let aggregated_to_try = <RewardsAggregatedDHXForAllMinersForDate<T>>::get(&start_of_requested_date_millis);
                    if let Some(_rewards_aggregated_dhx_daily) = aggregated_to_try {
                        rewards_aggregated_dhx_daily = _rewards_aggregated_dhx_daily;
                    } else {
                        log::error!("Unable to retrieve balance for rewards_aggregated_dhx_daily");
                    }


                    let rewards_aggregated_dhx_daily_as_u128;
                    let _rewards_aggregated_dhx_daily_as_u128 = Self::convert_balance_to_u128(rewards_aggregated_dhx_daily.clone());
                    match _rewards_aggregated_dhx_daily_as_u128.clone() {
                        Err(_e) => {
                            log::error!("Unable to convert balance to u128 for rewards_aggregated_dhx_daily_as_u128");
                            return 0;
                        },
                        Ok(x) => {
                            rewards_aggregated_dhx_daily_as_u128 = x;
                        }
                    }

                    // Add, handling overflow
                    let new_rewards_aggregated_dhx_daily_as_u128;
                    let _new_rewards_aggregated_dhx_daily_as_u128 =
                        rewards_aggregated_dhx_daily_as_u128.clone().checked_add(daily_reward_for_miner_as_u128.clone());
                    match _new_rewards_aggregated_dhx_daily_as_u128 {
                        None => {
                            log::error!("Unable to add daily_reward_for_miner to rewards_aggregated_dhx_daily due to StorageOverflow");
                            return 0;
                        },
                        Some(x) => {
                            new_rewards_aggregated_dhx_daily_as_u128 = x;
                        }
                    }

                    log::info!("new_rewards_aggregated_dhx_daily_as_u128: {:?}", new_rewards_aggregated_dhx_daily_as_u128.clone());
                    // println!("[eligible] block: {:#?}, miner: {:#?}, date_start: {:#?} new_rewards_aggregated_dhx_daily_as_u128: {:#?}", _n, miner_count, start_of_requested_date_millis, new_rewards_aggregated_dhx_daily_as_u128);

                    let new_rewards_aggregated_dhx_daily;
                    let _new_rewards_aggregated_dhx_daily = Self::convert_u128_to_balance(new_rewards_aggregated_dhx_daily_as_u128.clone());
                    match _new_rewards_aggregated_dhx_daily {
                        Err(_e) => {
                            log::error!("Unable to convert u128 to balance for new_rewards_aggregated_dhx_daily");
                            return 0;
                        },
                        Ok(ref x) => {
                            new_rewards_aggregated_dhx_daily = x;
                        }
                    }

                    // add to storage item that accumulates total rewards for all registered miners for the day
                    <RewardsAggregatedDHXForAllMinersForDate<T>>::insert(
                        start_of_requested_date_millis.clone(),
                        new_rewards_aggregated_dhx_daily.clone(),
                    );
                    log::info!("Added RewardsAggregatedDHXForAllMinersForDate for miner {:?} {:?} {:?}", start_of_requested_date_millis.clone(), miner.clone(), new_rewards_aggregated_dhx_daily.clone());

                    // add to storage item that maps the date to the registered miner and the calculated reward
                    // (prior to possibly reducing it so they get a proportion of the daily rewards that are available)
                    <RewardsAccumulatedDHXForMinerForDate<T>>::insert(
                        (
                            start_of_requested_date_millis.clone(),
                            miner.clone(),
                        ),
                        daily_reward_for_miner.clone(),
                    );
                    log::info!("Added RewardsAccumulatedDHXForMinerForDate for miner {:?} {:?} {:?}", start_of_requested_date_millis.clone(), miner.clone(), daily_reward_for_miner.clone());

                // if they stop bonding the min dhx, and
                // if cooling_off_period_days_remaining.1 is Some(0),
                // and if cooling_off_period_days_remaining.2 is 1 (where they had just been bonding and getting rewards)
                // so since we detected they are no longer bonding above the min. then we should start at max. remaining days
                // before starting to decrement on subsequent blocks
                } else if
                    cooling_off_period_days_remaining.1 == 0u32 &&
                    cooling_off_period_days_remaining.2 == 1u32 &&
                    is_bonding_min_dhx == false
                {
                    // Write the new value to storage
                    <CoolingOffPeriodDaysRemaining<T>>::insert(
                        miner.clone(),
                        (
                            start_of_requested_date_millis.clone(),
                            cooling_off_period_days.clone(),
                            2u32, // they have unbonded again, waiting to finish cooling down period
                        ),
                    );

                    log::info!("Unbonding detected for miner. Starting cooling down period {:?} {:?}", miner.clone(), cooling_off_period_days.clone());

                // if cooling_off_period_days_remaining.0 is not the start of the current date
                //   (since if they just started un-bonding and we just set days remaining to 7, or we already decremented
                //   a miner's days remaining for the current date, then we want to wait until the next day until we
                //   decrement another day).
                // if cooling_off_period_days_remaining.1 is Some(above 0), then decrement,
                // but not yet completely unbonded so cannot withdraw yet
                // note: we don't care if they stop bonding below the min. dhx during the cooling off period,
                // as the user needs to learn that they should always been bonding the min. to
                // maintain rewards, otherwise they have to wait for entire cooling down period and
                // then the cooling off period again.
                //
                } else if
                    cooling_off_period_days_remaining.0 != start_of_requested_date_millis.clone() &&
                    cooling_off_period_days_remaining.1 > 0u32 &&
                    cooling_off_period_days_remaining.2 == 2u32
                    // && is_bonding_min_dhx == false
                {
                    let old_cooling_off_period_days_remaining = cooling_off_period_days_remaining.1.clone();

                    // Subtract, handling overflow
                    let new_cooling_off_period_days_remaining;
                    let _new_cooling_off_period_days_remaining =
                        old_cooling_off_period_days_remaining.checked_sub(One::one());
                    match _new_cooling_off_period_days_remaining {
                        None => {
                            log::error!("Unable to subtract one from cooling_off_period_days_remaining due to StorageOverflow");
                            return 0;
                        },
                        Some(x) => {
                            new_cooling_off_period_days_remaining = x;
                        }
                    }

                    // Write the new value to storage
                    <CoolingOffPeriodDaysRemaining<T>>::insert(
                        miner.clone(),
                        (
                            start_of_requested_date_millis.clone(),
                            new_cooling_off_period_days_remaining.clone(),
                            2u32, // they have unbonded again, waiting to finish cooling down period
                        ),
                    );

                    // println!("[reduce] block: {:#?}, miner: {:#?}, date_start: {:#?} new_cooling_off_period_days_remaining: {:#?}", _n, miner_count, start_of_requested_date_millis, new_cooling_off_period_days_remaining);
                    log::info!("Unbonded miner. Reducing cooling down period dates remaining {:?} {:?}", miner.clone(), new_cooling_off_period_days_remaining.clone());

                // if cooling_off_period_days_remaining.0 is not the start of the current date
                //   (since if we decremented days remaining to from 1 to 0 days left for a miner
                //   then we want to wait until the next day before we set cooling_off_period_days_remaining.2 to 0u32
                //   to allow them to be completely unbonded and withdraw).
                // if cooling_off_period_days_remaining.1 is Some(0), do not subtract anymore, they are
                // completely unbonded so can withdraw
                } else if
                    cooling_off_period_days_remaining.0 != start_of_requested_date_millis.clone() &&
                    cooling_off_period_days_remaining.1 == 0u32 &&
                    cooling_off_period_days_remaining.2 == 2u32
                    // && is_bonding_min_dhx == false
                {
                    // Write the new value to storage
                    <CoolingOffPeriodDaysRemaining<T>>::insert(
                        miner.clone(),
                        (
                            start_of_requested_date_millis.clone(),
                            0u32,
                            0u32, // they are completely unbonded again
                        ),
                    );

                    log::info!("Unbonded miner. Cooling down period finished so allow them to withdraw {:?}", miner.clone());
                }
            }

            log::info!("Finished initial loop of registered miners");

            // fetch accumulated total rewards for all registered miners for the day
            // TODO - we've done this twice, create a function to fetch it
            let mut rewards_aggregated_dhx_daily: BalanceOf<T> = 0u32.into(); // initialize
            let aggregated_to_try = <RewardsAggregatedDHXForAllMinersForDate<T>>::get(&start_of_requested_date_millis);
            if let Some(_rewards_aggregated_dhx_daily) = aggregated_to_try {
                rewards_aggregated_dhx_daily = _rewards_aggregated_dhx_daily;
            } else {
                log::error!("Unable to retrieve balance for rewards_aggregated_dhx_daily. Cooling off period may not be finished yet");
                // Note: it would be an issue if we got past the first loop of looping through the registered miners
                // and still hadn't added to the aggregated rewards for the day
                return 0;
            }
            // println!("[multiplier] block: {:#?}, miner: {:#?}, date_start: {:#?} rewards_aggregated_dhx_daily: {:#?}", _n, miner_count, start_of_requested_date_millis, rewards_aggregated_dhx_daily);

            if rewards_aggregated_dhx_daily == 0u32.into() {
                log::error!("rewards_aggregated_dhx_daily must be greater than 0 to distribute rewards");
                return 0;
            }

            let rewards_aggregated_dhx_daily_as_u128;
            let _rewards_aggregated_dhx_daily_as_u128 = Self::convert_balance_to_u128(rewards_aggregated_dhx_daily.clone());
            match _rewards_aggregated_dhx_daily_as_u128.clone() {
                Err(_e) => {
                    log::error!("Unable to convert balance to u128 for rewards_aggregated_dhx_daily_as_u128");
                    return 0;
                },
                Ok(x) => {
                    rewards_aggregated_dhx_daily_as_u128 = x;
                }
            }
            log::info!("rewards_aggregated_dhx_daily_as_u128: {:?}", rewards_aggregated_dhx_daily_as_u128.clone());

            // TODO - we've done this twice, create a function to fetch it
            let rewards_allowance_dhx_daily_u128;
            let dhx_to_try = <RewardsAllowanceDHXDaily<T>>::get();
            if let Some(_rewards_allowance_dhx_daily_u128) = dhx_to_try {
                rewards_allowance_dhx_daily_u128 = _rewards_allowance_dhx_daily_u128;
            } else {
                log::error!("Unable to convert Moment to i64 for requested_date");
                return 0;
            }

            if rewards_allowance_dhx_daily_u128 == 0u128 {
                log::error!("rewards_allowance_dhx_daily must be greater than 0 to distribute rewards");
                return 0;
            }

            let rewards_allowance_dhx_daily;
            let _rewards_allowance_dhx_daily = Self::convert_u128_to_balance(rewards_allowance_dhx_daily_u128.clone());
            match _rewards_allowance_dhx_daily {
                Err(_e) => {
                    log::error!("Unable to convert u128 to balance for rewards_allowance_dhx_daily");
                    return 0;
                },
                Ok(ref x) => {
                    rewards_allowance_dhx_daily = x;
                }
            }

            // previously when we looped through all the registered dhx miners we calculated the
            // reward for each registered miner based on the 10:1 ratio, and stored that along with
            // the corresponding day in storage. since that loop we've fetched the total
            // aggregated rewards that all reg miners are eligible for on that day as `rewards_aggregated_dhx_daily`,
            // lets say it adds up to 8000 DHX, but say we only have 5000 DHX availabe to distribute
            // from `rewards_allowance_dhx_daily`, so we'll constrain the rewards they'll receive further by
            // applying a `distribution_multiplier_for_day_u128` of (5000/8000)*reg_miner_reward on each of
            // the rewards that are distributed to them.

            // if the aggregated rewards isn't more than the daily rewards allowance available
            // then just set the multiplier to 1, so they actually get the previously calculated reward rather
            // than a scaled down proportion.
            //
            // e.g. 1: if miner rewards are 2000 & 4000 DHX respectively, this is greater than 5000 DHX daily allowance
            // so we'd have a multiplier of 5000/6000 = 5/6, so they'd receive ~1666 & 3333 DHX respectively.
            // e.g. 2: if miner rewards are 2000 & 2000 DHX respectively, this is less than 5000 DHX daily allowance
            // so we'd just use a multiplier of 1, so they'd receive 2000 & 2000 DHX respectively.
            // https://docs.rs/fixed/0.5.4/fixed/struct.FixedU128.html
            let mut distribution_multiplier_for_day_fixed128 = FixedU128::from_num(1);

            if rewards_aggregated_dhx_daily_as_u128.clone() > rewards_allowance_dhx_daily_u128.clone() {
                // Divide, handling overflow

                // Note: If the rewards_allowance_dhx_daily_u128 is 5000 DHX, its 5000000000000000000000,
                // but we can't convert to u64 since largest value is 18446744073709551615.
                // Since we expect the rewards_aggregated_dhx_daily_as_u128 to be at least 1 DHX (i.e. 1000000000000000000),
                // we could just divide both numbers by 1000000000000000000, so we'd have say 5000 and 1 instead,
                // since we're just using these values to get a multiplier output.

                let mut manageable_rewards_allowance_dhx_daily_u128 = 0u128;
                if let Some(_manageable_rewards_allowance_dhx_daily_u128) =
                    rewards_allowance_dhx_daily_u128.clone().checked_div(1000000000000000000u128) {
                        manageable_rewards_allowance_dhx_daily_u128 = _manageable_rewards_allowance_dhx_daily_u128;
                } else {
                    log::error!("Unable to divide rewards_allowance_dhx_daily_u128 to make it smaller");
                    return 0;
                }

                let mut rewards_allowance_dhx_daily_u64 = 0u64;
                if let Some(_rewards_allowance_dhx_daily_u64) =
				    TryInto::<u64>::try_into(manageable_rewards_allowance_dhx_daily_u128.clone()).ok() {
                        rewards_allowance_dhx_daily_u64 = _rewards_allowance_dhx_daily_u64;
                } else {
                    log::error!("Unable to convert u128 to u64 for rewards_allowance_dhx_daily_u128");
                    return 0;
                }

                let mut manageable_rewards_aggregated_dhx_daily_as_u128 = 0u128;
                if let Some(_manageable_rewards_aggregated_dhx_daily_as_u128) = rewards_aggregated_dhx_daily_as_u128.clone().checked_div(1000000000000000000u128) {
                    manageable_rewards_aggregated_dhx_daily_as_u128 = _manageable_rewards_aggregated_dhx_daily_as_u128;
                } else {
                    log::error!("Unable to divide manageable_rewards_aggregated_dhx_daily_as_u128 to make it smaller");
                    return 0;
                }

                let mut rewards_aggregated_dhx_daily_as_u64 = 0u64;
                if let Some(_rewards_aggregated_dhx_daily_as_u64) =
				    TryInto::<u64>::try_into(manageable_rewards_aggregated_dhx_daily_as_u128.clone()).ok() {
                        rewards_aggregated_dhx_daily_as_u64 = _rewards_aggregated_dhx_daily_as_u64;
                } else {
                    log::error!("Unable to convert u128 to u64 for rewards_aggregated_dhx_daily_as_u128");
                    return 0;
                }

                // See https://github.com/ltfschoen/substrate-node-template/pull/6/commits/175ef4805d07093042431c5814dda52da1ebde18
                let _fraction_distribution_multiplier_for_day_fixed128 =
                    U64F64::from_num(manageable_rewards_allowance_dhx_daily_u128.clone())
                        .checked_div(U64F64::from_num(manageable_rewards_aggregated_dhx_daily_as_u128.clone()));
                let _distribution_multiplier_for_day_fixed128 = _fraction_distribution_multiplier_for_day_fixed128.clone();
                match _distribution_multiplier_for_day_fixed128 {
                    None => {
                        log::error!("Unable to divide rewards_allowance_dhx_daily_u128 due to StorageOverflow by rewards_aggregated_dhx_daily_as_u128");
                        return 0;
                    },
                    Some(x) => {
                        distribution_multiplier_for_day_fixed128 = x;
                    }
                }
            }
            log::info!("distribution_multiplier_for_day_fixed128 {:#?}", distribution_multiplier_for_day_fixed128);
            // println!("[multiplier] block: {:#?}, miner: {:#?}, date_start: {:#?} distribution_multiplier_for_day_fixed128: {:#?}", _n, miner_count, start_of_requested_date_millis, distribution_multiplier_for_day_fixed128);

            // Initialise outside the loop as we need this value after the loop after we finish iterating through all the miners
            let mut rewards_allowance_dhx_remaining_today_as_u128 = 0u128;

            miner_count = 0;
            for (index, miner) in reg_dhx_miners.iter().enumerate() {
                miner_count += 1;
                log::info!("rewards loop - miner_count {:#?}", miner_count);
                log::info!("rewards loop - miner {:#?}", miner);

                // only run the following once per day per miner until rewards_allowance_dhx_for_date is exhausted
                // but since we're giving each registered miner a proportion of the daily reward allowance
                // (if their aggregated rewards is above daily allowance) each proportion is rounded down,
                // it shouldn't become exhausted anyway
                let is_already_distributed = <RewardsAllowanceDHXForDateDistributed<T>>::get(start_of_requested_date_millis.clone());
                if is_already_distributed == Some(true) {
                    log::error!("Unable to distribute further rewards allowance today");
                    return 0;
                }

                let daily_reward_for_miner_as_u128;
                let daily_reward_for_miner_to_try = <RewardsAccumulatedDHXForMinerForDate<T>>::get(
                    (
                        start_of_requested_date_millis.clone(),
                        miner.clone(),
                    ),
                );
                if let Some(_daily_reward_for_miner_to_try) = daily_reward_for_miner_to_try.clone() {
                    let _daily_reward_for_miner_as_u128 = Self::convert_balance_to_u128(_daily_reward_for_miner_to_try.clone());
                    match _daily_reward_for_miner_as_u128.clone() {
                        Err(_e) => {
                            log::error!("Unable to convert balance to u128 for daily_reward_for_miner_as_u128");
                            return 0;
                        },
                        Ok(x) => {
                            daily_reward_for_miner_as_u128 = x;
                        }
                    }
                } else {
                    // If any of the miner's don't have a reward, we won't waste storing that,
                    // so we want to move to the next miner in the loop
                    log::error!("Unable to retrieve reward balance for daily_reward_for_miner {:?}", miner.clone());
                    continue;
                }
                log::info!("daily_reward_for_miner_as_u128: {:?}", daily_reward_for_miner_as_u128.clone());

                let mut manageable_daily_reward_for_miner_as_u128 = 0u128;
                if let Some(_manageable_daily_reward_for_miner_as_u128) =
                    daily_reward_for_miner_as_u128.clone().checked_div(1000000000000000000u128) {
                        manageable_daily_reward_for_miner_as_u128 = _manageable_daily_reward_for_miner_as_u128;
                } else {
                    log::error!("Unable to divide daily_reward_for_miner_as_u128 to make it smaller");
                    return 0;
                }

                // Multiply, handling overflow
                // TODO - probably have to initialise below proportion_of_daily_reward_for_miner_fixed128 to 0u128,
                // and convert distribution_multiplier_for_day_fixed128 to u64,
                // and convert daily_reward_for_miner_as_u128 to u64 too, like i did earlier.
                // but it works so this doesn't seem necessary.
                let proportion_of_daily_reward_for_miner_fixed128;
                let _proportion_of_daily_reward_for_miner_fixed128 =
                    U64F64::from_num(distribution_multiplier_for_day_fixed128.clone()).checked_mul(U64F64::from_num(manageable_daily_reward_for_miner_as_u128.clone()));
                match _proportion_of_daily_reward_for_miner_fixed128 {
                    None => {
                        log::error!("Unable to multiply proportion_of_daily_reward_for_miner_fixed128 with daily_reward_for_miner_as_u128 due to StorageOverflow");
                        return 0;
                    },
                    Some(x) => {
                        proportion_of_daily_reward_for_miner_fixed128 = x;
                    }
                }
                log::info!("proportion_of_daily_reward_for_miner_fixed128: {:?}", proportion_of_daily_reward_for_miner_fixed128.clone());

                // round down to nearest integer. we need to round down, because if we round up then if there are
                // 3x registered miners with 5000 DHX rewards allowance per day then they would each get 1667 rewards,
                // but there would only be 1666 remaining after the first two, so the last one would miss out.
                // so if we round down they each get 1666 DHX and there is 2 DHX from the daily allocation that doesn't get distributed at all.
                let proportion_of_daily_reward_for_miner_u128: u128 = proportion_of_daily_reward_for_miner_fixed128.floor().to_num::<u128>();

                // we lose some accuracy doing this conversion, but at least we split the bulk of the rewards proportionally and fairly
                let mut restored_proportion_of_daily_reward_for_miner_u128 = 0u128;
                if let Some(_restored_proportion_of_daily_reward_for_miner_u128) =
                    proportion_of_daily_reward_for_miner_u128.clone().checked_mul(1000000000000000000u128) {
                        restored_proportion_of_daily_reward_for_miner_u128 = _restored_proportion_of_daily_reward_for_miner_u128;
                } else {
                    log::error!("Unable to multiply proportion_of_daily_reward_for_miner_fixed128 to restore it larger again");
                    return 0;
                }

                // println!("[rewards] block: {:#?}, miner: {:#?}, date_start: {:#?} restored_proportion_of_daily_reward_for_miner_u128: {:#?}", _n, miner_count, start_of_requested_date_millis, restored_proportion_of_daily_reward_for_miner_u128);

                let treasury_account_id: T::AccountId = <pallet_treasury::Pallet<T>>::account_id();
                let max_payout = pallet_balances::Pallet::<T>::usable_balance(treasury_account_id.clone());
                log::info!("Treasury account id: {:?}", treasury_account_id.clone());
                log::info!("Miner to receive reward: {:?}", miner.clone());
                log::info!("Treasury balance max payout: {:?}", max_payout.clone());

                let proportion_of_daily_reward_for_miner;
                let _proportion_of_daily_reward_for_miner = Self::convert_u128_to_balance(restored_proportion_of_daily_reward_for_miner_u128.clone());
                match _proportion_of_daily_reward_for_miner {
                    Err(_e) => {
                        log::error!("Unable to convert u128 to balance for proportion_of_daily_reward_for_miner");
                        return 0;
                    },
                    Ok(ref x) => {
                        proportion_of_daily_reward_for_miner = x;
                    }
                }

                let max_payout_as_u128;
                if let Some(_max_payout_as_u128) = TryInto::<u128>::try_into(max_payout).ok() {
                    max_payout_as_u128 = _max_payout_as_u128;
                } else {
                    log::error!("Unable to convert Balance to u128 for max_payout");
                    return 0;
                }
                log::info!("max_payout_as_u128: {:?}", max_payout_as_u128.clone());

                // Store output `rewards_allowance_dhx_remaining_today_as_u128` outside the loop
                let rewards_allowance_dhx_remaining_today_to_try = <RewardsAllowanceDHXForDate<T>>::get(&start_of_requested_date_millis);
                // Validate inputs so the daily_rewards is less or equal to the existing_allowance
                if let Some(_rewards_allowance_dhx_remaining_today_to_try) = rewards_allowance_dhx_remaining_today_to_try.clone() {
                    let _rewards_allowance_dhx_remaining_today_as_u128 = Self::convert_balance_to_u128(_rewards_allowance_dhx_remaining_today_to_try.clone());
                    match _rewards_allowance_dhx_remaining_today_as_u128.clone() {
                        Err(_e) => {
                            log::error!("Unable to convert balance to u128");
                            return 0;
                        },
                        Ok(x) => {
                            rewards_allowance_dhx_remaining_today_as_u128 = x;
                        }
                    }
                    log::info!("rewards_allowance_dhx_remaining_today_as_u128: {:?}", rewards_allowance_dhx_remaining_today_as_u128.clone());
                } else {
                    log::error!("Unable to retrieve balance from value provided.");
                    return 0;
                }

                // check if miner's reward is less than or equal to: rewards_allowance_dhx_daily_remaining
                if restored_proportion_of_daily_reward_for_miner_u128.clone() > 0u128 &&
                    rewards_allowance_dhx_remaining_today_as_u128.clone() >= restored_proportion_of_daily_reward_for_miner_u128.clone() &&
                    max_payout_as_u128.clone() >= restored_proportion_of_daily_reward_for_miner_u128.clone()
                {
                    // pay the miner their daily reward
                    info!("Paying the miner a proportion of the remaining daily reward allowance");

                    let tx_result;
                    let _tx_result = <T as Config>::Currency::transfer(
                        &treasury_account_id,
                        &miner.clone(),
                        proportion_of_daily_reward_for_miner.clone(),
                        ExistenceRequirement::KeepAlive
                    );
                    match _tx_result {
                        Err(_e) => {
                            log::error!("Unable to transfer from treasury to miner {:?}", miner.clone());
                            return 0;
                        },
                        Ok(ref x) => {
                            tx_result = x;
                        }
                    }
                    info!("Transfer to the miner tx_result: {:?}", tx_result.clone());

                    info!("Success paying the reward to the miner: {:?}", restored_proportion_of_daily_reward_for_miner_u128.clone());

                    // TODO - move into function `reduce_remaining_rewards_allowance_dhx_for_date`?

                    // Subtract, handling overflow
                    let new_rewards_allowance_dhx_remaining_today_as_u128;
                    let _new_rewards_allowance_dhx_remaining_today_as_u128 =
                        rewards_allowance_dhx_remaining_today_as_u128.clone().checked_sub(restored_proportion_of_daily_reward_for_miner_u128.clone());
                    match _new_rewards_allowance_dhx_remaining_today_as_u128 {
                        None => {
                            log::error!("Unable to subtract restored_proportion_of_daily_reward_for_miner_u128 from rewards_allowance_dhx_remaining_today due to StorageOverflow");
                            return 0;
                        },
                        Some(x) => {
                            new_rewards_allowance_dhx_remaining_today_as_u128 = x;
                        }
                    }

                    let new_rewards_allowance_dhx_remaining_today;
                    let _new_rewards_allowance_dhx_remaining_today = Self::convert_u128_to_balance(new_rewards_allowance_dhx_remaining_today_as_u128.clone());
                    match _new_rewards_allowance_dhx_remaining_today {
                        Err(_e) => {
                            log::error!("Unable to convert u128 to balance for new_rewards_allowance_dhx_remaining_today");
                            return 0;
                        },
                        Ok(ref x) => {
                            new_rewards_allowance_dhx_remaining_today = x;
                        }
                    }

                    // Write the new value to storage
                    <RewardsAllowanceDHXForDate<T>>::insert(
                        start_of_requested_date_millis.clone(),
                        new_rewards_allowance_dhx_remaining_today.clone(),
                    );

                    // println!("[paid] block: {:#?}, miner: {:#?}, date_start: {:#?} new_rewards_allowance_dhx_remaining_today: {:#?}", _n, miner_count, start_of_requested_date_millis, new_rewards_allowance_dhx_remaining_today);

                    // emit event with reward payment history rather than bloating storage
                    Self::deposit_event(Event::TransferredRewardsAllowanceDHXToMinerForDate(
                        start_of_requested_date_millis.clone(),
                        proportion_of_daily_reward_for_miner.clone(),
                        new_rewards_allowance_dhx_remaining_today.clone(),
                        miner.clone(),
                    ));

                    log::info!("TransferredRewardsAllowanceDHXToMinerForDate {:?} {:?} {:?} {:?}",
                        start_of_requested_date_millis.clone(),
                        proportion_of_daily_reward_for_miner.clone(),
                        new_rewards_allowance_dhx_remaining_today.clone(),
                        miner.clone(),
                    );

                    continue;
                } else {
                    log::error!("Insufficient remaining rewards allowance to pay daily reward to miner");

                    break;
                }
            }

            let rewards_allowance_dhx_remaining_today;
            let _rewards_allowance_dhx_remaining_today = Self::convert_u128_to_balance(rewards_allowance_dhx_remaining_today_as_u128.clone());
            match _rewards_allowance_dhx_remaining_today {
                Err(_e) => {
                    log::error!("Unable to convert u128 to balance for rewards_allowance_dhx_remaining_today");
                    return 0;
                },
                Ok(ref x) => {
                    rewards_allowance_dhx_remaining_today = x;
                }
            }

            <RewardsAllowanceDHXForDateDistributed<T>>::insert(
                start_of_requested_date_millis.clone(),
                true
            );

            // println!("[distributed] block: {:#?}, miner: {:#?}, date_start: {:#?} ", _n, miner_count, start_of_requested_date_millis);

            Self::deposit_event(Event::DistributedRewardsAllowanceDHXForDate(
                start_of_requested_date_millis.clone(),
                rewards_allowance_dhx_remaining_today.clone(),
            ));

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
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_registered_dhx_miner(origin: OriginFor<T>) -> DispatchResult {
            let _sender: T::AccountId = ensure_signed(origin)?;

            <RegisteredDHXMiners<T>>::append(_sender.clone());
            log::info!("register_dhx_miner - account_id: {:?}", &_sender);

            Self::deposit_event(Event::SetRegisteredDHXMiner(
                _sender.clone(),
            ));

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_min_bonded_dhx_daily(origin: OriginFor<T>, min_bonded_dhx_daily: BalanceOf<T>) -> DispatchResult {
            let _sender: T::AccountId = ensure_signed(origin)?;

            let min_bonded_dhx_daily_as_u128 = Self::convert_balance_to_u128(min_bonded_dhx_daily.clone())?;

            <MinBondedDHXDaily<T>>::put(&min_bonded_dhx_daily.clone());
            log::info!("set_min_bonded_dhx_daily: {:?}", &min_bonded_dhx_daily_as_u128);

            Self::deposit_event(Event::SetMinBondedDHXDailyStored(
                min_bonded_dhx_daily_as_u128.clone(),
                _sender.clone(),
            ));

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_cooling_off_period_days(origin: OriginFor<T>, cooling_off_period_days: u32) -> DispatchResult {
            let _sender: T::AccountId = ensure_signed(origin)?;

            <CoolingOffPeriodDays<T>>::put(&cooling_off_period_days.clone());
            log::info!("cooling_off_period_days: {:?}", &cooling_off_period_days);

            Self::deposit_event(Event::SetCoolingOffPeriodDaysStored(
                cooling_off_period_days.clone(),
            ));

            Ok(())
        }

        // customised by governance at any time. this function allows us to change it each year
        // https://docs.google.com/spreadsheets/d/1W2AzOH9Cs9oCR8UYfYCbpmd9X7hp-USbYXL7AuwMY_Q/edit#gid=970997021
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_bonded_dhx_of_account_for_date(origin: OriginFor<T>, account_id: T::AccountId) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            // Note: we DO need the following as we're using the current timestamp, rather than a function parameter.
            let timestamp: <T as pallet_timestamp::Config>::Moment = <pallet_timestamp::Pallet<T>>::get();
            let requested_date_as_u64 = Self::convert_moment_to_u64_in_milliseconds(timestamp.clone())?;
            log::info!("set_bonded_dhx_of_account_for_date - requested_date_as_u64: {:?}", requested_date_as_u64.clone());

            // convert the requested date/time to the start of that day date/time to signify that date for lookup
            // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
            let start_of_requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(requested_date_as_u64.clone())?;

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
            <BondedDHXForAccountForDate<T>>::insert(start_of_requested_date_millis.clone(), &bonded_data);
            log::info!("set_bonded_dhx_of_account_for_date - account_id: {:?}", &account_id);
            log::info!("set_bonded_dhx_of_account_for_date - bonded_data: {:?}", &bonded_data);

            // Emit an event.
            Self::deposit_event(Event::SetBondedDHXOfAccountForDateStored(
                start_of_requested_date_millis.clone(),
                bonded_data.clone(),
                account_id.clone(),
                _who.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // TODO: we need to change this in future so it is only modifiable by governance,
        // rather than just any user
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_allowance_dhx_daily(origin: OriginFor<T>, rewards_allowance: BalanceOf<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            // TODO - change to match with Ok/Error
            let rewards_allowance_as_u128 = Self::convert_balance_to_u128(rewards_allowance.clone())?;

            // Update storage
            <RewardsAllowanceDHXDaily<T>>::put(&rewards_allowance_as_u128);
            log::info!("set_rewards_allowance_dhx_daily - rewards_allowance: {:?}", &rewards_allowance_as_u128);

            // Emit an event.
            Self::deposit_event(Event::SetRewardsAllowanceDHXDailyStored(
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
            let start_of_requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(timestamp.clone())?;

            // Update storage. Override the default that may have been set in on_initialize
            <RewardsAllowanceDHXForDate<T>>::insert(start_of_requested_date_millis.clone(), &rewards_allowance);
            log::info!("set_rewards_allowance_dhx_for_date - rewards_allowance: {:?}", &rewards_allowance);

            // Emit an event.
            Self::deposit_event(Event::SetRewardsAllowanceDHXForDateStored(
                start_of_requested_date_millis.clone(),
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

            let start_of_requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(timestamp.clone())?;

            // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
            ensure!(<RewardsAllowanceDHXForDate<T>>::contains_key(&start_of_requested_date_millis), DispatchError::Other("Date key must exist to reduce allowance."));

            let existing_allowance_to_try = <RewardsAllowanceDHXForDate<T>>::get(&start_of_requested_date_millis);

            // Validate inputs so the daily_rewards is less or equal to the existing_allowance
            let existing_allowance_as_u128;
            if let Some(_existing_allowance_to_try) = existing_allowance_to_try.clone() {
                existing_allowance_as_u128 = Self::convert_balance_to_u128(_existing_allowance_to_try.clone())?;
                log::info!("change_remaining_rewards_allowance_dhx_for_date - existing_allowance_as_u128: {:?}", existing_allowance_as_u128.clone());
            } else {
                return Err(DispatchError::Other("Unable to retrieve balance from value provided"));
            }

            let daily_rewards_as_u128;
            daily_rewards_as_u128 = Self::convert_balance_to_u128(daily_rewards.clone())?;
            log::info!("change_remaining_rewards_allowance_dhx_for_date - daily_rewards_as_u128: {:?}", daily_rewards_as_u128.clone());

            ensure!(daily_rewards_as_u128 > 0u128, DispatchError::Other("Daily rewards must be greater than zero"));
            ensure!(existing_allowance_as_u128 >= daily_rewards_as_u128, DispatchError::Other("Daily rewards cannot exceed current remaining allowance"));

            let new_remaining_allowance_as_balance;
            if change == 0 {
                // Decrementing the value will error in the event of underflow.
                let new_remaining_allowance_as_u128 = existing_allowance_as_u128.checked_sub(daily_rewards_as_u128).ok_or(Error::<T>::StorageUnderflow)?;
                new_remaining_allowance_as_balance = Self::convert_u128_to_balance(new_remaining_allowance_as_u128.clone())?;
                log::info!("change_remaining_rewards_allowance_dhx_for_date - Decreasing rewards_allowance_dhx_for_date at Date: {:?}", &start_of_requested_date_millis);
            } else {
                // Incrementing the value will error in the event of overflow.
                let new_remaining_allowance_as_u128 = existing_allowance_as_u128.checked_add(daily_rewards_as_u128).ok_or(Error::<T>::StorageOverflow)?;
                new_remaining_allowance_as_balance = Self::convert_u128_to_balance(new_remaining_allowance_as_u128.clone())?;
                log::info!("change_remaining_rewards_allowance_dhx_for_date - Increasing rewards_allowance_dhx_for_date at Date: {:?}", &start_of_requested_date_millis);
            }

            // Update storage
            <RewardsAllowanceDHXForDate<T>>::mutate(
                &start_of_requested_date_millis,
                |allowance| {
                    if let Some(_allowance) = allowance {
                        *_allowance = new_remaining_allowance_as_balance.clone();
                    }
                },
            );

            // Emit an event.
            Self::deposit_event(Event::ChangedRewardsAllowanceDHXForDateStored(
                start_of_requested_date_millis.clone(),
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
            log::info!("convert_u64_in_milliseconds_to_start_of_date - date_as_u64_secs: {:?}", date_as_u64_secs.clone());

            let date_start_millis = date.and_hms(0, 0, 0).timestamp() * 1000;
            log::info!("convert_u64_in_milliseconds_to_start_of_date - date_start_millis: {:?}", date_start_millis.clone());
            log::info!("convert_u64_in_milliseconds_to_start_of_date - Timestamp requested Date: {:?}", date);
            return Ok(date_start_millis);
        }

        fn convert_balance_to_u128(balance: BalanceOf<T>) -> Result<u128, DispatchError> {
            let balance_as_u128;

            if let Some(_balance_as_u128) = TryInto::<u128>::try_into(balance).ok() {
                balance_as_u128 = _balance_as_u128;
            } else {
                return Err(DispatchError::Other("Unable to convert Balance to u128 for balance"));
            }
            log::info!("convert_balance_to_u128 balance_as_u128 - {:?}", balance_as_u128.clone());

            return Ok(balance_as_u128);
        }

        fn convert_balance_to_u128_from_pallet_balance(balance: BalanceFromBalancePallet<T>) -> Result<u128, DispatchError> {
            let balance_as_u128;

            if let Some(_balance_as_u128) = TryInto::<u128>::try_into(balance).ok() {
                balance_as_u128 = _balance_as_u128;
            } else {
                return Err(DispatchError::Other("Unable to convert Balance to u128 for balance"));
            }
            log::info!("convert_balance_to_u128_from_pallet_balance - balance_as_u128: {:?}", balance_as_u128.clone());

            return Ok(balance_as_u128);
        }

        fn convert_u128_to_balance(balance_as_u128: u128) -> Result<BalanceOf<T>, DispatchError> {
            let balance;
            if let Some(_balance) = TryInto::<BalanceOf<T>>::try_into(balance_as_u128).ok() {
                balance = _balance;
            } else {
                return Err(DispatchError::Other("Unable to convert u128 to Balance for balance"));
            }
            log::info!("convert_u128_to_balance balance - {:?}", balance.clone());

            return Ok(balance);
        }

        fn convert_blocknumber_to_u64(blocknumber: T::BlockNumber) -> Result<u64, DispatchError> {
            let mut blocknumber_u64 = 0u64;
            if let Some(_blocknumber_u64) = TryInto::<u64>::try_into(blocknumber).ok() {
                blocknumber_u64 = _blocknumber_u64;
            } else {
                log::error!("Unable to convert BlockNumber to u64");
            }
            log::info!("blocknumber_u64: {:?}", blocknumber_u64.clone());

            return Ok(blocknumber_u64);
        }
    }
}
