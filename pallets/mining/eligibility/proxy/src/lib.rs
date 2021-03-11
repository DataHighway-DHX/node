#![cfg_attr(not(feature = "std"), no_std)]

use account_set::AccountSet;
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
// use serde::{Serialize, Deserialize};
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
use sp_std::{
    convert::TryInto,
    prelude::*,
};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait + roaming_operators::Trait + pallet_treasury::Trait + pallet_balances::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    // Loosely coupled
    type MembershipSource: AccountSet<AccountId = Self::AccountId>;
    type MiningEligibilityProxyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningEligibilityProxy(pub [u8; 16]);

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive())]
pub struct MiningEligibilityProxyResult<U, V, W, X> {
    pub proxy_claim_requestor_account_id: U, /* Supernode (proxy) account id requesting DHX rewards as proxy to
                                              * distribute to its miners */
    pub proxy_claim_total_reward_amount: V,
    pub proxy_claim_rewardees_data: W,
    pub proxy_claim_block_redeemed: X,
}

#[derive(Encode, Decode, Default, Clone, Eq, PartialEq, Debug)]
// #[derive(Encode, Decode, Default, Clone, Eq, PartialEq, Debug, Serialize)]
#[cfg_attr(feature = "std", derive())]
pub struct MiningEligibilityProxyClaimRewardeeData<U, V, W, X> {
    pub proxy_claim_rewardee_account_id: U, // Rewardee miner associated with supernode (proxy) account id
    pub proxy_claim_reward_amount: V,       // Reward in DHX tokens for specific rewardee miner
    pub proxy_claim_start_block: W,         // Start block associated with mining claim
    pub proxy_claim_interval_blocks: X,     /* Blocks after the start block that the mining claim requesting rewards
                                             * covers */
}

type RewardeeData<T> = MiningEligibilityProxyClaimRewardeeData<
    <T as frame_system::Trait>::AccountId,
    BalanceOf<T>,
    <T as frame_system::Trait>::BlockNumber,
    <T as frame_system::Trait>::BlockNumber,
>;

decl_event!(
    pub enum Event<T> where
        AccountId = <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningEligibilityProxyIndex,
        BalanceOf = BalanceOf<T>,
        <T as frame_system::Trait>::BlockNumber,
        RewardeeData = RewardeeData<T>,
    {
        Created(AccountId, MiningEligibilityProxyIndex),
        MiningEligibilityProxyResultSet(
            AccountId,
            MiningEligibilityProxyIndex,
            BalanceOf,
            Vec<RewardeeData>,
            BlockNumber,
        ),
        IsAMember(AccountId),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningEligibilityProxy {
        /// Stores all the mining_eligibility_proxys, key is the mining_eligibility_proxy id / index
        pub MiningEligibilityProxys get(fn mining_eligibility_proxy): map hasher(opaque_blake2_256) T::MiningEligibilityProxyIndex => Option<MiningEligibilityProxy>;

        /// Stores the total number of mining_eligibility_proxys. i.e. the next mining_eligibility_proxy index
        pub MiningEligibilityProxyCount get(fn mining_eligibility_proxy_count): T::MiningEligibilityProxyIndex;

        /// Stores mining_eligibility_proxy owner
        pub MiningEligibilityProxyOwners get(fn mining_eligibility_proxy_owner): map hasher(opaque_blake2_256) T::MiningEligibilityProxyIndex => Option<T::AccountId>;

        /// Stores mining_eligibility_proxy_result
        pub MiningEligibilityProxyResults get(fn mining_eligibility_proxy_eligibility_results): map hasher(opaque_blake2_256) T::MiningEligibilityProxyIndex =>
            Option<MiningEligibilityProxyResult<
                T::AccountId,
                BalanceOf<T>,
                Vec<RewardeeData<T>>,
                T::BlockNumber,
            >>;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The caller is not a member
        NotAMember,
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: <T as frame_system::Trait>::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_eligibility_proxy
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_eligibility_proxy_id = Self::next_mining_eligibility_proxy_id()?;

            // Geneeligibility a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_eligibility_proxy
            let mining_eligibility_proxy = MiningEligibilityProxy(unique_id);
            Self::insert_mining_eligibility_proxy(&sender, mining_eligibility_proxy_id, mining_eligibility_proxy);

            Self::deposit_event(RawEvent::Created(sender, mining_eligibility_proxy_id));
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn proxy_eligibility_claim(
            origin,
            mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
            _proxy_claim_total_reward_amount: BalanceOf<T>,
            _proxy_claim_rewardees_data: Option<Vec<RewardeeData<T>>>,
        ) -> Result<(), DispatchError> {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_origin_whitelisted_member_supernodes(sender.clone()).is_ok(), "Only whitelisted Supernode account members may request proxy rewards");

            ensure!(Self::is_supernode_claim_reasonable(_proxy_claim_total_reward_amount).is_ok(), "Supernode claim has been deemed unreasonable");

            if let Some(rewardees_data) = _proxy_claim_rewardees_data {
                Self::is_valid_reward_data(rewardees_data.clone());

                debug::info!("Transferring claim to proxy Supernode");
                // Distribute the reward to the account that has locked the funds
                let treasury_account_id: T::AccountId = <pallet_treasury::Module<T>>::account_id();

                let reward_to_pay_as_balance_to_try = TryInto::<BalanceOf<T>>::try_into(_proxy_claim_total_reward_amount).ok();
                if let Some(reward_to_pay) = reward_to_pay_as_balance_to_try {
                    <T as Trait>::Currency::transfer(
                        &treasury_account_id,
                        &sender,
                        reward_to_pay,
                        ExistenceRequirement::KeepAlive
                    );
                }

                debug::info!("Setting the proxy eligibility results");

                Self::set_mining_eligibility_proxy_eligibility_result(
                    sender.clone(),
                    mining_eligibility_proxy_id,
                    _proxy_claim_total_reward_amount,
                    rewardees_data,
                );

                return Ok(());
            } else {
                debug::info!("Proxy claim rewardees data missing");
                return Err(DispatchError::Other("Proxy claim rewardees data missing"));
            }
        }
    }
}

impl<T: Trait> Module<T> {
    /// Checks whether the caller is a member of the set of account IDs provided by the
    /// MembershipSource type. Emits an event if they are, and errors if not.
    pub fn is_origin_whitelisted_member_supernodes(sender: T::AccountId) -> Result<(), DispatchError> {
        let caller = sender.clone();

        // Get the members from the `membership-supernodes` pallet
        let members = T::MembershipSource::accounts();

        // Check whether the caller is a member
        // https://crates.parity.io/frame_support/traits/trait.Contains.html
        // ensure!(members.contains(&caller), Error::<T>::NotAMember);
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
        let current_block = <frame_system::Module<T>>::block_number();
        let mut rewardees_data_count = 0;
        let mut is_valid = 1;
        let calc_min_cooldown_period = 20000 * 7;
        // FIXME - use cooldown in config runtime or move to abstract constant instead of hard-code here
        let MIN_COOLDOWN_PERIOD: T::BlockNumber = calc_min_cooldown_period.into(); // 7 days @ 20k blocks produced per day

        // Iterate through all rewardees data
        for (index, rewardees_data) in _proxy_claim_rewardees_data.iter().enumerate() {
            rewardees_data_count += 1;
            debug::info!("rewardees_data_count {:#?}", rewardees_data_count);

            if let _proxy_claim_start_block = rewardees_data.proxy_claim_start_block {
                if let _proxy_claim_interval_blocks = rewardees_data.proxy_claim_interval_blocks {
                    if _proxy_claim_start_block < current_block {
                        debug::info!("invalid _proxy_claim_start_block: {:#?}", _proxy_claim_start_block);
                        is_valid == 0;
                        break;
                    } else if _proxy_claim_start_block + _proxy_claim_interval_blocks < MIN_COOLDOWN_PERIOD.into() {
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

    pub fn exists_mining_eligibility_proxy_result(
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_eligibility_proxy_eligibility_results(mining_eligibility_proxy_id) {
            Some(_value) => Ok(()),
            None => Err(DispatchError::Other("MiningEligibilityProxyResult does not exist")),
        }
    }

    pub fn has_value_for_mining_eligibility_proxy_result_index(
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
    ) -> Result<(), DispatchError> {
        debug::info!("Checking if mining_eligibility_proxy_result has a value that is defined");
        let fetched_mining_eligibility_proxy_result =
            <MiningEligibilityProxyResults<T>>::get(mining_eligibility_proxy_id);
        if let Some(_value) = fetched_mining_eligibility_proxy_result {
            debug::info!("Found value for mining_eligibility_proxy_result");
            return Ok(());
        }
        debug::info!("No value for mining_eligibility_proxy_result");
        Err(DispatchError::Other("No value for mining_eligibility_proxy_result"))
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

    /// Set mining_eligibility_proxy_result
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

        // Ensure that the caller is owner of the mining_eligibility_proxy_result they are trying to change
        Self::is_mining_eligibility_proxy_owner(mining_eligibility_proxy_id, _proxy_claim_requestor_account_id.clone());

        let proxy_claim_requestor_account_id = _proxy_claim_requestor_account_id.clone();
        let proxy_claim_total_reward_amount = _proxy_claim_total_reward_amount.clone();
        let proxy_claim_rewardees_data = _proxy_claim_rewardees_data.clone();
        let current_block = <frame_system::Module<T>>::block_number();
        let proxy_claim_block_redeemed = current_block;

        // Check if a mining_eligibility_proxy_result already exists with the given mining_eligibility_proxy_id
        // to determine whether to insert new or mutate existing.
        if Self::has_value_for_mining_eligibility_proxy_result_index(mining_eligibility_proxy_id).is_ok() {
            debug::info!("Mutating values");
            <MiningEligibilityProxyResults<T>>::mutate(
                mining_eligibility_proxy_id,
                |mining_eligibility_proxy_result| {
                    if let Some(_mining_eligibility_proxy_result) = mining_eligibility_proxy_result {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been
                        // provided
                        _mining_eligibility_proxy_result.proxy_claim_requestor_account_id =
                            proxy_claim_requestor_account_id.clone();
                        _mining_eligibility_proxy_result.proxy_claim_total_reward_amount =
                            proxy_claim_total_reward_amount.clone();
                        _mining_eligibility_proxy_result.proxy_claim_rewardees_data =
                            proxy_claim_rewardees_data.clone();
                        _mining_eligibility_proxy_result.proxy_claim_block_redeemed =
                            proxy_claim_block_redeemed.clone();
                    }
                },
            );

            debug::info!("Checking mutated values");
            let fetched_mining_eligibility_proxy_result =
                <MiningEligibilityProxyResults<T>>::get(mining_eligibility_proxy_id);
            if let Some(_mining_eligibility_proxy_result) = fetched_mining_eligibility_proxy_result {
                debug::info!(
                    "Latest field proxy_claim_requestor_account_id {:#?}",
                    _mining_eligibility_proxy_result.proxy_claim_requestor_account_id
                );
                debug::info!(
                    "Latest field proxy_claim_total_reward_amount {:#?}",
                    _mining_eligibility_proxy_result.proxy_claim_total_reward_amount
                );
                // debug::info!(
                //     "Latest field proxy_claim_rewardees_data {:#?}",
                //     serde_json::to_string_pretty(&_mining_eligibility_proxy_result.proxy_claim_rewardees_data)
                // );
                debug::info!(
                    "Latest field proxy_claim_block_redeemed {:#?}",
                    _mining_eligibility_proxy_result.proxy_claim_block_redeemed
                );
            }
        } else {
            debug::info!("Inserting values");

            // Create a new mining mining_eligibility_proxy_result instance with the input params
            let mining_eligibility_proxy_result_instance = MiningEligibilityProxyResult {
                // Since each parameter passed into the function is optional (i.e. `Option`)
                // we will assign a default value if a parameter value is not provided.
                proxy_claim_requestor_account_id: proxy_claim_requestor_account_id.clone(),
                proxy_claim_total_reward_amount: proxy_claim_total_reward_amount.clone(),
                proxy_claim_rewardees_data: proxy_claim_rewardees_data.clone(),
                proxy_claim_block_redeemed: proxy_claim_block_redeemed.clone(),
            };

            <MiningEligibilityProxyResults<T>>::insert(
                mining_eligibility_proxy_id,
                &mining_eligibility_proxy_result_instance,
            );

            debug::info!("Checking inserted values");
            let fetched_mining_eligibility_proxy_result =
                <MiningEligibilityProxyResults<T>>::get(mining_eligibility_proxy_id);
            if let Some(_mining_eligibility_proxy_result) = fetched_mining_eligibility_proxy_result {
                debug::info!(
                    "Inserted field proxy_claim_requestor_account_id {:#?}",
                    _mining_eligibility_proxy_result.proxy_claim_requestor_account_id
                );
                debug::info!(
                    "Inserted field proxy_claim_total_reward_amount {:#?}",
                    _mining_eligibility_proxy_result.proxy_claim_total_reward_amount
                );
                // debug::info!(
                //     "Inserted field proxy_claim_rewardees_data {:#?}",
                //     serde_json::to_string_pretty(&_mining_eligibility_proxy_result.proxy_claim_rewardees_data)
                // );
                debug::info!(
                    "Inserted field proxy_claim_block_redeemed {:#?}",
                    _mining_eligibility_proxy_result.proxy_claim_block_redeemed
                );
            }
        }

        Self::deposit_event(RawEvent::MiningEligibilityProxyResultSet(
            proxy_claim_requestor_account_id,
            mining_eligibility_proxy_id,
            proxy_claim_total_reward_amount,
            proxy_claim_rewardees_data,
            proxy_claim_block_redeemed,
        ));
    }
}
