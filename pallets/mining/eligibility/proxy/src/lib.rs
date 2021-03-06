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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + roaming_operators::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MiningEligibilityProxyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningEligibilityProxyClaimTotalRewardAmount: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type MiningEligibilityProxyClaimBlockRedeemed: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

// type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as
// frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningEligibilityProxy(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningEligibilityProxyResult<U, V, W, X> {
    pub proxy_claim_requestor_account_id: U, // Supernode (proxy) account id requesting DHX rewards as proxy to distribute to its miners
    pub proxy_claim_total_reward_amount: V,
    pub proxy_claim_rewardees_data: W,
    pub proxy_claim_block_redeemed: X,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningEligibilityProxyClaimRewardeeData<U, V, W, X> {
    pub proxy_claim_rewardee_account_id: U, // Rewardee miner associated with supernode (proxy) account id
    pub proxy_claim_reward_amount: V, // Reward in DHX tokens for specific rewardee miner
    pub proxy_claim_start_block: W, // Start block associated with mining claim
    pub proxy_claim_interval_blocks: X, // Blocks after the start block that the mining claim requesting rewards covers
}

type RewardeeData<T> = MiningEligibilityProxyClaimRewardeeData<
    <T as frame_system::Trait>::AccountId,
    <T as Trait>::MiningEligibilityProxyIndex,
    <T as Trait>::MiningEligibilityProxyClaimTotalRewardAmount,
    <T as frame_system::Trait>::BlockNumber,
>;

decl_event!(
    pub enum Event<T> where
        AccountId = <T as frame_system::Trait>::AccountId,
        <T as Trait>::MiningEligibilityProxyIndex,
        <T as Trait>::MiningEligibilityProxyClaimTotalRewardAmount,
        <T as frame_system::Trait>::BlockNumber,
    {
        Created(AccountId, MiningEligibilityProxyIndex),
        MiningEligibilityProxyResultSet(
          AccountId, MiningEligibilityProxyIndex,
          MiningEligibilityProxyClaimTotalRewardAmount, Vec<RewardeeData<T>>,
          BlockNumber
        ),
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
                T::MiningEligibilityProxyClaimTotalRewardAmount,
                Vec<RewardeeData<T>>,
                T::BlockNumber,
            >>;
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
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
            _proxy_claim_total_reward_amount: Option<T::MiningEligibilityProxyClaimTotalRewardAmount>,
            _proxy_claim_rewardees_data: Option<Vec<RewardeeData<T>>>,
        ) {
            let sender = ensure_signed(origin)?;

            ensure!(is_origin_whitelisted_supernode(sender.clone()), "Only whitelisted Supernode account members may request proxy rewards");

            // ensure!(is_supernode_claim_reasonable(_proxy_claim_total_reward_amount, _proxy_claim_rewardees_data), "Supernode claim has been deemed unreasonable");

            if let Some(rewardees_data) = _proxy_claim_rewardees_data {
                ensure!(is_valid_reward_data(rewardees_data), "Rewardees data is invalid");
            } else {
                debug::info!("Proxy claim rewardees data missing");
            }

            debug::info!("Setting the proxy eligibility results");

            Self::set_mining_eligibility_proxy_eligibility_result(
                sender.clone(),
                mining_eligibility_proxy_id,
                _proxy_claim_total_reward_amount,
                _proxy_claim_rewardees_data,
            );
        }

        // FIXME - implement this and fix the type errors and uncomment it in the integration tests
        // /// Calculate mining_eligibility_proxy_result
        // pub fn calculate_mining_eligibility_proxy_result(
        //     origin,
        //     mining_config_token_id: T::MiningConfigTokenIndex,
        //     mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
        // ) {
        //     let sender = ensure_signed(origin)?;

        //     // Ensure that the mining_eligibility_proxy_id whose config we want to change actually exists
        //     let is_mining_eligibility_proxy = Self::exists_mining_eligibility_proxy(mining_eligibility_proxy_id).is_ok();
        //     ensure!(is_mining_eligibility_proxy, "MiningEligibilityProxy does not exist");

        //     // Ensure that the caller is owner of the mining_eligibility_proxy_result they are trying to change
        //     ensure!(Self::mining_eligibility_proxy_owner(mining_eligibility_proxy_id) == Some(sender.clone()), "Only owner can set mining_eligibility_proxy_result");

        //     let DEFAULT_RATE_CONFIG = 0;
        //     let mut proxy_claim_total_reward_amount = 0.into();
        //     let mut part_proxy_claim_total_reward_amount = 0.into();
        //     let mut proxy_claim_rewardees_data = 0.into();
        //     let mut token_token_max_token = 0.into();

        //     let mut current_token_type;
        //     let mut current_token_lock_amount;
        //     // Get the config associated with the given configuration_token
        //     if let Some(configuration_token_config) = <mining_config_token::Module<T>>::mining_config_token_token_configs(mining_config_token_id) {
        //       if let token_type = configuration_token_config.token_type {
        //         if token_type != "".to_string() {
        //           current_token_type = token_type.clone();

        //           if let token_lock_amount = configuration_token_config.token_lock_amount {
        //             if token_lock_amount != 0 {
        //               current_token_lock_amount = token_lock_amount;

        //               // Get list of all sampling_token_ids that correspond to the given mining_config_token_id
        //               // of type MiningSamplingTokenIndex
        //               let sampling_token_ids = <mining_sampling_token::Module<T>>
        //                 ::token_config_samplings(mining_config_token_id);

        //               let mut sample_count = 0;
        //               let mut current_sample_tokens_locked = 0;
        //               let mut current_token_rate = 0;
        //               let mut current_token_max_tokens = 0;
        //               let mut total = 0;
        //               // Iteratve through all the associated samples
        //               for (index, sampling_token_id) in sampling_token_ids.iter().enumerate() {
        //                 // Retrieve the current corresponding sampling_token_config
        //                 // of type MiningSamplingTokenConfig
        //                 if let Some(current_sampling_token_config) = <mining_sampling_token::Module<T>>::mining_samplings_token_samplings_configs(
        //                   (mining_config_token_id, sampling_token_id)
        //                 ) {
        //                   if let tokens_locked = current_sampling_token_config.token_sample_locked_amount {
        //                     sample_count += 1;

        //                     if tokens_locked == 0 {
        //                       debug::info!("Mining rate sample has nothing locked. Skipping to next sampling.");
        //                       continue;
        //                     }
        //                     current_sample_tokens_locked = tokens_locked;

        //                     if let Some(token_rates_config) = <mining_rates_token::Module<T>>::mining_rates_token_rates_configs(DEFAULT_RATE_CONFIG) {

        //                       if current_token_type == "MXC".to_string() {
        //                         current_token_rate = token_rates_config.token_token_mxc;
        //                       } else if current_token_type == "IOTA".to_string() {
        //                         current_token_rate = token_rates_config.token_token_iota;
        //                       } else if current_token_type == "DOT".to_string() {
        //                         current_token_rate = token_rates_config.token_token_dot;
        //                       }
        //                       current_token_max_tokens = token_rates_config.token_token_max_token;
        //                       proxy_claim_rewardees_data = current_token_rate * (current_sample_tokens_locked / current_token_lock_amount);

        //                       part_proxy_claim_total_reward_amount = part_proxy_claim_total_reward_amount + proxy_claim_rewardees_data * current_token_max_tokens;
        //                     } else {
        //                       debug::info!("Mining rate config missing");
        //                       // break;
        //                       return Err(DispatchError::Other("Mining rate config missing"));
        //                     }
        //                   }
        //                 }
        //               }
        //               proxy_claim_total_reward_amount = part_proxy_claim_total_reward_amount / sample_count;
        //               debug::info!("Calculate eligibilty based on average {:#?}", proxy_claim_total_reward_amount);
        //             }
        //           }
        //         }
        //       }
        //     }

        //     // Check if a mining_eligibility_proxy_result already exists with the given mining_eligibility_proxy_id
        //     // to determine whether to insert new or mutate existing.
        //     if Self::has_value_for_mining_eligibility_proxy_result_index(mining_config_token_id, mining_eligibility_proxy_id).is_ok() {
        //         debug::info!("Mutating values");
        //         <MiningEligibilityProxyResults<T>>::mutate((mining_config_token_id, mining_eligibility_proxy_id), |mining_eligibility_proxy_result| {
        //             if let Some(_mining_eligibility_proxy_result) = mining_eligibility_proxy_result {
        //                 // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
        //                 _mining_eligibility_proxy_result.proxy_claim_total_reward_amount = proxy_claim_total_reward_amount.clone();
        //                 _mining_eligibility_proxy_result.proxy_claim_rewardees_data = proxy_claim_rewardees_data.clone();
        //                 // _mining_eligibility_proxy_result.proxy_claim_block_redeemed = proxy_claim_block_redeemed.clone();
        //                 // _mining_eligibility_proxy_result.proxy_claim_requestor_account_id = proxy_claim_requestor_account_id.clone();
        //             }
        //         });
        //         debug::info!("Checking mutated values");
        //         let fetched_mining_eligibility_proxy_result = <MiningEligibilityProxyResults<T>>::get((mining_config_token_id, mining_eligibility_proxy_id));
        //         if let Some(_mining_eligibility_proxy_result) = fetched_mining_eligibility_proxy_result {
        //             debug::info!("Latest field proxy_claim_total_reward_amount {:#?}", _mining_eligibility_proxy_result.proxy_claim_total_reward_amount);
        //             debug::info!("Latest field proxy_claim_rewardees_data {:#?}", _mining_eligibility_proxy_result.proxy_claim_rewardees_data);
        //             // debug::info!("Latest field proxy_claim_block_redeemed {:#?}", _mining_eligibility_proxy_result.proxy_claim_block_redeemed);
        //             // debug::info!("Latest field proxy_claim_requestor_account_id {:#?}", _mining_eligibility_proxy_result.proxy_claim_requestor_account_id);
        //         }
        //     } else {
        //         debug::info!("Inserting values");

        //         // Create a new mining mining_eligibility_proxy_result instance with the input params
        //         let mining_eligibility_proxy_result_instance = MiningEligibilityProxyResult {
        //             // Since each parameter passed into the function is optional (i.e. `Option`)
        //             // we will assign a default value if a parameter value is not provided.
        //             proxy_claim_total_reward_amount: proxy_claim_total_reward_amount.clone(),
        //             proxy_claim_rewardees_data: proxy_claim_rewardees_data.clone(),
        //             // proxy_claim_block_redeemed: proxy_claim_block_redeemed.clone(),
        //             // proxy_claim_requestor_account_id: proxy_claim_requestor_account_id.clone(),
        //         };

        //         <MiningEligibilityProxyResults<T>>::insert(
        //             (mining_config_token_id, mining_eligibility_proxy_id),
        //             &mining_eligibility_proxy_result_instance
        //         );

        //         debug::info!("Checking inserted values");
        //         let fetched_mining_eligibility_proxy_result = <MiningEligibilityProxyResults<T>>::get((mining_config_token_id, mining_eligibility_proxy_id));
        //         if let Some(_mining_eligibility_proxy_result) = fetched_mining_eligibility_proxy_result {
        //             debug::info!("Inserted field proxy_claim_total_reward_amount {:#?}", _mining_eligibility_proxy_result.proxy_claim_total_reward_amount);
        //             debug::info!("Inserted field proxy_claim_rewardees_data {:#?}", _mining_eligibility_proxy_result.proxy_claim_rewardees_data);
        //             // debug::info!("Inserted field proxy_claim_block_redeemed {:#?}", _mining_eligibility_proxy_result.proxy_claim_block_redeemed);
        //             // debug::info!("Inserted field proxy_claim_requestor_account_id {:#?}", _mining_eligibility_proxy_result.proxy_claim_requestor_account_id);
        //         }
        //     }

        //     Self::deposit_event(RawEvent::MiningEligibilityProxyResultSet(
        //       sender,
        //       mining_config_token_id,
        //       mining_eligibility_proxy_id,
        //       proxy_claim_total_reward_amount,
        //       proxy_claim_rewardees_data,
        //       // proxy_claim_block_redeemed,
        //       // proxy_claim_requestor_account_id
        //     ));
        // }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_origin_whitelisted_supernode(
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            // FIXME - implement this pallet
            <member_supernodes::Module<T>>::is_member_supernode(sender.clone()).is_ok(),
            "Sender is not a whitelisted Supernode member"
        );
        Ok(())
    }

    pub fn is_valid_reward_data(
        _proxy_claim_rewardees_data: Vec<RewardeeData<T>
    ) -> Result<(), DispatchError> {
        let current_block = <frame_system::Module<T>>::block_number();
        let mut rewardees_data_count = 0;
        let mut is_valid = 1;
        // FIXME - use cooldown in config runtime or move to abstract constant instead of hard-code here
        let MIN_COOLDOWN_PERIOD = 20000 * 7; // 7 days @ 20k blocks produced per day

        // Iterate through all rewardees data
        for (index, rewardees_data) in _proxy_claim_rewardees_data.iter().enumerate() {
            rewardees_data_count += 1;
            debug::info!("rewardees_data_count {:#?}", rewardees_data_count);

            if let Some(_proxy_claim_start_block) = _proxy_claim_rewardees_data.proxy_claim_start_block {
                if let Some(_proxy_claim_interval_blocks) = _proxy_claim_rewardees_data.proxy_claim_interval_blocks {
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
    ) -> Result<MiningEligibilityProxy, DispatchError> {
        match Self::mining_eligibility_proxy(mining_eligibility_proxy_id) {
            Some(value) => Ok(value),
            None => Err(DispatchError::Other("MiningEligibilityProxy does not exist")),
        }
    }

    pub fn exists_mining_eligibility_proxy_result(
        mining_eligibility_proxy_id: T::MiningEligibilityProxyIndex,
    ) -> Result<(), DispatchError> {
        match Self::mining_eligibility_proxy_eligibility_results(mining_eligibility_proxy_id)
        {
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
        _proxy_claim_total_reward_amount: Option<T::MiningEligibilityProxyClaimTotalRewardAmount>,
        _proxy_claim_rewardees_data: Option<Vec<RewardeeData<T>>>,
    ) {
        // Ensure that the mining_eligibility_proxy_id whose config we want to change actually exists
        let is_mining_eligibility_proxy = Self::exists_mining_eligibility_proxy(mining_eligibility_proxy_id).is_ok();
        ensure!(is_mining_eligibility_proxy, "MiningEligibilityProxy does not exist");

        // Ensure that the caller is owner of the mining_eligibility_proxy_result they are trying to change
        Self::is_mining_eligibility_proxy_owner(mining_eligibility_proxy_id, _proxy_claim_requestor_account_id);

        let proxy_claim_requestor_account_id = _proxy_claim_requestor_account_id;
        // FIXME - change to ensure and check that a value is provided or early exit
        let proxy_claim_total_reward_amount = match _proxy_claim_total_reward_amount.clone() {
            Some(value) => value,
            None => 1.into() // Default
        };
        // FIXME - change to ensure and check that data structure is valid or early exit
        let proxy_claim_rewardees_data = match _proxy_claim_rewardees_data {
            Some(value) => value,
            None => 1.into() // Default
        };
        let current_block = <frame_system::Module<T>>::block_number();
        let proxy_claim_block_redeemed = current_block;

        // Check if a mining_eligibility_proxy_result already exists with the given mining_eligibility_proxy_id
        // to determine whether to insert new or mutate existing.
        if Self::has_value_for_mining_eligibility_proxy_result_index(mining_eligibility_proxy_id).is_ok() {
            debug::info!("Mutating values");
            <MiningEligibilityProxyResults<T>>::mutate(mining_eligibility_proxy_id, |mining_eligibility_proxy_result| {
                if let Some(_mining_eligibility_proxy_result) = mining_eligibility_proxy_result {
                    // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                    _mining_eligibility_proxy_result.proxy_claim_requestor_account_id = proxy_claim_requestor_account_id.clone();
                    _mining_eligibility_proxy_result.proxy_claim_total_reward_amount = proxy_claim_total_reward_amount.clone();
                    _mining_eligibility_proxy_result.proxy_claim_rewardees_data = proxy_claim_rewardees_data.clone();
                    _mining_eligibility_proxy_result.proxy_claim_block_redeemed = proxy_claim_block_redeemed.clone();
                }
            });

            debug::info!("Checking mutated values");
            let fetched_mining_eligibility_proxy_result = <MiningEligibilityProxyResults<T>>::get(mining_eligibility_proxy_id);
            if let Some(_mining_eligibility_proxy_result) = fetched_mining_eligibility_proxy_result {
                debug::info!("Latest field proxy_claim_requestor_account_id {:#?}", _mining_eligibility_proxy_result.proxy_claim_requestor_account_id);
                debug::info!("Latest field proxy_claim_total_reward_amount {:#?}", _mining_eligibility_proxy_result.proxy_claim_total_reward_amount);
                debug::info!("Latest field proxy_claim_rewardees_data {:#?}", _mining_eligibility_proxy_result.proxy_claim_rewardees_data);
                debug::info!("Latest field proxy_claim_block_redeemed {:#?}", _mining_eligibility_proxy_result.proxy_claim_block_redeemed);
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
                &mining_eligibility_proxy_result_instance
            );

            debug::info!("Checking inserted values");
            let fetched_mining_eligibility_proxy_result = <MiningEligibilityProxyResults<T>>::get(mining_eligibility_proxy_id);
            if let Some(_mining_eligibility_proxy_result) = fetched_mining_eligibility_proxy_result {
                debug::info!("Inserted field proxy_claim_requestor_account_id {:#?}", _mining_eligibility_proxy_result.proxy_claim_requestor_account_id);
                debug::info!("Inserted field proxy_claim_total_reward_amount {:#?}", _mining_eligibility_proxy_result.proxy_claim_total_reward_amount);
                debug::info!("Inserted field proxy_claim_rewardees_data {:#?}", _mining_eligibility_proxy_result.proxy_claim_rewardees_data);
                debug::info!("Inserted field proxy_claim_block_redeemed {:#?}", _mining_eligibility_proxy_result.proxy_claim_block_redeemed);
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
