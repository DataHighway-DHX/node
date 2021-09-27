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
            u32, // days remaining
            // 0: unbonded (i.e. never bonded, or finished cool-down period and no longer bonding)
            // 1: bonded/bonding (i.e. waiting in the cool-down period before start getting rewards)
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
        pub registered_dhx_miners: Vec<T::AccountId>,
        pub min_bonded_dhx_daily: BalanceOf<T>,
        pub cooling_off_period_days: u32,
        pub cooling_off_period_days_remaining: Vec<(T::AccountId, (u32, u32))>,
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
                registered_dhx_miners: vec![
                    Default::default(),
                    Default::default(),
                ],
                min_bonded_dhx_daily: Default::default(),
                cooling_off_period_days: Default::default(),
                cooling_off_period_days_remaining: Default::default(),
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
            <MinBondedDHXDaily<T>>::put(&self.min_bonded_dhx_daily);
            <CoolingOffPeriodDays<T>>::put(&self.cooling_off_period_days);
            for (a, (b, c)) in &self.cooling_off_period_days_remaining {
                <CoolingOffPeriodDaysRemaining<T>>::insert(a, (b, c));
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

            // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
            let contains_key = <RewardsAllowanceDHXForDate<T>>::contains_key(&start_of_requested_date_millis);
            let mut is_key_added = false;
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
                log::info!("start_of_requested_date_millis: {:?}", start_of_requested_date_millis.clone());
                log::info!("rewards_allowance: {:?}", &rewards_allowance_dhx_daily);
                is_key_added = true;
            }

            // check again whether the start_of_requested_date has been added to storage
            if is_key_added == false {
                log::error!("Unable to add start_of_requested_date to storage");
                return 0;
            }

            // only run the following once per day until rewards_allowance_dhx_for_date is exhausted
            let is_already_distributed = <RewardsAllowanceDHXForDateDistributed<T>>::get(start_of_requested_date_millis.clone());
            if is_already_distributed == Some(true) {
                log::error!("Unable to distribute further rewards allowance today");
                return 0;
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
            // TODO - iterate through the registered miners in random order, otherwise the same miners get the rewards each day
            // and possibly the same miners miss out. if the miners at the start of the list have large rewards they
            // could possibly exhaust the daily allocation of rewards just by themselves each day
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

                let locked: BalanceLock<<T as pallet_balances::Config>::Balance> =
                    <pallet_balances::Pallet<T>>::locks(miner.clone()).into_inner().clone()[0];
                log::info!("miner locks {:#?}", locked.clone());

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

                // assume DHX Miner only has one lock for simplicity. retrieve the amount locked
                // TODO - isn't there a vector of locked amounts?
                let locks_first_amount = 10u128;

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
                if locks_first_amount > min_bonded_dhx_daily_u128 {
                    is_bonding_min_dhx = true;
                }

                let cooling_off_period_days;
                let cooling_off_period_days_to_try = <CoolingOffPeriodDays<T>>::get();
                if let Some(_cooling_off_period_days_to_try) = cooling_off_period_days_to_try {
                    cooling_off_period_days = _cooling_off_period_days_to_try;
                } else {
                    log::error!("Unable to retrieve cooling off period days");
                    return 0;
                }

                let cooling_off_period_days_remaining;
                let cooling_off_period_days_remaining_to_try = <CoolingOffPeriodDaysRemaining<T>>::get(miner.clone());
                if let Some(_cooling_off_period_days_remaining_to_try) = cooling_off_period_days_remaining_to_try {
                    cooling_off_period_days_remaining = _cooling_off_period_days_remaining_to_try;
                } else {
                    log::error!("Unable to retrieve cooling off period days remaining for given miner");
                    return 0;
                }
                // if cooling_off_period_days_remaining.1 is 0u32, it means we haven't recognised they that are bonding yet (unbonded),
                // they aren't currently bonding, they haven't started cooling off to start bonding,
                // or have already finished cooling down after bonding.
                // so if we detect they are now bonding above the min. then we should start at max. remaining days
                // before starting to decrement on subsequent blocks
                if cooling_off_period_days_remaining.1 == 0u32 && is_bonding_min_dhx == true {
                    <CoolingOffPeriodDaysRemaining<T>>::insert(
                        miner.clone(),
                        (
                            cooling_off_period_days.clone(),
                            1u32, // they are bonded again, waiting to start getting rewards
                        ),
                    );
                // if cooling_off_period_days_remaining.0 is Some(above 0), then decrement, but not eligible yet for rewards.
                } else if cooling_off_period_days_remaining.0 > 0 && is_bonding_min_dhx == true {
                    let old_cooling_off_period_days_remaining = cooling_off_period_days_remaining.0.clone();

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
                            new_cooling_off_period_days_remaining.clone(),
                            1u32, // they are bonded again, waiting to start getting rewards
                        ),
                    );
                // if cooling_off_period_days_remaining.0 is Some(0),
                // and if cooling_off_period_days_remaining.1 is 0
                // and then no more cooling off days, but don't decrement,
                // and say they are eligible for reward payments
                } else if
                    cooling_off_period_days_remaining.0 == 0u32 &&
                    cooling_off_period_days_remaining.1 == 0u32 &&
                    is_bonding_min_dhx == true
                {
                    let rewards_allowance_dhx_daily;
                    let rewards_allowance_dhx_daily_to_try = <RewardsAllowanceDHXDaily<T>>::get();
                    if let Some(_rewards_allowance_dhx_daily_to_try) = rewards_allowance_dhx_daily_to_try {
                        rewards_allowance_dhx_daily = _rewards_allowance_dhx_daily_to_try;
                    } else {
                        log::error!("Unable to retrieve rewards_allowance_dhx_daily");
                        return 0;
                    }

                    // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
                    if <RewardsAllowanceDHXForDate<T>>::contains_key(&start_of_requested_date_millis) == false {
                        log::error!("Date key must exist to check its remaining allowance.");
                        return 0;
                    }

                    let existing_allowance_to_try = <RewardsAllowanceDHXForDate<T>>::get(&start_of_requested_date_millis);

                    // Validate inputs so the daily_rewards is less or equal to the existing_allowance
                    let existing_allowance_as_u128;
                    if let Some(_existing_allowance_to_try) = existing_allowance_to_try.clone() {
                        let _existing_allowance_as_u128 = Self::convert_balance_to_u128(_existing_allowance_to_try.clone());
                        match _existing_allowance_as_u128.clone() {
                            Err(_e) => {
                                log::error!("Unable to convert balance to u128");
                                return 0;
                            },
                            Ok(x) => {
                                existing_allowance_as_u128 = x;
                            }
                        }
                        log::info!("existing_allowance_as_u128: {:?}", existing_allowance_as_u128.clone());
                    } else {
                        log::error!("Unable to retrieve balance from value provided.");
                        return 0;
                    }

                    let rewards_allowance_dhx_remaining_today_as_u128 = existing_allowance_as_u128.clone();

                    // TODO - calculate the miner's reward for this day, as a proportion taking other eligible miner's
                    // who are eligible for daily rewards into account since we want to split them fairly
                    let daily_reward_for_miner_as_u128 = 100u128; // hard coded

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

                    let treasury_account_id: T::AccountId = <pallet_treasury::Pallet<T>>::account_id();
                    let max_payout = pallet_balances::Pallet::<T>::usable_balance(treasury_account_id.clone());
                    log::info!("Treasury account id: {:?}", treasury_account_id.clone());
                    log::info!("Miner to receive reward: {:?}", miner.clone());
                    log::info!("Treasury balance max payout: {:?}", max_payout.clone());

                    // let daily_reward_for_miner_as_u128;
                    // if let Some(_daily_reward_for_miner_as_u128) = TryInto::<u128>::try_into(daily_reward_for_miner).ok() {
                    //     daily_reward_for_miner_as_u128 = _daily_reward_for_miner_as_u128;
                    // } else {
                    //     log::error!("Unable to convert Balance to u128 for daily_reward_for_miner");
                    //     return ();
                    // }
                    // log::info!("daily_reward_for_miner_as_u128: {:?}", daily_reward_for_miner_as_u128.clone());

                    let max_payout_as_u128;
                    if let Some(_max_payout_as_u128) = TryInto::<u128>::try_into(max_payout).ok() {
                        max_payout_as_u128 = _max_payout_as_u128;
                    } else {
                        log::error!("Unable to convert Balance to u128 for max_payout");
                        return 0;
                    }
                    log::info!("max_payout_as_u128: {:?}", max_payout_as_u128.clone());

                    // check if miner's reward is less than or equal to: rewards_allowance_dhx_daily_remaining
                    if daily_reward_for_miner_as_u128.clone() > 0u128 &&
                        rewards_allowance_dhx_remaining_today_as_u128.clone() >= daily_reward_for_miner_as_u128.clone() &&
                        max_payout_as_u128.clone() >= daily_reward_for_miner_as_u128.clone()
                    {
                        // pay the miner their daily reward
                        info!("Paying the miner a proportion of the remaining daily reward allowance");

                        <T as Config>::Currency::transfer(
                            &treasury_account_id,
                            &miner.clone(),
                            daily_reward_for_miner.clone(),
                            ExistenceRequirement::KeepAlive
                        );

                        info!("Success paying the reward to the miner: {:?}", daily_reward_for_miner_as_u128.clone());

                        // TODO - move into function `reduce_remaining_rewards_allowance_dhx_for_date`?

                        // Subtract, handling overflow
                        let new_rewards_allowance_dhx_remaining_today_as_u128;
                        let _new_rewards_allowance_dhx_remaining_today_as_u128 =
                            rewards_allowance_dhx_remaining_today_as_u128.clone().checked_sub(daily_reward_for_miner_as_u128.clone());
                        match _new_rewards_allowance_dhx_remaining_today_as_u128 {
                            None => {
                                log::error!("Unable to subtract daily_reward_for_miner from rewards_allowance_dhx_remaining_today due to StorageOverflow");
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

                        // emit event with reward payment history rather than bloating storage
                        Self::deposit_event(Event::TransferredRewardsAllowanceDHXToMinerForDate(
                            start_of_requested_date_millis.clone(),
                            daily_reward_for_miner.clone(),
                            new_rewards_allowance_dhx_remaining_today.clone(),
                            miner.clone(),
                        ));
                    } else {
                        log::error!("Insufficient remaining rewards allowance to pay daily reward to miner");

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

                        Self::deposit_event(Event::DistributedRewardsAllowanceDHXForDate(
                            start_of_requested_date_millis.clone(),
                            rewards_allowance_dhx_remaining_today.clone(),
                        ));

                        return 0;
                    }
                // if they stop bonding the min dhx, and
                // if cooling_off_period_days_remaining.0 is Some(0),
                // and if cooling_off_period_days_remaining.1 is 1 (where they had just been bonding and getting rewards)
                // so since we detected they are no longer bonding above the min. then we should start at max. remaining days
                // before starting to decrement on subsequent blocks
                } else if
                    cooling_off_period_days_remaining.0 == 0u32 &&
                    cooling_off_period_days_remaining.1 == 1u32 &&
                    is_bonding_min_dhx == false
                {
                    // Write the new value to storage
                    <CoolingOffPeriodDaysRemaining<T>>::insert(
                        miner.clone(),
                        (
                            cooling_off_period_days.clone(),
                            2u32, // they have unbonded again, waiting to finish cooling down period
                        ),
                    );
                // if cooling_off_period_days_remaining.0 is Some(above 0), then decrement,
                // but not yet completely unbonded so cannot withdraw yet
                // note: we don't care if they stop bonding below the min. dhx during the cooling off period,
                // as the user needs to learn that they should always been bonding the min. to
                // maintain rewards, otherwise they have to wait for entire cooling down period and
                // then the cooling off period again.
                //
                } else if cooling_off_period_days_remaining.0 > 0u32 &&
                    cooling_off_period_days_remaining.1 == 2u32
                    // && is_bonding_min_dhx == false
                {
                    let old_cooling_off_period_days_remaining = cooling_off_period_days_remaining.0.clone();

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
                            new_cooling_off_period_days_remaining.clone(),
                            2u32, // they have unbonded again, waiting to finish cooling down period
                        ),
                    );
                // if cooling_off_period_days_remaining.0 is Some(0), do not subtract anymore, they are
                // completely unbonded so can withdraw
                } else if cooling_off_period_days_remaining.0 == 0u32 &&
                    cooling_off_period_days_remaining.1 == 2u32
                    // && is_bonding_min_dhx == false
                {
                    // Write the new value to storage
                    <CoolingOffPeriodDaysRemaining<T>>::insert(
                        miner.clone(),
                        (
                            0u32,
                            0u32, // they are completely unbonded again
                        ),
                    );
                }
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
            log::info!("account_id: {:?}", &account_id);
            log::info!("bonded_data: {:?}", &bonded_data);

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

        // customised by governance at any time
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_allowance_dhx_daily(origin: OriginFor<T>, rewards_allowance: BalanceOf<T>) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let rewards_allowance_as_u128 = Self::convert_balance_to_u128(rewards_allowance.clone())?;

            // Update storage
            <RewardsAllowanceDHXDaily<T>>::put(&rewards_allowance_as_u128);
            log::info!("rewards_allowance: {:?}", &rewards_allowance_as_u128);

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
            log::info!("rewards_allowance: {:?}", &rewards_allowance);

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
                log::info!("Decreasing rewards_allowance_dhx_for_date at Date: {:?}", &start_of_requested_date_millis);
            } else {
                // Incrementing the value will error in the event of overflow.
                let new_remaining_allowance_as_u128 = existing_allowance_as_u128.checked_add(daily_rewards_as_u128).ok_or(Error::<T>::StorageOverflow)?;
                new_remaining_allowance_as_balance = Self::convert_u128_to_balance(new_remaining_allowance_as_u128.clone())?;
                log::info!("Increasing rewards_allowance_dhx_for_date at Date: {:?}", &start_of_requested_date_millis);
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
