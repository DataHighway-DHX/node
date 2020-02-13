#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_io::hashing::{blake2_128};
use sp_runtime::traits::{Bounded, Member, One, SimpleArithmetic};
use frame_support::traits::{Currency, ExistenceRequirement, Randomness};
/// A runtime module for managing non-fungible tokens
use frame_support::{decl_event, decl_error, dispatch, decl_module, decl_storage, ensure, Parameter, debug};
use system::ensure_signed;
use sp-std::prelude::*; // Imports Vec

// FIXME - remove this, only used this approach since do not know how to use BalanceOf using only mining-speed-boosts runtime module
use roaming_operators;

/// The module's configuration trait.
pub trait Trait: system::Trait + roaming_operators::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type MiningSpeedBoostConfigurationTokenMiningIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    // Mining Speed Boost Token Mining Config
    type MiningSpeedBoostConfigurationTokenMiningTokenType: Parameter + Member + Default;
    type MiningSpeedBoostConfigurationTokenMiningTokenLockedAmount: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    // // Mining Speed Boost Eligibility
    // type MiningSpeedBoostEligibilityCalculatedEligibility: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    // type MiningSpeedBoostEligibilityTokenLockedPercentage: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    // type MiningSpeedBoostEligibilityHardwareUptimePercentage: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    // type MiningSpeedBoostEligibilityDateAudited: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
    // type MiningSpeedBoostEligibilityAuditorAccountID: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;

}

type BalanceOf<T> = <<T as roaming_operators::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MiningSpeedBoostConfigurationTokenMining(pub [u8; 16]);

// #[cfg_attr(feature = "std", derive(Debug))]
// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// pub struct MiningSpeedBoostRates<U, V, W, X, Y> {
//     pub token_mxc: U,
//     pub token_iota: V,
//     pub token_dot: W,
//     pub hardware_secure: X,
//     pub hardware_insecure: Y,
// }

// #[cfg_attr(feature = "std", derive(Debug))]
// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// pub struct MiningSpeedBoostRatesMax<U, V, W, X> {
//     pub token: U,
//     pub hardware: V,
//     pub loyalty: W,
//     pub combination: X,
// }

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct MiningSpeedBoostConfigurationTokenMiningTokenConfig<U, V, W, X, Y> {
    pub token_type: U,
    pub token_locked_amount: V,
    pub token_lock_period: W,
    pub token_lock_period_start_date: X,
    pub token_lock_period_end_date: Y,
}

// #[cfg_attr(feature = "std", derive(Debug))]
// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// pub struct MiningSpeedBoostConfigurationHardwareMiningHardwareConfig<U, V, W, X, Y, Z> {
//     pub hardware_secure: U,
//     pub hardware_type: V,
//     pub hardware_id: W,
//     pub hardware_dev_eui: X,
//     pub hardware_lock_period_start_date: Y,
//     pub hardware_lock_period_end_date: Z,
// }

// #[cfg_attr(feature = "std", derive(Debug))]
// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// pub struct MiningSpeedBoostSample<U, V> {
//     pub random_sample_date: U,
//     pub random_sample_tokens_locked: V,
// }

// // TODO - Configure Auditing of Eligibility
// #[cfg_attr(feature = "std", derive(Debug))]
// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// pub struct MiningSpeedBoostEligibilityResult<U, V, W> {
//     pub eligibility_calculated_eligibility: U,
//     pub eligibility_token_locked_percentage: V,
//     pub eligibility_hardware_uptime_percentage: W,
//     // pub eligibility_date_audited: X,
//     // pub eligibility_auditor_account_id: Y,
// }

// #[cfg_attr(feature = "std", derive(Debug))]
// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// pub struct MiningSpeedBoostClaim<U, V> {
//     pub reward_amount: U,
//     pub reward_date_redeemed: V,
// }

decl_event!(
    pub enum Event<T> where
        <T as system::Trait>::AccountId,
        <T as Trait>::MiningSpeedBoostConfigurationTokenMiningIndex,
        <T as Trait>::MiningSpeedBoostConfigurationTokenMiningTokenType,
        <T as Trait>::MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod,
        <T as Trait>::MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate,
        <T as Trait>::MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate,
        Balance = BalanceOf<T>,
    {
        /// A mining_speed_boosts_configuration_token_mining is created. (owner, mining_speed_boosts_configuration_token_mining_id)
        Created(AccountId, MiningSpeedBoostConfigurationTokenMiningIndex),
        /// A mining_speed_boosts_configuration_token_mining is transferred. (from, to, mining_speed_boosts_configuration_token_mining_id)
        Transferred(AccountId, AccountId, MiningSpeedBoostConfigurationTokenMiningIndex),
        MiningSpeedBoostConfigurationTokenMiningTokenConfigSet(
            AccountId, MiningSpeedBoostConfigurationTokenMiningIndex, MiningSpeedBoostConfigurationTokenMiningTokenType, Balance,
            MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod, MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate,
            MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate
        ),
        // SampleSet(
        //     AccountId, MiningSpeedBoostOracleIndex, MiningSpeedBoostSampleHash, MiningSpeedBoostSampleDate
        // ),
        // EligibilitySet(
        //     AccountId, MiningSpeedBoostEligibilityTokenMiningIndex, MiningSpeedBoostEligibilityCalculatedEligibility 
        // ),
        // RewardSet(
        //     AccountId, MiningSpeedBoostClaimIndex, MiningSpeedBoostClaimHash, MiningSpeedBoostClaimAmount, MiningSpeedBoostClaimDateRedeemed
        // )
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MiningSpeedBoostConfigurationTokenMining {
        /// Stores all the mining_speed_boosts_configuration_token_minings, key is the mining_speed_boosts_configuration_token_mining id / index
        pub MiningSpeedBoostConfigurationTokenMinings get(fn mining_speed_boosts_configuration_token_mining): map T::MiningSpeedBoostConfigurationTokenMiningIndex => Option<MiningSpeedBoostConfigurationTokenMining>;

        /// Stores the total number of mining_speed_boosts_configuration_token_minings. i.e. the next mining_speed_boosts_configuration_token_mining index
        pub MiningSpeedBoostConfigurationTokenMiningCount get(fn mining_speed_boosts_configuration_token_mining_count): T::MiningSpeedBoostConfigurationTokenMiningIndex;

        /// Stores mining_speed_boosts_configuration_token_mining owner
        pub MiningSpeedBoostConfigurationTokenMiningOwners get(fn mining_speed_boosts_configuration_token_mining_owner): map T::MiningSpeedBoostConfigurationTokenMiningIndex => Option<T::AccountId>;

        /// Stores mining_speed_boosts_configuration_token_mining_token_config
        pub MiningSpeedBoostConfigurationTokenMiningTokenConfigs get(fn mining_speed_boosts_configuration_token_mining_token_configs): map T::MiningSpeedBoostConfigurationTokenMiningIndex =>
            Option<MiningSpeedBoostConfigurationTokenMiningTokenConfig<T::MiningSpeedBoostConfigurationTokenMiningTokenType, BalanceOf<T>, T::MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod,
                T::MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate, T::MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate>>;

        // /// Stores mining_speed_boosts_random_samples
        // pub MiningSpeedBoostSamples get(fn mining_speed_boosts_random_sample): map (T::MiningSpeedBoostOracleIndex, T::MiningSpeedBoostSampleHash) =>
        //     Option<MiningSpeedBoostSample<T::MiningSpeedBoostSampleDate, T::MiningSpeedBoostSampleTokensLocked>>;

        // /// Stores mining_speed_boosts_random_eligibility
        // pub MiningSpeedBoostEligibility get(fn mining_speed_boosts_eligibility): map T::MiningSpeedBoostEligibilityTokenMiningIndex =>
        //     Option<MiningSpeedBoostEligibilityResult<
        //         T::MiningSpeedBoostEligibilityCalculatedEligibility, T::MiningSpeedBoostEligibilityTokenLockedPercentage, T::MiningSpeedBoostEligibilityHardwareUptimePercentage
        //     >>;
        // }

        // /// Stores mining_speed_boosts_claim
        // pub MiningSpeedBoostClaim get(fn mining_speed_boosts_claim): map (T::MiningSpeedBoostClaimIndex, T::MiningSpeedBoostClaimHash) =>
        //     Option<MiningSpeedBoostClaim<
        //         T::MiningSpeedBoostClaimHash, T::MiningSpeedBoostClaimAmount, T::MiningSpeedBoostClaimDateRedeemed
        //     >>;
        // }
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new mining mining_speed_boosts_configuration_token_mining
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let mining_speed_boosts_configuration_token_mining_id = Self::next_mining_speed_boosts_configuration_token_mining_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store mining_speed_boosts_configuration_token_mining
            let mining_speed_boosts_configuration_token_mining = MiningSpeedBoostConfigurationTokenMining(unique_id);
            Self::insert_mining_speed_boosts_configuration_token_mining(&sender, mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_configuration_token_mining);

            Self::deposit_event(RawEvent::Created(sender, mining_speed_boosts_configuration_token_mining_id));
        }

        /// Transfer a mining_speed_boosts_configuration_token_mining to new owner
        pub fn transfer(origin, to: T::AccountId, mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::mining_speed_boosts_configuration_token_mining_owner(mining_speed_boosts_configuration_token_mining_id) == Some(sender.clone()), "Only owner can transfer mining mining_speed_boosts_configuration_token_mining");

            Self::update_owner(&to, mining_speed_boosts_configuration_token_mining_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, mining_speed_boosts_configuration_token_mining_id));
        }

        /// Set mining_speed_boosts_configuration_token_mining_token_config
        pub fn set_mining_speed_boosts_configuration_token_mining_token_config(
            origin,
            mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex,
            _token_type: Option<T::MiningSpeedBoostConfigurationTokenMiningTokenType>,
            _token_locked_amount: Option<BalanceOf<T>>,
            _token_lock_period: Option<T::MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod>,
            _token_lock_period_start_date: Option<T::MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate>,
            _token_lock_period_end_date: Option<T::MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate>,
        ) {
            let sender = ensure_signed(origin)?;

            // Ensure that the mining_speed_boosts_configuration_token_mining_id whose config we want to change actually exists
            let is_mining_speed_boosts_configuration_token_mining = Self::exists_mining_speed_boosts_configuration_token_mining(mining_speed_boosts_configuration_token_mining_id).is_ok();
            ensure!(is_mining_speed_boosts_configuration_token_mining, "MiningSpeedBoostConfigurationTokenMining does not exist");

            // Ensure that the caller is owner of the mining_speed_boosts_configuration_token_mining_token_config they are trying to change
            ensure!(Self::mining_speed_boosts_configuration_token_mining_owner(mining_speed_boosts_configuration_token_mining_id) == Some(sender.clone()), "Only owner can set mining_speed_boosts_configuration_token_mining_token_config");

            let token_type = match _token_type.clone() {
                Some(value) => value,
                None => Default::default() // Default
            };
            let token_locked_amount = match _token_locked_amount {
                Some(value) => value,
                None => 0.into() // Default
            };
            let token_lock_period = match _token_lock_period {
                Some(value) => value,
                None => 3.into() // Default
            };
            let token_lock_period_start_date = match _token_lock_period_start_date {
                Some(value) => value,
                None => Default::default() // Default
            };
            let token_lock_period_end_date = match _token_lock_period_end_date {
                Some(value) => value,
                None => Default::default() // Default
            };

            // Check if a mining_speed_boosts_configuration_token_mining_token_config already exists with the given mining_speed_boosts_configuration_token_mining_id
            // to determine whether to insert new or mutate existing.
            if Self::has_value_for_mining_speed_boosts_configuration_token_mining_token_config_index(mining_speed_boosts_configuration_token_mining_id).is_ok() {
                debug::info!("Mutating values");
                <MiningSpeedBoostConfigurationTokenMiningTokenConfigs<T>>::mutate(mining_speed_boosts_configuration_token_mining_id, |mining_speed_boosts_configuration_token_mining_token_config| {
                    if let Some(_mining_speed_boosts_configuration_token_mining_token_config) = mining_speed_boosts_configuration_token_mining_token_config {
                        // Only update the value of a key in a KV pair if the corresponding parameter value has been provided
                        _mining_speed_boosts_configuration_token_mining_token_config.token_type = token_type.clone();
                        _mining_speed_boosts_configuration_token_mining_token_config.token_locked_amount = token_locked_amount.clone();
                        _mining_speed_boosts_configuration_token_mining_token_config.token_lock_period = token_lock_period.clone();
                        _mining_speed_boosts_configuration_token_mining_token_config.token_lock_period_start_date = token_lock_period_start_date.clone();
                        _mining_speed_boosts_configuration_token_mining_token_config.token_lock_period_end_date = token_lock_period_end_date.clone();
                    }
                });
                debug::info!("Checking mutated values");
                let fetched_mining_speed_boosts_configuration_token_mining_token_config = <MiningSpeedBoostConfigurationTokenMiningTokenConfigs<T>>::get(mining_speed_boosts_configuration_token_mining_id);
                if let Some(_mining_speed_boosts_configuration_token_mining_token_config) = fetched_mining_speed_boosts_configuration_token_mining_token_config {
                    debug::info!("Latest field token_type {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_type);
                    debug::info!("Latest field token_locked_amount {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_locked_amount);
                    debug::info!("Latest field token_lock_period {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_lock_period);
                    debug::info!("Latest field token_lock_period_start_date {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_lock_period_start_date);
                    debug::info!("Latest field token_lock_period_end_date {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_lock_period_end_date);
                }
            } else {
                debug::info!("Inserting values");

                // Create a new mining mining_speed_boosts_configuration_token_mining_token_config instance with the input params
                let mining_speed_boosts_configuration_token_mining_token_config_instance = MiningSpeedBoostConfigurationTokenMiningTokenConfig {
                    // Since each parameter passed into the function is optional (i.e. `Option`)
                    // we will assign a default value if a parameter value is not provided.
                    token_type: token_type.clone(),
                    token_locked_amount: token_locked_amount.clone(),
                    token_lock_period: token_lock_period.clone(),
                    token_lock_period_start_date: token_lock_period_start_date.clone(),
                    token_lock_period_end_date: token_lock_period_end_date.clone()
                };

                <MiningSpeedBoostConfigurationTokenMiningTokenConfigs<T>>::insert(
                    mining_speed_boosts_configuration_token_mining_id,
                    &mining_speed_boosts_configuration_token_mining_token_config_instance
                );

                debug::info!("Checking inserted values");
                let fetched_mining_speed_boosts_configuration_token_mining_token_config = <MiningSpeedBoostConfigurationTokenMiningTokenConfigs<T>>::get(mining_speed_boosts_configuration_token_mining_id);
                if let Some(_mining_speed_boosts_configuration_token_mining_token_config) = fetched_mining_speed_boosts_configuration_token_mining_token_config {
                    debug::info!("Inserted field token_type {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_type);
                    debug::info!("Inserted field token_locked_amount {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_locked_amount);
                    debug::info!("Inserted field token_lock_period {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_lock_period);
                    debug::info!("Inserted field token_lock_period_start_date {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_lock_period_start_date);
                    debug::info!("Inserted field token_lock_period_end_date {:#?}", _mining_speed_boosts_configuration_token_mining_token_config.token_lock_period_end_date);
                }
            }

            Self::deposit_event(RawEvent::MiningSpeedBoostConfigurationTokenMiningTokenConfigSet(
                sender,
                mining_speed_boosts_configuration_token_mining_id,
                token_type,
                token_locked_amount,
                token_lock_period,
                token_lock_period_start_date,
                token_lock_period_end_date
            ));
        }
    }
}

impl<T: Trait> Module<T> {
	pub fn is_mining_speed_boosts_configuration_token_mining_owner(mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex, sender: T::AccountId) -> Result<(), &'static str> {
        ensure!(
            Self::mining_speed_boosts_configuration_token_mining_owner(&mining_speed_boosts_configuration_token_mining_id)
                .map(|owner| owner == sender)
                .unwrap_or(false),
            "Sender is not owner of MiningSpeedBoost"
        );
        Ok(())
    }

    pub fn exists_mining_speed_boosts_configuration_token_mining(mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex) -> Result<MiningSpeedBoostConfigurationTokenMining, &'static str> {
        match Self::mining_speed_boosts_configuration_token_mining(mining_speed_boosts_configuration_token_mining_id) {
            Some(value) => Ok(value),
            None => Err("MiningSpeedBoostConfigurationTokenMining does not exist")
        }
    }

    pub fn exists_mining_speed_boosts_configuration_token_mining_token_config(mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex) -> Result<(), &'static str> {
        match Self::mining_speed_boosts_configuration_token_mining_token_configs(mining_speed_boosts_configuration_token_mining_id) {
            Some(value) => Ok(()),
            None => Err("MiningSpeedBoostConfigurationTokenMiningTokenConfig does not exist")
        }
    }

    pub fn has_value_for_mining_speed_boosts_configuration_token_mining_token_config_index(mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex)
        -> Result<(), &'static str> {
        debug::info!("Checking if mining_speed_boosts_configuration_token_mining_token_config has a value that is defined");
        let fetched_mining_speed_boosts_configuration_token_mining_token_config = <MiningSpeedBoostConfigurationTokenMiningTokenConfigs<T>>::get(mining_speed_boosts_configuration_token_mining_id);
        if let Some(value) = fetched_mining_speed_boosts_configuration_token_mining_token_config {
            debug::info!("Found value for mining_speed_boosts_configuration_token_mining_token_config");
            return Ok(());
        }
        debug::info!("No value for mining_speed_boosts_configuration_token_mining_token_config");
        Err("No value for mining_speed_boosts_configuration_token_mining_token_config")
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <system::Module<T>>::extrinsic_index(),
            <system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_mining_speed_boosts_configuration_token_mining_id() -> Result<T::MiningSpeedBoostConfigurationTokenMiningIndex, &'static str> {
        let mining_speed_boosts_configuration_token_mining_id = Self::mining_speed_boosts_configuration_token_mining_count();
        if mining_speed_boosts_configuration_token_mining_id == <T::MiningSpeedBoostConfigurationTokenMiningIndex as Bounded>::max_value() {
            return Err("MiningSpeedBoostConfigurationTokenMining count overflow");
        }
        Ok(mining_speed_boosts_configuration_token_mining_id)
    }

    fn insert_mining_speed_boosts_configuration_token_mining(owner: &T::AccountId, mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex, mining_speed_boosts_configuration_token_mining: MiningSpeedBoostConfigurationTokenMining) {
        // Create and store mining mining_speed_boosts_configuration_token_mining
        <MiningSpeedBoostConfigurationTokenMinings<T>>::insert(mining_speed_boosts_configuration_token_mining_id, mining_speed_boosts_configuration_token_mining);
        <MiningSpeedBoostConfigurationTokenMiningCount<T>>::put(mining_speed_boosts_configuration_token_mining_id + One::one());
        <MiningSpeedBoostConfigurationTokenMiningOwners<T>>::insert(mining_speed_boosts_configuration_token_mining_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, mining_speed_boosts_configuration_token_mining_id: T::MiningSpeedBoostConfigurationTokenMiningIndex) {
        <MiningSpeedBoostConfigurationTokenMiningOwners<T>>::insert(mining_speed_boosts_configuration_token_mining_id, to);
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

	use sp_core::H256;
	use frame_support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ();
        type TransferFee = ();
        type CreationFee = ();
    }
    impl transaction_payment::Trait for Test {
        type Currency = Balances;
        type OnTransactionPayment = ();
        type TransactionBaseFee = ();
        type TransactionByteFee = ();
        type WeightToFee = ();
        type FeeMultiplierUpdate = ();
    }
    // FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
    impl roaming_operators::Trait for Test {
        type Event = ();
        type Currency = Balances;
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl Trait for Test {
        type Event = ();
        type MiningSpeedBoostConfigurationTokenMiningIndex = u64;
        // Mining Speed Boost Token Mining Config
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningSpeedBoostConfigurationTokenMiningTokenType = Vec<u8>;
        // type MiningSpeedBoostConfigurationTokenMiningTokenType = MiningSpeedBoostConfigurationTokenMiningTokenTypes;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockedAmount = u64;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod = u32;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate = u64;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate = u64;
    }
    //type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type MiningSpeedBoostConfigurationTokenMiningTestModule = Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }
}
