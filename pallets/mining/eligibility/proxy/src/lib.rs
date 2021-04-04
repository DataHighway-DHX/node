#![cfg_attr(not(feature = "std"), no_std)]

use account_set::AccountSet;
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
    debug,
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
use frame_system::ensure_signed;
use module_primitives::{
    constants::time::MILLISECS_PER_BLOCK,
    types::*,
};
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
    prelude::*,
};
use substrate_fixed::types::U32F32;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + roaming_operators::Trait
    + pallet_treasury::Trait
    + pallet_balances::Trait
    + pallet_timestamp::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    // Loosely coupled
    type MembershipSource: AccountSet<AccountId = Self::AccountId>;
    type MiningEligibilityProxyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RewardsOfDay: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type Date = i64;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive())]
pub struct MiningEligibilityProxy(pub [u8; 16]);

#[derive(Encode, Decode, Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive())]
pub struct MiningEligibilityProxyRewardRequest<U, V, W, X> {
    pub proxy_claim_requestor_account_id: U, /* Supernode (proxy) account id requesting DHX rewards as proxy to
                                              * distribute to its miners */
    pub proxy_claim_total_reward_amount: V,
    pub proxy_claim_rewardees_data: W,
    pub proxy_claim_timestamp_redeemed: X,
}

#[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive())]
pub struct MiningEligibilityProxyClaimRewardeeData<U, V, W, X> {
    pub proxy_claim_rewardee_account_id: U, // Rewardee miner associated with supernode (proxy) account id
    pub proxy_claim_reward_amount: V,       // Reward in DHX tokens for specific rewardee miner
    pub proxy_claim_start_date: W,          // Start date associated with mining claim
    pub proxy_claim_end_date: X,            // Blocks after start date covered by claim requesting mining rewards
}

#[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive())]
pub struct RewardRequestorData<U, V, W, X, Y> {
    pub mining_eligibility_proxy_id: U,
    pub total_amt: V,
    pub rewardee_count: W,
    pub member_kind: X,
    pub requested_date: Y,
}

#[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive())]
pub struct RewardTransferData<U, V, W, X, Y, Z> {
    pub mining_eligibility_proxy_id: U,
    pub is_sent: V,
    pub total_amt: W,
    pub rewardee_count: X,
    pub member_kind: Y,
    pub requested_date: Z,
}

#[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive())]
pub struct RewardDailyData<U, V, W, X, Y> {
    pub mining_eligibility_proxy_id: U,
    pub total_amt: V,
    // Assume that the requestor is also the recipient
    pub proxy_claim_requestor_account_id: W,
    pub member_kind: X,
    pub rewarded_date: Y,
}

type RewardeeData<T> =
    MiningEligibilityProxyClaimRewardeeData<<T as frame_system::Trait>::AccountId, BalanceOf<T>, Date, Date>;

type RequestorData<T> = RewardRequestorData<
    <T as Trait>::MiningEligibilityProxyIndex,
    BalanceOf<T>,
    u64,
    u32,
    <T as pallet_timestamp::Trait>::Moment,
>;

type TransferData<T> = RewardTransferData<
    <T as Trait>::MiningEligibilityProxyIndex,
    bool,
    BalanceOf<T>,
    u64,
    u32,
    <T as pallet_timestamp::Trait>::Moment,
>;

type DailyData<T> = RewardDailyData<
    <T as Trait>::MiningEligibilityProxyIndex,
    BalanceOf<T>,
    <T as frame_system::Trait>::AccountId,
    u32,
    Date,
>;

decl_event!(
    pub enum Event<T> where
        AccountId = <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningEligibilityProxyIndex,
        BalanceOf = BalanceOf<T>,
        RewardeeData = RewardeeData<T>,
        RequestorData = RequestorData<T>,
        TransferData = TransferData<T>,
        DailyData = DailyData<T>,
        <T as Trait>::RewardsOfDay,
    {
        Created(AccountId, MiningEligibilityProxyIndex),
        MiningEligibilityProxyRewardRequestSet(
            AccountId,
            MiningEligibilityProxyIndex,
            BalanceOf,
            Vec<RewardeeData>,
            Date,
        ),
        MiningEligibilityProxyRewardRequestorSet(
            AccountId,
            RequestorData,
        ),
        MiningEligibilityProxyRewardTransferSet(
            AccountId,
            TransferData,
        ),
        RewardsPerDaySet(
            Date,
            DailyData,
        ),
        RewardsOfDayCalculated(RewardsOfDay),
        IsAMember(AccountId),
        /// Substrate-fixed total rewards for a given day has been updated.
        TotalRewardsPerDayUpdated(BalanceOf, Date, AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        NoneValue,
        /// Some math operation overflowed
        Overflow,
    }
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningEligibilityProxy {
        /// Stores all the mining_eligibility_proxys, key is the mining_eligibility_proxy id / index
        pub MiningEligibilityProxys get(fn mining_eligibility_proxy): map hasher(opaque_blake2_256) T::MiningEligibilityProxyIndex => Option<MiningEligibilityProxy>;

        /// Stores the total number of mining_eligibility_proxys. i.e. the next mining_eligibility_proxy index
        pub MiningEligibilityProxyCount get(fn mining_eligibility_proxy_count): T::MiningEligibilityProxyIndex;

        /// Stores mining_eligibility_proxy owner
        pub MiningEligibilityProxyOwners get(fn mining_eligibility_proxy_owner): map hasher(opaque_blake2_256) T::MiningEligibilityProxyIndex => Option<T::AccountId>;

        /// Stores mining_eligibility_proxy_reward_request
        pub MiningEligibilityProxyRewardRequests get(fn mining_eligibility_proxy_eligibility_reward_requests): map hasher(opaque_blake2_256) T::MiningEligibilityProxyIndex =>
            Option<MiningEligibilityProxyRewardRequest<
                T::AccountId,
                BalanceOf<T>,
                Vec<RewardeeData<T>>, // TODO - change to store MiningEligibilityProxyRewardeeIndex that is created to store then instead
                <T as pallet_timestamp::Trait>::Moment,
            >>;

        // IGNORE - seems unnessary since can get from MiningEligibilityProxyRewardRequests
        // pub MiningEligibilityProxyRewardees get(fn mining_eligibility_proxy_eligibility_rewardees): map hasher(opaque_blake2_256) T::MiningEligibilityProxyRewardeeIndex =>
        //     Option<MiningEligibilityProxyRewardee<
        //         u64,                        // reward_hash
        //         T::AccountId,               // requestor_account_id
        //         BalanceOf<T>,               // reward_amount
        //         T::Moment,                  // timestamp
        //         T::BlockNumber,             // block_start
        //         T::BlockNumber,             // block_interval
        //     >>;

        /// Stores reward_requests for given rewardee
        ///
        /// requestor_acct_id > (reward_hash, total_amt, rewardee_count, member_kind, date)
        /// where member_kind is either supernode_center 1 or supernode 2 or individual 3
        pub MiningEligibilityProxyRewardRequestors get(fn reward_requestors):
            map hasher(opaque_blake2_256) T::AccountId =>
                Option<Vec<RewardRequestorData<
                    <T as Trait>::MiningEligibilityProxyIndex,
                    BalanceOf<T>,
                    u64,
                    u32,
                    <T as pallet_timestamp::Trait>::Moment,
                >>>;

        /// Stores reward_transfers for given rewardee
        /// IMPORTANT NOTE: REQUESTOR MAY DIFFER FROM REQUESTEE
        /// HENCE DIFFERENT STORAGE FROM `reward_requestors`
        ///
        /// rewardee_acct_id > (reward_hash, bool, total_amt, rewardee_count, member_kind, date)
        pub MiningEligibilityProxyRewardTransfers get(fn reward_transfers):
            map hasher(opaque_blake2_256) T::AccountId =>
                Option<Vec<RewardTransferData<
                    <T as Trait>::MiningEligibilityProxyIndex,
                    bool,
                    BalanceOf<T>,
                    u64,
                    u32,
                    <T as pallet_timestamp::Trait>::Moment,
                >>>;

        /// Substrate-fixed, value starts at 0 (additive identity)
        pub TotalRewardsPerDay get(fn total_rewards_daily):
            map hasher(opaque_blake2_256) Date => Option<BalanceOf<T>>;

        /// Stores accumulation of daily_rewards_sent on a given date
        /// Note: Must store date/time value as the same for all rewards sent on same day
        pub RewardsPerDay get(fn rewards_daily):
            map hasher(opaque_blake2_256) Date =>
                Option<Vec<RewardDailyData<
                    <T as Trait>::MiningEligibilityProxyIndex,
                    BalanceOf<T>,
                    <T as frame_system::Trait>::AccountId,
                    u32,
                    Date,
                >>>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: <T as frame_system::Trait>::Origin {
        fn deposit_event() = default;

        // /// Emits an event with the calculated rewards for a given day.
        // /// A separate custom RPC endpoint is a preferred alternative to return this data to the user
        // #[weight = 10_000 + T::DbWeight::get().writes(1)]
        // pub fn calc_rewards_of_day(
        //     origin,
        //     _proxy_claim_reward_day: Option<T::Moment>,
        // ) -> Result<(), DispatchError> {
        //     let sender = ensure_signed(origin)?;

        //     if let Some(reward_day) = _proxy_claim_reward_day {
        //         debug::info!("Retrieving total rewards of day {:#?}", reward_day);

        //         let _rewards_for_reward_day = Self::rewards_daily(&reward_day);

        //         if let Some(rewards_for_reward_day) = _rewards_for_reward_day {
        //             let mut total_daily_rewards: BalanceOf<T> = 0.into();
        //             let rewards_for_reward_day_count = rewards_for_reward_day.len();
        //             let mut reward_data_current_index = 0;

        //             for (index, reward_data) in rewards_for_reward_day.iter().enumerate() {
        //                 reward_data_current_index += 1;
        //                 debug::info!("reward_data_current_index {:#?}", reward_data_current_index);

        //                 if let _total_amt = reward_data.total_amt {
        //                     total_daily_rewards += _total_amt;
        //                 } else {
        //                     continue;
        //                 }
        //             }

        //             let _total_daily_rewards_to_try = TryInto::<u32>::try_into(total_daily_rewards).ok();
        //             if let Some(total_daily_rewards_to_try) = _total_daily_rewards_to_try {
        //                 Self::deposit_event(RawEvent::RewardsOfDayCalculated(total_daily_rewards_to_try.into()));
        //                 return Ok(())
        //             } else {
        //                 return Err(DispatchError::Other("Unable to convert Balance to u64 to calculate daily rewards"));
        //             }

        //         } else {
        //             return Err(DispatchError::Other("Invalid rewards_for_reward_day data"));
        //         }
        //     } else {
        //         return Err(DispatchError::Other("Invalid reward_day provided"));
        //     }
        // }

        /// Transfer tokens claimed by the Supernode Centre on behalf of a Supernode from the
        /// on-chain DHX DAO unlocked reserves of the Treasury account to the Supernode Centre's address,
        /// but only if the claimed amount is deemed reasonable and if there is valid data
        /// provided about the recipient accounts associated with the Supernode.
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn proxy_eligibility_claim(
            origin,
            _proxy_claim_total_reward_amount: BalanceOf<T>,
            _proxy_claim_rewardees_data: Option<Vec<RewardeeData<T>>>,
        ) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;

            // get the current block & current date/time
            let current_block = <frame_system::Module<T>>::block_number();
            let requested_date = <pallet_timestamp::Module<T>>::get();

            ensure!(Self::is_origin_whitelisted_member_supernodes(sender.clone()).is_ok(), "Only whitelisted Supernode account members may request proxy rewards");

            let member_kind = T::MembershipSource::account_kind(sender.clone());
            debug::info!("Requestor account kind: {:?}", member_kind.clone());

            // TODO - determine whether we'll allow the recipient to be provided by the sender
            // and how to restrict who the recipients are by membership or similar
            let recipient_member_kind = T::MembershipSource::account_kind(sender.clone());
            debug::info!("Recipient account kind: {:?}", recipient_member_kind.clone());

            let mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex;
            match Self::create(sender.clone()) {
                Ok(proxy_id) => {
                    mining_eligibility_proxy_id = proxy_id.into();
                },
                Err(_) => {
                    return Err(DispatchError::Other("Proxy claim rewardees data missing"));
                }
            }

            // TODO
            // ensure!(Self::is_supernode_claim_reasonable(_proxy_claim_total_reward_amount).is_ok(), "Supernode claim has been deemed unreasonable");

            if let Some(rewardees_data) = _proxy_claim_rewardees_data {
                // TODO
                Self::is_valid_reward_data(rewardees_data.clone());

                debug::info!("Transferring claim to proxy Supernode");
                // Distribute the reward to the account that has locked the funds
                let treasury_account_id: T::AccountId = <pallet_treasury::Module<T>>::account_id();
                // Only available in Substrate 3 is pot()
                // let max_payout = pallet_treasury::Module::<T>::pot();
                let max_payout = pallet_balances::Module::<T>::usable_balance(treasury_account_id.clone());
                debug::info!("Treasury account id: {:?}", treasury_account_id.clone());
                debug::info!("Requestor to receive reward: {:?}", sender.clone());
                debug::info!("Treasury balance max payout: {:?}", max_payout.clone());

                let reward_to_pay_as_balance_to_try = TryInto::<BalanceOf<T>>::try_into(_proxy_claim_total_reward_amount).ok();
                if let Some(reward_to_pay) = reward_to_pay_as_balance_to_try {
                    // ensure!(max_payout > reward_to_pay, "Reward cannot exceed treasury balance");

                    // Store Requestor of the reward

                    let _rewardees_data_len: usize = rewardees_data.len();
                    // Try to convert usize into u64
                    // note: rewardees_data_len.clone().try_into().unwrap(),
                    let rewardees_data_len_to_try = TryInto::<u64>::try_into(_rewardees_data_len).ok();

                    if let Some(rewardees_data_len) = rewardees_data_len_to_try {
                        let reward_requestor_data: RequestorData<T> = RewardRequestorData {
                            mining_eligibility_proxy_id: mining_eligibility_proxy_id.clone(),
                            total_amt: reward_to_pay.clone(),
                            rewardee_count: rewardees_data_len.clone(),
                            member_kind: member_kind.clone(),
                            requested_date: requested_date.clone(),
                        };

                        debug::info!("Setting the proxy eligibility reward requestor");

                        Self::insert_mining_eligibility_proxy_reward_requestor(
                            &sender.clone(),
                            reward_requestor_data.clone(),
                        );

                        debug::info!("Inserted reward Requestor: {:?}", sender.clone());
                        debug::info!("Inserted reward Requestor Data: {:?}", reward_requestor_data.clone());

                        debug::info!("Treasury paying reward");

                        <T as Trait>::Currency::transfer(
                            &treasury_account_id,
                            &sender,
                            reward_to_pay.clone(),
                            ExistenceRequirement::KeepAlive
                        );

                        debug::info!("Success paying the reward amount: {:?}", reward_to_pay.clone());

                        let requested_date = <pallet_timestamp::Module<T>>::get();

                        // convert the current date/time to the start of the current day date/time.
                        // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000

                        let requested_date_as_u64;
                        if let Some(_requested_date_as_u64) = TryInto::<u64>::try_into(requested_date).ok() {
                            requested_date_as_u64 = _requested_date_as_u64;
                        } else {
                            return Err(DispatchError::Other("Unable to convert Moment to u64 for requested_date"));
                        }
                        let sent_date = NaiveDateTime::from_timestamp(i64::try_from(requested_date_as_u64.clone() / 1000u64).unwrap(), 0).date();

                        debug::info!("Timestamp sent Date: {:?}", sent_date);
                        // check if the start of the current day date/time entry exists as a key for `rewards_daily`
                        //
                        // if so, retrieve the latest `rewards_daily` data stored for the start of that day date/time
                        // i.e. (account_id, balance_rewarded, block_number), and add the new reward value to it.
                        //
                        // else just insert that as a new entry

                        let reward_amount_item: DailyData<T> = RewardDailyData {
                            mining_eligibility_proxy_id: mining_eligibility_proxy_id.clone(),
                            total_amt: _proxy_claim_total_reward_amount.clone(),
                            proxy_claim_requestor_account_id: sender.clone(),
                            member_kind: recipient_member_kind.clone(),
                            rewarded_date: sent_date.and_hms(0, 0, 0).timestamp(),
                        };

                        debug::info!("Appended new rewards_per_day storage item");

                        <RewardsPerDay<T>>::append(
                            sent_date.and_hms(0, 0, 0).timestamp(),
                            reward_amount_item.clone(),
                        );

                        debug::info!("Appended new rewards_per_day at Date: {:?}", sent_date);
                        debug::info!("Appended new rewards_per_day in storage item: {:?}", reward_amount_item.clone());

                        let rewards_per_day_retrieved = <RewardsPerDay<T>>::get(
                            sent_date.and_hms(0, 0, 0).timestamp(),
                        );
                        debug::info!("Retrieved new rewards_per_day storage item: {:?}", rewards_per_day_retrieved.clone());

                        // Update in storage the total rewards distributed so far for the current day
                        // so users may query state and have the latest calculated total returned.
                        match Self::total_rewards_daily(sent_date.and_hms(0, 0, 0).timestamp()) {
                            None => {
                                debug::info!("Creating new total rewards entry for a given day");

                                <TotalRewardsPerDay<T>>::insert(
                                    sent_date.and_hms(0, 0, 0).timestamp(),
                                    _proxy_claim_total_reward_amount.clone(),
                                );

                                debug::info!("Created new total_rewards_daily at Date: {:?}",  sent_date.and_hms(0, 0, 0).timestamp());
                                debug::info!("Creating new total_rewards_daily at Date with Amount: {:?}", _proxy_claim_total_reward_amount.clone());

                                // Emit event
                                Self::deposit_event(RawEvent::TotalRewardsPerDayUpdated(
                                    _proxy_claim_total_reward_amount.clone(),
                                    sent_date.and_hms(0, 0, 0).timestamp(),
                                    sender.clone(),
                                ));
                            },
                            Some(old_total_rewards_for_day) => {
                                debug::info!("TotalRewardsPerDay entry mapping already exists for given day. Updating...");

                                // Add, handling overflow
                                let new_total_rewards_for_day =
                                    old_total_rewards_for_day.checked_add(&_proxy_claim_total_reward_amount.clone()).ok_or(Error::<T>::Overflow)?;
                                // Write the new value to storage
                                <TotalRewardsPerDay<T>>::mutate(
                                    sent_date.and_hms(0, 0, 0).timestamp(),
                                    |reward_moment| {
                                        if let Some(_reward_moment) = reward_moment {
                                            *_reward_moment = new_total_rewards_for_day.clone();
                                        }

                                        debug::info!("Updated total_rewards_daily at Date: {:?}",  sent_date);
                                        debug::info!("Updated total_rewards_daily at Date. Existing Amount: {:?}", old_total_rewards_for_day.clone());
                                        debug::info!("Updated total_rewards_daily at Date. Reward Amount: {:?}", _proxy_claim_total_reward_amount.clone());
                                        debug::info!("Updated total_rewards_daily at Date. New Amount: {:?}", new_total_rewards_for_day.clone());
                                    },
                                );

                                // Emit event
                                Self::deposit_event(RawEvent::TotalRewardsPerDayUpdated(
                                    new_total_rewards_for_day.clone(),
                                    sent_date.and_hms(0, 0, 0).timestamp(),
                                    sender.clone(),
                                ));
                            }
                        }

                        let reward_transfer_data: TransferData<T> = RewardTransferData {
                            mining_eligibility_proxy_id: mining_eligibility_proxy_id.clone(),
                            is_sent: true,
                            total_amt: reward_to_pay.clone(),
                            rewardee_count: rewardees_data_len.clone(),
                            member_kind: member_kind.clone(),
                            requested_date: requested_date.clone(),
                        };

                        debug::info!("Setting the proxy eligibility reward transfer");

                        Self::insert_mining_eligibility_proxy_reward_transfer(
                            &sender.clone(),
                            reward_transfer_data.clone(),
                        );

                        debug::info!("Inserted proxy_reward_transfer for Sender: {:?}", sender.clone());
                        debug::info!("Inserted proxy_reward_transfer for Sender with Data: {:?}", reward_transfer_data.clone());

                        let reward_daily_data: DailyData<T> = RewardDailyData {
                            mining_eligibility_proxy_id: mining_eligibility_proxy_id.clone(),
                            total_amt: reward_to_pay.clone(),
                            proxy_claim_requestor_account_id: sender.clone(),
                            member_kind: member_kind.clone(),
                            rewarded_date: sent_date.and_hms(0, 0, 0).timestamp(),
                        };

                        debug::info!("Setting the proxy eligibility reward daily");

                        // FIXME - get the time right at the start of the day that `reward_daily_data`
                        // corresponds to and only store that.
                        Self::insert_mining_eligibility_proxy_reward_daily(
                            &sent_date.and_hms(0, 0, 0).timestamp(),
                            reward_daily_data.clone(),
                        );

                        debug::info!("Inserted proxy_reward_daily for Moment: {:?}", requested_date.clone());
                        debug::info!("Inserted proxy_reward_daily for Moment with Data: {:?}", reward_daily_data.clone());

                    }
                }

                debug::info!("Setting the proxy eligibility results");

                Self::set_mining_eligibility_proxy_eligibility_result(
                    sender.clone(),
                    mining_eligibility_proxy_id.clone(),
                    _proxy_claim_total_reward_amount.clone(),
                    rewardees_data.clone(),
                );

                debug::info!("Inserted proxy_eligibility_result for Proxy ID: {:?}", mining_eligibility_proxy_id.clone());
                debug::info!("Inserted proxy_eligibility_result for Proxy ID with reward amount: {:?}", _proxy_claim_total_reward_amount.clone());
                debug::info!("Inserted proxy_eligibility_result for Proxy ID with rewardees_data: {:?}", rewardees_data.clone());

                return Ok(());
            } else {
                debug::info!("Proxy claim rewardees data missing");
                return Err(DispatchError::Other("Proxy claim rewardees data missing"));
            }
        }
    }
}

impl<T: Trait> Module<T> {
    /// Create a new mining mining_eligibility_proxy
    // #[weight = 10_000 + T::DbWeight::get().writes(1)]
    pub fn create(sender: T::AccountId) -> Result<T::MiningEligibilityProxyIndex, DispatchError> {
        let mining_eligibility_proxy_id = Self::next_mining_eligibility_proxy_id()?;

        // Generate a random 128bit value
        let unique_id = Self::random_value(&sender);

        // Create and store mining_eligibility_proxy
        let mining_eligibility_proxy = MiningEligibilityProxy(unique_id);
        Self::insert_mining_eligibility_proxy(&sender, mining_eligibility_proxy_id, mining_eligibility_proxy);

        Self::deposit_event(RawEvent::Created(sender, mining_eligibility_proxy_id));
        return Ok(mining_eligibility_proxy_id);
    }

    /// Checks whether the caller is a member of the set of account IDs provided by the
    /// MembershipSource type. Emits an event if they are, and errors if not.
    pub fn is_origin_whitelisted_member_supernodes(sender: T::AccountId) -> Result<(), DispatchError> {
        let caller = sender.clone();

        // Get the members from the `membership-supernodes` pallet
        let members = T::MembershipSource::accounts();

        // Check whether the caller is a member
        // https://crates.parity.io/frame_support/traits/trait.Contains.html
        ensure!(members.contains(&caller), DispatchError::Other("Not a member"));

        // If the previous call didn't error, then the caller is a member, so emit the event
        Self::deposit_event(RawEvent::IsAMember(caller));

        Ok(())
    }

    pub fn is_supernode_claim_reasonable(proxy_claim_total_reward_amount: BalanceOf<T>) -> Result<(), DispatchError> {
        let current_block = <frame_system::Module<T>>::block_number();
        // block reward max is 5000 DHX per day until year 2023, so by 2024 we'd be up to
        // 20000 * 4 * 365 = 29200000 block, then reduces to 4800 DHX per day, and so on per halving cycle.
        // assume worse case scenario of only one supernode requesting
        // rewards on behalf of users that collectively earnt the max DHX produced on that day.
        if proxy_claim_total_reward_amount > 5000.into() && current_block < 29200000.into() {
            return Err(DispatchError::Other("Unreasonable proxy reward claim"));
        }
        Ok(())
    }

    pub fn is_valid_reward_data(_proxy_claim_rewardees_data: Vec<RewardeeData<T>>) -> Result<(), DispatchError> {
        ensure!(_proxy_claim_rewardees_data.len() > 0, "Rewardees data is invalid as no elements");
        let current_timestamp = <pallet_timestamp::Module<T>>::get();
        // convert the current date/time to the start of the current day date/time.
        // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000

        let current_timestamp_as_u64;
        if let Some(_current_timestamp_as_u64) = TryInto::<u64>::try_into(current_timestamp).ok() {
            current_timestamp_as_u64 = _current_timestamp_as_u64;
        } else {
            return Err(DispatchError::Other("Unable to convert Moment to u64 for current_timestamp"));
        }

        let current_date =
            NaiveDateTime::from_timestamp(i64::try_from(current_timestamp_as_u64.clone() / 1000u64).unwrap(), 0).date();

        let mut rewardees_data_count = 0;
        let mut is_valid = 1;
        // FIXME - use cooldown in config runtime or move to abstract constant instead of hard-code here
        let MIN_COOLDOWN_PERIOD_DAYS: Duration = Duration::days(7);

        // Iterate through all rewardees data
        for (index, rewardees_data) in _proxy_claim_rewardees_data.iter().enumerate() {
            rewardees_data_count += 1;
            debug::info!("rewardees_data_count {:#?}", rewardees_data_count);

            if let _proxy_claim_start_date = &rewardees_data.proxy_claim_start_date {
                if let _proxy_claim_end_date = &rewardees_data.proxy_claim_end_date {
                    let proxy_claim_start_date = NaiveDateTime::from_timestamp(*_proxy_claim_start_date, 0).date();
                    let proxy_claim_end_date = NaiveDateTime::from_timestamp(*_proxy_claim_end_date, 0).date();
                    let claim_duration = proxy_claim_end_date.signed_duration_since(proxy_claim_start_date);

                    if proxy_claim_end_date < current_date {
                        debug::info!("invalid proxy_claim_end_date must be prior to current_date: {:#?}", proxy_claim_end_date);
                        is_valid == 0;
                        break;
                    } else if claim_duration > MIN_COOLDOWN_PERIOD_DAYS {
                        debug::info!("unable to claim reward for lock duration less than cooldown period");
                        is_valid == 0;
                        break;
                    } else {
                        continue;
                    }
                }
            }
        }
        if is_valid == 0 {
            return Err(DispatchError::Other("Invalid rewardees data"));
        }
        Ok(())
    }

    pub fn is_mining_eligibility_proxy_owner(
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::mining_eligibility_proxy_owner(&mining_eligibility_proxy_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningEligibilityProxy"
        );
        Ok(())
    }

    pub fn exists_mining_eligibility_proxy(
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_eligibility_proxy(mining_eligibility_proxy_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningEligibilityProxy does not exist")),
        }
    }

    pub fn exists_mining_eligibility_proxy_reward_request(
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_eligibility_proxy_eligibility_reward_requests(mining_eligibility_proxy_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningEligibilityProxyRewardRequest does not exist")),
        }
    }

    pub fn has_value_for_mining_eligibility_proxy_reward_request_index(
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_eligibility_proxy_reward_request has a value that is defined");
        let fetched_mining_eligibility_proxy_reward_request =
            <MiningEligibilityProxyRewardRequests<T>>::get(mining_eligibility_proxy_id);
        if let Some(_value) = fetched_mining_eligibility_proxy_reward_request {
            debug::info!("Found value for mining_eligibility_proxy_reward_request");
            return Ok(());
        }
        debug::info!("No value for mining_eligibility_proxy_reward_request");
        Err(DispatchError::Other("No value for mining_eligibility_proxy_reward_request"))
    }

    pub fn has_value_for_mining_eligibility_proxy_reward_requestor_account_id(
        requestor: &T::AccountId,
    ) -> Result<(), DispatchError> {
        debug::info!(
            "Checking if mining_eligibility_proxy_reward_requestor has a value for the given account id that is \
             defined"
        );
        let fetched_mining_eligibility_proxy_reward_requestor =
            <MiningEligibilityProxyRewardRequestors<T>>::get(requestor);
        if let Some(_value) = fetched_mining_eligibility_proxy_reward_requestor {
            debug::info!("Found value for mining_eligibility_proxy_reward_requestor");
            return Ok(());
        }
        debug::info!("No value for mining_eligibility_proxy_reward_requestor");
        Err(DispatchError::Other("No value for mining_eligibility_proxy_reward_requestor"))
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

    fn next_mining_eligibility_proxy_id() -> Result<T::MiningEligibilityProxyIndex, DispatchError> {
        let mining_eligibility_proxy_id = Self::mining_eligibility_proxy_count();
        if mining_eligibility_proxy_id == <T::MiningEligibilityProxyIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("MiningEligibilityProxy count overflow"));
        }
        Ok(mining_eligibility_proxy_id)
    }

    fn insert_mining_eligibility_proxy(
        owner: &T::AccountId,
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
        mining_eligibility_proxy: MiningEligibilityProxy,
    ) {
        // Create and store mining mining_eligibility_proxy
        <MiningEligibilityProxys<T>>::insert(mining_eligibility_proxy_id, mining_eligibility_proxy);
        <MiningEligibilityProxyCount<T>>::put(mining_eligibility_proxy_id + One::one());
        <MiningEligibilityProxyOwners<T>>::insert(mining_eligibility_proxy_id, owner.clone());
    }

    fn insert_mining_eligibility_proxy_reward_requestor(
        requestor: &T::AccountId,
        reward_requestor_data: RequestorData<T>,
    ) {
        debug::info!("Appending reward requestor data");

        <MiningEligibilityProxyRewardRequestors<T>>::append(requestor.clone(), &reward_requestor_data.clone());

        Self::deposit_event(RawEvent::MiningEligibilityProxyRewardRequestorSet(
            requestor.clone(),
            reward_requestor_data.clone(),
        ));
    }

    fn insert_mining_eligibility_proxy_reward_transfer(transfer: &T::AccountId, reward_transfer_data: TransferData<T>) {
        debug::info!("Appending reward transfer data");

        <MiningEligibilityProxyRewardTransfers<T>>::append(transfer.clone(), &reward_transfer_data.clone());

        Self::deposit_event(RawEvent::MiningEligibilityProxyRewardTransferSet(
            transfer.clone(),
            reward_transfer_data.clone(),
        ));
    }

    fn insert_mining_eligibility_proxy_reward_daily(date: &Date, reward_daily_data: DailyData<T>) {
        debug::info!("Appending reward daily data");

        <RewardsPerDay<T>>::append(date.clone(), &reward_daily_data.clone());

        Self::deposit_event(RawEvent::RewardsPerDaySet(date.clone(), reward_daily_data.clone()));
    }

    /// Set mining_eligibility_proxy_reward_request
    fn set_mining_eligibility_proxy_eligibility_result(
        _proxy_claim_requestor_account_id: T::AccountId,
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
        _proxy_claim_total_reward_amount: BalanceOf<T>,
        _proxy_claim_rewardees_data: Vec<RewardeeData<T>>,
    ) {
        // Ensure that the mining_eligibility_proxy_id whose config we want to change actually exists
        let is_mining_eligibility_proxy = Self::exists_mining_eligibility_proxy(mining_eligibility_proxy_id);
        // FIXME - why does this cause error `expected `()`, found enum `std::result::Result``
        // its because there is no return type in this function.
        // ensure!(is_mining_eligibility_proxy.is_ok(), "MiningEligibilityProxy does not exist");
        if !is_mining_eligibility_proxy.is_ok() {
            debug::info!("Error no supernode exists with given id");
        }

        // Ensure that the caller is owner of the mining_eligibility_proxy_reward_request they are trying to change
        Self::is_mining_eligibility_proxy_owner(mining_eligibility_proxy_id, _proxy_claim_requestor_account_id.clone());

        let proxy_claim_requestor_account_id = _proxy_claim_requestor_account_id.clone();
        let proxy_claim_total_reward_amount = _proxy_claim_total_reward_amount.clone();
        let proxy_claim_rewardees_data = _proxy_claim_rewardees_data.clone();
        let current_block = <frame_system::Module<T>>::block_number();
        let proxy_claim_block_redeemed = current_block;
        let proxy_claim_timestamp_redeemed = <pallet_timestamp::Module<T>>::get();

        // Check if a mining_eligibility_proxy_reward_request already exists with the given mining_eligibility_proxy_id
        // to determine whether to insert new or mutate existing.
        if Self::has_value_for_mining_eligibility_proxy_reward_request_index(mining_eligibility_proxy_id).is_ok() {
            debug::info!("Mutating values");
            <MiningEligibilityProxyRewardRequests<T>>::mutate(
                mining_eligibility_proxy_id,
                |mining_eligibility_proxy_reward_request| {
                    if let Some(_mining_eligibility_proxy_reward_request) = mining_eligibility_proxy_reward_request {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been
                        // provided
                        _mining_eligibility_proxy_reward_request.proxy_claim_requestor_account_id =
                            proxy_claim_requestor_account_id.clone();
                        _mining_eligibility_proxy_reward_request.proxy_claim_total_reward_amount =
                            proxy_claim_total_reward_amount.clone();
                        _mining_eligibility_proxy_reward_request.proxy_claim_rewardees_data =
                            proxy_claim_rewardees_data.clone();
                        _mining_eligibility_proxy_reward_request.proxy_claim_timestamp_redeemed =
                            proxy_claim_timestamp_redeemed.clone();
                    }
                },
            );

            debug::info!("Checking mutated values");
            let fetched_mining_eligibility_proxy_reward_request =
                <MiningEligibilityProxyRewardRequests<T>>::get(mining_eligibility_proxy_id);
            if let Some(_mining_eligibility_proxy_reward_request) = fetched_mining_eligibility_proxy_reward_request {
                debug::info!(
                    "Latest field proxy_claim_requestor_account_id {:#?}",
                    _mining_eligibility_proxy_reward_request.proxy_claim_requestor_account_id
                );
                debug::info!(
                    "Latest field proxy_claim_total_reward_amount {:#?}",
                    _mining_eligibility_proxy_reward_request.proxy_claim_total_reward_amount
                );
                // debug::info!(
                //     "Latest field proxy_claim_rewardees_data {:#?}",
                //     serde_json::to_string_pretty(&_mining_eligibility_proxy_reward_request.
                // proxy_claim_rewardees_data) );
                debug::info!(
                    "Latest field proxy_claim_timestamp_redeemed {:#?}",
                    _mining_eligibility_proxy_reward_request.proxy_claim_timestamp_redeemed
                );
            }
        } else {
            debug::info!("Inserting values");

            // Create a new mining mining_eligibility_proxy_reward_request instance with the input params
            let mining_eligibility_proxy_reward_request_instance = MiningEligibilityProxyRewardRequest {
                // Since each parameter passed into the function is optional (i.e. `Option`)
                // we will assign a default value if a parameter value is not provided.
                proxy_claim_requestor_account_id: proxy_claim_requestor_account_id.clone(),
                proxy_claim_total_reward_amount: proxy_claim_total_reward_amount.clone(),
                proxy_claim_rewardees_data: proxy_claim_rewardees_data.clone(),
                // proxy_claim_block_redeemed: proxy_claim_block_redeemed.clone(),
                proxy_claim_timestamp_redeemed: proxy_claim_timestamp_redeemed.clone(),
            };

            <MiningEligibilityProxyRewardRequests<T>>::insert(
                mining_eligibility_proxy_id,
                &mining_eligibility_proxy_reward_request_instance,
            );

            debug::info!("Checking inserted values");
            let fetched_mining_eligibility_proxy_reward_request =
                <MiningEligibilityProxyRewardRequests<T>>::get(mining_eligibility_proxy_id);
            if let Some(_mining_eligibility_proxy_reward_request) = fetched_mining_eligibility_proxy_reward_request {
                debug::info!(
                    "Inserted field proxy_claim_requestor_account_id {:#?}",
                    _mining_eligibility_proxy_reward_request.proxy_claim_requestor_account_id
                );
                debug::info!(
                    "Inserted field proxy_claim_total_reward_amount {:#?}",
                    _mining_eligibility_proxy_reward_request.proxy_claim_total_reward_amount
                );
                // TODO
                // debug::info!(
                //     "Inserted field proxy_claim_rewardees_data {:#?}",
                //     serde_json::to_string_pretty(&_mining_eligibility_proxy_reward_request.
                // proxy_claim_rewardees_data) );
                // debug::info!(
                //     "Inserted field proxy_claim_block_redeemed {:#?}",
                //     _mining_eligibility_proxy_reward_request.proxy_claim_block_redeemed
                // );
                debug::info!(
                    "Inserted field proxy_claim_timestamp_redeemed {:#?}",
                    _mining_eligibility_proxy_reward_request.proxy_claim_timestamp_redeemed
                );
            }
        }

        let proxy_claim_timestamp_redeemed_as_u64 =
            TryInto::<u64>::try_into(proxy_claim_timestamp_redeemed).ok().unwrap();
        let proxy_claim_date_redeemed = NaiveDateTime::from_timestamp(
            i64::try_from(proxy_claim_timestamp_redeemed_as_u64.clone() / 1000u64).unwrap(),
            0,
        )
        .date();

        Self::deposit_event(RawEvent::MiningEligibilityProxyRewardRequestSet(
            proxy_claim_requestor_account_id,
            mining_eligibility_proxy_id,
            proxy_claim_total_reward_amount,
            proxy_claim_rewardees_data,
            proxy_claim_date_redeemed.and_hms(0, 0, 0).timestamp(),
        ));
    }
}
