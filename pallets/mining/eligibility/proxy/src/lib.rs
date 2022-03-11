#![cfg_attr(not(feature = "std"), no_std)]

use log::{warn, info};
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
use scale_info::TypeInfo;
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

/// The module's configuration trait.
pub trait Config:
    frame_system::Config
    + pallet_treasury::Config
    + pallet_balances::Config
    + pallet_timestamp::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
    type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
    // Loosely coupled
    type MembershipSource: AccountSet<AccountId = Self::AccountId>;
    type MiningEligibilityProxyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type RewardsOfDay: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type Date = i64;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, TypeInfo)]
#[cfg_attr(feature = "std", derive())]
pub struct MiningEligibilityProxy(pub [u8; 16]);

#[derive(Encode, Decode, Debug, Default, Clone, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive())]
pub struct MiningEligibilityProxyRewardRequest<U, V, W> {
    pub proxy_claim_requestor_account_id: U, /* Supernode (proxy) account id requesting DHX rewards as proxy to
                                              * distribute to its miners */
    pub proxy_claim_total_reward_amount: V,
    pub proxy_claim_timestamp_redeemed: W,
}

#[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive())]
// #[scale_info(skip_type_params(U))]
pub struct MiningEligibilityProxyClaimRewardeeData<U, V, W, X> {
    pub proxy_claim_rewardee_account_id: U, // Rewardee miner associated with supernode (proxy) account id
    pub proxy_claim_reward_amount: V,       // Reward in DHX tokens for specific rewardee miner
    pub proxy_claim_start_date: W,          // Start date associated with mining claim
    pub proxy_claim_end_date: X,            // End date covered by claim requesting mining rewards
}

#[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive())]
pub struct RewardRequestorData<U, V, W, X, Y> {
    pub mining_eligibility_proxy_id: U,
    pub total_amt: V,
    pub rewardee_count: W,
    pub member_kind: X,
    pub requested_date: Y,
}

#[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq, TypeInfo)]
#[cfg_attr(feature = "std", derive())]
pub struct RewardTransferData<U, V, W, X, Y> {
    pub mining_eligibility_proxy_id: U,
    pub total_amt: V,
    pub rewardee_count: W,
    pub member_kind: X,
    pub requested_date: Y,
}

#[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq, TypeInfo)]
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
    MiningEligibilityProxyClaimRewardeeData<<T as frame_system::Config>::AccountId, BalanceOf<T>, Date, Date>;

type RequestorData<T> = RewardRequestorData<
    <T as Config>::MiningEligibilityProxyIndex,
    BalanceOf<T>,
    u64,
    u32,
    <T as pallet_timestamp::Config>::Moment,
>;

type TransferData<T> = RewardTransferData<
    <T as Config>::MiningEligibilityProxyIndex,
    BalanceOf<T>,
    u64,
    u32,
    <T as pallet_timestamp::Config>::Moment,
>;

type DailyData<T> = RewardDailyData<
    <T as Config>::MiningEligibilityProxyIndex,
    BalanceOf<T>,
    <T as frame_system::Config>::AccountId,
    u32,
    Date,
>;

decl_event!(
    pub enum Event<T> where
        AccountId = <T as frame_system::Config>::AccountId,
        <T as Config>::MiningEligibilityProxyIndex,
        BalanceOf = BalanceOf<T>,
        RewardeeData = RewardeeData<T>,
        RequestorData = RequestorData<T>,
        TransferData = TransferData<T>,
        DailyData = DailyData<T>,
        <T as Config>::RewardsOfDay,
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
        CompletedReward(MiningEligibilityProxyIndex),
        // IsPremining(bool, AccountId),
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
    trait Store for Module<T: Config> as MiningEligibilityProxy {
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
                <T as pallet_timestamp::Config>::Moment,
            >>;

        /// Stores mining_eligibility_proxy_rewardees
        pub MiningEligibilityProxyRewardees get(fn mining_eligibility_proxy_rewardees): map hasher(opaque_blake2_256) T::MiningEligibilityProxyIndex =>
            Option<Vec<RewardeeData<T>>>;

        /// Stores reward_requests for given rewardee
        ///
        /// requestor_acct_id > (reward_hash, total_amt, rewardee_count, member_kind, date)
        /// where member_kind is either supernode_center 1 or supernode 2 or individual 3
        pub MiningEligibilityProxyRewardRequestors get(fn reward_requestors):
            map hasher(opaque_blake2_256) T::AccountId =>
                Option<Vec<RewardRequestorData<
                    <T as Config>::MiningEligibilityProxyIndex,
                    BalanceOf<T>,
                    u64,
                    u32,
                    <T as pallet_timestamp::Config>::Moment,
                >>>;

        /// Stores reward_transfers for given rewardee
        /// IMPORTANT NOTE: REQUESTOR MAY DIFFER FROM REQUESTEE
        /// HENCE DIFFERENT STORAGE FROM `reward_requestors`
        ///
        /// rewardee_acct_id > (reward_hash, total_amt, rewardee_count, member_kind, date)
        pub MiningEligibilityProxyRewardTransfers get(fn reward_transfers):
            map hasher(opaque_blake2_256) T::AccountId =>
                Option<Vec<RewardTransferData<
                    <T as Config>::MiningEligibilityProxyIndex,
                    BalanceOf<T>,
                    u64,
                    u32,
                    <T as pallet_timestamp::Config>::Moment,
                >>>;

        /// Substrate-fixed, value starts at 0 (additive identity)
        pub TotalRewardsPerDay get(fn total_rewards_daily):
            map hasher(opaque_blake2_256) Date => Option<BalanceOf<T>>;

        /// Stores accumulation of daily_rewards_sent on a given date
        /// Note: Must store date/time value as the same for all rewards sent on same day
        pub RewardsPerDay get(fn rewards_daily):
            map hasher(opaque_blake2_256) Date =>
                Option<Vec<RewardDailyData<
                    <T as Config>::MiningEligibilityProxyIndex,
                    BalanceOf<T>,
                    <T as frame_system::Config>::AccountId,
                    u32,
                    Date,
                >>>;

        /// Stores a boolean value of `true` only at the end of calling extrinsic
        /// `proxy_eligibility_claim` to signify that all the input validation has passed and
        /// all information has been stored on-chain (i.e. if an error occurs midway through
        /// `proxy_eligibility_claim` and for example some data is not stored in `TotalRewardsPerDay`
        /// then this value will NOT be `true`)
        pub MiningEligibilityProxyStatus get(fn proxy_status):
            map hasher(opaque_blake2_256) T::MiningEligibilityProxyIndex => bool;

        pub IsPremine get(fn is_premine): bool;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
        fn deposit_event() = default;

        // Toggle premine status to enable or disable daily reward limits in `is_supernode_claim_reasonable`
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn set_is_premine(
            origin,
            _is_premine: bool,
        ) -> Result<(), DispatchError> {
            let sender = ensure_root(origin)?;

            IsPremine::put(_is_premine.clone());

            // Self::deposit_event(RawEvent::IsPremining(
            //     _is_premine.clone(),
            //     sender.clone(),
            // ));

            Ok(())
        }

        /// Transfer tokens claimed by the Supernode Centre on behalf of a Supernode from the
        /// on-chain DHX DAO unlocked reserves of the Treasury account to the Supernode Centre's address,
        /// but only if the claimed amount is deemed reasonable and if there is valid data
        /// provided about the recipient accounts associated with the Supernode.
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn proxy_eligibility_claim(
            origin,
            _proxy_claim_total_reward_amount: BalanceOf<T>,
            _proxy_claim_rewardees_data: Vec<RewardeeData<T>>,
        ) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;

            // get the current block & current date/time
            let current_block = <frame_system::Pallet<T>>::block_number();
            let requested_date = <pallet_timestamp::Pallet<T>>::get();

            // convert the current date/time to the start of the current day date/time.
            // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000

            let requested_date_as_u64;
            if let Some(_requested_date_as_u64) = TryInto::<u64>::try_into(requested_date).ok() {
                requested_date_as_u64 = _requested_date_as_u64;
            } else {
                return Err(DispatchError::Other("Unable to convert Moment to i64 for requested_date"));
            }
            info!("requested_date_as_u64: {:?}", requested_date_as_u64.clone());

            let requested_date_as_u64_secs = requested_date_as_u64.clone() / 1000u64;
            // https://docs.rs/chrono/0.4.6/chrono/naive/struct.NaiveDateTime.html#method.from_timestamp
            let sent_date = NaiveDateTime::from_timestamp(i64::try_from(requested_date_as_u64_secs).unwrap(), 0).date();
            info!("requested_date_as_u64_secs: {:?}", requested_date_as_u64_secs.clone());
            info!("sent_date: {:?}", sent_date.clone());

            let sent_date_millis = sent_date.and_hms(0, 0, 0).timestamp() * 1000;
            info!("sent_date_millis: {:?}", sent_date_millis.clone());
            info!("Timestamp sent Date: {:?}", sent_date);

            ensure!(Self::is_origin_whitelisted_member_supernodes(sender.clone()).is_ok(), "Only whitelisted Supernode account members may request proxy rewards");

            let member_kind = T::MembershipSource::account_kind(sender.clone());
            info!("Requestor account kind: {:?}", member_kind.clone());

            // TODO - determine whether we'll allow the recipient to be provided by the sender
            // and how to restrict who the recipients are by membership or similar
            let recipient_member_kind = T::MembershipSource::account_kind(sender.clone());
            info!("Recipient account kind: {:?}", recipient_member_kind.clone());

            // Validate inputs (i.e. run `is_valid_reward_data` before we generate the `mining_eligibility_proxy_id` or insert any data in storage
            // as we do not want it to panic if inputs are invalid and have have only partially added some data in storage,
            // as we'd end up with numerous `mining_eligibility_proxy_id` with incomplete data.

            let is_premine = IsPremine::get();
            if is_premine != true {
                ensure!(Self::is_supernode_claim_reasonable(_proxy_claim_total_reward_amount, sent_date_millis.clone()).is_ok(), "Supernode claim has been deemed unreasonable");
            }

            match Self::is_valid_reward_data(_proxy_claim_total_reward_amount.clone(), _proxy_claim_rewardees_data.clone()) {
                Ok(_) => {
                    info!("Valid reward data");
                },
                Err(dispatch_error) => {
                    return Err(dispatch_error);
                }
            }

            // The rewards shall be distributed to the account that has locked the funds
            let treasury_account_id: T::AccountId = <pallet_treasury::Pallet<T>>::account_id();
            // Only available in Substrate 3 is pot()
            // let max_payout = pallet_treasury::Module::<T>::pot();
            let max_payout = pallet_balances::Module::<T>::usable_balance(treasury_account_id.clone());
            info!("Treasury account id: {:?}", treasury_account_id.clone());
            info!("Requestor to receive reward: {:?}", sender.clone());
            info!("Treasury balance max payout: {:?}", max_payout.clone());

            // Validate inputs so the total_reward_amount is less than the max_payout

            let reward_to_pay = _proxy_claim_total_reward_amount;

            let reward_to_pay_as_u128;
            if let Some(_reward_to_pay_as_u128) = TryInto::<u128>::try_into(reward_to_pay).ok() {
                reward_to_pay_as_u128 = _reward_to_pay_as_u128;
            } else {
                return Err(DispatchError::Other("Unable to convert Balance to u128 for reward_to_pay"));
            }
            info!("reward_to_pay_as_u128: {:?}", reward_to_pay_as_u128.clone());

            let max_payout_as_u128;
            if let Some(_max_payout_as_u128) = TryInto::<u128>::try_into(max_payout).ok() {
                max_payout_as_u128 = _max_payout_as_u128;
            } else {
                return Err(DispatchError::Other("Unable to convert Balance to u128 for max_payout"));
            }
            info!("max_payout_as_u128: {:?}", max_payout_as_u128.clone());

            ensure!(reward_to_pay_as_u128 > 0u128, "Reward must be greater than zero");
            ensure!(max_payout_as_u128 > reward_to_pay_as_u128, "Reward cannot exceed treasury balance");

            let mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex;

            match Self::create(sender.clone()) {
                Ok(proxy_id) => {
                    mining_eligibility_proxy_id = proxy_id.into();
                },
                Err(_) => {
                    return Err(DispatchError::Other("Proxy claim rewardees data missing"));
                }
            }

            info!("Transferring claim to proxy Supernode");

            // Store Requestor of the reward

            let _rewardees_data_len: usize = _proxy_claim_rewardees_data.len();
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

                info!("Setting the proxy eligibility reward requestor");

                Self::insert_mining_eligibility_proxy_reward_requestor(
                    &sender.clone(),
                    reward_requestor_data.clone(),
                );

                info!("Inserted reward Requestor: {:?}", sender.clone());
                info!("Inserted reward Requestor Data: {:?}", reward_requestor_data.clone());

                info!("Treasury paying reward");

                <T as Config>::Currency::transfer(
                    &treasury_account_id,
                    &sender,
                    reward_to_pay.clone(),
                    ExistenceRequirement::KeepAlive
                );

                info!("Success paying the reward amount: {:?}", reward_to_pay.clone());

                let reward_amount_item: DailyData<T> = RewardDailyData {
                    mining_eligibility_proxy_id: mining_eligibility_proxy_id.clone(),
                    total_amt: reward_to_pay.clone(),
                    proxy_claim_requestor_account_id: sender.clone(),
                    member_kind: recipient_member_kind.clone(),
                    rewarded_date: sent_date_millis.clone(),
                };

                Self::insert_mining_eligibility_proxy_reward_daily(
                    &sent_date_millis.clone(),
                    reward_amount_item.clone(),
                );

                info!("Appended new rewards_per_day at Date: {:?}", sent_date_millis.clone());
                info!("Appended new rewards_per_day in storage item: {:?}", reward_amount_item.clone());

                let rewards_per_day_retrieved = <RewardsPerDay<T>>::get(
                    sent_date_millis.clone(),
                );
                info!("Retrieved new rewards_per_day storage item: {:?}", rewards_per_day_retrieved.clone());

                // Update in storage the total rewards distributed so far for the current day
                // so users may query state and have the latest calculated total returned.
                match Self::total_rewards_daily(sent_date_millis.clone()) {
                    None => {
                        info!("Creating new total rewards entry for a given day");

                        <TotalRewardsPerDay<T>>::insert(
                            sent_date_millis.clone(),
                            _proxy_claim_total_reward_amount.clone(),
                        );

                        info!("Created new total_rewards_daily at Date millis: {:?}", sent_date_millis.clone());
                        info!("Creating new total_rewards_daily at Date with Amount: {:?}", _proxy_claim_total_reward_amount.clone());

                        // Emit event
                        Self::deposit_event(RawEvent::TotalRewardsPerDayUpdated(
                            _proxy_claim_total_reward_amount.clone(),
                            sent_date_millis.clone(),
                            sender.clone(),
                        ));
                    },
                    Some(old_total_rewards_for_day) => {
                        info!("TotalRewardsPerDay entry mapping already exists for given day. Updating...");

                        // Add, handling overflow
                        let new_total_rewards_for_day =
                            old_total_rewards_for_day.checked_add(&_proxy_claim_total_reward_amount.clone()).ok_or(Error::<T>::Overflow)?;
                        // Write the new value to storage
                        <TotalRewardsPerDay<T>>::mutate(
                            sent_date_millis.clone(),
                            |reward_moment| {
                                if let Some(_reward_moment) = reward_moment {
                                    *_reward_moment = new_total_rewards_for_day.clone();
                                }

                                info!("Updated total_rewards_daily at Date: {:?}",  sent_date);
                                info!("Updated total_rewards_daily at Date. Existing Amount: {:?}", old_total_rewards_for_day.clone());
                                info!("Updated total_rewards_daily at Date. Reward Amount: {:?}", _proxy_claim_total_reward_amount.clone());
                                info!("Updated total_rewards_daily at Date. New Amount: {:?}", new_total_rewards_for_day.clone());
                            },
                        );

                        // Emit event
                        Self::deposit_event(RawEvent::TotalRewardsPerDayUpdated(
                            new_total_rewards_for_day.clone(),
                            sent_date_millis.clone(),
                            sender.clone(),
                        ));
                    }
                }

                // This is only really necessary in addition to `RewardRequestorData` if
                // the sender of the data is different from the recipient of the rewards
                // (if this extrinsic function accepted a recipient argument other than the sender)
                let reward_transfer_data: TransferData<T> = RewardTransferData {
                    mining_eligibility_proxy_id: mining_eligibility_proxy_id.clone(),
                    total_amt: reward_to_pay.clone(),
                    rewardee_count: rewardees_data_len.clone(),
                    member_kind: member_kind.clone(),
                    requested_date: requested_date.clone(),
                };

                info!("Setting the proxy eligibility reward transfer");

                Self::insert_mining_eligibility_proxy_reward_transfer(
                    &sender.clone(),
                    reward_transfer_data.clone(),
                );

                info!("Inserted proxy_reward_transfer for Sender: {:?}", sender.clone());
                info!("Inserted proxy_reward_transfer for Sender with Data: {:?}", reward_transfer_data.clone());

                info!("Setting the proxy eligibility reward_request");

                Self::set_mining_eligibility_proxy_eligibility_reward_request(
                    sender.clone(),
                    mining_eligibility_proxy_id.clone(),
                    _proxy_claim_total_reward_amount.clone(),
                    _proxy_claim_rewardees_data.clone(),
                );

                info!("Inserted proxy_eligibility_reward_request for Proxy ID: {:?}", mining_eligibility_proxy_id.clone());
                info!("Inserted proxy_eligibility_reward_request for Proxy ID with reward amount: {:?}", _proxy_claim_total_reward_amount.clone());
                info!("Inserted proxy_eligibility_reward_request for Proxy ID with _proxy_claim_rewardees_data: {:?}", _proxy_claim_rewardees_data.clone());

                <MiningEligibilityProxyStatus<T>>::insert(
                    mining_eligibility_proxy_id.clone(),
                    true,
                );

                Self::deposit_event(RawEvent::CompletedReward(
                    mining_eligibility_proxy_id.clone(),
                ));

                info!("Completed Transfer");

                return Ok(());
            } else {
                warn!("Unable to convert _proxy_claim_rewardees_data");
                return Err(DispatchError::Other("Unable to convert _proxy_claim_rewardees_data"));
            }
        }
    }
}

impl<T: Config> Module<T> {
    /// Create a new mining mining_eligibility_proxy
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

    pub fn is_supernode_claim_reasonable(
        proxy_claim_total_reward_amount: BalanceOf<T>,
        sent_date_millis: Date,
    ) -> Result<(), DispatchError> {
        let current_block = <frame_system::Pallet<T>>::block_number();
        // block reward max is 5000 DHX per day until year 2023, so by 2024 we'd be up to
        // 20000 * 4 * 365 = 29200000 block, then reduces to 4800 DHX per day, and so on per halving cycle.
        // assume worse case scenario of only one supernode requesting
        // rewards on behalf of users that collectively earnt the max DHX produced on that day.
        let mut is_valid = 1;
        let DAILY_WITHDRAWAL_LIMIT = 5000000000000000000000u128;

        let proxy_claim_total_reward_amount_as_u128 =
            TryInto::<u128>::try_into(proxy_claim_total_reward_amount).ok().unwrap();

        if let Some(total_rewards_per_day_retrieved) = <TotalRewardsPerDay<T>>::get(sent_date_millis.clone()) {
            let total_rewards_per_day_retrieved_as_u128 =
                    TryInto::<u128>::try_into(total_rewards_per_day_retrieved).ok().unwrap();
            info!("Retrieved new total_rewards_per_day_retrieved_as_u128 storage item: {:?}", total_rewards_per_day_retrieved_as_u128.clone());

            let sum = total_rewards_per_day_retrieved_as_u128 + proxy_claim_total_reward_amount_as_u128;
            // println!("sum {:#?}", sum);
            info!("sum {:#?}", sum);
            if sum > DAILY_WITHDRAWAL_LIMIT.clone().into() {
                // println!("Sum exceeds daily withdrawal limit");
                warn!("Sum exceeds daily withdrawal limit");
                is_valid = 0;
            }
        } else if proxy_claim_total_reward_amount_as_u128 > DAILY_WITHDRAWAL_LIMIT.clone().into() {
            // println!("Total reward amount exceeds daily withdrawal limit");
            warn!("Total reward amount exceeds daily withdrawal limit");
            is_valid = 0;
        }

        // println!("proxy_claim_total_reward_amount {:#?}", proxy_claim_total_reward_amount);
        // println!("proxy_claim_total_reward_amount_as_u128 {:#?}", proxy_claim_total_reward_amount_as_u128);
        // println!("DAILY_WITHDRAWAL_LIMIT {:#?}", DAILY_WITHDRAWAL_LIMIT);
        info!("proxy_claim_total_reward_amount {:#?}", proxy_claim_total_reward_amount);
        info!("proxy_claim_total_reward_amount_as_u128 {:#?}", proxy_claim_total_reward_amount_as_u128);
        info!("DAILY_WITHDRAWAL_LIMIT {:#?}", DAILY_WITHDRAWAL_LIMIT);

        if is_valid == 0 {
            return Err(DispatchError::Other("Supernode claim has been deemed unreasonable"));
        }

        Ok(())
    }

    pub fn is_valid_reward_data(_proxy_claim_total_reward_amount: BalanceOf<T>, _proxy_claim_rewardees_data: Vec<RewardeeData<T>>) -> Result<(), DispatchError> {
        ensure!(_proxy_claim_rewardees_data.len() > 0, "Rewardees data is invalid as no elements");

        let current_timestamp = <pallet_timestamp::Pallet<T>>::get();
        // convert the current date/time to the start of the current day date/time.
        // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
        let current_timestamp_as_u64;
        if let Some(_current_timestamp_as_u64) = TryInto::<u64>::try_into(current_timestamp).ok() {
            current_timestamp_as_u64 = _current_timestamp_as_u64;
        } else {
            return Err(DispatchError::Other("Unable to convert Moment to u64 for current_timestamp"));
        }

        let current_timestamp_as_u64_secs = current_timestamp_as_u64.clone() / 1000u64;
        let current_date =
            NaiveDateTime::from_timestamp(i64::try_from(current_timestamp_as_u64_secs).unwrap(), 0).date();

        let mut rewardees_data_count = 0;
        let mut is_valid = 1;
        let MIN_COOLDOWN_PERIOD_DAYS: Duration = Duration::days(7); // 7 days @ 20k blocks produced per day

        // Iterate through all rewardees data
        for (index, rewardees_data) in _proxy_claim_rewardees_data.iter().enumerate() {
            rewardees_data_count += 1;
            info!("rewardees_data_count {:#?}", rewardees_data_count);

            if let _proxy_claim_start_date = &rewardees_data.proxy_claim_start_date {
                if let _proxy_claim_end_date = &rewardees_data.proxy_claim_end_date {
                    let proxy_claim_start_date_secs = _proxy_claim_start_date / 1000i64;
                    let proxy_claim_end_date_secs = _proxy_claim_end_date / 1000i64;
                    let proxy_claim_start_date = NaiveDateTime::from_timestamp(proxy_claim_start_date_secs, 0).date();
                    let proxy_claim_end_date = NaiveDateTime::from_timestamp(proxy_claim_end_date_secs, 0).date();
                    let claim_duration = proxy_claim_end_date.signed_duration_since(proxy_claim_start_date);

                    if proxy_claim_end_date >= current_date {
                        warn!("invalid proxy_claim_end_date must be prior to current_date: {:#?}", proxy_claim_end_date);
                        is_valid = 0;
                        break;
                    } else if claim_duration <= MIN_COOLDOWN_PERIOD_DAYS {
                        warn!("unable to claim reward for lock duration less than cooldown period");
                        is_valid = 0;
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

        // Check that sum _proxy_claim_total_reward_amount equals sum of all the rewardee's proxy_claim_reward_amount
        info!("Verifying that total reward amount requested equals sum of all rewardee data claim amounts");

        let mut sum_reward_amounts = 0u128;
        rewardees_data_count = 0; // Reset count

        // Iterate through all rewardees data
        for (index, rewardees_data) in _proxy_claim_rewardees_data.iter().enumerate() {
            rewardees_data_count += 1;
            info!("rewardees_data_count {:#?}", rewardees_data_count);

            if let _proxy_claim_reward_amount = rewardees_data.proxy_claim_reward_amount.clone() {
                let _proxy_claim_reward_amount_as_u128 =
                    TryInto::<u128>::try_into(_proxy_claim_reward_amount).ok().unwrap();
                sum_reward_amounts += _proxy_claim_reward_amount_as_u128;
            } else {
                warn!("unable to interpret proxy_claim_reward_amount");
                is_valid = 0;
                break;
            }
        }
        let _proxy_claim_total_reward_amount_as_u128 =
            TryInto::<u128>::try_into(_proxy_claim_total_reward_amount).ok().unwrap();
        if sum_reward_amounts != _proxy_claim_total_reward_amount_as_u128 {
            is_valid = 0;
            return Err(DispatchError::Other("Inconsistent data provided as total reward amount requested does not equal sum of all rewardee data claim amounts"));
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

    pub fn has_value_for_mining_eligibility_proxy_reward_requestor_account_id(
        requestor: &T::AccountId,
    ) -> Result<(), DispatchError> {
        info!(
            "Checking if mining_eligibility_proxy_reward_requestor has a value for the given account id that is \
             defined"
        );
        let fetched_mining_eligibility_proxy_reward_requestor =
            <MiningEligibilityProxyRewardRequestors<T>>::get(requestor);
        if let Some(_value) = fetched_mining_eligibility_proxy_reward_requestor {
            info!("Found value for mining_eligibility_proxy_reward_requestor");
            return Ok(());
        }
        warn!("No value for mining_eligibility_proxy_reward_requestor");
        Err(DispatchError::Other("No value for mining_eligibility_proxy_reward_requestor"))
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <frame_system::Pallet<T>>::extrinsic_index(),
            <frame_system::Pallet<T>>::block_number(),
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
        info!("Appending reward requestor data");

        <MiningEligibilityProxyRewardRequestors<T>>::append(requestor.clone(), &reward_requestor_data.clone());

        Self::deposit_event(RawEvent::MiningEligibilityProxyRewardRequestorSet(
            requestor.clone(),
            reward_requestor_data.clone(),
        ));
    }

    fn insert_mining_eligibility_proxy_reward_transfer(transfer: &T::AccountId, reward_transfer_data: TransferData<T>) {
        info!("Appending reward transfer data");

        <MiningEligibilityProxyRewardTransfers<T>>::append(transfer.clone(), &reward_transfer_data.clone());

        Self::deposit_event(RawEvent::MiningEligibilityProxyRewardTransferSet(
            transfer.clone(),
            reward_transfer_data.clone(),
        ));
    }

    fn insert_mining_eligibility_proxy_reward_daily(sent_date: &Date, reward_daily_data: DailyData<T>) {
        info!("Appending reward daily data");

        <RewardsPerDay<T>>::append(sent_date.clone(), &reward_daily_data.clone());

        Self::deposit_event(RawEvent::RewardsPerDaySet(sent_date.clone(), reward_daily_data.clone()));
    }

    /// Set mining_eligibility_proxy_reward_request
    fn set_mining_eligibility_proxy_eligibility_reward_request(
        _proxy_claim_requestor_account_id: T::AccountId,
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
        _proxy_claim_total_reward_amount: BalanceOf<T>,
        _proxy_claim_rewardees_data: Vec<RewardeeData<T>>,
    ) {
        // Ensure that the mining_eligibility_proxy_id whose config we want to change actually exists
        let is_mining_eligibility_proxy = Self::exists_mining_eligibility_proxy(mining_eligibility_proxy_id);

        if !is_mining_eligibility_proxy.is_ok() {
            warn!("Error no supernode exists with given id");
        }

        // Ensure that the caller is owner of the mining_eligibility_proxy_reward_request they are trying to change
        Self::is_mining_eligibility_proxy_owner(mining_eligibility_proxy_id, _proxy_claim_requestor_account_id.clone());

        let proxy_claim_requestor_account_id = _proxy_claim_requestor_account_id.clone();
        let proxy_claim_total_reward_amount = _proxy_claim_total_reward_amount.clone();
        let proxy_claim_rewardees_data = _proxy_claim_rewardees_data.clone();
        let current_block = <frame_system::Pallet<T>>::block_number();
        let proxy_claim_block_redeemed = current_block;
        let proxy_claim_timestamp_redeemed = <pallet_timestamp::Pallet<T>>::get();

        info!("Inserting reward requests");

        // Create a new mining mining_eligibility_proxy_reward_request instance with the input params
        let mining_eligibility_proxy_reward_request_instance = MiningEligibilityProxyRewardRequest {
            // Since each parameter passed into the function is optional (i.e. `Option`)
            // we will assign a default value if a parameter value is not provided.
            proxy_claim_requestor_account_id: proxy_claim_requestor_account_id.clone(),
            proxy_claim_total_reward_amount: proxy_claim_total_reward_amount.clone(),
            proxy_claim_timestamp_redeemed: proxy_claim_timestamp_redeemed.clone(),
        };

        <MiningEligibilityProxyRewardRequests<T>>::insert(
            mining_eligibility_proxy_id,
            mining_eligibility_proxy_reward_request_instance.clone(),
        );

        info!("Insert rewardees {:#?}", proxy_claim_rewardees_data.clone());
        <MiningEligibilityProxyRewardees<T>>::insert(
            mining_eligibility_proxy_id,
            proxy_claim_rewardees_data.clone(),
        );

        let proxy_claim_timestamp_redeemed_as_u64 =
            TryInto::<u64>::try_into(proxy_claim_timestamp_redeemed).ok().unwrap();
        let proxy_claim_date_redeemed = NaiveDateTime::from_timestamp(
            i64::try_from(proxy_claim_timestamp_redeemed_as_u64.clone() / 1000u64).unwrap(),
            0,
        )
        .date();

        let date_redeemed_millis = proxy_claim_date_redeemed.and_hms(0, 0, 0).timestamp() * 1000;
        info!("proxy_claim_date_redeemed.timestamp {:#?}", date_redeemed_millis.clone());

        Self::deposit_event(RawEvent::MiningEligibilityProxyRewardRequestSet(
            proxy_claim_requestor_account_id,
            mining_eligibility_proxy_id,
            proxy_claim_total_reward_amount,
            proxy_claim_rewardees_data,
            date_redeemed_millis.clone(),
        ));
    }
}
