#![cfg_attr(not(feature = "std"), no_std)]

//! A pallet that ...

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
    dispatch::DispatchResult,
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
    self as system,
    ensure_signed,
};
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
// use hex_literal::hex;
// use sp_core::crypto::UncheckedFrom;
use sp_std::prelude::*;

pub trait Config:
    frame_system::Config + roaming_operators::Config + pallet_treasury::Config + pallet_balances::Config + pallet_timestamp::Config + pallet_elections_phragmen::Config
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
    type Randomness: Randomness<Self::Hash>;
    // type AccountId: UncheckedFrom<<Self as frame_system::Config>::Hash> + AsRef<[u8]>;
    type MiningRewardsTokenIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive())]
pub struct MiningRewardsToken(pub [u8; 16]);

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
        BalanceOf = BalanceOf<T>,
    {
        Test(AccountId, BalanceOf),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Config> as MiningRewardsToken {
        /// Stores all the mining_rewards_tokens, key is the mining_rewards_token id / index
        pub MiningRewardsTokens get(fn mining_rewards_token): map hasher(opaque_blake2_256) T::MiningRewardsTokenIndex => Option<MiningRewardsToken>;

        /// Stores the total number of mining_rewards_tokens. i.e. the next mining_rewards_token index
        pub MiningRewardsTokenCount get(fn mining_rewards_token_count): T::MiningRewardsTokenIndex;

        /// Stores mining_rewards_token owner
        pub MiningRewardsTokenOwners get(fn mining_rewards_token_owner): map hasher(opaque_blake2_256) T::MiningRewardsTokenIndex => Option<T::AccountId>;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        fn on_finalize(current_block_number: T::BlockNumber) {
            debug::info!("rewards - on_finalize");
            debug::info!("rewards - current block number {:#?}", current_block_number);

            let members_and_stake = <pallet_elections_phragmen::Module<T>>::members()
                .into_iter()
                .map(|m|
                    (m.who, m.stake)
                ).collect::<Vec<_>>();

            debug::info!("members_and_stake {:#?}", members_and_stake);
        }
    }
}
