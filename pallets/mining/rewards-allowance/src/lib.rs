//! <!-- markdown-link-check-disable -->
//! # Rewards Allowance with Offchain Worker Pallet
//!
//! TODO - add description
//!
//! Run `cargo doc --package rewards-allowance --open` to view this module's
//! documentation.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//!
//! ## Overview
//!
//! Offchain Worker (OCW) will be triggered after every block, fetch the mPower of current
//! of registered DHX users and prepare either signed or unsigned transaction to feed the
//! result back on chain.
//!
//! Additional logic in OCW is put in place to prevent spamming the network with both signed
//! and unsigned transactions, and custom `UnsignedValidator` makes sure that there is only
//! one unsigned transaction floating in the network.
//!
//! The on-chain logic will integrate their mPower values in the calculation of their
//! accumulated and aggregated rewards allowance each day.
//!
//! TODO - add further overview
//!
#![cfg_attr(not(feature = "std"), no_std)]

// We need this to allow use of `format!` in a no_std environment
#![crate_type = "dylib"]
#[macro_use]
extern crate alloc;

use core::str;
use chrono::{
    NaiveDateTime,
};
use codec::{
    Decode,
    Encode,
    Error,
    Input,
    Output,
};
use frame_support::{
    dispatch::DispatchResult,
    traits::{
        Currency,
        ExistenceRequirement,
        Get,
    },
};
use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendUnsignedTransaction, Signer, SubmitTransaction,
	},
};
use hex;                // to use hex::encode("...");
use hex_literal::{      // to use hex!("...");
    hex as write_hex,
};
// use serde::{Deserialize, Serialize};
use lite_json::json::JsonValue;
use log::{warn, info};
use module_primitives::{
    types::{
        AccountId,
        Balance,
        Signature,
    },
};
use pallet_balances::{BalanceLock};
use rand::{seq::SliceRandom, Rng};
use sp_core::{
    crypto::{KeyTypeId, Public},
    sr25519,
};
use sp_runtime::{
	offchain::{
		http,
		storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
		Duration,
	},
    transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
    traits::{
        IdentifyAccount,
        One,
        Verify,
        Zero,
    },
	RuntimeDebug,
};
use sp_std::{
    convert::{
        TryFrom,
        TryInto,
    },
    // vec::Vec,
    prelude::*, // Imports Vec
};
use substrate_fixed::{
    types::{
        extra::U3,
        U16F16,
        U32F32,
        U64F64,
    },
    FixedU32,
    FixedU128,
};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"mpow");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
        MultiSigner,
        MultiSignature,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	// implemented for off-chain workers in runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

    // implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // this is only a default for test purposes and fallback.
    // set this to 0u128 in production
    pub const default_bonded_amount_u128: u128 = 25_133_000_000_000_000_000_000u128;
    const TEN: u128 = 10_000_000_000_000_000_000_u128; // 10
    pub const FIVE_THOUSAND_DHX: u128 = 5_000_000_000_000_000_000_000_u128; // 5000

    // type BalanceOf<T> = <T as pallet_balances::Config>::Balance;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type BalanceFromBalancePallet<T> = <T as pallet_balances::Config>::Balance;
    type Date = i64;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: CreateSignedTransaction<Call<Self>>
        + frame_system::Config
        + pallet_democracy::Config
        + pallet_balances::Config
        + pallet_timestamp::Config
        + pallet_treasury::Config {

        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The overarching dispatch call type.
		type Call: From<Call<Self>>;

        type Currency: Currency<Self::AccountId>;

		// Configuration parameters

		/// A grace period after we send transaction.
		///
		/// To avoid sending too many transactions, we only attempt to send one
		/// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
		/// sending between distinct runs of this offchain worker.
		#[pallet::constant]
		type GracePeriod: Get<Self::BlockNumber>;

		/// Number of blocks of cooldown after unsigned transaction is included.
		///
		/// This ensures that we only accept unsigned transactions once, every `UnsignedInterval`
		/// blocks.
		#[pallet::constant]
		type UnsignedInterval: Get<Self::BlockNumber>;

		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple pallets send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;
    }

    #[derive(Debug)]
    struct MPowerAccountData<U, V> {
        acct_id: U,
        mpower: V,
    }

    #[derive(Debug)]
    struct MPowerJSONResponseData<U, V> {
        data: Vec<MPowerAccountData<U, V>>,
    }

    /// Payload used to hold mPower data required to submit a transaction.
    #[cfg_attr(feature = "std", derive(Debug))]
    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
    pub struct MPowerPayload<U, V, W, X> {
        pub account_id_registered_dhx_miner: U,
        pub mpower_registered_dhx_miner: V,
        pub received_at_date: W,
        pub received_at_block_number: X,
    }

    #[cfg_attr(feature = "std", derive(Debug))]
    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
    pub struct FetchedMPowerForAccountsForDatePayload<U, V, W> {
        pub start_of_requested_date_millis: U,
        pub all_miner_public_keys_fetched: V,
        pub total_mpower_fetched: W,
    }

    type MPowerPayloadData<T> = MPowerPayload<
        Vec<u8>, // <T as frame_system::Config>::AccountId,
        u128,
        Date,
        <T as frame_system::Config>::BlockNumber,
    >;

    type FetchedMPowerForAccountsForDatePayloadData = FetchedMPowerForAccountsForDatePayload<
        Date,
        Vec<Vec<u8>>,
        u128,
    >;

    // type MPowerAccountDataType<T> = MPowerAccountData<
    //     <T as frame_system::Config>::AccountId,
    //     u128,
    // >;

    enum TransactionType {
        Raw,
        None,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);
    // #[pallet::generate_storage_info]
    // pub struct Pallet<T>(PhantomData<T>);

    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage
    // Learn more about declaring storage items:
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
    #[pallet::storage]
    #[pallet::getter(fn bonded_dhx_of_account_for_date)]
    pub(super) type BondedDHXForAccountForDate<T: Config> = StorageMap<_, Blake2_128Concat,
        (
            Date,
            Vec<u8>, // public key of AccountId
        ),
        BalanceOf<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn rewards_allowance_dhx_for_date_remaining)]
    pub(super) type RewardsAllowanceDHXForDateRemaining<T: Config> = StorageMap<_, Blake2_128Concat,
        Date,
        BalanceOf<T>
    >;

    // checks if all eligible registered dhx miners have already received their reward for a specific date
    #[pallet::storage]
    #[pallet::getter(fn rewards_allowance_dhx_for_date_remaining_distributed)]
    pub(super) type RewardsAllowanceDHXForDateRemainingDistributed<T: Config> = StorageMap<_, Blake2_128Concat,
        Date,
        bool
    >;

    // checks if a specific registered dhx miner has already received their reward for a specific date
    #[pallet::storage]
    #[pallet::getter(fn rewards_allowance_dhx_for_miner_for_date_remaining_distributed)]
    pub(super) type RewardsAllowanceDHXForMinerForDateRemainingDistributed<T: Config> = StorageMap<_, Blake2_128Concat,
        (
            Date,
            Vec<u8>, // public key of AccountId
        ),
        bool
    >;

    #[pallet::storage]
    #[pallet::getter(fn rewards_allowance_dhx_daily)]
    pub(super) type RewardsAllowanceDHXDaily<T: Config> = StorageValue<_, BalanceOf<T>>;

    // store for ease of changing by governance
    // global pause
    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_paused)]
    pub(super) type RewardsMultiplierPaused<T: Config> = StorageValue<_, bool>;

    // global reset
    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_reset)]
    pub(super) type RewardsMultiplierReset<T: Config> = StorageValue<_, bool>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_default_change)]
    pub(super) type RewardsMultiplierDefaultChange<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_current_change)]
    pub(super) type RewardsMultiplierCurrentChange<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_next_change)]
    pub(super) type RewardsMultiplierNextChange<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_default_period_days)]
    pub(super) type RewardsMultiplierDefaultPeriodDays<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_next_period_days)]
    pub(super) type RewardsMultiplierNextPeriodDays<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_current_period_days_total)]
    pub(super) type RewardsMultiplierCurrentPeriodDaysTotal<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_current_period_days_remaining)]
    pub(super) type RewardsMultiplierCurrentPeriodDaysRemaining<T: Config> = StorageValue<_,
        (
            Date, // date when current period started
            Date, // previous date that was entered during countdown
            u32, // total days for this period
            u32, // days remaining
        ),
    >;

    // set to 1 for addition, to change the
    // min. bonded dhx value after every rewards_multiplier_next_period_days.
    #[pallet::storage]
    #[pallet::getter(fn rewards_multiplier_operation)]
    pub(super) type RewardsMultiplierOperation<T: Config> = StorageValue<_, u8>;

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
            Vec<u8>,
        ),
        BalanceOf<T>,
    >;

	/// Those who registered that they want to participate in DHX Mining
	///
	/// TWOX-NOTE: Safe, as increasing integer keys are safe.
    #[pallet::storage]
    #[pallet::getter(fn registered_dhx_miners)]
    pub(super) type RegisteredDHXMiners<T: Config> = StorageValue<_, Vec<Vec<u8>>>;

    #[pallet::storage]
    #[pallet::getter(fn rewards_eligible_miners_for_date)]
    pub(super) type RewardsEligibleMinersForDate<T: Config> = StorageMap<_, Blake2_128Concat,
        Date,
        Vec<Vec<u8>>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn min_bonded_dhx_daily)]
    pub(super) type MinBondedDHXDaily<T: Config> = StorageValue<_, BalanceOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn min_bonded_dhx_daily_default)]
    pub(super) type MinBondedDHXDailyDefault<T: Config> = StorageValue<_, BalanceOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn min_mpower_daily)]
    pub(super) type MinMPowerDaily<T: Config> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn min_mpower_daily_default)]
    pub(super) type MinMPowerDailyDefault<T: Config> = StorageValue<_, u128>;

    #[pallet::storage]
    #[pallet::getter(fn challenge_period_days)]
    pub(super) type ChallengePeriodDays<T: Config> = StorageValue<_, u64>;

    #[pallet::storage]
    #[pallet::getter(fn cooling_down_period_days)]
    pub(super) type CoolingDownPeriodDays<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn cooling_down_period_days_remaining)]
    pub(super) type CoolingDownPeriodDaysRemaining<T: Config> = StorageMap<_, Blake2_128Concat,
        Vec<u8>, // public key of AccountId
        (
            // date when cooling down period started for a given miner, or the date when we last reduced their cooling down period.
            // we do not reduce their cooling down period days remaining if we've already set this to a date that is the
            // current date for a miner (i.e. only reduce the days remaining once per day per miner)
            Date,
            u32, // days remaining
            u32, // 0: during cooldown period, 1: cooldown period finished (don't reset cooldown days remaining unless bond min. then unbond again)
        ),
    >;

    // Offchain workers

    /// Recently submitted mPower data.
    #[pallet::storage]
    #[pallet::getter(fn mpower_of_account_for_date)]
    pub(super) type MPowerForAccountForDate<T: Config> = StorageMap<_, Blake2_128Concat,
        (
            Date, // converted to start of date
            Vec<u8>, // T::AccountId,
        ),
        u128, // mPower
    >;

    /// Recently submitted finished fetching mPower data.
    #[pallet::storage]
    #[pallet::getter(fn finished_fetching_mpower_for_accounts_for_date)]
    pub(super) type FinishedFetchingMPowerForAccountsForDate<T: Config> = StorageMap<_, Blake2_128Concat,
        Date, // converted to start of date
        (
            Vec<Vec<u8>>,
            u128, // mPower
        ),
    >;

	/// Defines the block when next unsigned transaction (specifically for fetching the mPower) will be accepted.
	///
	/// To prevent spam of unsigned (and unpayed!) transactions on the network,
	/// we only allow one transaction every `T::UnsignedInterval` blocks.
	/// This storage entry defines when new transaction is going to be accepted.
	#[pallet::storage]
	#[pallet::getter(fn next_unsigned_at_for_fetched_mpower)]
	pub(super) type NextUnsignedAtForFetchedMPower<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	/// Defines the block when next unsigned transaction (specifically for finishing fetching the mPower) will be accepted.
	#[pallet::storage]
	#[pallet::getter(fn next_unsigned_at_for_finished_fetching_mpower)]
	pub(super) type NextUnsignedAtForFinishedFetchingMPower<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    // The genesis config type.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub rewards_allowance_dhx_for_date_remaining: Vec<(Date, BalanceOf<T>)>,
        pub rewards_allowance_dhx_for_date_remaining_distributed: Vec<(Date, bool)>,
        pub rewards_allowance_dhx_for_miner_for_date_remaining_distributed: Vec<((Date, Vec<u8>), bool)>,
        pub rewards_allowance_dhx_daily: BalanceOf<T>,
        pub rewards_multiplier_paused: bool,
        pub rewards_multiplier_reset: bool,
        pub rewards_multiplier_default_change: u32,
        pub rewards_multiplier_current_change: u32,
        pub rewards_multiplier_next_change: u32,
        pub rewards_multiplier_default_period_days: u32,
        pub rewards_multiplier_next_period_days: u32,
        pub rewards_multiplier_current_period_days_total: u32,
        pub rewards_multiplier_current_period_days_remaining: (Date, Date, u32, u32),
        pub rewards_multiplier_operation: u8,
        pub rewards_aggregated_dhx_for_all_miners_for_date: Vec<(Date, BalanceOf<T>)>,
        pub rewards_accumulated_dhx_for_miner_for_date: Vec<((Date, Vec<u8>), BalanceOf<T>)>,
        pub registered_dhx_miners: Vec<Vec<u8>>,
        pub rewards_eligible_miners_for_date: Vec<(Date, Vec<Vec<u8>>)>,
        pub min_bonded_dhx_daily: BalanceOf<T>,
        pub min_bonded_dhx_daily_default: BalanceOf<T>,
        pub min_mpower_daily: u128,
        pub min_mpower_daily_default: u128,
        pub challenge_period_days: u64,
        pub cooling_down_period_days: u32,
        pub cooling_down_period_days_remaining: Vec<(Vec<u8>, (Date, u32, u32))>,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                rewards_allowance_dhx_for_date_remaining: Default::default(),
                rewards_allowance_dhx_for_date_remaining_distributed: Default::default(),
                rewards_allowance_dhx_for_miner_for_date_remaining_distributed: Default::default(),
                // 5000 UNIT, where UNIT token has 18 decimal places
                rewards_allowance_dhx_daily: Default::default(),
                rewards_multiplier_paused: false,
                rewards_multiplier_reset: false,
                rewards_multiplier_default_change: 10u32,
                rewards_multiplier_current_change: 10u32,
                rewards_multiplier_next_change: 10u32,
                // FIXME - setup for different amount of days each month and leap years
                rewards_multiplier_default_period_days: 90u32,
                // FIXME - setup for different amount of days each month and leap years
                rewards_multiplier_next_period_days: 90u32,
                // FIXME - setup for different amount of days each month and leap years
                rewards_multiplier_current_period_days_total: 90u32,
                rewards_multiplier_current_period_days_remaining: Default::default(),
                rewards_multiplier_operation: 1u8,
                rewards_aggregated_dhx_for_all_miners_for_date: Default::default(),
                rewards_accumulated_dhx_for_miner_for_date: Default::default(),
                registered_dhx_miners: vec![
                    Default::default(),
                    Default::default(),
                ],
                rewards_eligible_miners_for_date: Default::default(),
                min_bonded_dhx_daily: Default::default(),
                min_bonded_dhx_daily_default: Default::default(),
                min_mpower_daily: 5u128,
                min_mpower_daily_default: 5u128,
                challenge_period_days: Default::default(),
                cooling_down_period_days: Default::default(),
                // Note: this doesn't seem to work, even if it's just `vec![Default::default()]` it doesn't use
                // the defaults in chain_spec.rs, so we set defaults later with `let mut cooling_down_period_days_remaining`
                cooling_down_period_days_remaining: vec![
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
            for (a, b) in &self.rewards_allowance_dhx_for_date_remaining {
                <RewardsAllowanceDHXForDateRemaining<T>>::insert(a, b);
            }
            for (a, b) in &self.rewards_allowance_dhx_for_date_remaining_distributed {
                <RewardsAllowanceDHXForDateRemainingDistributed<T>>::insert(a, b);
            }
            for ((a, b), c) in &self.rewards_allowance_dhx_for_miner_for_date_remaining_distributed {
                <RewardsAllowanceDHXForMinerForDateRemainingDistributed<T>>::insert((a, b), c);
            }
            <RewardsAllowanceDHXDaily<T>>::put(&self.rewards_allowance_dhx_daily);
            <RewardsMultiplierPaused<T>>::put(&self.rewards_multiplier_paused);
            <RewardsMultiplierReset<T>>::put(&self.rewards_multiplier_reset);
            <RewardsMultiplierDefaultChange<T>>::put(&self.rewards_multiplier_default_change);
            <RewardsMultiplierCurrentChange<T>>::put(&self.rewards_multiplier_current_change);
            <RewardsMultiplierNextChange<T>>::put(&self.rewards_multiplier_next_change);
            <RewardsMultiplierDefaultPeriodDays<T>>::put(&self.rewards_multiplier_default_period_days);
            <RewardsMultiplierNextPeriodDays<T>>::put(&self.rewards_multiplier_next_period_days);
            <RewardsMultiplierCurrentPeriodDaysTotal<T>>::put(&self.rewards_multiplier_current_period_days_total);
            <RewardsMultiplierCurrentPeriodDaysRemaining<T>>::put(&self.rewards_multiplier_current_period_days_remaining);
            for (a) in &self.registered_dhx_miners {
                <RegisteredDHXMiners<T>>::append(a);
            }
            for (a, b) in &self.rewards_eligible_miners_for_date {
                <RewardsEligibleMinersForDate<T>>::insert(a, b);
            }
            <RewardsMultiplierOperation<T>>::put(&self.rewards_multiplier_operation);
            for (a, b) in &self.rewards_aggregated_dhx_for_all_miners_for_date {
                <RewardsAggregatedDHXForAllMinersForDate<T>>::insert(a, b);
            }
            for ((a, b), c) in &self.rewards_accumulated_dhx_for_miner_for_date {
                <RewardsAccumulatedDHXForMinerForDate<T>>::insert((a, b), c);
            }
            <MinBondedDHXDaily<T>>::put(&self.min_bonded_dhx_daily);
            <MinBondedDHXDailyDefault<T>>::put(&self.min_bonded_dhx_daily_default);
            <MinMPowerDaily<T>>::put(&self.min_mpower_daily);
            <MinMPowerDailyDefault<T>>::put(&self.min_mpower_daily_default);
            <ChallengePeriodDays<T>>::put(&self.challenge_period_days);
            <CoolingDownPeriodDays<T>>::put(&self.cooling_down_period_days);
            for (a, (b, c, d)) in &self.cooling_down_period_days_remaining {
                <CoolingDownPeriodDaysRemaining<T>>::insert(a, (b, c, d));
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
        Date = "Date",
        T::BlockNumber = "BlockNumber",
    )]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Storage of provided accounts as a registered DHX miners
        /// \[sender, registered_dhx_miners]
        SetRegisteredDHXMiners(Vec<Vec<u8>>),

        /// Storage of the minimum DHX that must be bonded by each registered DHX miner each day
        /// to be eligible for rewards
        /// \[amount_dhx\]
        SetMinBondedDHXDailyStored(BalanceOf<T>),

        /// Storage of the minimum mPower that must be held by each registered DHX miner each day
        /// to be eligible for rewards
        /// \[amount_mpower\]
        SetMinMPowerDailyStored(u128),

        /// Storage of the default challenge period in days
        /// \[challenge_period_days\]
        SetChallengePeriodDaysStored(u64),

        /// Storage of the default cooling down period in days
        /// \[cooling_down_period_days\]
        SetCoolingDownPeriodDaysStored(u32),

        /// Storage of the bonded DHX of an account on a specific date.
        /// \[date, amount_dhx_bonded, account_dhx_bonded\]
        SetBondedDHXOfAccountForDateStored(Date, BalanceOf<T>, Vec<u8>),

        /// Storage of the default daily reward allowance in DHX by an origin account.
        /// \[amount_dhx\]
        SetRewardsAllowanceDHXDailyStored(BalanceOf<T>),

        /// Change the stored reward allowance in DHX for a specific date by an origin account, and
        /// where change is 0 for an decrease or any other value like 1 for an increase to the remaining
        /// rewards allowance.
        /// \[date, change_amount_dhx, change\]
        ChangedRewardsAllowanceDHXForDateRemainingStored(Date, BalanceOf<T>, u8),

        /// Transferred a proportion of the daily DHX rewards allowance to a DHX Miner on a given date
        /// \[date, miner_reward, remaining_rewards_allowance_today, miner_public_key\]
        TransferredRewardsAllowanceDHXToMinerForDate(Date, BalanceOf<T>, BalanceOf<T>, Vec<u8>),

        /// Exhausted distributing all the daily DHX rewards allowance to DHX Miners on a given date.
        /// Note: There may be some leftover for the day so we record it here
        /// \[date, remaining_rewards_allowance_today\]
        DistributedRewardsAllowanceDHXForDateRemaining(Date, BalanceOf<T>),

        /// Changed the min. bonded DHX daily using a value, using addition operation
        /// \[start_date_period, new_min_dhx_bonded, modified_old_min_dhx_bonded_using_ratio, operation_used, next_period_days\]
        ChangedMinBondedDHXDailyUsingNewRewardsMultiplier(Date, BalanceOf<T>, u32, u8, u32),

        /// Storage of a new reward operation (1u8: addition) by an origin account.
        /// \[operation\]
        SetRewardsMultiplierOperationStored(u8),

        /// Storage of a new reward multiplier default period in days (i.e. 90 for 3 months) by an origin account.
        /// \[days\]
        SetRewardsMultiplierDefaultPeriodDaysStored(u32),

        /// Storage of a new reward multiplier next period in days (i.e. 90 for 3 months) by an origin account.
        /// \[days\]
        SetRewardsMultiplierNextPeriodDaysStored(u32),

        /// Storage of new rewards multiplier paused status
        /// \[new_status]
        ChangeRewardsMultiplierPausedStatusStored(bool),

        /// Storage of new rewards multiplier reset status
        /// \[new_status]
        ChangeRewardsMultiplierResetStatusStored(bool),

        // Off-chain workers

		/// Event generated when new mPower data is accepted to contribute to the rewards allowance.
		/// \[start_date_requested, registered_dhx_miner_account_id, mpower\]
        NewMPowerForAccountForDate(Date, Vec<u8>, u128),

		/// Event generated when new finished fetching mPower data is stored.
		/// \[start_date_requested, all_miner_public_keys_fetched, total_mpower_fetched\]
        NewFinishedFetchingMPowerForAccountsForDate(Date, Vec<Vec<u8>>, u128),
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
		/// Offchain Worker entry point.
		///
		/// By implementing `fn offchain_worker` you declare a new offchain worker.
		/// This function will be called when the node is fully synced and a new best block is
		/// succesfuly imported.
		/// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		/// You can use `Local Storage` API to coordinate runs of the worker.
        ///
        /// https://paritytech.github.io/substrate/master/frame_support/traits/trait.OffchainWorker.html
        /// https://paritytech.github.io/substrate/master/frame_support/traits/trait.Hooks.html
        ///
        /// Implementing this trait on a module allows you to perform long-running tasks that make
        /// (by default) validators generate transactions that feed results of those long-running
        /// computations back on chain.
        /// NOTE: This function runs off-chain, so it can access the block state, but cannot preform
        /// any alterations. More specifically alterations are not forbidden, but they are not persisted
        /// in any way after the worker has finished.
		fn offchain_worker(block_number: T::BlockNumber) {
            log::info!("offchain_worker: {:?}", block_number.clone());
            println!("offchain_worker: {:?}", block_number.clone());
			// Note that having logs compiled to WASM may cause the size of the blob to increase
			// significantly. You can use `RuntimeDebug` custom derive to hide details of the types
			// in WASM. The `sp-api` crate also provides a feature `disable-logging` to disable
			// all logging and thus, remove any logging from the WASM.

			// Since off-chain workers are just part of the runtime code, they have direct access
			// to the storage and other included pallets.
			//
			// We can easily import `frame_system` and retrieve a block hash of the parent block.
			// let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			// log::debug!("offchain_workers current block: {:?} (parent hash: {:?})", block_number, parent_hash);

            let should_process_block = Self::should_process_block(block_number.clone());
            if should_process_block == false {
                return;
            }

            // TODO - this timestamp section is duplicated in the on_initialize function,
            // so move it into its owh private function

            // Anything that needs to be done at the start of the block.
            let timestamp: <T as pallet_timestamp::Config>::Moment = <pallet_timestamp::Pallet<T>>::get();
            log::info!("offchain_worker - block_number: {:?}", block_number.clone());
            log::info!("offchain_worker - timestamp: {:?}", timestamp.clone());
            let requested_date_as_u64;
            let _requested_date_as_u64 = Self::convert_moment_to_u64_in_milliseconds(timestamp.clone());
            match _requested_date_as_u64 {
                Err(_e) => {
                    log::error!("offchain_worker - Unable to convert Moment to u64 in millis for timestamp");
                    return;
                },
                Ok(ref x) => {
                    requested_date_as_u64 = x;
                }
            }
            log::info!("offchain_worker - requested_date_as_u64: {:?}", requested_date_as_u64.clone());
            println!("offchain_worker - requested_date_as_u64: {:?}", requested_date_as_u64.clone());

            // do not run when block number is 1, which is when timestamp is 0 because this
            // timestamp corresponds to 1970-01-01
            if requested_date_as_u64.clone() == 0u64 {
                return;
            }

            let start_of_requested_date_millis;
            let _start_of_requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(requested_date_as_u64.clone());
            match _start_of_requested_date_millis {
                Err(_e) => {
                    log::error!("offchain_worker - Unable to convert u64 in milliseconds to start_of_requested_date_millis");
                    return;
                },
                Ok(ref x) => {
                    start_of_requested_date_millis = x;
                }
            }
            log::info!("offchain_worker - start_of_requested_date_millis: {:?}", start_of_requested_date_millis.clone());
            println!("offchain_worker - start_of_requested_date_millis: {:?}", start_of_requested_date_millis.clone());

			// after fetching the mpower values store by sending an unsigned transactions
			let should_send = Self::choose_transaction_type(block_number.clone());
            let mut mpower_data_vec = vec![];
            match should_send {
				TransactionType::Raw => {
                    let _mpower_res = Self::fetch_mpower_process(block_number.clone(), start_of_requested_date_millis.clone());
                    match _mpower_res.clone() {
                        Err(e) => {
                            log::error!("offchain_worker - offchain_workers error fetching mpower: {}", e);
                            return;
                        },
                        Ok(x) => {
                            mpower_data_vec = x;
                        }
                    }
                },
				TransactionType::None => {
                    log::error!("offchain_worker - offchain_workers error unknown transaction type");
                    return;
                },
			};

            // TODO - remove all the fetched accounts from vector that aren't in the list of
            // reg_dhx_miners, since we add the list of registered dhx miners from a signed account to
            // and use it incase the API endpoint is compromised by hackers and fake accounts and mpower data are provided

            // TODO - change reg_dhx_miners functionality so it may be stored via an extrinsic function
            // and voted on by governance, and so it just includes a list of account addresses that we check against.
            // if the list needs to be changed then just call it and add all of them again once approved.

            // TODO - verify these fetched public keys correspond to registered dhx miners,
            // and if not we should only store mpower data corresponding to those that are
            let mut all_miner_public_keys_fetched = vec![];
            let mut total_mpower_fetched = 0u128;
            for (index, mpower_data_item) in mpower_data_vec.iter().enumerate() {
                log::info!("offchain_worker - mpower_data_vec {:?}, {:?}", index.clone(), mpower_data_item.account_id_registered_dhx_miner.clone());
                log::info!("offchain_worker - mpower_data_vec {:?}, {:?}", index.clone(), mpower_data_item.mpower_registered_dhx_miner.clone());
                all_miner_public_keys_fetched.push(mpower_data_item.account_id_registered_dhx_miner.clone());

                let new_total_mpower_fetched = total_mpower_fetched.clone();
                let _total_mpower_fetched =
                    new_total_mpower_fetched.checked_add(mpower_data_item.mpower_registered_dhx_miner.clone());
                match _total_mpower_fetched {
                    None => {
                        log::error!("Unable to add mpower_data_item.mpower_registered_dhx_miner with total_mpower_fetched due to StorageOverflow");
                        return;
                    },
                    Some(x) => {
                        total_mpower_fetched = x;
                    }
                }
            }
            log::info!("offchain_worker - total_mpower_fetched {:?}, {:?}", total_mpower_fetched.clone(), block_number.clone());

            let fetched_mpower_data: FetchedMPowerForAccountsForDatePayloadData = FetchedMPowerForAccountsForDatePayload {
                start_of_requested_date_millis: start_of_requested_date_millis.clone(),
                all_miner_public_keys_fetched: all_miner_public_keys_fetched.clone(),
                total_mpower_fetched: total_mpower_fetched.clone(),
            };

            let res_store_mpower = Self::store_mpower_raw_unsigned(
                block_number.clone(),
                start_of_requested_date_millis.clone(),
                mpower_data_vec.clone()
            );
            log::info!("offchain_worker - finished store_mpower_raw_unsigned at block: {:?}", block_number.clone());
            if let Err(e) = res_store_mpower {
                log::error!("offchain_worker - offchain_workers error storing mpower: {}", e);
            }

            let res_store_finished = Self::store_finished_fetching_mpower_raw_unsigned(
                block_number.clone(),
                fetched_mpower_data.clone()
            );
            log::info!("offchain_worker - finished store_finished_fetching_mpower_raw_unsigned at block: {:?}", block_number.clone());
            if let Err(e) = res_store_finished {
                log::error!("offchain_worker - offchain_workers error storing finished fetching mpower: {}", e);
            }

            return;
        }

        // `on_initialize` is executed at the beginning of the block before any extrinsic are
        // dispatched.
        //
        // This function must return the weight consumed by `on_initialize` and `on_finalize`.
        // TODO - update with the weight consumed
        fn on_initialize(block_number: T::BlockNumber) -> Weight {
            let should_process_block = Self::should_process_block(block_number.clone());
            if should_process_block == false {
                return 0;
            }

            // Anything that needs to be done at the start of the block.
            let timestamp: <T as pallet_timestamp::Config>::Moment = <pallet_timestamp::Pallet<T>>::get();
            log::info!("block_number: {:?}", block_number.clone());
            println!("block_number: {:?}", block_number.clone());
            log::info!("timestamp: {:?}", timestamp.clone());
            println!("timestamp: {:?}", timestamp.clone());
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
            println!("requested_date_as_u64: {:?}", requested_date_as_u64.clone());

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
            println!("start_of_requested_date_millis: {:?}", start_of_requested_date_millis.clone());

            // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
            let contains_key = <RewardsAllowanceDHXForDateRemaining<T>>::contains_key(&start_of_requested_date_millis);
            log::info!("contains_key for date: {:?}, {:?}", start_of_requested_date_millis.clone(), contains_key.clone());

            let mut rewards_allowance_dhx_daily: BalanceOf<T> = 5000u32.into(); // initialize;

            // add the start_of_requested_date to storage if it doesn't already exist
            if contains_key == false {
                if let Some(_rewards_allowance_dhx_daily) = <RewardsAllowanceDHXDaily<T>>::get() {
                    rewards_allowance_dhx_daily = _rewards_allowance_dhx_daily;
                } else {
                    log::error!("Unable to get rewards_allowance_dhx_daily");
                    return 0;
                }

                // Update storage. Use RewardsAllowanceDHXDaily as fallback incase not previously set prior to block
                <RewardsAllowanceDHXForDateRemaining<T>>::insert(start_of_requested_date_millis.clone(), &rewards_allowance_dhx_daily);
                <RewardsAllowanceDHXForDateRemainingDistributed<T>>::insert(start_of_requested_date_millis.clone(), false);
                log::info!("on_initialize");
                log::info!("rewards_allowance: {:?}", &rewards_allowance_dhx_daily);
            }

            let mut rm_paused = false; // have to initialise
            if let Some(_rm_paused) = <RewardsMultiplierPaused<T>>::get() {
                rm_paused = _rm_paused;
            } else {
                log::info!("Unable to get rm_paused");
            }

            let mut rm_reset = false;
            if let Some(_rm_reset) = <RewardsMultiplierReset<T>>::get() {
                rm_reset = _rm_reset;
            } else {
                log::info!("Unable to get rm_reset");
            }

            let mut rm_default_change = 10u32;
            if let Some(_rm_default_change) = <RewardsMultiplierDefaultChange<T>>::get() {
                rm_default_change = _rm_default_change;
            } else {
                log::info!("Unable to get rm_default_change");
            }

            let mut rm_current_change = 10u32;
            if let Some(_rm_current_change) = <RewardsMultiplierCurrentChange<T>>::get() {
                rm_current_change = _rm_current_change;
            } else {
                log::info!("Unable to get rm_current_change");
            }

            let mut rm_next_change = 10u32;
            if let Some(_rm_next_change) = <RewardsMultiplierNextChange<T>>::get() {
                rm_next_change = _rm_next_change;
            } else {
                log::info!("Unable to get rm_next_change");
            }

            let mut rm_default_period_days = 90u32;
            if let Some(_rm_default_period_days) = <RewardsMultiplierDefaultPeriodDays<T>>::get() {
                rm_default_period_days = _rm_default_period_days;
            } else {
                log::info!("Unable to get rm_default_period_days");
            }

            let mut rm_next_period_days = 90u32;
            if let Some(_rm_next_period_days) = <RewardsMultiplierNextPeriodDays<T>>::get() {
                rm_next_period_days = _rm_next_period_days;
            } else {
                log::info!("Unable to get rm_next_period_days");
            }

            let mut min_bonded_dhx_daily_default: BalanceOf<T> = 10u32.into();
            let mut min_bonded_dhx_daily_default_u128;
            let _min_bonded_dhx_daily_default = Self::get_min_bonded_dhx_daily_default();
            match _min_bonded_dhx_daily_default {
                Err(_e) => {
                    log::error!("Unable to retrieve any min. bonded DHX daily default as BalanceOf and u128");
                    return 0;
                },
                Ok(ref x) => {
                    min_bonded_dhx_daily_default = x.0;
                    min_bonded_dhx_daily_default_u128 = x.1;
                }
            }
            println!("min_bonded_dhx_daily_default_u128: {:?}", min_bonded_dhx_daily_default_u128.clone());

            let mut min_mpower_daily_default: u128 = 5u128;
            if let Some(_min_mpower_daily_default) = <MinMPowerDailyDefault<T>>::get() {
                min_mpower_daily_default = _min_mpower_daily_default;
            } else {
                log::info!("Unable to get min_mpower_daily_default");
            }
            println!("min_mpower_daily_default {:?}", min_mpower_daily_default);

            let mut rm_current_period_days_remaining = (
                0.into(),
                0.into(),
                90u32,
                90u32,
            );
            if let Some(_rm_current_period_days_remaining) = <RewardsMultiplierCurrentPeriodDaysRemaining<T>>::get() {
                rm_current_period_days_remaining = _rm_current_period_days_remaining;
            } else {
                log::info!("Unable to get rm_current_period_days_remaining");
            }

            log::info!("rm_paused: {:?}", &rm_paused);
            log::info!("rm_reset: {:?}", &rm_reset);
            log::info!("rm_default_change: {:?}", &rm_default_change);
            log::info!("rm_current_change: {:?}", &rm_current_change);
            log::info!("rm_next_change: {:?}", &rm_next_change);
            log::info!("rm_default_period_days: {:?}", &rm_default_period_days);
            log::info!("rm_next_period_days: {:?}", &rm_next_period_days);
            log::info!("rm_current_period_days_remaining: {:?}", &rm_current_period_days_remaining);

            println!("rm_paused: {:?}", &rm_paused);
            println!("rm_reset: {:?}", &rm_reset);
            println!("rm_default_change: {:?}", &rm_default_change);
            println!("rm_current_change: {:?}", &rm_current_change);
            println!("rm_next_change: {:?}", &rm_next_change);
            println!("rm_default_period_days: {:?}", &rm_default_period_days);
            println!("rm_next_period_days: {:?}", &rm_next_period_days);
            println!("rm_current_period_days_remaining: {:?}", &rm_current_period_days_remaining);

            // pause the process of automatically changing to the next period change and next period day
            // until unpaused again by governance
            if rm_paused != true {
                if rm_reset == true {
                    <RewardsMultiplierCurrentChange<T>>::put(rm_default_change.clone());
                    <RewardsMultiplierNextChange<T>>::put(rm_default_change.clone());
                    <RewardsMultiplierNextPeriodDays<T>>::put(rm_default_period_days.clone());
                    <RewardsMultiplierReset<T>>::put(false);
                    <MinBondedDHXDaily<T>>::put(min_bonded_dhx_daily_default.clone());
                    <MinMPowerDaily<T>>::put(min_mpower_daily_default.clone());
                }

                let is_block_two = Self::is_block_two(block_number.clone());

                // start on block #2 since timestamp is 0 in blocks before that
                if is_block_two.clone() == true {
                    // initialise values in storage that cannot be set in genesis and apply to local variables
                    // incase its just after genesis when values are not yet set in storage
                    <RewardsMultiplierCurrentPeriodDaysRemaining<T>>::put(
                        (
                            start_of_requested_date_millis.clone(), // start date of period
                            start_of_requested_date_millis.clone(), // previous date (is today) of period
                            rm_default_period_days.clone(),
                            rm_default_period_days.clone(),
                        )
                    );
                    if let Some(_rm_current_period_days_remaining) = <RewardsMultiplierCurrentPeriodDaysRemaining<T>>::get() {
                        rm_current_period_days_remaining = _rm_current_period_days_remaining;
                    } else {
                        log::info!("Unable to get rm_current_period_days_remaining");
                    }
                } else {
                    // any block after block #2

                    // if the value we stored in RewardsMultiplierCurrentPeriodDaysRemaining to represent the previous day's
                    // date is not the current date (since we don't want it to happen again until the next day)

                    if rm_current_period_days_remaining.1 != start_of_requested_date_millis.clone() {
                        // if there are still days remaining in the countdown
                        if rm_current_period_days_remaining.3 > 0u32 {
                            println!("[reducing_multiplier_days] block: {:#?}, date_start: {:#?} remain_days: {:#?}", block_number, rm_current_period_days_remaining.0, rm_current_period_days_remaining.3);
                            let old_rm_current_period_days_remaining = rm_current_period_days_remaining.3.clone();

                            // Subtract, handling overflow
                            let new_rm_current_period_days_remaining;
                            let _new_rm_current_period_days_remaining =
                                old_rm_current_period_days_remaining.checked_sub(One::one());
                            match _new_rm_current_period_days_remaining {
                                None => {
                                    log::error!("Unable to subtract one from rm_current_period_days_remaining due to StorageOverflow");
                                    return 0;
                                },
                                Some(x) => {
                                    new_rm_current_period_days_remaining = x;
                                }
                            }

                            // Write the new value to storage
                            <RewardsMultiplierCurrentPeriodDaysRemaining<T>>::put(
                                (
                                    rm_current_period_days_remaining.0, // retain original value
                                    start_of_requested_date_millis.clone(), // insert today's date for the previous date
                                    rm_current_period_days_remaining.2, // retain original value
                                    new_rm_current_period_days_remaining.clone(),
                                ),
                            );
                            log::info!("Reduced RewardsMultiplierCurrentPeriodDaysRemaining {:?} {:?}", start_of_requested_date_millis.clone(), new_rm_current_period_days_remaining.clone());
                        } else {
                            // if no more days remaining
                            println!("[reducing_multiplier_days] no more remaining days");

                            // run an operation with the the next change and the current min bonded dhx daily to determine the
                            // new min. bonded dhx daily for the next period

                            let mut min_bonded_dhx_daily: BalanceOf<T> = 10u32.into();
                            let mut min_bonded_dhx_daily_u128;
                            let _min_bonded_dhx_daily = Self::get_min_bonded_dhx_daily();
                            match _min_bonded_dhx_daily {
                                Err(_e) => {
                                    log::error!("Unable to retrieve any min. bonded DHX daily as BalanceOf and u128");
                                    return 0;
                                },
                                Ok(ref x) => {
                                    min_bonded_dhx_daily = x.0;
                                    min_bonded_dhx_daily_u128 = x.1;
                                }
                            }
                            println!("min_bonded_dhx_daily_u128: {:?}", min_bonded_dhx_daily_u128.clone());

                            let rewards_multipler_operation;
                            if let Some(_rewards_multipler_operation) = <RewardsMultiplierOperation<T>>::get() {
                                rewards_multipler_operation = _rewards_multipler_operation;
                            } else {
                                log::error!("Unable to retrieve rewards_multipler_operation");
                                return 0;
                            }

                            let mut new_min_bonded_dhx_daily_u128 = 0u128; // initialize

                            println!("rewards_multipler_operation: {:?}", rewards_multipler_operation.clone());

                            // prepare for 'add' operation

                            let mut rm_next_change_u128_short = 0u128; // initialize
                            if let Some(_rm_next_change_u128_short) = TryInto::<u128>::try_into(rm_next_change.clone()).ok() {
                                rm_next_change_u128_short = _rm_next_change_u128_short;
                            } else {
                                log::error!("Unable to convert u32 to u128");
                                return 0;
                            }

                            let ONE = 1000000000000000000u128;
                            let mut rm_next_change_as_fixedu128 = FixedU128::from_num(0);
                            let _rm_next_change_as_fixedu128 =
                                U64F64::from_num(rm_next_change_u128_short.clone()).checked_mul(U64F64::from_num(ONE));
                            match _rm_next_change_as_fixedu128 {
                                None => {
                                    log::error!("Unable to mult rm_next_change by ONE due to StorageOverflow");
                                    return 0;
                                },
                                Some(x) => {
                                    rm_next_change_as_fixedu128 = x;
                                }
                            }
                            println!("rm_next_change_as_fixedu128: {:?}", rm_next_change_as_fixedu128.clone());
                            // round down the fixed point number to the nearest integer of type u128
                            let rm_next_change_u128: u128 = rm_next_change_as_fixedu128.floor().to_num::<u128>();
                            println!("rm_next_change_u128: {:?}", rm_next_change_as_fixedu128.clone());

                            // case of addition
                            if rewards_multipler_operation == 1u8 {
                                // To 'add' rm_next_change u32 to min_bonded_dhx_daily_u128 we first need to convert
                                // the say 10 u32 value to u128, then multiply it by 1000000000000000000 so
                                // has same 18 decimal places representation, and only then 'add' it to
                                // min_bonded_dhx_daily_u128

                                let _new_min_bonded_dhx_daily_u128 =
                                    (rm_next_change_u128).checked_add(min_bonded_dhx_daily_u128.clone());
                                match _new_min_bonded_dhx_daily_u128 {
                                    None => {
                                        log::error!("Unable to add min_bonded_dhx_daily_u128 with rm_next_change_u128 due to StorageOverflow");
                                        return 0;
                                    },
                                    Some(x) => {
                                        new_min_bonded_dhx_daily_u128 = x;
                                    }
                                }
                            } else {
                                log::error!("Unsupported rewards_multipler_operation value");
                                return 0;
                            }

                            println!("new_min_bonded_dhx_daily_u128 {:?}", new_min_bonded_dhx_daily_u128);

                            let new_min_bonded_dhx_daily;
                            let _new_min_bonded_dhx_daily = Self::convert_u128_to_balance(new_min_bonded_dhx_daily_u128.clone());
                            match _new_min_bonded_dhx_daily {
                                Err(_e) => {
                                    log::error!("Unable to convert u128 to balance for new_min_bonded_dhx_daily");
                                    return 0;
                                },
                                Ok(ref x) => {
                                    new_min_bonded_dhx_daily = x;
                                }
                            }
                            log::info!("new_min_bonded_dhx_daily: {:?}", new_min_bonded_dhx_daily.clone());
                            println!("new_min_bonded_dhx_daily: {:?}", new_min_bonded_dhx_daily.clone());

                            <MinBondedDHXDaily<T>>::put(new_min_bonded_dhx_daily.clone());
                            log::info!("New MinBondedDHXDaily {:?} {:?}", start_of_requested_date_millis.clone(), new_min_bonded_dhx_daily_u128.clone());
                            println!("New MinBondedDHXDaily {:?} {:?}", start_of_requested_date_millis.clone(), new_min_bonded_dhx_daily_u128.clone());

                            // FIXME - can we automatically change the next period days value to (~90 days depending on days in included months 28, 29, 30, or 31)
                            // depending on the date? and do this from genesis too?

                            Self::deposit_event(Event::ChangedMinBondedDHXDailyUsingNewRewardsMultiplier(
                                start_of_requested_date_millis.clone(),
                                new_min_bonded_dhx_daily.clone(),
                                rm_next_change.clone(),
                                rewards_multipler_operation.clone(),
                                rm_next_period_days.clone(),
                            ));

                            // Set the current change (for this next period) to the value that was set as the
                            // next change (perhaps by governance)
                            <RewardsMultiplierCurrentChange<T>>::put(rm_next_change.clone());

                            <RewardsMultiplierNextChange<T>>::put(rm_next_change.clone());

                            // Set the current period in days (for this next period) to the value that was set as the
                            // next period days (perhaps by governance)
                            <RewardsMultiplierNextPeriodDays<T>>::put(rm_next_period_days.clone());

                            <RewardsMultiplierCurrentPeriodDaysTotal<T>>::put(rm_next_period_days.clone());

                            // Restart the days remaining for the next period
                            <RewardsMultiplierCurrentPeriodDaysRemaining<T>>::put(
                                (
                                    start_of_requested_date_millis.clone(), // insert today's date for the start date of the new period
                                    start_of_requested_date_millis.clone(), // insert today's date for the previous date
                                    rm_next_period_days.clone(), // total days
                                    rm_next_period_days.clone(), // remaining days
                                ),
                            );
                            log::info!("Restarting RewardsMultiplierCurrentPeriodDaysRemaining {:?} {:?}", start_of_requested_date_millis.clone(), rm_next_period_days.clone());

                            let new_min_bonded_dhx_daily;
                            let _new_min_bonded_dhx_daily = Self::convert_u128_to_balance(new_min_bonded_dhx_daily_u128.clone());
                            match _new_min_bonded_dhx_daily {
                                Err(_e) => {
                                    log::error!("Unable to convert u128 to balance for new_min_bonded_dhx_daily");
                                    return 0;
                                },
                                Ok(ref x) => {
                                    new_min_bonded_dhx_daily = x;
                                }
                            }
                            log::info!("new_min_bonded_dhx_daily: {:?}", new_min_bonded_dhx_daily.clone());
                            println!("new_min_bonded_dhx_daily: {:?}", new_min_bonded_dhx_daily.clone());
                        }
                    }
                }
            }
            // we only check accounts that have registered that they want to participate in DHX Mining
            let reg_dhx_miners;
            if let Some(_reg_dhx_miners) = <RegisteredDHXMiners<T>>::get() {
                reg_dhx_miners = _reg_dhx_miners;
            } else {
                log::error!("Unable to retrieve any registered DHX Miners");
                println!("Unable to retrieve any registered DHX Miners");
                return 0;
            }
            if reg_dhx_miners.len() == 0 {
                log::error!("Registered DHX Miners has no elements");
                println!("Unable to retrieve any registered DHX Miners");
                return 0;
            };

            // TODO - uncomment this

            // // only continue with aggregating and accumulating rewards and using mPower data that
            // // was fetched offchain when the unsigned tx
            // // from offchain_workers that indicates we have finished fetching mpower data for the
            // // day has been finished
            // let data_finished_fetching_for_date =
            //     <FinishedFetchingMPowerForAccountsForDate<T>>::get(start_of_requested_date_millis.clone());

            // match data_finished_fetching_for_date {
            //     None => {
            //         log::warn!("Skipping this block as we have not yet finished fetching mpower data for today from offchain workers {:?}", start_of_requested_date_millis.clone());
            //         return 0;
            //     },
            //     Some(x) => {
            //     }
            // }

            let mut miner_count = 0;

            for (index, miner_public_key) in reg_dhx_miners.iter().enumerate() {
                miner_count += 1;
                log::info!("miner_count {:#?}", miner_count);
                println!("miner_count {:#?}", miner_count);
                log::info!("miner_public_key {:?}", miner_public_key);
                println!("miner_public_key {:?}", miner_public_key);
                // let locks_until_block_for_account = <pallet_balances::Pallet<T>>::locks(miner_public_key.clone());
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

                // initialise so they have no locks and are ineligible for rewards
                let mut locks_first_amount_as_u128 = 0u128.clone();
                let miner_account_id: T::AccountId;

                // convert type public key Vec<u8> to type T::AccountId
                let _miner_account_id = Decode::decode(&mut miner_public_key.as_slice().clone());
                match _miner_account_id.clone() {
                    Err(_e) => {
                        log::error!("Unable to decode miner_public_key");
                        println!("Unable to decode miner_public_key");
                        return 0;
                    },
                    Ok(x) => {
                        miner_account_id = x;
                    }
                }

                let locked_vec = <pallet_balances::Pallet<T>>::locks(miner_account_id.clone()).into_inner();
                if locked_vec.len() != 0 {
                    println!("locked_vec: {:?}", locked_vec);
                    let locks_first_amount: <T as pallet_balances::Config>::Balance =
                        <pallet_balances::Pallet<T>>::locks(miner_account_id.clone()).into_inner().clone()[0].amount;

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
                }
                log::info!("locks_first_amount_as_u128: {:?}", locks_first_amount_as_u128.clone());
                println!("locks_first_amount_as_u128 {:#?}", locks_first_amount_as_u128);

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

                // let miner_public_key = miner.clone().encode();
                log::info!("Public key {:?}", miner_public_key);
                println!("Public key {:?}", miner_public_key);

                let bonded_dhx_current_u128;
                let _bonded_dhx_current_u128 = Self::set_bonded_dhx_of_account_for_date(
                    miner_public_key.clone(),
                    locks_first_amount_as_u128.clone()
                );
                match _bonded_dhx_current_u128 {
                    Err(_e) => {
                        log::error!("Unable to set_bonded_dhx_of_account_for_date");
                        return 0;
                    },
                    Ok(ref x) => {
                        bonded_dhx_current_u128 = x;
                    }
                }

                let mut min_bonded_dhx_daily: BalanceOf<T> = 10u32.into();
                let mut min_bonded_dhx_daily_u128;
                let _min_bonded_dhx_daily = Self::get_min_bonded_dhx_daily();
                match _min_bonded_dhx_daily {
                    Err(_e) => {
                        log::error!("Unable to retrieve any min. bonded DHX daily as BalanceOf and u128");
                        return 0;
                    },
                    Ok(ref x) => {
                        min_bonded_dhx_daily = x.0;
                        min_bonded_dhx_daily_u128 = x.1;
                    }
                }

                let mut is_bonding_min_dhx = false;
                if locks_first_amount_as_u128 >= min_bonded_dhx_daily_u128 {
                    is_bonding_min_dhx = true;
                }
                log::info!("is_bonding_min_dhx: {:?} {:?}", is_bonding_min_dhx.clone(), miner_public_key.clone());
                println!("is_bonding_min_dhx {:#?}", is_bonding_min_dhx);
                println!("min_bonded_dhx_daily_u128 {:#?}", min_bonded_dhx_daily_u128);

                // the cooling-off period of 7 days and DHX Mining starts when they start bonding at least 10 DHX,
                // but they don't necessarily have to have the min. 1 mPower yet, however you will only start earning
                // dailing mining rewards when you have at least the min. 1 mPower (i.e. by locking at least 1 MXC or similar task)
                // (so they could be DHX Mining and not getting any rewards until they do so)
                let mut min_mpower_daily_u128: u128 = 1u128;
                if let Some(_min_mpower_daily_u128) = <MinMPowerDaily<T>>::get() {
                    min_mpower_daily_u128 = _min_mpower_daily_u128;
                } else {
                    log::error!("Unable to retrieve min. mPower daily as u128");
                }
                println!("min_mpower_daily_u128 {:#?}", min_mpower_daily_u128);

                // fetch the mPower of the miner currently being iterated to check if it's greater than the min. mPower that is required
                let mut mpower_current_u128: u128 = 0u128;
                let _mpower_current_u128 = <MPowerForAccountForDate<T>>::get((start_of_requested_date_millis.clone(), miner_public_key.clone()));
                match _mpower_current_u128 {
                    None => {
                        log::error!("Unable to get_mpower_of_account_for_date {:?}", start_of_requested_date_millis.clone());
                        println!("Unable to get_mpower_of_account_for_date {:?}", start_of_requested_date_millis.clone());
                    },
                    Some(x) => {
                        mpower_current_u128 = x;
                    }
                }
                // Note: this was the hard-code mPower data that may be used incase we just want to mock
                // the storage value of mpower for the current miner being iterated if we retrieved it from offchain workers
                // and stored their mpower on-chain using an unsigned transaction

                // let _mpower_data = (
                //     Some(0u128),
                //     start_of_requested_date_millis.clone(),
                //     1u64,
                // );
                // match _mpower_data.0 {
                //     None => {
                //         log::error!("Unable to get_mpower_of_account_for_date {:?}", start_of_requested_date_millis.clone());
                //         println!("Unable to get_mpower_of_account_for_date {:?}", start_of_requested_date_millis.clone());
                //     },
                //     Some(x) => {
                //         mpower_current_u128 = x;
                //     }
                // }
                log::info!("mpower_current_u128 {:#?}, {:?}", mpower_current_u128, start_of_requested_date_millis.clone());
                println!("mpower_current_u128 {:#?}, {:?}", mpower_current_u128, start_of_requested_date_millis.clone());

                let mut has_min_mpower_daily = false;
                // min. mpower daily must be greater than or equal to 1 otherwise they don't get any rewards
                if mpower_current_u128 >= min_mpower_daily_u128 && min_mpower_daily_u128 >= 1u128 {
                    has_min_mpower_daily = true;
                }
                log::info!("has_min_mpower_daily: {:?} {:?}", has_min_mpower_daily.clone(), miner_public_key.clone());
                println!("has_min_mpower_daily {:#?}", has_min_mpower_daily);

                // TODO - after fetching their mPower from the off-chain workers function where we iterate through
                // the registered DHX miners too, we need to incorporate it
                // into the recording the aggregated and accumulated rewards and the distribution of those rewards that
                // are done in on_initialize.
                // See Dhx-pop-mining-automatic.md in https://mxc.atlassian.net/browse/MMD-717 that explains off-chain worker
                // aspects

                let cooling_down_period_days;
                if let Some(_cooling_down_period_days) = <CoolingDownPeriodDays<T>>::get() {
                    cooling_down_period_days = _cooling_down_period_days;
                } else {
                    log::error!("Unable to retrieve cooling down period days");
                    println!("Unable to retrieve cooling down period days");
                    return 0;
                }

                // fallback incase not already set
                let mut cooling_down_period_days_remaining = (
                    start_of_requested_date_millis.clone(),
                    0u32,
                    0u32,
                );
                if let Some(_cooling_down_period_days_remaining) = <CoolingDownPeriodDaysRemaining<T>>::get(miner_public_key.clone()) {
                    // we do not change cooling_down_period_days_remaining.0 to the default value in the chain_spec.rs of 0,
                    // instead we want to use today's date `start_of_requested_date_millis.clone()` by default, as we did above.
                    if _cooling_down_period_days_remaining.0 != 0 {
                        cooling_down_period_days_remaining.0 = _cooling_down_period_days_remaining.0;
                    }
                    cooling_down_period_days_remaining.1 = _cooling_down_period_days_remaining.1;
                    cooling_down_period_days_remaining.2 = _cooling_down_period_days_remaining.2;
                } else {
                    log::info!("Unable to retrieve cooling down period days remaining for given miner, using default {:?}", miner_public_key.clone());
                    println!("Unable to retrieve cooling down period days remaining for given miner, using default {:?}", miner_public_key.clone());
                }

                log::info!("cooling_down_period_days_remaining {:?} {:?} {:?}", start_of_requested_date_millis.clone(), cooling_down_period_days_remaining, miner_public_key.clone());
                println!("cooling_down_period_days_remaining {:?} {:?} {:?}", start_of_requested_date_millis.clone(), cooling_down_period_days_remaining, miner_public_key.clone());

                // if they stop bonding the min dhx, and
                // if cooling_down_period_days_remaining.1 is Some(0)
                // so since we detected they are no longer bonding above the min.
                // then we should start at max. remaining days before starting to decrement on subsequent blocks
                if
                    cooling_down_period_days_remaining.1 == 0u32 &&
                    is_bonding_min_dhx == false

                {
                    // case 1: just finished unbonding period
                    if cooling_down_period_days_remaining.2 == 1u32 {
                        <CoolingDownPeriodDaysRemaining<T>>::insert(
                            miner_public_key.clone(),
                            (
                                start_of_requested_date_millis.clone(),
                                0u32,
                                0u32,
                            ),
                        );
                    }

                    // // case 2: after finished unbonding period, and where .2 set to 2u32 already
                    // if cooling_down_period_days_remaining.2 == 2u32 {
                    //     <CoolingDownPeriodDaysRemaining<T>>::insert(
                    //         miner_public_key.clone(),
                    //         (
                    //             start_of_requested_date_millis.clone(),
                    //             cooling_down_period_days_remaining.1,
                    //             0u32,
                    //         ),
                    //     );
                    // }
                    // // case 1: just started unbonding after being
                    // if cooling_down_period_days_remaining.2 == 0u32 {
                    //     <CoolingDownPeriodDaysRemaining<T>>::insert(
                    //         miner_public_key.clone(),
                    //         (
                    //             start_of_requested_date_millis.clone(),
                    //             cooling_down_period_days.clone(),
                    //             1u32,
                    //         ),
                    //     );
                    // }
                    // // case 2: midway through unbonding
                    // if cooling_down_period_days_remaining.2 == 1u32 {
                    //     <CoolingDownPeriodDaysRemaining<T>>::insert(
                    //         miner_public_key.clone(),
                    //         (
                    //             start_of_requested_date_millis.clone(),
                    //             cooling_down_period_days_remaining.1,
                    //             1u32,
                    //         ),
                    //     );
                    // // case 3: just finished unbonding
                    // } else if cooling_down_period_days_remaining.2 == 0u32 {
                    //     <CoolingDownPeriodDaysRemaining<T>>::insert(
                    //         miner_public_key.clone(),
                    //         (
                    //             start_of_requested_date_millis.clone(),
                    //             cooling_down_period_days_remaining.1,
                    //             0u32,
                    //         ),
                    //     );
                    // }
                    // // case 4: finished unbonding period


                // if cooling_down_period_days_remaining.0 is not the start of the current date
                //   (since if they just started un-bonding
                //   and we just set days remaining to 7, or we already decremented
                //   a miner's days remaining for the current date, then we want to wait until the next day until we
                //   decrement another day).
                // if cooling_down_period_days_remaining.1 is Some(above 0), then decrement,
                // but not yet completely unbonded so cannot claim rewards yet
                } else if
                    cooling_down_period_days_remaining.0 != start_of_requested_date_millis.clone() &&
                    cooling_down_period_days_remaining.1 > 0u32
                    //  &&
                    // is_bonding_min_dhx == false
                {
                    let old_cooling_down_period_days_remaining = cooling_down_period_days_remaining.1.clone();

                    // Subtract, handling overflow
                    let new_cooling_down_period_days_remaining;
                    let _new_cooling_down_period_days_remaining =
                        old_cooling_down_period_days_remaining.checked_sub(One::one());
                    match _new_cooling_down_period_days_remaining {
                        None => {
                            log::error!("Unable to subtract one from cooling_down_period_days_remaining due to StorageOverflow");
                            println!("Unable to subtract one from cooling_down_period_days_remaining due to StorageOverflow");

                            return 0;
                        },
                        Some(x) => {
                            new_cooling_down_period_days_remaining = x;
                        }
                    }

                    // Write the new value to storage
                    <CoolingDownPeriodDaysRemaining<T>>::insert(
                        miner_public_key.clone(),
                        (
                            start_of_requested_date_millis.clone(),
                            new_cooling_down_period_days_remaining.clone(),
                            1u32,
                        ),
                    );

                    println!("[reduce] block: {:#?}, miner: {:#?}, date_start: {:#?} new_cooling_down_period_days_remaining: {:#?}", block_number, miner_count, start_of_requested_date_millis, new_cooling_down_period_days_remaining);
                    log::info!("Unbonded miner. Reducing cooling down period dates remaining {:?} {:?}", miner_public_key.clone(), new_cooling_down_period_days_remaining.clone());
                }

                // to calculate eligible rewards allowance each day then just need to be:
                // - bonding the min. dhx
                // - not unbonded at any block on the day being processed (since otherwise they have to wait the unbonding period)
                if
                    cooling_down_period_days_remaining.1 == 0u32 &&
                    is_bonding_min_dhx == true
                {
                    log::info!("[eligible] block: {:#?}, miner: {:#?}, date_start: {:#?} remain_days: {:#?}", block_number, miner_count, start_of_requested_date_millis, cooling_down_period_days_remaining);
                    println!("[eligible] block: {:#?}, miner: {:#?}, date_start: {:#?} remain_days: {:#?}", block_number, miner_count, start_of_requested_date_millis, cooling_down_period_days_remaining);

                    // only accumulate the DHX reward for each registered miner once per day
                    // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
                    if <RewardsAccumulatedDHXForMinerForDate<T>>::contains_key(
                        (
                            start_of_requested_date_millis.clone(),
                            miner_public_key.clone(),
                        )
                    ) == true {
                        continue;
                    }

                    let _result_calculate_rewards_eligibility = Self::calculate_rewards_eligibility_of_miner_for_day(
                        miner_public_key.clone(),
                        start_of_requested_date_millis.clone(),
                        locks_first_amount_as_u128.clone(),
                        rewards_allowance_dhx_daily.clone(),
                        min_bonded_dhx_daily_u128.clone(),
                        mpower_current_u128.clone(),
                        block_number.clone(),
                        miner_count.clone(),
                        reg_dhx_miners.clone(),
                    );
                    match _result_calculate_rewards_eligibility {
                        Err(_e) => {
                            return 0;
                        },
                        Ok(ref x) => {
                        }
                    }

                    log::info!("Finished calculating eligible rewards date_start: {:#?}", start_of_requested_date_millis);
                    println!("Finished calculating eligible rewards date_start: {:#?}", start_of_requested_date_millis);
                }
            }

            log::info!("Finished loop of registered miners for block");
            println!("Finished loop of registered miners for block");

            return 0;
        }

        // `on_finalize` is executed at the end of block after all extrinsic are dispatched.
        fn on_finalize(block_number: T::BlockNumber) {
            // Perform necessary data/state clean up here.
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_registered_dhx_miners(origin: OriginFor<T>, register_dhx_miners: Vec<Vec<u8>>) -> DispatchResult {
            let _sender = ensure_root(origin)?;

            for miner_public_key in register_dhx_miners.iter().rev() {
                // log::info!("{:?}", miner);

                <RegisteredDHXMiners<T>>::append(miner_public_key.clone());
                log::info!("set_registered_dhx_miners: {:?}", &_sender);
            }

            Self::deposit_event(Event::SetRegisteredDHXMiners(
                register_dhx_miners.clone(),
            ));

            Ok(())
        }

        // only modifiable by governance as root rather than just any user
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_min_bonded_dhx_daily(origin: OriginFor<T>, min_bonded_dhx_daily: BalanceOf<T>) -> DispatchResult {
            let _sender = ensure_root(origin)?;

            <MinBondedDHXDaily<T>>::put(&min_bonded_dhx_daily.clone());
            log::info!("set_min_bonded_dhx_daily: {:?}", &min_bonded_dhx_daily);

            Self::deposit_event(Event::SetMinBondedDHXDailyStored(
                min_bonded_dhx_daily.clone(),
            ));

            Ok(())
        }

        // only modifiable by governance as root rather than just any user
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_min_mpower_daily(origin: OriginFor<T>, min_mpower_daily: u128) -> DispatchResult {
            let _sender = ensure_root(origin)?;

            <MinMPowerDaily<T>>::put(&min_mpower_daily.clone());
            log::info!("set_min_mpower_daily: {:?}", &min_mpower_daily);

            Self::deposit_event(Event::SetMinMPowerDailyStored(
                min_mpower_daily.clone(),
            ));

            Ok(())
        }

        // only modifiable by governance as root rather than just any user
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_challenge_period_days(origin: OriginFor<T>, challenge_period_days: u64) -> DispatchResult {
            let _sender = ensure_root(origin)?;

            <ChallengePeriodDays<T>>::put(&challenge_period_days.clone());
            log::info!("challenge_period_days: {:?}", &challenge_period_days);

            Self::deposit_event(Event::SetChallengePeriodDaysStored(
                challenge_period_days.clone(),
            ));

            Ok(())
        }

        // only modifiable by governance as root rather than just any user
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_cooling_down_period_days(origin: OriginFor<T>, cooling_down_period_days: u32) -> DispatchResult {
            let _sender = ensure_root(origin)?;

            <CoolingDownPeriodDays<T>>::put(&cooling_down_period_days.clone());
            log::info!("cooling_down_period_days: {:?}", &cooling_down_period_days);

            Self::deposit_event(Event::SetCoolingDownPeriodDaysStored(
                cooling_down_period_days.clone(),
            ));

            Ok(())
        }

        // only modifiable by governance as root rather than just any user
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_allowance_dhx_daily(origin: OriginFor<T>, rewards_allowance: BalanceOf<T>) -> DispatchResult {
            let _who = ensure_root(origin)?;
            // Update storage
            <RewardsAllowanceDHXDaily<T>>::put(&rewards_allowance.clone());
            log::info!("set_rewards_allowance_dhx_daily - rewards_allowance: {:?}", &rewards_allowance);

            // Emit an event.
            Self::deposit_event(Event::SetRewardsAllowanceDHXDailyStored(
                rewards_allowance.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // customised by governance at any time. this function allows us to change it each year
        // https://docs.google.com/spreadsheets/d/1W2AzOH9Cs9oCR8UYfYCbpmd9X7hp-USbYXL7AuwMY_Q/edit#gid=970997021
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_allowance_dhx_for_date_remaining(origin: OriginFor<T>, rewards_allowance: BalanceOf<T>, timestamp: u64) -> DispatchResult {
            let _who = ensure_root(origin)?;

            // Note: we do not need the following as we're not using the current timestamp, rather the function parameter.
            // let current_date = <pallet_timestamp::Pallet<T>>::get();
            // let requested_date_as_u64 = Self::convert_moment_to_u64_in_milliseconds(timestamp.clone())?;
            // log::info!("requested_date_as_u64: {:?}", requested_date_as_u64.clone());

            // Note: to get current timestamp `<pallet_timestamp::Pallet<T>>::get()`
            // convert the requested date/time to the start of that day date/time to signify that date for lookup
            // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
            let start_of_requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(timestamp.clone())?;

            // Update storage. Override the default that may have been set in on_initialize
            <RewardsAllowanceDHXForDateRemaining<T>>::insert(start_of_requested_date_millis.clone(), &rewards_allowance);
            log::info!("set_rewards_allowance_dhx_for_date_remaining - rewards_allowance: {:?}", &rewards_allowance);

            // Emit an event.
            Self::deposit_event(Event::ChangedRewardsAllowanceDHXForDateRemainingStored(
                start_of_requested_date_millis.clone(),
                rewards_allowance.clone(),
                1u8, // increment
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // only modifiable by governance as root rather than just any user
        // parameter `change: u8` value may be 0 or 1 (or any other value) to represent that we want to make a
        // corresponding decrease or increase to the remaining dhx rewards allowance for a given date.
        pub fn change_rewards_allowance_dhx_for_date_remaining(origin: OriginFor<T>, daily_rewards: BalanceOf<T>, timestamp: u64, change: u8) -> DispatchResult {
            let _who = ensure_root(origin)?;

            let start_of_requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(timestamp.clone())?;

            // https://substrate.dev/rustdocs/latest/frame_support/storage/trait.StorageMap.html
            ensure!(<RewardsAllowanceDHXForDateRemaining<T>>::contains_key(&start_of_requested_date_millis), DispatchError::Other("Date key must exist to reduce allowance."));

            // Validate inputs so the daily_rewards is less or equal to the existing_allowance
            let existing_allowance_as_u128;
            if let Some(_existing_allowance) = <RewardsAllowanceDHXForDateRemaining<T>>::get(&start_of_requested_date_millis) {
                existing_allowance_as_u128 = Self::convert_balance_to_u128(_existing_allowance.clone())?;
                log::info!("change_rewards_allowance_dhx_for_date_remaining - existing_allowance_as_u128: {:?}", existing_allowance_as_u128.clone());
            } else {
                return Err(DispatchError::Other("Unable to retrieve balance from value provided"));
            }

            let daily_rewards_as_u128;
            daily_rewards_as_u128 = Self::convert_balance_to_u128(daily_rewards.clone())?;
            log::info!("change_rewards_allowance_dhx_for_date_remaining - daily_rewards_as_u128: {:?}", daily_rewards_as_u128.clone());

            ensure!(daily_rewards_as_u128 > 0u128, DispatchError::Other("Daily rewards must be greater than zero"));
            ensure!(existing_allowance_as_u128 >= daily_rewards_as_u128, DispatchError::Other("Daily rewards cannot exceed current remaining allowance"));

            let new_remaining_allowance_as_balance;
            if change == 0 {
                // Decrementing the value will error in the event of underflow.
                let new_remaining_allowance_as_u128 = existing_allowance_as_u128.checked_sub(daily_rewards_as_u128).ok_or(Error::<T>::StorageUnderflow)?;
                new_remaining_allowance_as_balance = Self::convert_u128_to_balance(new_remaining_allowance_as_u128.clone())?;
                log::info!("change_rewards_allowance_dhx_for_date_remaining - Decreasing rewards_allowance_dhx_for_date_remaining at Date: {:?}", &start_of_requested_date_millis);
            } else {
                // Incrementing the value will error in the event of overflow.
                let new_remaining_allowance_as_u128 = existing_allowance_as_u128.checked_add(daily_rewards_as_u128).ok_or(Error::<T>::StorageOverflow)?;
                new_remaining_allowance_as_balance = Self::convert_u128_to_balance(new_remaining_allowance_as_u128.clone())?;
                log::info!("change_rewards_allowance_dhx_for_date_remaining - Increasing rewards_allowance_dhx_for_date_remaining at Date: {:?}", &start_of_requested_date_millis);
            }

            // Update storage
            <RewardsAllowanceDHXForDateRemaining<T>>::mutate(
                &start_of_requested_date_millis,
                |allowance| {
                    if let Some(_allowance) = allowance {
                        *_allowance = new_remaining_allowance_as_balance.clone();
                    }
                },
            );

            // Emit an event.
            Self::deposit_event(Event::ChangedRewardsAllowanceDHXForDateRemainingStored(
                start_of_requested_date_millis.clone(),
                new_remaining_allowance_as_balance.clone(),
                change.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // only modifiable by governance as root rather than just any user
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_multiplier_operation(origin: OriginFor<T>, operation: u8) -> DispatchResult {
            let _who = ensure_root(origin)?;
            <RewardsMultiplierOperation<T>>::put(&operation.clone());
            log::info!("set_rewards_multiplier_operation - operation: {:?}", &operation);

            // Emit an event.
            Self::deposit_event(Event::SetRewardsMultiplierOperationStored(
                operation.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // only modifiable by governance as root rather than just any user
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_multiplier_default_period_days(origin: OriginFor<T>, days: u32) -> DispatchResult {
            let _who = ensure_root(origin)?;
            <RewardsMultiplierDefaultPeriodDays<T>>::put(&days.clone());
            log::info!("set_rewards_multiplier_default_period_days - days: {:?}", &days);

            // Emit an event.
            Self::deposit_event(Event::SetRewardsMultiplierDefaultPeriodDaysStored(
                days.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // only modifiable by governance as root rather than just any user
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_rewards_multiplier_next_period_days(origin: OriginFor<T>, days: u32) -> DispatchResult {
            let _who = ensure_root(origin)?;
            <RewardsMultiplierNextPeriodDays<T>>::put(&days.clone());
            log::info!("set_rewards_multiplier_next_period_days - days: {:?}", &days);

            // Emit an event.
            Self::deposit_event(Event::SetRewardsMultiplierNextPeriodDaysStored(
                days.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // extrinsic that governance may choose to call to set the mpower of an account for a date
        // if it needs to be corrected in future before they claim
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn change_mpower_of_account_for_date(origin: OriginFor<T>, account_id: Vec<u8>, start_of_requested_date_millis: Date, mpower: u128) -> DispatchResult {
            let _who = ensure_root(origin)?;

            let mpower_current_u128 = mpower.clone();

            log::info!("change_mpower_of_account_for_date {:?} {:?} {:?}",
                account_id.clone(),
                start_of_requested_date_millis.clone(),
                mpower_current_u128.clone(),
            );

            let _set_mpower = Self::set_mpower_of_account_for_date(
                account_id.clone(),
                start_of_requested_date_millis.clone(),
                mpower_current_u128.clone(),
            );
            match _set_mpower.clone() {
                Err(_e) => {
                    log::warn!("Unable to change mpower using set_mpower_of_account_for_date");
                    return Err(DispatchError::Other("Unable to change mpower using set_mpower_of_account_for_date"));
                },
                Ok(x) => {
                    log::info!("changed mpower using set_mpower_of_account_for_date to: {:?} {:?}", account_id.clone(), x);
                }
            }

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // TODO - add change_finished_fetching_mpower_for_accounts_for_date

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn claim_rewards_of_account_for_date(
            origin: OriginFor<T>,
            miner_public_key: Vec<u8>, // we need to ensure this is the same account as the origin
            start_of_requested_date_millis: Date
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            // FIXME - these need to be the same, or ideally we need to find out how to get the public key associated with the origin
            // and not need to pass the miner_public_key as 2nd parameter at all.
            //
            // https://github.com/paritytech/subport/issues/290
            // assert_eq!(_sender.clone(), miner_public_key.clone());

            log::info!("claim_rewards_of_account_for_date - miner_public_key: {:?} {:?}", miner_public_key.clone(), start_of_requested_date_millis.clone());
            println!("claim_rewards_of_account_for_date - miner_public_key: {:?} {:?}", miner_public_key.clone(), start_of_requested_date_millis.clone());

            let block_number = <frame_system::Pallet<T>>::block_number();

            // check that the current date start_of_current_date_millis is at least 7 days after the provided start_of_requested_date_millis
            // so there is sufficient time for the community to audit the reward eligibility.
            // we could use `CoolingDownPeriodDays` for this, but probably better to call it a `ChallengePeriodDays`
            // since it serves a different purpose (`CoolingDownPeriodDays` gives rewards for first 7 days after they
            // start bonding in bulk on the 8th day, and then the next day after each day after that, whereas
            // `ChallengePeriodDays` you can only claim each daily rewards manually 7 days after it, even for the first
            // 7 days after they start bonding)

            let current_timestamp = <pallet_timestamp::Pallet<T>>::get();
            let current_timestamp_as_u64 = Self::convert_moment_to_u64_in_milliseconds(current_timestamp.clone())?;
            log::info!("current_timestamp_as_u64: {:?}", current_timestamp_as_u64.clone());
            println!("current_timestamp_as_u64: {:?}", current_timestamp_as_u64.clone());
            // convert the current timestamp to the start of that day date/time
            // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
            let start_of_current_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(current_timestamp_as_u64.clone())?;

            // // Prevent them from claiming during their unbonding period
            // let mut is_still_unbonding = false;
            // if let Some(_cooling_down_period_days_remaining) = <CoolingDownPeriodDaysRemaining<T>>::get(miner_public_key.clone()) {
            //     if _cooling_down_period_days_remaining.2 == 1 {
            //         is_still_unbonding = true;
            //     }
            // } else {
            //     log::info!("Unable to retrieve cooling down period days remaining for given miner {:?}", miner_public_key.clone());
            //     println!("Unable to retrieve cooling down period days remaining for given miner {:?}", miner_public_key.clone());
            //     return Err(DispatchError::Other("Unable to retrieve cooling down period days remaining for given miner"));
            // }
            // if is_still_unbonding == true {
            //     log::info!("Unbonding still in progress for given miner {:?}", miner_public_key.clone());
            //     println!("Unbonding still in progress for given miner {:?}", miner_public_key.clone());
            //     return Err(DispatchError::Other("Unbonding still in progress for given miner"));
            // }

            // where there are 86400000 milliseconds in a day
            // there are 7 * 86400000 = 604800000 milliseconds in 7 days
            // so we want to make sure `start_of_current_date_millis` - `start_of_requested_date_millis` > 604800000

            let mut is_more_than_challenge_period = false;
            let _is_more_than_challenge_period = Self::is_more_than_challenge_period(start_of_requested_date_millis.clone());
            match _is_more_than_challenge_period.clone() {
                Err(_e) => {
                    log::error!("Unable to claim until after waiting the challenge period {:?}", _e);
                    println!("Unable to claim until after waiting the challenge period {:?}", _e);

                    return Err(_e);
                },
                Ok(x) => {
                    log::info!("Proceeding since waited at least the challenge period");
                    println!("Proceeding since waited at least the challenge period");
                    is_more_than_challenge_period = true;
                }
            }

            // // TODO - consider the miner's mPower that we have fetched. it should have been added earlier above
            // // to the aggregated (all miners for that day) and accumulated (specific miner for a day) rewards

            // fetch accumulated total rewards for all registered miners for the day
            // TODO - we've done this twice, create a function to fetch it
            let mut rewards_aggregated_dhx_daily: BalanceOf<T> = 0u32.into(); // initialize
            if let Some(_rewards_aggregated_dhx_daily) = <RewardsAggregatedDHXForAllMinersForDate<T>>::get(&start_of_requested_date_millis) {
                rewards_aggregated_dhx_daily = _rewards_aggregated_dhx_daily;
            } else {
                log::error!("Unable to retrieve balance for rewards_aggregated_dhx_daily. cooling down period or challenge period may not be finished yet");
                println!("Unable to retrieve balance for rewards_aggregated_dhx_daily. cooling down period or challenge period may not be finished yet");
                // Note: it would be an issue if we got past the first loop of looping through the registered miners
                // and still hadn't added to the aggregated rewards for the day
                return Err(DispatchError::Other("Unable to retrieve balance for rewards_aggregated_dhx_daily. cooling down period or challenge period may not be finished yet"));
            }
            println!("[multiplier] block: {:#?}, date_start: {:#?} rewards_aggregated_dhx_daily: {:#?}", block_number, start_of_requested_date_millis, rewards_aggregated_dhx_daily);

            if rewards_aggregated_dhx_daily == 0u32.into() {
                log::error!("rewards_aggregated_dhx_daily must be greater than 0 to distribute rewards");
                println!("rewards_aggregated_dhx_daily must be greater than 0 to distribute rewards");
                return Err(DispatchError::Other("rewards_aggregated_dhx_daily must be greater than 0 to distribute rewards"));
            }

            let rewards_aggregated_dhx_daily_as_u128;
            let _rewards_aggregated_dhx_daily_as_u128 = Self::convert_balance_to_u128(rewards_aggregated_dhx_daily.clone());
            match _rewards_aggregated_dhx_daily_as_u128.clone() {
                Err(_e) => {
                    log::error!("Unable to convert balance to u128 for rewards_aggregated_dhx_daily_as_u128");
                    println!("Unable to convert balance to u128 for rewards_aggregated_dhx_daily_as_u128");
                    return Err(DispatchError::Other("Unable to convert balance to u128 for rewards_aggregated_dhx_daily_as_u128"));
                },
                Ok(x) => {
                    rewards_aggregated_dhx_daily_as_u128 = x;
                }
            }
            log::info!("rewards_aggregated_dhx_daily_as_u128: {:?}", rewards_aggregated_dhx_daily_as_u128.clone());
            println!("rewards_aggregated_dhx_daily_as_u128: {:?}", rewards_aggregated_dhx_daily_as_u128.clone());

            // TODO - we've done this twice, create a function to fetch it
            let rewards_allowance_dhx_daily;
            if let Some(_rewards_allowance_dhx_daily) = <RewardsAllowanceDHXDaily<T>>::get() {
                rewards_allowance_dhx_daily = _rewards_allowance_dhx_daily;
            } else {
                log::error!("Unable to get rewards_allowance_dhx_daily");
                println!("Unable to get rewards_allowance_dhx_daily");
                return Err(DispatchError::Other("Unable to get rewards_allowance_dhx_daily"));
            }

            let rewards_allowance_dhx_daily_u128;
            let _rewards_allowance_dhx_daily_u128 = Self::convert_balance_to_u128(rewards_allowance_dhx_daily.clone());
            match _rewards_allowance_dhx_daily_u128.clone() {
                Err(_e) => {
                    log::error!("Unable to convert balance to u128 for rewards_allowance_dhx_daily_u128");
                    println!("Unable to convert balance to u128 for rewards_allowance_dhx_daily_u128");
                    return Err(DispatchError::Other("Unable to convert balance to u128 for rewards_allowance_dhx_daily_u128"));
                },
                Ok(x) => {
                    rewards_allowance_dhx_daily_u128 = x;
                }
            }

            log::info!("rewards_allowance_dhx_daily_u128: {:?}", rewards_allowance_dhx_daily_u128.clone());
            println!("rewards_allowance_dhx_daily_u128: {:?}", rewards_allowance_dhx_daily_u128.clone());

            if rewards_allowance_dhx_daily_u128 == 0u128 {
                log::error!("rewards_allowance_dhx_daily must be greater than 0 to distribute rewards");
                println!("rewards_allowance_dhx_daily must be greater than 0 to distribute rewards");
                return Err(DispatchError::Other("rewards_allowance_dhx_daily must be greater than 0 to distribute rewards"));
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
            let mut distribution_multiplier_for_day_fixed128 = FixedU128::from_num(1); // initialize

            if rewards_aggregated_dhx_daily_as_u128.clone() > rewards_allowance_dhx_daily_u128.clone() {
                log::info!("rewards_aggregated_dhx_daily > rewards_allowance_dhx_daily");
                println!("rewards_aggregated_dhx_daily > rewards_allowance_dhx_daily");
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
                    println!("Unable to divide rewards_allowance_dhx_daily_u128 to make it smaller");
                    return Err(DispatchError::Other("Unable to divide rewards_allowance_dhx_daily_u128 to make it smaller"));
                }
                log::info!("manageable_rewards_allowance_dhx_daily_u128: {:?}", manageable_rewards_allowance_dhx_daily_u128.clone());
                println!("manageable_rewards_allowance_dhx_daily_u128: {:?}", manageable_rewards_allowance_dhx_daily_u128.clone());

                let mut rewards_allowance_dhx_daily_u64 = 0u64;
                if let Some(_rewards_allowance_dhx_daily_u64) =
				    TryInto::<u64>::try_into(manageable_rewards_allowance_dhx_daily_u128.clone()).ok() {
                        rewards_allowance_dhx_daily_u64 = _rewards_allowance_dhx_daily_u64;
                } else {
                    log::error!("Unable to convert u128 to u64 for rewards_allowance_dhx_daily_u128");
                    println!("Unable to convert u128 to u64 for rewards_allowance_dhx_daily_u128");
                    return Err(DispatchError::Other("Unable to convert u128 to u64 for rewards_allowance_dhx_daily_u128"));
                }
                log::info!("rewards_allowance_dhx_daily_u64: {:?}", rewards_allowance_dhx_daily_u64.clone());
                println!("rewards_allowance_dhx_daily_u64: {:?}", rewards_allowance_dhx_daily_u64.clone());

                let mut manageable_rewards_aggregated_dhx_daily_as_u128 = 0u128;
                if let Some(_manageable_rewards_aggregated_dhx_daily_as_u128) = rewards_aggregated_dhx_daily_as_u128.clone().checked_div(1000000000000000000u128) {
                    manageable_rewards_aggregated_dhx_daily_as_u128 = _manageable_rewards_aggregated_dhx_daily_as_u128;
                } else {
                    log::error!("Unable to divide manageable_rewards_aggregated_dhx_daily_as_u128 to make it smaller");
                    println!("Unable to divide manageable_rewards_aggregated_dhx_daily_as_u128 to make it smaller");
                    return Err(DispatchError::Other("Unable to divide manageable_rewards_aggregated_dhx_daily_as_u128 to make it smaller"));
                }
                log::info!("manageable_rewards_aggregated_dhx_daily_as_u128: {:?}", manageable_rewards_aggregated_dhx_daily_as_u128.clone());
                println!("manageable_rewards_aggregated_dhx_daily_as_u128: {:?}", manageable_rewards_aggregated_dhx_daily_as_u128.clone());

                let mut rewards_aggregated_dhx_daily_as_u64 = 0u64;
                if let Some(_rewards_aggregated_dhx_daily_as_u64) =
				    TryInto::<u64>::try_into(manageable_rewards_aggregated_dhx_daily_as_u128.clone()).ok() {
                        rewards_aggregated_dhx_daily_as_u64 = _rewards_aggregated_dhx_daily_as_u64;
                } else {
                    log::error!("Unable to convert u128 to u64 for rewards_aggregated_dhx_daily_as_u128");
                    println!("Unable to convert u128 to u64 for rewards_aggregated_dhx_daily_as_u128");
                    return Err(DispatchError::Other("Unable to convert u128 to u64 for rewards_aggregated_dhx_daily_as_u128"));
                }
                log::info!("rewards_aggregated_dhx_daily_as_u64: {:?}", rewards_aggregated_dhx_daily_as_u64.clone());
                println!("rewards_aggregated_dhx_daily_as_u64: {:?}", rewards_aggregated_dhx_daily_as_u64.clone());

                // See https://github.com/ltfschoen/substrate-node-template/pull/6/commits/175ef4805d07093042431c5814dda52da1ebde18
                let _fraction_distribution_multiplier_for_day_fixed128 =
                    U64F64::from_num(manageable_rewards_allowance_dhx_daily_u128.clone())
                        .checked_div(U64F64::from_num(manageable_rewards_aggregated_dhx_daily_as_u128.clone()));
                let _distribution_multiplier_for_day_fixed128 = _fraction_distribution_multiplier_for_day_fixed128.clone();
                match _distribution_multiplier_for_day_fixed128 {
                    None => {
                        log::error!("Unable to divide rewards_allowance_dhx_daily_u128 due to StorageOverflow by rewards_aggregated_dhx_daily_as_u128");
                        println!("Unable to divide rewards_allowance_dhx_daily_u128 due to StorageOverflow by rewards_aggregated_dhx_daily_as_u128");
                        return Err(DispatchError::Other("Unable to divide rewards_allowance_dhx_daily_u128 due to StorageOverflow by rewards_aggregated_dhx_daily_as_u128"));
                    },
                    Some(x) => {
                        distribution_multiplier_for_day_fixed128 = x;
                    }
                }
            } else {
                log::info!("rewards_aggregated_dhx_daily <= rewards_allowance_dhx_daily");
                println!("rewards_aggregated_dhx_daily <= rewards_allowance_dhx_daily");
            }
            log::info!("distribution_multiplier_for_day_fixed128 {:#?}", distribution_multiplier_for_day_fixed128);
            println!("distribution_multiplier_for_day_fixed128 {:#?}", distribution_multiplier_for_day_fixed128);

            log::info!("[multiplier] block: {:#?}, date_start: {:#?} distribution_multiplier_for_day_fixed128: {:#?}", block_number, start_of_requested_date_millis, distribution_multiplier_for_day_fixed128);
            println!("[multiplier] block: {:#?}, date_start: {:#?} distribution_multiplier_for_day_fixed128: {:#?}", block_number, start_of_requested_date_millis, distribution_multiplier_for_day_fixed128);

            // Initialise outside the loop as we need this value after the loop after we finish iterating through all the miners
            let mut rewards_allowance_dhx_remaining_today_as_u128 = 0u128;

            // only run the following once per day per miner until rewards_allowance_dhx_for_date_remaining is exhausted
            // but since we're giving each registered miner a proportion of the daily reward allowance
            // (if their aggregated rewards is above daily allowance) each proportion is rounded down,
            // it shouldn't become exhausted anyway
            // let is_already_distributed = <RewardsAllowanceDHXForDateRemainingDistributed<T>>::get(start_of_requested_date_millis.clone());
            // if is_already_distributed == Some(true) {
            //     log::error!("Unable to distribute further rewards allowance today");
            //     return 0;
            // }

            let is_already_distributed_to_miner = <RewardsAllowanceDHXForMinerForDateRemainingDistributed<T>>::get(
                (
                    start_of_requested_date_millis.clone(),
                    miner_public_key.clone(),
                )
            );
            log::info!("is_already_distributed_to_miner {:#?}", is_already_distributed_to_miner);
            println!("is_already_distributed_to_miner {:#?}", is_already_distributed_to_miner);

            if is_already_distributed_to_miner == Some(true) {
                log::error!("Unable to distribute further rewards allowance to the registered dhx miner for this day");
                println!("Unable to distribute further rewards allowance to the registered dhx miner for this day");
                return Err(DispatchError::Other("Unable to distribute further rewards allowance to the registered dhx miner for this day"));
            }

            let daily_reward_for_miner_as_u128;
            let daily_reward_for_miner_to_try = <RewardsAccumulatedDHXForMinerForDate<T>>::get(
                (
                    start_of_requested_date_millis.clone(),
                    miner_public_key.clone(),
                ),
            );
            if let Some(_daily_reward_for_miner_to_try) = daily_reward_for_miner_to_try.clone() {
                let _daily_reward_for_miner_as_u128 = Self::convert_balance_to_u128(_daily_reward_for_miner_to_try.clone());
                match _daily_reward_for_miner_as_u128.clone() {
                    Err(_e) => {
                        log::error!("Unable to convert balance to u128 for daily_reward_for_miner_as_u128");
                        println!("Unable to distribute further rewards allowance to the registered dhx miner for this day");
                        return Err(DispatchError::Other("Unable to convert balance to u128 for daily_reward_for_miner_as_u128"));
                    },
                    Ok(x) => {
                        daily_reward_for_miner_as_u128 = x;
                    }
                }
            } else {
                // If any of the miner's don't have a reward, we won't waste storing that,
                // so we want to move to the next miner in the loop
                log::error!("Unable to retrieve reward balance for daily_reward_for_miner {:?}", miner_public_key.clone());
                println!("Unable to retrieve reward balance for daily_reward_for_miner {:?}", miner_public_key.clone());
                return Err(DispatchError::Other("Unable to retrieve reward balance for daily_reward_for_miner"));
            }
            log::info!("daily_reward_for_miner_as_u128: {:?}", daily_reward_for_miner_as_u128.clone());
            println!("daily_reward_for_miner_as_u128: {:?}", daily_reward_for_miner_as_u128.clone());

            let mut manageable_daily_reward_for_miner_as_u128 = 0u128;
            if let Some(_manageable_daily_reward_for_miner_as_u128) =
                daily_reward_for_miner_as_u128.clone().checked_div(1000000000000000000u128) {
                    manageable_daily_reward_for_miner_as_u128 = _manageable_daily_reward_for_miner_as_u128;
            } else {
                log::error!("Unable to divide daily_reward_for_miner_as_u128 to make it smaller");
                println!("Unable to divide daily_reward_for_miner_as_u128 to make it smaller");
                return Err(DispatchError::Other("Unable to divide daily_reward_for_miner_as_u128 to make it smaller"));
            }
            log::info!("manageable_daily_reward_for_miner_as_u128 {:#?}", manageable_daily_reward_for_miner_as_u128);
            println!("manageable_daily_reward_for_miner_as_u128 {:#?}", manageable_daily_reward_for_miner_as_u128);

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
                    println!("Unable to multiply proportion_of_daily_reward_for_miner_fixed128 with daily_reward_for_miner_as_u128 due to StorageOverflow");
                    return Err(DispatchError::Other("Unable to multiply proportion_of_daily_reward_for_miner_fixed128 with daily_reward_for_miner_as_u128 due to StorageOverflow"));
                },
                Some(x) => {
                    proportion_of_daily_reward_for_miner_fixed128 = x;
                }
            }
            log::info!("proportion_of_daily_reward_for_miner_fixed128: {:?}", proportion_of_daily_reward_for_miner_fixed128.clone());
            println!("proportion_of_daily_reward_for_miner_fixed128: {:?}", proportion_of_daily_reward_for_miner_fixed128.clone());

            // round down to nearest integer. we need to round down, because if we round up then if there are
            // 3x registered miners with 5000 DHX rewards allowance per day then they would each get 1667 rewards,
            // but there would only be 1666 remaining after the first two, so the last one would miss out.
            // so if we round down they each get 1666 DHX and there is 2 DHX from the daily allocation that doesn't get distributed at all.
            let proportion_of_daily_reward_for_miner_u128: u128 = proportion_of_daily_reward_for_miner_fixed128.floor().to_num::<u128>();
            log::info!("proportion_of_daily_reward_for_miner_u128: {:?}", proportion_of_daily_reward_for_miner_u128.clone());
            println!("proportion_of_daily_reward_for_miner_u128: {:?}", proportion_of_daily_reward_for_miner_u128.clone());


            // we lose some accuracy doing this conversion, but at least we split the bulk of the rewards proportionally and fairly
            let mut restored_proportion_of_daily_reward_for_miner_u128 = 0u128;
            if let Some(_restored_proportion_of_daily_reward_for_miner_u128) =
                proportion_of_daily_reward_for_miner_u128.clone().checked_mul(1000000000000000000u128) {
                    restored_proportion_of_daily_reward_for_miner_u128 = _restored_proportion_of_daily_reward_for_miner_u128;
            } else {
                log::error!("Unable to multiply proportion_of_daily_reward_for_miner_fixed128 to restore it larger again");
                println!("Unable to multiply proportion_of_daily_reward_for_miner_fixed128 to restore it larger again");
                return Err(DispatchError::Other("Unable to multiply proportion_of_daily_reward_for_miner_fixed128 to restore it larger again"));
            }
            log::info!("restored_proportion_of_daily_reward_for_miner_u128: {:?}", restored_proportion_of_daily_reward_for_miner_u128.clone());
            println!("restored_proportion_of_daily_reward_for_miner_u128: {:?}", restored_proportion_of_daily_reward_for_miner_u128.clone());

            log::info!("[rewards] block: {:#?}, date_start: {:#?} restored_proportion_of_daily_reward_for_miner_u128: {:#?}", block_number, start_of_requested_date_millis, restored_proportion_of_daily_reward_for_miner_u128);
            println!("[rewards] block: {:#?}, date_start: {:#?} restored_proportion_of_daily_reward_for_miner_u128: {:#?}", block_number, start_of_requested_date_millis, restored_proportion_of_daily_reward_for_miner_u128);

            let treasury_account_id: T::AccountId = <pallet_treasury::Pallet<T>>::account_id();
            let max_payout = pallet_balances::Pallet::<T>::usable_balance(treasury_account_id.clone());
            log::info!("Treasury account id: {:?}", treasury_account_id.clone());
            log::info!("Miner to receive reward: {:?}", miner_public_key.clone());
            log::info!("Treasury balance max payout: {:?}", max_payout.clone());
            println!("Treasury account id: {:?}", treasury_account_id.clone());
            println!("Miner to receive reward: {:?}", miner_public_key.clone());
            println!("Treasury balance max payout: {:?}", max_payout.clone());

            let proportion_of_daily_reward_for_miner;
            let _proportion_of_daily_reward_for_miner = Self::convert_u128_to_balance(restored_proportion_of_daily_reward_for_miner_u128.clone());
            match _proportion_of_daily_reward_for_miner {
                Err(_e) => {
                    log::error!("Unable to convert u128 to balance for proportion_of_daily_reward_for_miner");
                    println!("Unable to convert u128 to balance for proportion_of_daily_reward_for_miner");
                    return Err(DispatchError::Other("Unable to convert u128 to balance for proportion_of_daily_reward_for_miner"));
                },
                Ok(ref x) => {
                    proportion_of_daily_reward_for_miner = x;
                }
            }
            log::info!("proportion_of_daily_reward_for_miner: {:?}", proportion_of_daily_reward_for_miner.clone());
            println!("proportion_of_daily_reward_for_miner: {:?}", proportion_of_daily_reward_for_miner.clone());

            let max_payout_as_u128;
            if let Some(_max_payout_as_u128) = TryInto::<u128>::try_into(max_payout).ok() {
                max_payout_as_u128 = _max_payout_as_u128;
            } else {
                log::error!("Unable to convert Balance to u128 for max_payout");
                println!("Unable to convert Balance to u128 for max_payout");
                return Err(DispatchError::Other("Unable to convert Balance to u128 for max_payout"));
            }
            log::info!("max_payout_as_u128: {:?}", max_payout_as_u128.clone());
            println!("max_payout_as_u128: {:?}", max_payout_as_u128.clone());

            // TODO - rename `rewards_allowance_dhx_remaining_today` to
            // `rewards_allowance_dhx_remaining_for_date_rewards_being_claimed`

            // Store output `rewards_allowance_dhx_remaining_today_as_u128` outside the loop
            // Validate inputs so the daily_rewards is less or equal to the existing_allowance
            if let Some(_rewards_allowance_dhx_remaining_today) = <RewardsAllowanceDHXForDateRemaining<T>>::get(&start_of_requested_date_millis) {
                let _rewards_allowance_dhx_remaining_today_as_u128 = Self::convert_balance_to_u128(_rewards_allowance_dhx_remaining_today.clone());
                match _rewards_allowance_dhx_remaining_today_as_u128.clone() {
                    Err(_e) => {
                        log::error!("Unable to convert balance to u128");
                        println!("Unable to convert balance to u128");
                        return Err(DispatchError::Other("Unable to convert balance to u128"));
                    },
                    Ok(x) => {
                        rewards_allowance_dhx_remaining_today_as_u128 = x;
                    }
                }
                log::info!("rewards_allowance_dhx_remaining_today_as_u128: {:?}", rewards_allowance_dhx_remaining_today_as_u128.clone());
                println!("rewards_allowance_dhx_remaining_today_as_u128: {:?}", rewards_allowance_dhx_remaining_today_as_u128.clone());
            } else {
                log::error!("Unable to retrieve balance from value provided.");
                println!("Unable to retrieve balance from value provided.");
                return Err(DispatchError::Other("Unable to retrieve balance from value provided."));
            }

            log::info!("[prepared-for-payment] block: {:#?}, date_start: {:#?} max payout: {:#?}, rewards remaining today {:?}, restored_proportion_of_daily_reward_for_miner_u128 {:?}", block_number, start_of_requested_date_millis, max_payout_as_u128, rewards_allowance_dhx_remaining_today_as_u128, restored_proportion_of_daily_reward_for_miner_u128);
            println!("[prepared-for-payment] block: {:#?}, date_start: {:#?} max payout: {:#?}, rewards remaining today {:?}, restored_proportion_of_daily_reward_for_miner_u128 {:?}", block_number, start_of_requested_date_millis, max_payout_as_u128, rewards_allowance_dhx_remaining_today_as_u128, restored_proportion_of_daily_reward_for_miner_u128);

            // check if miner's reward is less than or equal to: rewards_allowance_dhx_daily_remaining
            if restored_proportion_of_daily_reward_for_miner_u128.clone() > 0u128 &&
                rewards_allowance_dhx_remaining_today_as_u128.clone() >= restored_proportion_of_daily_reward_for_miner_u128.clone() &&
                max_payout_as_u128.clone() >= restored_proportion_of_daily_reward_for_miner_u128.clone()
            {
                // pay the miner their daily reward
                info!("Paying the miner a proportion of the remaining daily reward allowance");
                println!("Paying the miner a proportion of the remaining daily reward allowance");

                let tx_result;
                let _tx_result = <T as Config>::Currency::transfer(
                    &treasury_account_id,
                    &_sender.clone(),
                    proportion_of_daily_reward_for_miner.clone(),
                    ExistenceRequirement::KeepAlive
                );
                match _tx_result {
                    Err(_e) => {
                        log::error!("Unable to transfer from treasury to miner {:?}", miner_public_key.clone());
                        return Err(DispatchError::Other("Unable to transfer from treasury to miner"));
                    },
                    Ok(ref x) => {
                        tx_result = x;
                    }
                }
                info!("Transfer to the miner tx_result: {:?}", tx_result.clone());
                println!("Transfer to the miner tx_result: {:?}", tx_result.clone());

                info!("Success paying the reward to the miner: {:?}", restored_proportion_of_daily_reward_for_miner_u128.clone());
                println!("Success paying the reward to the miner: {:?}", restored_proportion_of_daily_reward_for_miner_u128.clone());

                // TODO - move into function `reduce_rewards_allowance_dhx_for_date_remaining`?

                // Subtract, handling overflow
                let new_rewards_allowance_dhx_remaining_today_as_u128;
                let _new_rewards_allowance_dhx_remaining_today_as_u128 =
                    rewards_allowance_dhx_remaining_today_as_u128.clone().checked_sub(restored_proportion_of_daily_reward_for_miner_u128.clone());
                match _new_rewards_allowance_dhx_remaining_today_as_u128 {
                    None => {
                        log::error!("Unable to subtract restored_proportion_of_daily_reward_for_miner_u128 from rewards_allowance_dhx_remaining_today due to StorageOverflow");
                        println!("Unable to subtract restored_proportion_of_daily_reward_for_miner_u128 from rewards_allowance_dhx_remaining_today due to StorageOverflow");
                        return Err(DispatchError::Other("Unable to subtract restored_proportion_of_daily_reward_for_miner_u128 from rewards_allowance_dhx_remaining_today due to StorageOverflow"));
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
                        println!("Unable to convert u128 to balance for new_rewards_allowance_dhx_remaining_today");
                        return Err(DispatchError::Other("Unable to convert u128 to balance for new_rewards_allowance_dhx_remaining_today"));
                    },
                    Ok(ref x) => {
                        new_rewards_allowance_dhx_remaining_today = x;
                    }
                }

                // Write the new value to storage
                <RewardsAllowanceDHXForDateRemaining<T>>::insert(
                    start_of_requested_date_millis.clone(),
                    new_rewards_allowance_dhx_remaining_today.clone(),
                );

                log::info!("[paid] block: {:#?}, date_start: {:#?} new_rewards_allowance_dhx_remaining_today: {:#?}", block_number, start_of_requested_date_millis, new_rewards_allowance_dhx_remaining_today);
                println!("[paid] block: {:#?}, date_start: {:#?} new_rewards_allowance_dhx_remaining_today: {:#?}", block_number, start_of_requested_date_millis, new_rewards_allowance_dhx_remaining_today);

                <RewardsAllowanceDHXForMinerForDateRemainingDistributed<T>>::insert(
                    (
                        start_of_requested_date_millis.clone(),
                        miner_public_key.clone(),
                    ),
                    true,
                );

                // emit event with reward payment history rather than bloating storage
                Self::deposit_event(Event::TransferredRewardsAllowanceDHXToMinerForDate(
                    start_of_requested_date_millis.clone(),
                    proportion_of_daily_reward_for_miner.clone(),
                    new_rewards_allowance_dhx_remaining_today.clone(),
                    miner_public_key.clone(),
                ));

                log::info!("TransferredRewardsAllowanceDHXToMinerForDate {:?} {:?} {:?} {:?}",
                    start_of_requested_date_millis.clone(),
                    proportion_of_daily_reward_for_miner.clone(),
                    new_rewards_allowance_dhx_remaining_today.clone(),
                    miner_public_key.clone(),
                );
                // do not split this across multiple lines otherwise not as easy to find/replace `println` with `println` when
                // switching between running tests with `println` and building the code with `cargo build --release` that doesn't
                // work with println
                println!("TransferredRewardsAllowanceDHXToMinerForDate {:?} {:?} {:?} {:?}", start_of_requested_date_millis.clone(), proportion_of_daily_reward_for_miner.clone(), new_rewards_allowance_dhx_remaining_today.clone(), miner_public_key.clone());
            } else {
                log::error!("Insufficient remaining rewards allowance to pay daily reward to miner");
                println!("Insufficient remaining rewards allowance to pay daily reward to miner");
                return Err(DispatchError::Other("Insufficient remaining rewards allowance to pay daily reward to miner"));
            }

            let rewards_allowance_dhx_remaining_today;
            let _rewards_allowance_dhx_remaining_today = Self::convert_u128_to_balance(rewards_allowance_dhx_remaining_today_as_u128.clone());
            match _rewards_allowance_dhx_remaining_today {
                Err(_e) => {
                    log::error!("Unable to convert u128 to balance for rewards_allowance_dhx_remaining_today");
                    println!("Unable to convert u128 to balance for rewards_allowance_dhx_remaining_today");
                    return Err(DispatchError::Other("Unable to convert u128 to balance for rewards_allowance_dhx_remaining_today"));
                },
                Ok(ref x) => {
                    rewards_allowance_dhx_remaining_today = x;
                }
            }
            info!("rewards_allowance_dhx_remaining_today: {:?}", rewards_allowance_dhx_remaining_today.clone());
            println!("rewards_allowance_dhx_remaining_today: {:?}", rewards_allowance_dhx_remaining_today.clone());

            log::info!("[distributed] block: {:#?}, date_start: {:#?} ", block_number, start_of_requested_date_millis);
            println!("[distributed] block: {:#?}, date_start: {:#?} ", block_number, start_of_requested_date_millis);

            Self::deposit_event(Event::DistributedRewardsAllowanceDHXForDateRemaining(
                start_of_requested_date_millis.clone(),
                rewards_allowance_dhx_remaining_today.clone(),
            ));

            // FIXME - previously when we distributed all rewards for a given date
            // we used `RewardsAllowanceDHXForDateRemainingDistributed` to record whether all the rewards for
            // that date had been distributed.
            //
            // but now registered_dhx_miners claim their rewards for a given date

            // we should only run `RewardsAllowanceDHXForDateRemainingDistributed` after all
            // the registered_dhx_miners that were stored under
            // a given date as the key for RewardsAccumulatedDHXForMinerForDate have been flagged as having been paid.
            // so since there may be some leftover rewards that don't get distributed for a given day due to rounding,
            // we can't say its all distributed when `rewards_allowance_dhx_remaining_today` is 0.
            // we now have a StorageMap RewardsAllowanceDHXForMinerForDateRemainingDistributed, that uses the same
            // date and miner_public_key as the key, and stores a flag of whether they have been paid for that date
            // already as a value, so they can't claim more than once for the same day

            // TODO - we won't use this yet. we could use it but we'll need to use `RewardsEligibleMinersForDate`,
            // which is where we store the registered_dhx_miners that are deemed eligible for rewards on a given date,
            // and also record which of those registered miners we've already distributed rewards to
            // using `RewardsAllowanceDHXForMinerForDateRemainingDistributed` and when there are 0 left to distribute to,
            // then we'd run this, but it doesn't seem that important to know
            // <RewardsAllowanceDHXForDateRemainingDistributed<T>>::insert(
            //     start_of_requested_date_millis.clone(),
            //     true
            // );

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        // Off-chain workers

        /// Submit new mPower data on-chain via unsigned transaction.
        ///
        /// Works exactly like the `submit_mpower` function, but since we allow sending the
        /// transaction without a signature, and hence without paying any fees,
        /// we need a way to make sure that only some transactions are accepted.
        /// This function can be called only once every `T::UnsignedInterval` blocks.
        /// Transactions that call that function are de-duplicated on the pool level
        /// via `validate_unsigned` implementation and also are rendered invalid if
        /// the function has already been called in current "session".
        ///
        /// TODO - verify the provided mPower to check that it is meaningful data
        /// TODO - replace u32 with data structured that contains the account id of each
        /// registered DHX miner and their mPower data for a date
        /// TODO - specify `weight` for unsigned calls as well, because even though
        /// they don't charge fees, we still don't want a single block to contain unlimited
        /// number of such transactions.
        #[pallet::weight(0)]
        pub fn submit_fetched_mpower_unsigned(
            origin: OriginFor<T>,
            _block_number: T::BlockNumber,
            start_of_requested_date_millis: Date,
            mpower_payload_vec: Vec<MPowerPayloadData<T>>,
        ) -> DispatchResultWithPostInfo {
            log::info!("submit_fetched_mpower_unsigned");
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Add the mpower vec on-chain, but mark it as coming from an empty address.
            Self::add_mpower(Default::default(), start_of_requested_date_millis.clone(), mpower_payload_vec.clone());
            log::info!("finished submit_fetched_mpower_unsigned");
            // now increment the block number at which we expect next unsigned transaction
            // that is specifically for fetching mpower.
            let current_block = <system::Pallet<T>>::block_number();
            <NextUnsignedAtForFetchedMPower<T>>::put(current_block + T::UnsignedInterval::get());
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn submit_finished_fetching_mpower_unsigned(
            origin: OriginFor<T>,
            _block_number: T::BlockNumber,
            fetched_mpower_data: FetchedMPowerForAccountsForDatePayloadData,
        ) -> DispatchResultWithPostInfo {
            log::info!("submit_finished_fetching_mpower_unsigned");
            // This ensures that the function can only be called via unsigned transaction.
            ensure_none(origin)?;
            // Add the data on-chain, but mark it as coming from an empty address.
            Self::add_finished_fetching_mpower(Default::default(), fetched_mpower_data.clone());
            // now increment the block number at which we expect next unsigned transaction
            // that is specifically for having "finished" fetching mpower.
            let current_block = <system::Pallet<T>>::block_number();
            <NextUnsignedAtForFinishedFetchingMPower<T>>::put(current_block + T::UnsignedInterval::get());
            Ok(().into())
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        /// Validate unsigned call to this module.
        ///
        /// By default unsigned transactions are disallowed, but implementing the validator
        /// here we make sure that some particular calls (the ones produced by offchain worker)
        /// are being whitelisted and marked as valid.
        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            if let Call::submit_fetched_mpower_unsigned(block_number, start_of_requested_date_millis, new_mpower_data_vec) = call {
                log::info!("validate_unsigned submit_fetched_mpower_unsigned");
                return Self::validate_transaction_parameters_for_fetched_mpower(block_number, start_of_requested_date_millis, new_mpower_data_vec);
            } else if let Call::submit_finished_fetching_mpower_unsigned(block_number, fetched_mpower_data) = call {
                log::info!("validate_unsigned validate_transaction_parameters_for_fetched_mpower");
                return Self::validate_transaction_parameters_for_finished_fetching_mpower(block_number, fetched_mpower_data);
            } else {
                log::info!("validate_unsigned invalid");
                return InvalidTransaction::Call.into();
            }
        }
    }

    // Private functions

    impl<T: Config> Pallet<T> {
        fn should_process_block(block_number: T::BlockNumber) -> bool {
            let block_one = 1u32;
            let block_one_as_block;
            if let Some(_block_one) = TryInto::<T::BlockNumber>::try_into(block_one).ok() {
                block_one_as_block = _block_one;
            } else {
                log::error!("Unable to convert u32 to BlockNumber");
                return false;
            }

            // skip block #1 since timestamp is 0 in blocks before block #2
            if block_number == block_one_as_block.clone() {
                return false;
            } else {
                return true;
            }
        }

        fn is_block_two(block_number: T::BlockNumber) -> bool {
            let block_two = 2u32;
            let block_two_as_block;
            if let Some(_block_two) = TryInto::<T::BlockNumber>::try_into(block_two).ok() {
                block_two_as_block = _block_two;
            } else {
                log::error!("Unable to convert u32 to BlockNumber");
                return false;
            }
            if block_number.clone() == block_two_as_block.clone() {
                return true;
            } else {
                return false;
            }
        }

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
            log::info!("convert_u64_in_milliseconds_to_start_of_date - date_as_u64_secs: {:?}", date_as_u64_secs.clone());
            // https://docs.rs/chrono/0.4.6/chrono/naive/struct.NaiveDateTime.html#method.from_timestamp
            let date = NaiveDateTime::from_timestamp(i64::try_from(date_as_u64_secs).unwrap(), 0).date();

            let date_start_millis = date.and_hms(0, 0, 0).timestamp() * 1000;
            log::info!("convert_u64_in_milliseconds_to_start_of_date - date_start_millis: {:?}", date_start_millis.clone());
            log::info!("convert_u64_in_milliseconds_to_start_of_date - Timestamp requested Date: {:?}", date);
            return Ok(date_start_millis);
        }

        fn convert_i64_in_milliseconds_to_start_of_date(date_as_i64_millis: i64) -> Result<Date, DispatchError> {
            let date_as_i64_secs = date_as_i64_millis.clone() / 1000i64;
            log::info!("convert_i64_in_milliseconds_to_start_of_date - date_as_i64_secs: {:?}", date_as_i64_secs.clone());
            // https://docs.rs/chrono/0.4.6/chrono/naive/struct.NaiveDateTime.html#method.from_timestamp
            let date = NaiveDateTime::from_timestamp(i64::try_from(date_as_i64_secs).unwrap(), 0).date();

            let date_start_millis = date.and_hms(0, 0, 0).timestamp() * 1000;
            log::info!("convert_i64_in_milliseconds_to_start_of_date - date_start_millis: {:?}", date_start_millis.clone());
            log::info!("convert_i64_in_milliseconds_to_start_of_date - Timestamp requested Date: {:?}", date);
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

        // fn convert_vec_u8_to_u128(data: &[u8]) -> Result<u128, DispatchError> {
        //     Ok(core::str::from_utf8(data)?.parse()?)
        // }

        // Convert a Vec<u8> that we received from an API endpoint that represents the mPower
        // associated with an account id into a u128 value.
        // ascii table: https://aws1.discourse-cdn.com/business5/uploads/rust_lang/original/3X/9/0/909baa7e3d9569489b07c791ca76f2223bd7bac2.webp
        pub fn convert_vec_u8_to_u128(data: &[u8]) -> Result<u128, DispatchError> {
            let mut out = 0u128;
            let mut multiplier = 1;

            for &val in data.iter().rev() {
                // log::info!("{:?}", val);

                let mut digit = 0u128;
                match val {
                    48u8 => {
                        digit = 0u128;
                    },
                    49u8 => {
                        digit = 1u128;
                    },
                    50u8 => {
                        digit = 2u128;
                    },
                    51u8 => {
                        digit = 3u128;
                    },
                    52u8 => {
                        digit = 4u128;
                    },
                    53u8 => {
                        digit = 5u128;
                    },
                    54u8 => {
                        digit = 6u128;
                    },
                    55u8 => {
                        digit = 7u128;
                    },
                    56u8 => {
                        digit = 8u128;
                    },
                    57u8 => {
                        digit = 9u128;
                    },
                    _ => {
                        log::error!("Non-digit ASCII char in input");
                        return Err(DispatchError::Other("Non-digit ASCII char in input"));
                    },
                }

                if digit != 0u128 && out != 0u128 {
                    multiplier *= 10;
                } else if digit != 0u128 && out == 0u128 {
                    multiplier *= 1;
                } else if digit == 0u128 && out != 0u128 {
                    multiplier *= 10;
                } else if digit == 0u128 && out == 0u128 {
                    multiplier *= 10;
                }

                out += multiplier * digit;
            }

            Ok(out)
        }

        fn set_bonded_dhx_of_account_for_date(account_public_key: Vec<u8>, bonded_dhx: u128) -> Result<u128, DispatchError> {
            // Note: we DO need the following as we're using the current timestamp, rather than a function parameter.
            let timestamp: <T as pallet_timestamp::Config>::Moment = <pallet_timestamp::Pallet<T>>::get();
            let requested_date_as_u64 = Self::convert_moment_to_u64_in_milliseconds(timestamp.clone())?;
            log::info!("set_bonded_dhx_of_account_for_date - requested_date_as_u64: {:?}", requested_date_as_u64.clone());

            // convert the requested date/time to the start of that day date/time to signify that date for lookup
            // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
            let start_of_requested_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(requested_date_as_u64.clone())?;

            let bonded_dhx_current_u128 = bonded_dhx.clone();

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

            // Update storage. Override the default that may have been set in on_initialize
            <BondedDHXForAccountForDate<T>>::insert(
                (
                    start_of_requested_date_millis.clone(),
                    account_public_key.clone(),
                ),
                bonded_dhx_current.clone(),
            );
            log::info!("set_bonded_dhx_of_account_for_date - account_public_key: {:?}", &account_public_key);
            log::info!("set_bonded_dhx_of_account_for_date - bonded_dhx_current: {:?}", &bonded_dhx_current);

            // Emit an event.
            Self::deposit_event(Event::SetBondedDHXOfAccountForDateStored(
                start_of_requested_date_millis.clone(),
                bonded_dhx_current.clone(),
                account_public_key.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(bonded_dhx_current_u128.clone())
        }

        pub fn set_mpower_of_account_for_date(account_public_key: Vec<u8>, start_of_requested_date_millis: Date, mpower: u128) -> Result<u128, DispatchError> {
            log::info!("set_mpower_of_account_for_date: {:?} {:?} {:?}", account_public_key.clone(), start_of_requested_date_millis.clone(), mpower.clone());
            let mpower_current_u128 = mpower.clone();

            // check if the new mpower value differs from the value that is already in storage
            // for the given key, and only insert if it is different
            let mpower_for_account_for_date = <MPowerForAccountForDate<T>>::get(
                (
                    start_of_requested_date_millis.clone(),
                    account_public_key.clone(),
                )
            );
            log::info!("existing mpower_for_account_for_date {:?}", mpower_for_account_for_date.clone());
            match mpower_for_account_for_date {
                None => {
                    log::info!("Adding new storage value of mPower for account for date of data retrieved from API {:?}", mpower_current_u128.clone());
                },
                Some(x) => {
                    log::warn!("Existing storage value of mPower for account for date of data retrieved from API {:?}", x);
                    return Err(DispatchError::Other("Existing storage value of mPower for account for date of data retrieved from API"));
                }
            }

            // Update storage. Override the default that may have been set in on_initialize
            <MPowerForAccountForDate<T>>::insert(
                (
                    start_of_requested_date_millis.clone(),
                    account_public_key.clone(),
                ),
                mpower_current_u128.clone(),
            );

            log::info!("Added MPowerForAccountForDate {:?} {:?} {:?}",
                start_of_requested_date_millis.clone(),
                account_public_key.clone(),
                mpower_current_u128.clone(),
            );

            // Emit an event.
            Self::deposit_event(Event::NewMPowerForAccountForDate(
                start_of_requested_date_millis.clone(),
                account_public_key.clone(),
                mpower_current_u128.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(mpower_current_u128.clone())
        }

        pub fn set_finished_fetching_mpower_for_accounts_for_date(all_miner_public_keys_fetched: Vec<Vec<u8>>, start_of_requested_date_millis: Date, total_mpower_fetched: u128) -> Result<u128, DispatchError> {
            log::info!("set_finished_fetching_mpower_for_accounts_for_date");
            // check if the new total_mpower_fetched value differs from the value that is already in storage
            // for the given key, and only insert if it is different
            let data_finished_fetching_for_date = <FinishedFetchingMPowerForAccountsForDate<T>>::get(start_of_requested_date_millis.clone());

            match data_finished_fetching_for_date {
                None => {
                },
                Some(x) => {
                    log::warn!("Existing storage value of finished fetching for date of data retrieved from API");
                    return Err(DispatchError::Other("Existing storage value of finished fetching for date of data retrieved from API"));
                }
            }

            // Update storage. Override the default that may have been set in on_initialize
            <FinishedFetchingMPowerForAccountsForDate<T>>::insert(
                start_of_requested_date_millis.clone(),
                (
                    all_miner_public_keys_fetched.clone(),
                    total_mpower_fetched.clone(),
                )
            );

            log::info!("Added FinishedFetchingMPowerForAccountsForDate {:?} {:?} {:?}",
                start_of_requested_date_millis.clone(),
                all_miner_public_keys_fetched.clone(),
                total_mpower_fetched.clone(),
            );

            // Emit an event.
            Self::deposit_event(Event::NewFinishedFetchingMPowerForAccountsForDate(
                start_of_requested_date_millis.clone(),
                all_miner_public_keys_fetched.clone(),
                total_mpower_fetched.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(total_mpower_fetched.clone())
        }

        fn get_min_bonded_dhx_daily() -> Result<(BalanceOf<T>, u128), DispatchError> {
            let mut min_bonded_dhx_daily: BalanceOf<T> = 10u32.into(); // initialize
            let mut min_bonded_dhx_daily_u128: u128 = TEN;
            if let Some(_min_bonded_dhx_daily) = <MinBondedDHXDaily<T>>::get() {
                min_bonded_dhx_daily = _min_bonded_dhx_daily;

                let _min_bonded_dhx_daily_u128 = Self::convert_balance_to_u128(min_bonded_dhx_daily.clone());
                match _min_bonded_dhx_daily_u128.clone() {
                    Err(_e) => {
                        log::error!("Unable to convert balance to u128 for min_bonded_dhx_daily_u128");
                        return Err(DispatchError::Other("Unable to convert balance to u128 for min_bonded_dhx_daily_u128"));
                    },
                    Ok(x) => {
                        min_bonded_dhx_daily_u128 = x;
                    }
                }
            } else {
                // fetch the min_bonded_dhx_daily_default instead as a fallback
                log::error!("Unable to retrieve any min. bonded DHX daily");

                let mut min_bonded_dhx_daily_default: BalanceOf<T> = 10u32.into();

                let _min_bonded_dhx_daily_default = Self::get_min_bonded_dhx_daily_default();
                match _min_bonded_dhx_daily_default {
                    Err(_e) => {
                        log::error!("Unable to retrieve any min. bonded DHX daily default as BalanceOf and u128");
                    },
                    Ok(ref x) => {
                        // Set the min. bonded DHX daily to the default value as fallback
                        min_bonded_dhx_daily = x.0;
                        min_bonded_dhx_daily_u128 = x.1;
                    }
                }
                println!("Reset to the min. bonded DHX daily default");
            }
            // Return a successful DispatchResultWithPostInfo
            Ok(
                (min_bonded_dhx_daily.clone(), min_bonded_dhx_daily_u128.clone())
            )
        }

        fn get_min_bonded_dhx_daily_default() -> Result<(BalanceOf<T>, u128), DispatchError> {
            let mut min_bonded_dhx_daily_default: BalanceOf<T> = 10u32.into(); // initialize
            if let Some(_min_bonded_dhx_daily_default) = <MinBondedDHXDailyDefault<T>>::get() {
                min_bonded_dhx_daily_default = _min_bonded_dhx_daily_default;
            } else {
                // TODO - if this fails we could try and fetch the min_bonded_dhx_daily_default instead as a fallback
                log::error!("Unable to retrieve any min. bonded DHX daily default");
                // return Err(DispatchError::Other("Unable to retrieve any min. bonded DHX daily default"));
            }

            let min_bonded_dhx_daily_default_u128;
            let _min_bonded_dhx_daily_default_u128 = Self::convert_balance_to_u128(min_bonded_dhx_daily_default.clone());
            match _min_bonded_dhx_daily_default_u128.clone() {
                Err(_e) => {
                    log::error!("Unable to convert balance to u128 for min_bonded_dhx_daily_default_u128");
                    return Err(DispatchError::Other("Unable to convert balance to u128 for min_bonded_dhx_daily_default_u128"));
                },
                Ok(x) => {
                    min_bonded_dhx_daily_default_u128 = x;
                }
            }
            // Return a successful DispatchResultWithPostInfo
            Ok(
                (min_bonded_dhx_daily_default.clone(), min_bonded_dhx_daily_default_u128.clone())
            )
        }

        pub fn change_rewards_multiplier_paused_status(new_status: bool) -> Result<bool, DispatchError> {

            <RewardsMultiplierPaused<T>>::put(new_status.clone());

            // Emit an event.
            Self::deposit_event(Event::ChangeRewardsMultiplierPausedStatusStored(
                new_status.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(new_status.clone())
        }

        pub fn change_rewards_multiplier_reset_status(new_status: bool) -> Result<bool, DispatchError> {

            <RewardsMultiplierReset<T>>::put(new_status.clone());

            // Emit an event.
            Self::deposit_event(Event::ChangeRewardsMultiplierResetStatusStored(
                new_status.clone(),
            ));

            // Return a successful DispatchResultWithPostInfo
            Ok(new_status.clone())
        }

        pub fn calculate_rewards_eligibility_of_miner_for_day(
            miner_public_key: Vec<u8>,
            start_of_requested_date_millis: i64,
            locks_first_amount_as_u128: u128,
            rewards_allowance_dhx_daily: BalanceOf<T>,
            min_bonded_dhx_daily_u128: u128,
            mpower_current_u128: u128,
            block_number: T::BlockNumber,
            miner_count: usize,
            reg_dhx_miners: Vec<Vec<u8>>,
        ) -> Result<(), &'static str> {
            log::info!("calculate_rewards_eligibility_of_miner_for_day");
            println!("calculate_rewards_eligibility_of_miner_for_day");

            // TODO - calculate the daily reward for the miner in DHX based on their mPower
            // and add that to the new_rewards_aggregated_dhx_daily_as_u128 (which currently only
            // includes the proportion of their reward based on their bonded DHX tokens) of all
            // miner's for that day, and also add that to the accumulated rewards for that specific
            // miner on that day.

            // calculate the daily reward for the miner in DHX based on their bonded DHX.
            // it should be a proportion taking other eligible miner's who are eligible for
            // daily rewards into account since we want to split them fairly.
            //
            // assuming min_bonded_dhx_daily is 10u128, and they have that minimum of 10 DHX bonded (10u128) for
            // the locks_first_amount_as_u128 value, then they are eligible for 1 DHX reward if they have 1 mPower
            //

            // Divide, handling overflow
            let mut div_bonded_as_u128 = 0u128;
            // note: this rounds down to the nearest integer
            let _div_bonded_as_u128 = locks_first_amount_as_u128.clone().checked_div(min_bonded_dhx_daily_u128.clone());
            match _div_bonded_as_u128 {
                None => {
                    log::error!("Unable to divide min_bonded_dhx_daily from locks_first_amount_as_u128 due to StorageOverflow");
                    println!("Unable to divide min_bonded_dhx_daily from locks_first_amount_as_u128 due to StorageOverflow");
                    return Err("Unable to divide min_bonded_dhx_daily from locks_first_amount_as_u128 due to StorageOverflow");
                },
                Some(x) => {
                    div_bonded_as_u128 = x;
                }
            }
            log::info!("div_bonded_as_u128: {:?}", div_bonded_as_u128.clone());

            // Multiply, handling overflow
            let mut daily_reward_for_miner_as_u128 = 0u128;
            // note: this rounds down to the nearest integer
            let _daily_reward_for_miner_as_u128 = mpower_current_u128.clone().checked_mul(div_bonded_as_u128.clone());
            match _daily_reward_for_miner_as_u128 {
                None => {
                    log::error!("Unable to divide min_bonded_dhx_daily from locks_first_amount_as_u128 due to StorageOverflow");
                    println!("Unable to divide min_bonded_dhx_daily from locks_first_amount_as_u128 due to StorageOverflow");
                    return Err("Unable to divide min_bonded_dhx_daily from locks_first_amount_as_u128 due to StorageOverflow");
                },
                Some(x) => {
                    daily_reward_for_miner_as_u128 = x;
                }
            }
            log::info!("daily_reward_for_miner_as_u128: {:?}", daily_reward_for_miner_as_u128.clone());
            println!("[eligible] block: {:#?}, miner: {:#?}, date_start: {:#?} daily_reward_for_miner_as_u128: {:#?}", block_number, miner_count, start_of_requested_date_millis, daily_reward_for_miner_as_u128);

            // if we have a rewards_aggregated_dhx_daily of 25.133 k DHX, then after the above manipulation
            // since we're dealing with a mixture of u128 and BalanceOf<T> so the values are more readable in the UI.
            // the reward will be represented as 2.5130 f DHX (where f is femto 10^-18, i.e. 0.000_000_000_000_002_513)
            // so we need to multiply it by 1_000_000_000_000_000_000 to be represented in DHX instead of femto DHX
            // before storing the value. we need to do the same for the rewards accumulated value before it is stored.
            // daily_reward_for_miner_as_u128 = daily_reward_for_miner_as_u128;
            if let Some(_daily_reward_for_miner_as_u128) = daily_reward_for_miner_as_u128.clone().checked_mul(1_000_000_000_000_000_000u128) {
                daily_reward_for_miner_as_u128 = _daily_reward_for_miner_as_u128;
            } else {
                log::error!("Unable to multiply daily_reward_for_miner_as_u128");
            }

            let mut daily_reward_for_miner;
            let _daily_reward_for_miner = Self::convert_u128_to_balance(daily_reward_for_miner_as_u128.clone());
            match _daily_reward_for_miner {
                Err(_e) => {
                    log::error!("Unable to convert u128 to balance for daily_reward_for_miner");
                    println!("Unable to convert u128 to balance for daily_reward_for_miner");
                    return Err("Unable to convert u128 to balance for daily_reward_for_miner");
                },
                Ok(ref x) => {
                    daily_reward_for_miner = x;
                }
            }
            log::info!("daily_reward_for_miner: {:?}", daily_reward_for_miner.clone());

            let mut rewards_aggregated_dhx_daily: BalanceOf<T> = 0u32.into(); // initialize
            if let Some(_rewards_aggregated_dhx_daily) = <RewardsAggregatedDHXForAllMinersForDate<T>>::get(&start_of_requested_date_millis) {
                rewards_aggregated_dhx_daily = _rewards_aggregated_dhx_daily;
            } else {
                log::error!("Unable to retrieve balance for rewards_aggregated_dhx_daily");
            }

            let rewards_aggregated_dhx_daily_as_u128;
            let _rewards_aggregated_dhx_daily_as_u128 = Self::convert_balance_to_u128(rewards_aggregated_dhx_daily.clone());
            match _rewards_aggregated_dhx_daily_as_u128.clone() {
                Err(_e) => {
                    log::error!("Unable to convert balance to u128 for rewards_aggregated_dhx_daily_as_u128");
                    println!("Unable to convert balance to u128 for rewards_aggregated_dhx_daily_as_u128");
                    return Err("Unable to convert balance to u128 for rewards_aggregated_dhx_daily_as_u128");
                },
                Ok(x) => {
                    rewards_aggregated_dhx_daily_as_u128 = x;
                }
            }

            // Add, handling overflow
            let mut new_rewards_aggregated_dhx_daily_as_u128;
            let _new_rewards_aggregated_dhx_daily_as_u128 =
                rewards_aggregated_dhx_daily_as_u128.clone().checked_add(daily_reward_for_miner_as_u128.clone());
            match _new_rewards_aggregated_dhx_daily_as_u128 {
                None => {
                    log::error!("Unable to add daily_reward_for_miner to rewards_aggregated_dhx_daily due to StorageOverflow");
                    println!("Unable to add daily_reward_for_miner to rewards_aggregated_dhx_daily due to StorageOverflow");
                    return Err("Unable to add daily_reward_for_miner to rewards_aggregated_dhx_daily due to StorageOverflow");
                },
                Some(x) => {
                    new_rewards_aggregated_dhx_daily_as_u128 = x;
                }
            }

            log::info!("new_rewards_aggregated_dhx_daily_as_u128: {:?}", new_rewards_aggregated_dhx_daily_as_u128.clone());
            println!("[eligible] block: {:#?}, miner: {:#?}, date_start: {:#?} new_rewards_aggregated_dhx_daily_as_u128: {:#?}", block_number, miner_count, start_of_requested_date_millis, new_rewards_aggregated_dhx_daily_as_u128);

            let new_rewards_aggregated_dhx_daily;
            let _new_rewards_aggregated_dhx_daily = Self::convert_u128_to_balance(new_rewards_aggregated_dhx_daily_as_u128.clone());
            match _new_rewards_aggregated_dhx_daily {
                Err(_e) => {
                    log::error!("Unable to convert u128 to balance for new_rewards_aggregated_dhx_daily");
                    println!("Unable to convert u128 to balance for new_rewards_aggregated_dhx_daily");
                    return Err("Unable to convert u128 to balance for new_rewards_aggregated_dhx_daily");
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
            log::info!("Added RewardsAggregatedDHXForAllMinersForDate for miner {:?} {:?} {:?}", start_of_requested_date_millis.clone(), miner_public_key.clone(), new_rewards_aggregated_dhx_daily.clone());
            println!("Added RewardsAggregatedDHXForAllMinersForDate for miner {:?} {:?} {:?}", start_of_requested_date_millis.clone(), miner_public_key.clone(), new_rewards_aggregated_dhx_daily.clone());

            // add to storage item that maps the date to the registered miner and the calculated reward
            // (prior to possibly reducing it so they get a proportion of the daily rewards that are available)
            <RewardsAccumulatedDHXForMinerForDate<T>>::insert(
                (
                    start_of_requested_date_millis.clone(),
                    miner_public_key.clone(),
                ),
                daily_reward_for_miner.clone(),
            );
            log::info!("Added RewardsAccumulatedDHXForMinerForDate for miner {:?} {:?} {:?}", start_of_requested_date_millis.clone(), miner_public_key.clone(), daily_reward_for_miner.clone());
            println!("Added RewardsAccumulatedDHXForMinerForDate for miner {:?} {:?} {:?}", start_of_requested_date_millis.clone(), miner_public_key.clone(), daily_reward_for_miner.clone());

            // Store a list of all the the registered_dhx_miners that are eligible for rewards on a given date
            // so we know they have been used as keys for other storage maps like `RewardsAccumulatedDHXForMinerForDate`
            // for future querying
            let mut rewards_eligible_miners_for_date: Vec<Vec<u8>> = vec![]; // initialize
            if let Some(_rewards_eligible_miners_for_date) = <RewardsEligibleMinersForDate<T>>::get(start_of_requested_date_millis.clone()) {
                rewards_eligible_miners_for_date = _rewards_eligible_miners_for_date;
            } else {
                log::warn!("Unable to retrieve rewards_eligible_miners_for_date");
                println!("Unable to retrieve rewards_eligible_miners_for_date");
            }
            log::info!("Retrieved existing rewards_eligible_miners_for_date {:?} {:?}", start_of_requested_date_millis.clone(), rewards_eligible_miners_for_date.clone());
            println!("Retrieved existing rewards_eligible_miners_for_date {:?} {:?}", start_of_requested_date_millis.clone(), rewards_eligible_miners_for_date.clone());

            log::warn!("Updating rewards_eligible_miners_for_date with miner {:?} {:?}", start_of_requested_date_millis.clone(), miner_public_key.clone());
            println!("Updating retrieve rewards_eligible_miners_for_date with miner {:?} {:?}", start_of_requested_date_millis.clone(), miner_public_key.clone());

            rewards_eligible_miners_for_date.push(miner_public_key.clone());
            <RewardsEligibleMinersForDate<T>>::insert(
                start_of_requested_date_millis.clone(),
                rewards_eligible_miners_for_date.clone(),
            );
            log::info!("date: {:?}, miner_count: {:?}, reg_dhx_miners.len: {:?}", start_of_requested_date_millis.clone(), miner_count.clone(), reg_dhx_miners.len());
            println!("date: {:?}, miner_count: {:?}, reg_dhx_miners.len: {:?}", start_of_requested_date_millis.clone(), miner_count.clone(), reg_dhx_miners.len());
            // if last miner being iterated then reset for next day
            if reg_dhx_miners.len() == miner_count {
                log::info!("date: {:?}, rewards_allowance_dhx_daily: {:?}", start_of_requested_date_millis.clone(), rewards_allowance_dhx_daily.clone());
                println!("date: {:?}, rewards_allowance_dhx_daily: {:?}", start_of_requested_date_millis.clone(), rewards_allowance_dhx_daily.clone());

                // reset to latest set by governance
                <RewardsAllowanceDHXForDateRemaining<T>>::insert(start_of_requested_date_millis.clone(), rewards_allowance_dhx_daily.clone());
            };

            Ok(())
        }

        // Offchain workers

        /// Chooses which transaction type to send.
        ///
        /// This function serves mostly to showcase `StorageValue` helper
        /// and local storage usage.
        ///
        /// Returns a type of transaction that should be produced in current run.
        ///
        /// TODO - figure out how to effectively use Local Storage and whether to use
        /// signed or unsigned transactions
        fn choose_transaction_type(block_number: T::BlockNumber) -> TransactionType {
            /// A friendlier name for the error that is going to be returned in case we are in the grace
            /// period.
            const RECENTLY_SENT: () = ();

            // Start off by creating a reference to Local Storage value.
            // Since the local storage is common for all offchain workers, it's a good practice
            // to prepend your entry with the module name.
            let val = StorageValueRef::persistent(b"mpow_ocw::last_send");
            // The Local Storage is persisted and shared between runs of the offchain workers,
            // and offchain workers may run concurrently. We can use the `mutate` function, to
            // write a storage entry in an atomic fashion. Under the hood it uses `compare_and_set`
            // low-level method of local storage API, which means that only one worker
            // will be able to "acquire a lock" and send a transaction if multiple workers
            // happen to be executed concurrently.
            let res = val.mutate(|last_send: Result<Option<T::BlockNumber>, StorageRetrievalError>| {
                match last_send {
                    // If we already have a value in storage and the block number is recent enough
                    // we avoid sending another transaction at this time.
                    Ok(Some(block)) if block_number < block + T::GracePeriod::get() =>
                        Err(RECENTLY_SENT),
                    // In every other case we attempt to acquire the lock and send a transaction.
                    _ => Ok(block_number),
                }
            });

            // The result of `mutate` call will give us a nested `Result` type.
            // The first one matches the return of the closure passed to `mutate`, i.e.
            // if we return `Err` from the closure, we get an `Err` here.
            // In case we return `Ok`, here we will have another (inner) `Result` that indicates
            // if the value has been set to the storage correctly - i.e. if it wasn't
            // written to in the meantime.
            match res {
                // The value has been set correctly, which means we can safely send a transaction now.
                Ok(block_number) => {
                    TransactionType::Raw
                },
                // We are in the grace period, we should not send a transaction this time.
                Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) => TransactionType::None,
                // We wanted to send a transaction, but failed to write the block number (acquire a
                // lock). This indicates that another offchain worker that was running concurrently
                // most likely executed the same logic and succeeded at writing to storage.
                // Thus we don't really want to send the transaction, knowing that the other run
                // already did.
                Err(MutateStorageError::ConcurrentModification(_)) => TransactionType::None,
            }
        }

        /// A helper function to fetch the mpower
        fn fetch_mpower_process(block_number: T::BlockNumber, start_of_requested_date_millis: Date) -> Result<Vec<MPowerPayloadData<T>>, &'static str> {
            // Make sure we don't fetch the mpower if unsigned transaction is going to be rejected
            // anyway.
            let next_unsigned_at_for_fetched_mpower = <NextUnsignedAtForFetchedMPower<T>>::get();
            if next_unsigned_at_for_fetched_mpower > block_number {
                return Err("Too early to send unsigned transaction")
            }

            // Make an external HTTP request to fetch the current mpower.
            // Note this call will block until response is received.
            let mpower_data_vec: Vec<MPowerPayloadData<T>> = Self::fetch_mpower(block_number.clone(), start_of_requested_date_millis.clone()).map_err(|_| "Failed to fetch mpower data vec")?;

            Ok(mpower_data_vec.clone())
        }

        /// A helper function to send a raw unsigned transaction to store the mpower data.
        fn store_mpower_raw_unsigned(block_number: T::BlockNumber, start_of_requested_date_millis: Date, mpower_data_vec: Vec<MPowerPayloadData<T>>) -> Result<(), &'static str> {
            log::info!("offchain_worker - store_mpower_raw_unsigned");
            // Received mpower data is wrapped into a call to `submit_fetched_mpower_unsigned` public function of this
            // pallet. This means that the transaction, when executed, will simply call that function
            // passing `mpower_data_vec` as an argument.
            let call = Call::submit_fetched_mpower_unsigned(block_number.clone(), start_of_requested_date_millis.clone(), mpower_data_vec.clone());

            // Now let's create a transaction out of this call and submit it to the pool.
            // Here we showcase two ways to send an unsigned transaction / unsigned payload (raw)
            //
            // TODO - By default unsigned transactions are disallowed, so we need to whitelist this case
            // by writing `UnsignedValidator`. Note that it's EXTREMELY important to carefuly
            // implement unsigned validation logic, as any mistakes can lead to opening DoS or spam
            // attack vectors. See validation logic docs for more details.
            //
            SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
                .map_err(|()| "Unable to submit unsigned transaction for submit_fetched_mpower.")?;

            Ok(())
        }

        /// A helper function to send a raw unsigned transaction to store finished fetching mpower data for use in on_initialize
        fn store_finished_fetching_mpower_raw_unsigned(block_number: T::BlockNumber, fetched_mpower_data: FetchedMPowerForAccountsForDatePayloadData) -> Result<(), &'static str> {
            log::info!("offchain_worker - store_finished_fetching_mpower_raw_unsigned");
            // Received mpower data is wrapped into a call to `submit_finished_fetching_mpower_unsigned` public function of this
            // pallet. This means that the transaction, when executed, will simply call that function
            // passing `fetched_mpower_data` as an argument.
            let call = Call::submit_finished_fetching_mpower_unsigned(block_number.clone(), fetched_mpower_data.clone());

            // Now let's create a transaction out of this call and submit it to the pool.
            // Here we showcase two ways to send an unsigned transaction / unsigned payload (raw)
            //
            // TODO - By default unsigned transactions are disallowed, so we need to whitelist this case
            // by writing `UnsignedValidator`. Note that it's EXTREMELY important to carefuly
            // implement unsigned validation logic, as any mistakes can lead to opening DoS or spam
            // attack vectors. See validation logic docs for more details.
            //
            SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
                .map_err(|()| "Unable to submit unsigned transaction for submit_finished_fetching_mpower.")?;

            Ok(())
        }

        /// Fetch current mPower and return the result.
        fn fetch_mpower(block_number: T::BlockNumber, start_of_requested_date_millis: Date) -> Result<Vec<MPowerPayloadData<T>>, http::Error> {
            log::info!("offchain_worker - fetch_mpower");

            // TODO - below is temporarily commented out whilst we are using mock API data

            // // We want to keep the offchain worker execution time reasonable, so we set a hard-coded
            // // deadline to 2s to complete the external call.
            // // You can also wait idefinitely for the response, however you may still get a timeout
            // // coming from the host machine.

            // // Initiate an external HTTP GET request.
            // // This is using high-level wrappers from `sp_runtime`, for the low-level calls that
            // // you can find in `sp_io`. The API is trying to be similar to `reqwest`, but
            // // since we are running in a custom WASM execution environment we can't simply
            // // import the library here.

            // // Example from Substrate
            // let request =
            //     http::Request::get("https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD");

            // // Example of request we may use
            // // let start_of_requested_date_millis = 1630195200000i64;
            // // let url = format!("https://api.datahighway.com/data/mpower-for-date?start_of_requested_date_millis={}", start_of_requested_date_millis);
            // // log::info!("Request URL: {}", url.clone());
            // // let request =
            // //     http::Request::get(&url);

            // // We set the deadline for sending of the request, note that awaiting response can
            // // have a separate deadline. Next we send the request, before that it's also possible
            // // to alter request headers or stream body content in case of non-GET requests.
            // let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

            // // The request is already being processed by the host, we are free to do anything
            // // else in the worker (we can send multiple concurrent requests too).
            // // At some point however we probably want to check the response though,
            // // so we can block current thread and wait for it to finish.
            // // Note that since the request is being driven by the host, we don't have to wait
            // // for the request to have it complete, we will just not read the response.
            // let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;

            // // Let's check the status code before we proceed to reading the response.
            // if response.code != 200 {
            //     log::warn!("offchain_worker - Unexpected status code: {}", response.code);
            //     return Err(http::Error::Unknown)
            // }

            // // Next we want to fully read the response body and collect it to a vector of bytes.
            // // Note that the return object allows you to read the body in chunks as well
            // // with a way to control the deadline.
            // let body = response.body().collect::<Vec<u8>>();

            // // Create a str slice from the body.
            // let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
            //     log::warn!("offchain_worker - No UTF8 body");
            //     http::Error::Unknown
            // })?;

            // log::info!("offchain_worker - Received HTTP Body: {}", body_str.clone());

            // FIXME - replace the below hard-coded example in future with use of the response body
            // Alice public key 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
            // Bob public key 0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48
            // Charlie public key 0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22
            //
            // Note: we're not currently using the `start_of_requested_date_millis` that is in the hard-coded data below,
            // and instead we're using the current date temporarily using
            // `received_at_date: received_date_as_millis.clone(),`
            let mpower_data = r#"{
                "data": [
                    { "acct_id": "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d", "mpower": "11", "start_of_requested_date_millis": "1630195200000" },
                    { "acct_id": "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", "mpower": "12", "start_of_requested_date_millis": "1630195200000" },
                    { "acct_id": "90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22", "mpower": "13", "start_of_requested_date_millis": "1630195200000" }
                ]
            }"#;

            let mpower_data_vec = match Self::parse_mpower_data(mpower_data, block_number.clone()) {
                Some(data) => Ok(data),
                None => {
                    log::warn!("offchain_worker - Unable to extract mpower data from the response");
                    Err(http::Error::Unknown)
                },
            }?;

            // log::info!("offchain_worker - Parsed mpower_data_vec: {:?}", mpower_data_vec);

            Ok(mpower_data_vec)
        }

        /// Parse the mPower from the given JSON string using `lite-json`.
        ///
        /// Returns `None` when parsing failed or `Some(mpower_data)` when parsing is successful.
        fn parse_mpower_data(mpower_data_str: &str, block_number: T::BlockNumber) -> Option<Vec<MPowerPayloadData<T>>> {
            // checking it works using serde_json, but cannot use in substrate as it uses std:
            // https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=09eee43b3354f2a798ca4394838fdef7

            let timestamp = <pallet_timestamp::Pallet<T>>::get();
            let received_date_as_u64 = Self::convert_moment_to_u64_in_milliseconds(timestamp.clone()).ok()?;
            log::info!("offchain_worker - received_date_as_u64: {:?}", received_date_as_u64.clone());
            // TODO - parse for mPower data and replace hard-coded response with output
            let received_date_as_millis = Self::convert_u64_in_milliseconds_to_start_of_date(received_date_as_u64.clone()).ok()?;

            let mpower_json_data = match lite_json::parse_json(mpower_data_str) {
                Err(e) => {
                    log::error!("Couldn't parse JSON {:?}", e);
                    println!("Couldn't parse JSON {:?}", e);
                    return None;
                },
                Ok(data) => data,
            };

            // let mpower_json_data: MPowerJSONResponseData<T::AccountId, u128> = match serde_json::from_str(mpower_data_str) {
            //     Err(e) => {
            //         println!("Couldn't parse JSON :( {:?}", e);
            //         return None;
            //     },
            //     Ok(data) => data,
            // };
            // log::info!("offchain_worker - mpower_json_data{:#?}", mpower_json_data);

            let mut mpower_data_vec: Vec<MPowerPayloadData<T>> = vec![];
            let mpower_array = match mpower_json_data {
                JsonValue::Object(obj) => {
                    let (_, v) = obj.into_iter().find(|(k, _)| k.iter().copied().eq("data".chars()))?;
                    match v {
                        JsonValue::Array(vec) => vec,
                        _ => return None,
                    }
                },
                _ => return None,
            };

            for (i, obj) in mpower_array.into_iter().enumerate() {
                let obj_acct_id = match obj.clone() {
                    JsonValue::Object(obj_data) => {
                        let (_, v) = obj_data.into_iter().find(|(k, _)| k.iter().copied().eq("acct_id".chars()))?;
                        match v {
                            JsonValue::String(val) => val,
                            _ => return None,
                        }
                    },
                    _ => return None,
                };

                let obj_mpower = match obj.clone() {
                    JsonValue::Object(obj_data) => {
                        let (_, v) = obj_data.into_iter().find(|(k, _)| k.iter().copied().eq("mpower".chars()))?;
                        match v {
                            JsonValue::String(val) => val,
                            _ => return None,
                        }
                    },
                    _ => return None,
                };

                log::info!("offchain_worker - obj_mpower char {:?} {:?}", i, obj_mpower.clone());

                // Convert from `Vec<char>` to `Vec<u8>` since we do not use String in the runtime
                // e.g. converts from `['1', '2', '3']` to `123`
                let obj_acct_id_str_hex: Vec<u8> = obj_acct_id.iter().map(|c| *c as u8).collect::<Vec<_>>();
                let obj_mpower_str_hex: Vec<u8> = obj_mpower.iter().map(|c| *c as u8).collect::<Vec<_>>();
                log::info!("offchain_worker - obj_mpower_str_hex {:?} {:?}", i, obj_mpower_str_hex.clone());

                // Decode from hex ascii format
                let obj_acct_id_str = hex::decode(obj_acct_id_str_hex.clone()).ok()?;
                log::info!("offchain_worker - Decoded acct_id i public key hex as Vec<u8> {:?} {:?}", i, obj_acct_id_str.clone());

                let mpower_u128 = Self::convert_vec_u8_to_u128(&obj_mpower_str_hex).ok()?;
                log::info!("offchain_worker - mpower_u128 {:?} {:?}", i, mpower_u128.clone());

                // let mut obj_mpower_as_u128 = 0u128; // initialize
                // if let Some(_obj_mpower_as_u128) = TryInto::<u128>::try_into(obj_mpower_str.clone()).ok() {
                //     obj_mpower_as_u128 = _obj_mpower_as_u128;
                // } else {
                //     log::error!("offchain_worker - Unable to convert Vec<u8> into u128");
                //     return None;
                // }
                // log::info!("offchain_worker - obj_mpower_as_u128 {:?} {:?}", i, obj_mpower_as_u128.clone());

                // Example only:
                // Alice public key 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
                //
                // Note: do not do `hex!["..."].encode(), since that will just encoding a vec,
                // which will include a length prefix, but we don't want that.
                // let example_acct_id_str: Vec<u8> = write_hex!["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"].into();
                // log::info!("offchain_worker - example_acct_id_str {:?}", example_acct_id_str);

                // let mpower_as_u128: u128 = obj_mpower.clone().parse().unwrap(); // convert from string to number

                // Example of how to access the Vec<u8> string representation of the account's public key hex
                // let reg_dhx_miners;
                // if let Some(_reg_dhx_miners) = <RegisteredDHXMiners<T>>::get() {
                //     reg_dhx_miners = _reg_dhx_miners;
                // } else {
                //     log::error!("offchain_worker - Unable to retrieve any registered DHX Miners");
                //     return None;
                // }
                // let first_reg_dhx_miner = &reg_dhx_miners[0];
                // log::info!("offchain_worker - first_reg_dhx_miner {:?}", first_reg_dhx_miner.encode());

                let mpower_data_elem: MPowerPayloadData<T> = MPowerPayload {
                    account_id_registered_dhx_miner: obj_acct_id_str.clone(),
                    mpower_registered_dhx_miner: mpower_u128.clone(),
                    received_at_date: received_date_as_millis.clone(),
                    received_at_block_number: block_number.clone(),
                };

                mpower_data_vec.push(mpower_data_elem);
            }

            // log::info!("offchain_worker - mpower_data_vec {:?}", mpower_data_vec);

            Some(mpower_data_vec)
        }

        /// Add new mPower on-chain.
        fn add_mpower(account_id: T::AccountId, start_of_requested_date_millis: Date, mpower_data_vec: Vec<MPowerPayloadData<T>>) -> Option<Vec<MPowerPayloadData<T>>> {
            // note: AccountId as Vec<u8> is [0, 0, ... 0] since its an unsigned transaction
            log::info!("offchain_worker - Processing mPower for account for date into storage: {:?}", start_of_requested_date_millis.clone());

            for (index, mpower_data_item) in mpower_data_vec.iter().enumerate() {
                log::info!("offchain_worker - Processing mPower for account for date into storage - loop: {:?} {:?} {:?} {:?}", index.clone(), mpower_data_item.account_id_registered_dhx_miner.clone(), start_of_requested_date_millis.clone(), mpower_data_item.mpower_registered_dhx_miner.clone());

                let _set_mpower = Self::set_mpower_of_account_for_date(
                    mpower_data_item.account_id_registered_dhx_miner.clone(),
                    start_of_requested_date_millis.clone(),
                    mpower_data_item.mpower_registered_dhx_miner.clone(),
                );
                match _set_mpower.clone() {
                    Err(_e) => {
                        log::warn!("Unable to set_mpower_of_account_for_date");
                        return None;
                    },
                    Ok(x) => {
                        log::info!("set set_mpower_of_account_for_date to: {:?} {:?}", mpower_data_item.account_id_registered_dhx_miner.clone(), x);
                        continue;
                    }
                }
            }

            Some(mpower_data_vec.clone())
        }

        /// Add new finished fetching mPower on-chain.
        fn add_finished_fetching_mpower(account_id: T::AccountId, fetched_mpower_data: FetchedMPowerForAccountsForDatePayloadData) -> Option<FetchedMPowerForAccountsForDatePayloadData> {
            // note: AccountId as Vec<u8> is [0, 0, ... 0] since its an unsigned transaction
            log::info!("offchain_worker - Processing finished fetching mPower for account for date into storage: {:?}", fetched_mpower_data.start_of_requested_date_millis.clone());

            Self::set_finished_fetching_mpower_for_accounts_for_date(
                fetched_mpower_data.all_miner_public_keys_fetched.clone(),
                fetched_mpower_data.start_of_requested_date_millis.clone(),
                fetched_mpower_data.total_mpower_fetched.clone(),
            );

            Some(fetched_mpower_data.clone())
        }

        fn validate_transaction_parameters_for_finished_fetching_mpower(
            block_number: &T::BlockNumber,
            fetched_mpower_data: &FetchedMPowerForAccountsForDatePayloadData,
        ) -> TransactionValidity {
            // Now let's check if the transaction has any chance to succeed.
            let next_unsigned_at_for_finished_fetching_mpower = <NextUnsignedAtForFinishedFetchingMPower<T>>::get();
            log::info!("offchain_worker - validate_transaction_parameters_for_finished_fetching_mpower - block_number {:?}", block_number.clone());
            log::info!("offchain_worker - validate_transaction_parameters_for_finished_fetching_mpower - next_unsigned_at_for_finished_fetching_mpower {:?}", next_unsigned_at_for_finished_fetching_mpower.clone());
            // TODO - do we need to use a modified version of this at all?
            if &next_unsigned_at_for_finished_fetching_mpower > block_number {
                return InvalidTransaction::Stale.into()
            }
            // Let's make sure to reject transactions from the future.
            let current_block = <system::Pallet<T>>::block_number();
            log::info!("offchain_worker - validate_transaction_parameters_for_finished_fetching_mpower - current_block {:?}", current_block.clone());
            if &current_block < block_number {
                return InvalidTransaction::Future.into()
            }

            // See Substrate client/transaction-pool for configuration details
            ValidTransaction::with_tag_prefix("OffchainWorkerFinishedFetchingMPower")
                .priority(T::UnsignedPriority::get().saturating_add(2u128 as _))
                .and_provides(next_unsigned_at_for_finished_fetching_mpower)
                // TODO - do we need to use .and_requires() and provide the `previous_unsigned_at`
                // to signify that we need the fetched_mpower unsigned tx to happen beforehand?
                .longevity(5)
                .propagate(true)
                .build()
        }

        fn validate_transaction_parameters_for_fetched_mpower(
            block_number: &T::BlockNumber,
            start_of_requested_date_millis: &Date,
            new_mpower_data: &Vec<MPowerPayloadData<T>>,
        ) -> TransactionValidity {
            // Now let's check if the transaction has any chance to succeed.
            let next_unsigned_at_for_fetched_mpower = <NextUnsignedAtForFetchedMPower<T>>::get();
            log::info!("offchain_worker - validate_transaction_parameters_for_fetched_mpower - block_number {:?}", block_number.clone());
            log::info!("offchain_worker - validate_transaction_parameters_for_fetched_mpower - next_unsigned_at_for_fetched_mpower {:?}", next_unsigned_at_for_fetched_mpower.clone());
            if &next_unsigned_at_for_fetched_mpower > block_number {
                return InvalidTransaction::Stale.into()
            }
            // Let's make sure to reject transactions from the future.
            let current_block = <system::Pallet<T>>::block_number();
            log::info!("offchain_worker - validate_transaction_parameters_for_fetched_mpower - current_block {:?}", current_block.clone());
            if &current_block < block_number {
                return InvalidTransaction::Future.into()
            }

            // See Substrate client/transaction-pool for configuration details
            ValidTransaction::with_tag_prefix("OffchainWorkerFetchedMPower")
                // Set base priority and hope it's included before any other
                // transactions in the pool
                .priority(T::UnsignedPriority::get().saturating_add(1u128 as _))
                // This transaction does not require anything else to go before into the pool.
                // In theory we could require `previous_unsigned_at` transaction to go first,
                // but it's not necessary in our case.
                //.and_requires()
                // We set the `provides` tag to be the same as `next_unsigned_at_for_fetched_mpower`. This makes
                // sure only one transaction produced after `next_unsigned_at_for_fetched_mpower` will ever
                // get to the transaction pool and will end up in the block.
                // We can still have multiple transactions compete for the same "spot",
                // and the one with higher priority will replace other one in the pool.
                .and_provides(next_unsigned_at_for_fetched_mpower)
                // The transaction is only valid for next 5 blocks. After that it's
                // going to be revalidated by the pool.
                .longevity(5)
                // It's fine to propagate that transaction to other peers, which means it can be
                // created even by nodes that don't produce blocks.
                // Note that sometimes it's better to keep it for yourself (if you are the block
                // producer), since for instance in some schemes others may copy your solution and
                // claim a reward.
                .propagate(true)
                .build()
        }

        // check that the current date start_of_current_date_millis is at least 7 days after the provided
        // start_of_requested_date_millis so there is sufficient time for the community to audit the reward eligibility,
        // where the start_of_requested_date_millis refers to a past date the claimant believes they became eligible for rewards on.
        //
        // `CoolingDownPeriodDays` notifies when to give rewards in bulk (previously automatically, now by claiming) on the 8th day in bulk to cover the
        // first say x (i.e. 7) days after they start bonding, and then on the next day after each day after that, and
        // it is also used to track the unbonding period where if they stop bonding obviously they cannot be eligible
        // for rewards and they have wait x (i.e. 7) days after unbonding before they can access the locked DHX tokens
        // that they were bonding with, whereas:
        //
        // `ChallengePeriodDays` is used only to ensure you can only claim each daily rewards manually 7 days after being found eligible,
        // even for the first 7 days after they start bonding)
        pub fn is_more_than_challenge_period(start_of_requested_date_millis: i64) -> Result<(), DispatchError> {
            let current_timestamp = <pallet_timestamp::Pallet<T>>::get();
            let current_timestamp_as_u64 = Self::convert_moment_to_u64_in_milliseconds(current_timestamp.clone())?;
            log::info!("current_timestamp_as_u64: {:?}", current_timestamp_as_u64.clone());
            // convert the current timestamp to the start of that day date/time
            // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
            let start_of_current_date_millis = Self::convert_u64_in_milliseconds_to_start_of_date(current_timestamp_as_u64.clone())?;

            // where there are 86400000 milliseconds in a day then challenge_period_millis is
            // 7 * 86400000 = 604800000 milliseconds if challenge_period_days is 7 days
            // so we want to make sure `start_of_current_date_millis` - `start_of_requested_date_millis` > 604800000

            let mut is_more_than_challenge_period = false;
            if let Some(_period_millis) = start_of_current_date_millis.clone().checked_sub(start_of_requested_date_millis.clone()) {
                let millis_per_day = 86400000u64;
                let mut challenge_period_days = 0u64;
                if let Some(_challenge_period_days) = <ChallengePeriodDays<T>>::get() {
                    challenge_period_days = _challenge_period_days;
                    log::info!("existing challenge_period_days: {:?}", challenge_period_days.clone());
                    println!("existing challenge_period_days: {:?}", challenge_period_days.clone());
                } else {
                    log::error!("Unable to get challenge_period_days");
                    return Err(DispatchError::Other("Unable to get challenge_period_days"));;
                }
                let mut challenge_period_millis = 0u64;
                if let Some(_challenge_period_millis) = challenge_period_days.clone().checked_mul(millis_per_day.clone()) {
                    challenge_period_millis = _challenge_period_millis;
                } else {
                    log::error!("Unable to multiply to determine challenge_period_millis");
                    return Err(DispatchError::Other("Unable to multiply to determine challenge_period_millis"));
                }

                let challenge_period_millis = challenge_period_days.clone() * millis_per_day.clone();

                let mut period_millis_u64 = 0u64; // initialize
                if let Some(_period_millis_u64) = TryInto::<u64>::try_into(_period_millis.clone()).ok() {
                    period_millis_u64 = _period_millis_u64;
                } else {
                    log::error!("Unable to convert i32 to u64 for period_millis");
                    return Err(DispatchError::Other("Unable to convert i32 to u64 for period_millis"));
                }

                // log::info!("period_millis_u64: {:?}", period_millis_u64.clone());
                // println!("period_millis_u64: {:?}", period_millis_u64.clone());
                if (period_millis_u64 >= challenge_period_millis.clone()) {
                    is_more_than_challenge_period = true;
                    println!("is_more_than_challenge_period: {:?}", is_more_than_challenge_period.clone());
                    return Ok(());
                } else {
                    return Err(DispatchError::Other("Not more than challenge period"));
                }
            } else {
                log::error!("Unable to subtract to determine if challenge period is satisfied");
                return Err(DispatchError::Other("Unable to subtract to determine if challenge period is satisfied"));
            }
        }
    }
}
