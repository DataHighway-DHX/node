// extern crate env as env;
extern crate membership_supernodes as membership_supernodes;
extern crate mining_claims_token as mining_claims_token;
extern crate mining_config_token as mining_config_token;
extern crate mining_eligibility_proxy as mining_eligibility_proxy;
extern crate mining_eligibility_token as mining_eligibility_token;
extern crate mining_execution_token as mining_execution_token;
extern crate mining_rates_token as mining_rates_token;
extern crate mining_sampling_token as mining_sampling_token;
extern crate roaming_operators as roaming_operators;

const INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE: u64 = 30000000;

#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_err,
        assert_ok,
        impl_outer_origin,
        parameter_types,
        traits::{
            Contains,
            ContainsLengthBound,
            Currency,
            EnsureOrigin,
        },
        weights::{
            IdentityFee,
            Weight,
        },
    };
    use frame_system::{
        EnsureRoot,
        RawOrigin,
    };
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{
            BlakeTwo256,
            IdentityLookup,
            Zero,
        },
        DispatchError,
        DispatchResult,
        ModuleId,
        Perbill,
        Percent,
        Permill,
    };
    use std::cell::RefCell;
    // Import Trait for each runtime module being tested
    use chrono::NaiveDate;
    use datahighway_runtime::{
        AccountId,
        Babe,
        Balance,
        BlockNumber,
        Moment,
        DAYS,
        SLOT_DURATION,
    };
    use membership_supernodes::{
        Module as MembershipSupernodesModule,
        Trait as MembershipSupernodesTrait,
    };
    use mining_claims_token::{
        MiningClaimsTokenClaimResult,
        Module as MiningClaimsTokenModule,
        Trait as MiningClaimsTokenTrait,
    };
    use mining_config_token::{
        MiningConfigTokenConfig,
        MiningConfigTokenRequirementsConfig,
        Module as MiningConfigTokenModule,
        Trait as MiningConfigTokenTrait,
    };
    use mining_eligibility_proxy::{
        Event as MiningEligibilityProxyEvent,
        MiningEligibilityProxyClaimRewardeeData,
        MiningEligibilityProxyRewardRequest,
        Module as MiningEligibilityProxyModule,
        RewardDailyData,
        RewardRequestorData,
        RewardTransferData,
        Trait as MiningEligibilityProxyTrait,
    };
    use mining_eligibility_token::{
        MiningEligibilityTokenResult,
        Module as MiningEligibilityTokenModule,
        Trait as MiningEligibilityTokenTrait,
    };
    use mining_execution_token::{
        MiningExecutionTokenExecutionResult,
        Module as MiningExecutionTokenModule,
        Trait as MiningExecutionTokenTrait,
    };
    use mining_rates_token::{
        MiningRatesTokenConfig,
        Module as MiningRatesTokenModule,
        Trait as MiningRatesTokenTrait,
    };
    use mining_sampling_token::{
        MiningSamplingTokenConfig,
        Module as MiningSamplingTokenModule,
        Trait as MiningSamplingTokenTrait,
    };
    use roaming_operators;

    // pub fn origin_of(who: &AccountId) -> <Runtime as frame_system::Trait>::Origin {
    // 	<Runtime as frame_system::Trait>::Origin::signed((*who).clone())
    // }

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
    impl frame_system::Trait for Test {
        type AccountData = pallet_balances::AccountData<u64>;
        type AccountId = u64;
        type AvailableBlockRatio = AvailableBlockRatio;
        type BaseCallFilter = ();
        type BlockExecutionWeight = ();
        type BlockHashCount = BlockHashCount;
        type BlockNumber = u64;
        type Call = ();
        type DbWeight = ();
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type ExtrinsicBaseWeight = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Header = Header;
        type Index = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type MaximumBlockLength = MaximumBlockLength;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumExtrinsicWeight = MaximumBlockWeight;
        type OnKilledAccount = ();
        type OnNewAccount = ();
        type Origin = Origin;
        type PalletInfo = ();
        type SystemWeightInfo = ();
        type Version = ();
    }
    parameter_types! {
        pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
    }
    impl pallet_timestamp::Trait for Test {
        type MinimumPeriod = MinimumPeriod;
        /// A timestamp: milliseconds since the unix epoch.
        type Moment = Moment;
        type OnTimestampSet = Babe;
        type WeightInfo = ();
    }
    parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
    }
    impl pallet_balances::Trait for Test {
        type AccountStore = System;
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type MaxLocks = ();
        type WeightInfo = ();
    }
    impl pallet_transaction_payment::Trait for Test {
        type Currency = Balances;
        type FeeMultiplierUpdate = ();
        type OnTransactionPayment = ();
        type TransactionByteFee = ();
        type WeightToFee = IdentityFee<u64>;
    }

    thread_local! {
        static TEN_TO_FOURTEEN: RefCell<Vec<u64>> = RefCell::new(vec![10,11,12,13,14]);
    }
    pub struct TenToFourteen;
    impl Contains<u64> for TenToFourteen {
        fn sorted_members() -> Vec<u64> {
            TEN_TO_FOURTEEN.with(|v| v.borrow().clone())
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn add(new: &u64) {
            TEN_TO_FOURTEEN.with(|v| {
                let mut members = v.borrow_mut();
                members.push(*new);
                members.sort();
            })
        }
    }
    impl ContainsLengthBound for TenToFourteen {
        fn max_len() -> usize {
            TEN_TO_FOURTEEN.with(|v| v.borrow().len())
        }

        fn min_len() -> usize {
            0
        }
    }

    parameter_types! {
        pub const ProposalBond: Permill = Permill::from_percent(5);
        pub const ProposalBondMinimum: u64 = 1_000_000_000_000_000_000;
        pub const SpendPeriod: BlockNumber = 1 * DAYS;
        pub const Burn: Permill = Permill::from_percent(0);
        pub const TipCountdown: BlockNumber = 1;
        pub const TipFindersFee: Percent = Percent::from_percent(20);
        pub const TipReportDepositBase: u64 = 1_000_000_000_000_000_000;
        pub const MaximumReasonLength: u32 = 16384;
        pub const BountyValueMinimum: u64 = 1;
        pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
        pub const BountyDepositBase: u64 = 80;
        pub const BountyDepositPayoutDelay: u32 = 3;
        pub const BountyUpdatePeriod: u32 = 20;
        pub const DataDepositPerByte: u64 = 1;
        pub const TreasuryModuleId: ModuleId = ModuleId(*b"py/trsry");
    }

    impl pallet_treasury::Trait for Test {
        type ApproveOrigin = EnsureRoot<u64>;
        type BountyCuratorDeposit = BountyCuratorDeposit;
        type BountyDepositBase = BountyDepositBase;
        type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
        type BountyUpdatePeriod = BountyUpdatePeriod;
        type BountyValueMinimum = BountyValueMinimum;
        type Burn = Burn;
        type BurnDestination = ();
        type Currency = Balances;
        type DataDepositPerByte = DataDepositPerByte;
        type Event = ();
        type MaximumReasonLength = MaximumReasonLength;
        type ModuleId = TreasuryModuleId;
        type OnSlash = ();
        type ProposalBond = ProposalBond;
        type ProposalBondMinimum = ProposalBondMinimum;
        type RejectOrigin = EnsureRoot<u64>;
        type SpendPeriod = SpendPeriod;
        type TipCountdown = TipCountdown;
        type TipFindersFee = TipFindersFee;
        type TipReportDepositBase = TipReportDepositBase;
        type Tippers = TenToFourteen;
        type WeightInfo = ();
    }

    // FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
    impl roaming_operators::Trait for Test {
        type Currency = Balances;
        type Event = ();
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl MiningConfigTokenTrait for Test {
        type Event = ();
        // type Currency = Balances;
        // type Randomness = Randomness;
        type MiningConfigTokenIndex = u64;
        type MiningConfigTokenLockAmount = u64;
        // Mining Speed Boost Token Mining Config
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningConfigTokenType = Vec<u8>;
    }
    impl MiningRatesTokenTrait for Test {
        type Event = ();
        type MiningRatesTokenIndex = u64;
        type MiningRatesTokenMaxLoyalty = u32;
        // Mining Speed Boost Max Rates
        type MiningRatesTokenMaxToken = u32;
        type MiningRatesTokenTokenDOT = u32;
        type MiningRatesTokenTokenIOTA = u32;
        // Mining Speed Boost Rate
        type MiningRatesTokenTokenMXC = u32;
    }
    impl MiningSamplingTokenTrait for Test {
        type Event = ();
        type MiningSamplingTokenIndex = u64;
        type MiningSamplingTokenSampleLockedAmount = u64;
    }
    impl MiningEligibilityTokenTrait for Test {
        type Event = ();
        type MiningEligibilityTokenCalculatedEligibility = u64;
        type MiningEligibilityTokenIndex = u64;
        type MiningEligibilityTokenLockedPercentage = u32;
        // type MiningEligibilityTokenAuditorAccountID = u64;
    }
    impl MiningEligibilityProxyTrait for Test {
        type Currency = Balances;
        type Event = ();
        type MembershipSource = MembershipSupernodes;
        type MiningEligibilityProxyIndex = u64;
        type RewardsOfDay = u64;
    }
    impl MiningClaimsTokenTrait for Test {
        type Event = ();
        type MiningClaimsTokenClaimAmount = u64;
        type MiningClaimsTokenIndex = u64;
    }
    impl MiningExecutionTokenTrait for Test {
        type Event = ();
        type MiningExecutionTokenIndex = u64;
    }
    impl MembershipSupernodesTrait for Test {
        type Event = ();
    }

    type System = frame_system::Module<Test>;
    pub type Balances = pallet_balances::Module<Test>;
    pub type Timestamp = pallet_timestamp::Module<Test>;
    pub type Treasury = pallet_treasury::Module<Test>;
    pub type MiningConfigTokenTestModule = MiningConfigTokenModule<Test>;
    pub type MiningRatesTokenTestModule = MiningRatesTokenModule<Test>;
    pub type MiningSamplingTokenTestModule = MiningSamplingTokenModule<Test>;
    pub type MiningEligibilityTokenTestModule = MiningEligibilityTokenModule<Test>;
    pub type MiningEligibilityProxyTestModule = MiningEligibilityProxyModule<Test>;
    pub type MiningClaimsTokenTestModule = MiningClaimsTokenModule<Test>;
    pub type MiningExecutionTokenTestModule = MiningExecutionTokenModule<Test>;
    pub type MembershipSupernodesTestModule = MembershipSupernodesModule<Test>;
    type Randomness = pallet_randomness_collective_flip::Module<Test>;
    type MembershipSupernodes = membership_supernodes::Module<Test>;

    // fn last_event() -> MiningEligibilityProxyEvent {
    //     System::events().pop().expect("Event expected").event
    // }

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    pub fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(0, INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE), (1, 10), (2, 20), (3, 30)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    // Create Users on Data Highway
    #[test]
    fn setup_users() {
        new_test_ext().execute_with(|| {
            assert_eq!(Balances::free_balance(0), INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
            assert_eq!(Balances::free_balance(3), 30);
            assert_eq!(Balances::reserved_balance(&1), 0);
            // FIXME - why can't I query `total_balance` from the Balances frame
            // assert_eq!(Balances::total_balance(&1), 0);
        });
    }

    #[test]
    fn integration_test() {
        new_test_ext().execute_with(|| {
            // env::config::set_test_env();

            // Create Mining Speed Boost Rates Token Mining

            // Call Functions
            assert_ok!(MiningRatesTokenTestModule::create(Origin::signed(0)));
            assert_ok!(MiningRatesTokenTestModule::set_mining_rates_token_rates_config(
                Origin::signed(0),
                0, // mining_rates_token_id
                // FIXME - convert all below types to Vec<u8> since float values? i.e. b"1.025".to_vec()
                Some(1), // token_token_mxc
                Some(1), // token_token_iota
                Some(1), // token_token_dot
                Some(1), // token_max_token
                Some(1), // token_max_loyalty
            ));

            // Verify Storage
            assert_eq!(MiningRatesTokenTestModule::mining_rates_token_count(), 1);
            assert!(MiningRatesTokenTestModule::mining_rates_token(0).is_some());
            assert_eq!(MiningRatesTokenTestModule::mining_rates_token_owner(0), Some(0));
            assert_eq!(
                MiningRatesTokenTestModule::mining_rates_token_rates_configs(0),
                Some(MiningRatesTokenConfig {
                    token_token_mxc: 1,
                    token_token_iota: 1,
                    token_token_dot: 1,
                    token_max_token: 1,
                    token_max_loyalty: 1,
                })
            );

            // Create Mining Speed Boost Configuration & Cooldown Configuration Token Mining

            // Call Functions
            assert_ok!(MiningConfigTokenTestModule::create(Origin::signed(0)));
            assert_ok!(MiningConfigTokenTestModule::set_mining_config_token_token_cooldown_config(
                Origin::signed(0),
                0,                     // mining_token_id
                Some(b"DHX".to_vec()), // token_type
                Some(10),              // token_lock_min_amount
                Some(7),               // token_lock_min_blocks
            ));
            assert_ok!(MiningConfigTokenTestModule::set_mining_config_token_token_config(
                Origin::signed(0),
                0,                     // mining_token_id
                Some(b"MXC".to_vec()), // token_type
                Some(100),             // token_lock_amount
                Some(12345),           // token_lock_start_block
                Some(23456),           // token_lock_interval_blocks
            ));

            // Verify Storage
            assert_eq!(MiningConfigTokenTestModule::mining_config_token_count(), 1);
            assert!(MiningConfigTokenTestModule::mining_config_token(0).is_some());
            assert_eq!(MiningConfigTokenTestModule::mining_config_token_owner(0), Some(0));
            assert_eq!(
                MiningConfigTokenTestModule::mining_config_token_token_cooldown_configs(0),
                Some(MiningConfigTokenRequirementsConfig {
                    token_type: b"DHX".to_vec(), // token_type
                    token_lock_min_amount: 10,   // token_lock_min_amount
                    token_lock_min_blocks: 7,    // token_lock_min_blocks
                })
            );
            assert_eq!(
                MiningConfigTokenTestModule::mining_config_token_token_configs(0),
                Some(MiningConfigTokenConfig {
                    token_type: b"MXC".to_vec(),       // token_type
                    token_lock_amount: 100,            // token_lock_amount
                    token_lock_start_block: 12345,     // token_lock_start_block
                    token_lock_interval_blocks: 23456, // token_lock_interval_blocks
                })
            );

            // Create Mining Speed Boost Sampling Token Mining

            // Call Functions
            assert_ok!(MiningSamplingTokenTestModule::create(Origin::signed(0)));
            assert_ok!(MiningSamplingTokenTestModule::set_mining_samplings_token_samplings_config(
                Origin::signed(0),
                0,           // mining_token_id
                0,           // mining_token_sample_id
                Some(23456), // token_sample_block
                Some(100),   // token_sample_locked_amount
            ));
            assert_ok!(MiningSamplingTokenTestModule::assign_sampling_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningSamplingTokenTestModule::mining_samplings_token_count(), 1);
            assert!(MiningSamplingTokenTestModule::mining_samplings_token(0).is_some());
            assert_eq!(MiningSamplingTokenTestModule::mining_samplings_token_owner(0), Some(0));
            assert_eq!(
                MiningSamplingTokenTestModule::mining_samplings_token_samplings_configs((0, 0)),
                Some(MiningSamplingTokenConfig {
                    token_sample_block: 23456,       // token_sample_block
                    token_sample_locked_amount: 100  // token_sample_locked_amount
                })
            );

            // Create Mining Speed Boost Eligibility Token Mining

            // On random sampling dates an oracle service audits and published logs
            // of how many tokens were locked and stores them in sampling instances
            // using the sampling token mining runtime module, where each sample belongs to a token
            // mining configuration (with start and end date) from the configuration
            // token mining runtime module.
            //
            // TODO - record the account id of the user who runs the oracle service and provides
            // the sampling of the logs.
            //
            // After the configuration's end date, the eligibility token mining runtime module
            // is used to aggregate the samplings that correspond to the configuration
            // and use that to calculate the eligibility of the token owner for receiving rewards.
            // The account id of the an auditor who may be involved in auditing the eligibility
            // outcome may also be recorded.
            // Note that we can find out all the samples associated with a
            // mining_config_token_id

            // Call Functions
            assert_ok!(MiningEligibilityTokenTestModule::create(Origin::signed(0)));
            // assert_eq!(
            //   MiningEligibilityTokenTestModule::calculate_mining_eligibility_token_result(
            //       Origin::signed(0),
            //       0, // mining_config_token_id
            //       0, // mining_eligibility_token_id
            //   ),
            //   Some(
            //     MiningEligibilityTokenResult {
            //       token_calculated_eligibility: 1.1
            //       // to determine eligibility for proportion (incase user hardware is not online around during the
            // whole lock period)       token_locked_percentage: 0.7, // i.e. 70%
            //       // token_block_audited: 123,
            //       // token_auditor_account_id: 123
            //     }
            //   )
            // ))

            // Override by DAO if necessary
            assert_ok!(MiningEligibilityTokenTestModule::set_mining_eligibility_token_eligibility_result(
                Origin::signed(0),
                0,       // mining_config_token_id
                0,       // mining_eligibility_token_id
                Some(1), // mining_token_calculated_eligibility
                Some(1), /* mining_token_locked_percentage
                          * 123, // mining_token_block_audited
                          * 123, // mining_token_auditor_account_id
                          * Some({
                          *   MiningEligibilityTokenResult {
                          *     token_calculated_eligibility: 1, // i.e. 1.1
                          *     // to determine eligibility for proportion (incase user hardware is not online
                          * around during the whole lock period)
                          *     token_locked_percentage: 1, // i.e. 0.7 for 70%
                          *     // token_block_audited: 123,
                          *     // token_auditor_account_id: 123
                          *   }
                          * }), */
            ));
            assert_ok!(MiningEligibilityTokenTestModule::assign_eligibility_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningEligibilityTokenTestModule::mining_eligibility_token_count(), 1);
            assert!(MiningEligibilityTokenTestModule::mining_eligibility_token(0).is_some());
            assert_eq!(MiningEligibilityTokenTestModule::mining_eligibility_token_owner(0), Some(0));
            assert_eq!(
                MiningEligibilityTokenTestModule::mining_eligibility_token_eligibility_results((0, 0)),
                Some(MiningEligibilityTokenResult {
                    token_calculated_eligibility: 1,
                    // to determine eligibility for proportion (incase user hardware is not online around during the
                    // whole lock period)
                    token_locked_percentage: 1, /* i.e. 70%
                                                 * token_block_audited: 123,
                                                 * token_auditor_account_id: 123 */
                })
            );

            // Create Mining Speed Boost Claims Token Mining

            // Call Functions
            assert_ok!(MiningClaimsTokenTestModule::create(Origin::signed(0)));
            assert_ok!(MiningClaimsTokenTestModule::assign_claim_to_configuration(Origin::signed(0), 0, 0));
            assert_ok!(MiningClaimsTokenTestModule::claim(
                Origin::signed(0),
                0, // mining_config_token_id
                0, // mining_eligibility_token_id
                0, // mining_claims_token_id
            ));
            // Override by DAO if necessary
            assert_ok!(MiningClaimsTokenTestModule::set_mining_claims_token_claims_result(
                Origin::signed(0),
                0,           // mining_config_token_id
                0,           // mining_eligibility_token_id
                0,           // mining_claims_token_id
                Some(1),     // token_claim_amount
                Some(34567), // token_claim_block_redeemed
            ));

            // Verify Storage
            assert_eq!(MiningClaimsTokenTestModule::mining_claims_token_count(), 1);
            assert!(MiningClaimsTokenTestModule::mining_claims_token(0).is_some());
            assert_eq!(MiningClaimsTokenTestModule::mining_claims_token_owner(0), Some(0));
            assert_eq!(
                MiningClaimsTokenTestModule::mining_claims_token_claims_results((0, 0)),
                Some(MiningClaimsTokenClaimResult {
                    token_claim_amount: 1,
                    token_claim_block_redeemed: 34567,
                })
            );

            // Create Mining Speed Boost Execution Token Mining

            // Call Functions
            assert_ok!(MiningExecutionTokenTestModule::create(Origin::signed(0)));
            assert_ok!(MiningExecutionTokenTestModule::assign_execution_to_configuration(Origin::signed(0), 0, 0));

            // Override by DAO if necessary
            //
            // Execute is called to start the mining if all checks pass
            assert_ok!(MiningExecutionTokenTestModule::set_mining_execution_token_execution_result(
                Origin::signed(0),
                0,           // mining_config_token_id
                0,           // mining_execution_token_id
                Some(12345), // token_execution_started_block
                Some(34567)  // token_execution_ended_block
            ));

            // Verify Storage
            assert_eq!(MiningExecutionTokenTestModule::mining_execution_token_count(), 1);
            assert!(MiningExecutionTokenTestModule::mining_execution_token(0).is_some());
            assert_eq!(MiningExecutionTokenTestModule::mining_execution_token_owner(0), Some(0));
            assert_eq!(
                MiningExecutionTokenTestModule::mining_execution_token_execution_results((0, 0)),
                Some(MiningExecutionTokenExecutionResult {
                    token_execution_executor_account_id: 0,
                    token_execution_started_block: 12345,
                    token_execution_ended_block: 34567,
                })
            );
            // TODO - check that the locked amount has actually been locked and check that a sampling, eligibility, and
            // claim were all run automatically afterwards assert!(false);

            // Mining Eligibility Proxy Tests
            //
            // A member of the Supernode Centre requests as a proxy on behalf of one or more Supernodes and their users
            // a claim for an amount of DHX tokens to be sent to them from the Treasury's DHX DAO unlocked reserves.
            // A separate endpoint will be added for Supernodes make requests themselves.
            // They provide data about the various user accounts belonging to that Supernode are to be
            // rewarded for their mining participation from a start block until an interval of blocks later,
            // and accounts that are to be rewarded must have completed their cooldown period after they started to
            // mine, and must not be in a cooldown period after requesting to stop mining.
            // Only the Root account may add and remove any account's membership of Member Supernodes, but in
            // practice only the Supernode Centre's account would be added with membership of Member Supernodes.
            //
            // Important note: Due to limitations with Substrate 2, the Treasury DHX DAO account
            // needs to manually be transferred the DHX tokens by the Sudo root user first, since instantiation at
            // genesis is only available in Substrate 3.

            // The implementation uses ensure_root, so only the Sudo root origin may add and remove members
            // (not account 0 or 1) of Member Supernodes
            assert_ok!(MembershipSupernodesTestModule::add_member(Origin::root(), 1, 1));
            assert_err!(MembershipSupernodesTestModule::add_member(Origin::signed(0), 1, 1), DispatchError::BadOrigin);

            let rewardee_data = MiningEligibilityProxyClaimRewardeeData {
                proxy_claim_rewardee_account_id: 3,
                proxy_claim_reward_amount: 1000,
                proxy_claim_start_date: 946681200000u64, // 1.1.2000
                proxy_claim_end_date: 947113200000u64, // 6.1.2000
            };
            let mut proxy_claim_rewardees_data: Vec<MiningEligibilityProxyClaimRewardeeData<u64, u64, u64, u64>> =
                Vec::new();
            proxy_claim_rewardees_data.push(rewardee_data);

            System::set_block_number(1);

            // 26th March 2021 @ ~2am is 1616724600000
            // where milliseconds/day         86400000
            Timestamp::set_timestamp(1616724600000u64);

            // Check balance of account Supernode Centre's proxy_claim_rewardee_account_id prior
            // to treasury rewarding it.
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::reserved_balance(1), 0);
            assert_eq!(Balances::total_balance(&1), 10);
            // Check balance of temporary treasury prior to paying the treasury.
            assert_eq!(Balances::usable_balance(0), INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
            assert_eq!(Balances::free_balance(0), INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);
            assert_eq!(Balances::reserved_balance(0), 0);
            assert_eq!(Balances::total_balance(&0), INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);

            // let _ = Balances::deposit_creating(&0, 30000);
            // Balances::make_free_balance_be(&Treasury::account_id(),
            // INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE);

            // Sudo transfers the temporary treasury DHX DAO reserves to the treasury after the genesis block
            // This is necessary because instantiable transfers to treasury in the genesis config are
            // only available in Substrate 3, but we are using Substrate 2 still.
            // origin, source, destination, balance
            assert_ok!(Balances::force_transfer(
                RawOrigin::Root.into(),
                0,
                Treasury::account_id(),
                INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE
            ));

            // Check the balance of the treasury has received the funds from the temporary account
            // to be use to pay the proxy_claim_reward_amount to proxy_claim_rewardee_account_id
            assert_eq!(
                Balances::free_balance(&Treasury::account_id()),
                INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE
            );
            // ::pot() is a private function in Substrate 2 but in Substrate 3 it is public
            // so currently we cannot use this until we upgrade to Substrate 3
            // assert_eq!(Treasury::pot(), 50000);

            // This will generate mining_eligibility_proxy_id 0
            assert_ok!(MiningEligibilityProxyTestModule::proxy_eligibility_claim(
                Origin::signed(1),
                1000, // _proxy_claim_total_reward_amount
                Some(proxy_claim_rewardees_data.clone()),
            ));

            // FIXME #20210312 - unable to get this to work or find help
            // https://matrix.to/#/!HzySYSaIhtyWrwiwEV:matrix.org/$1615538012148183moxRT:matrix.org?via=matrix.parity.io&via=matrix.org&via=corepaper.org
            // Check balance of account proxy_claim_rewardee_account_id after treasury rewards it.
            // assert_eq!(
            //     last_event(),
            //     MiningEligibilityProxyEvent::MiningEligibilityProxyRewardRequestSet(
            //         1u64, // proxy_claim_requestor_account_id
            //         0u64, // mining_eligibility_proxy_id
            //         1000u64, // proxy_claim_total_reward_amount
            //         proxy_claim_rewardees_data.clone(), // proxy_claim_rewardees_data
            //         1u64, // proxy_claim_block_redeemed
            //         1u64, // proxy_claim_timestamp_redeemed
            //     ),
            // );

            System::set_block_number(2);

            // 27th March 2021 @ ~2am is 1616811000000u64
            // https://currentmillis.com/
            Timestamp::set_timestamp(1616811000000u64);

            assert_eq!(Balances::free_balance(1), 1010);
            assert_eq!(Balances::reserved_balance(1), 0);
            assert_eq!(Balances::total_balance(&1), 1010);
            // Check balance of treasury after paying the proxy_claim_reward_amount.
            assert_eq!(
                Balances::free_balance(Treasury::account_id()),
                (INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE - 1000)
            );
            assert_eq!(Balances::reserved_balance(Treasury::account_id()), 0);
            assert_eq!(
                Balances::total_balance(&Treasury::account_id()),
                (INITIAL_DHX_DAO_TREASURY_UNLOCKED_RESERVES_BALANCE - 1000)
            );

            assert_ok!(MembershipSupernodesTestModule::remove_member(Origin::root(), 1, 1));
            assert_err!(
                MembershipSupernodesTestModule::remove_member(Origin::signed(0), 1, 1),
                DispatchError::BadOrigin
            );

            // This tries to generate mining_eligibility_proxy_id 0
            // assert_err!(
            //     MiningEligibilityProxyTestModule::proxy_eligibility_claim(
            //         Origin::signed(0),
            //         1000, // _proxy_claim_total_reward_amount
            //         // Some(proxy_claim_rewardees_data.clone()),
            //     ),
            //     "Only whitelisted Supernode account members may request proxy rewards"
            // );

            // Verify Storage
            assert_eq!(MiningEligibilityProxyTestModule::mining_eligibility_proxy_count(), 1);
            assert!(MiningEligibilityProxyTestModule::mining_eligibility_proxy(0).is_some());
            assert_eq!(MiningEligibilityProxyTestModule::mining_eligibility_proxy_owner(0), Some(1));

            // Check that data about the proxy claim and rewardee data has been stored.
            assert_eq!(
                MiningEligibilityProxyTestModule::mining_eligibility_proxy_eligibility_reward_requests(0),
                Some(MiningEligibilityProxyRewardRequest {
                    proxy_claim_requestor_account_id: 1u64,
                    proxy_claim_total_reward_amount: 1000u64,
                    proxy_claim_rewardees_data: proxy_claim_rewardees_data.clone(),
                    proxy_claim_timestamp_redeemed: 1616724600000u64, // current timestamp
                })
            );

            if let Some(reward_requestor_data) = MiningEligibilityProxyTestModule::reward_requestors(1) {
                // Check that data about the proxy claim reward requestor data has been stored.
                // Check latest request added to vector for requestor AccountId 0
                assert_eq!(
                    reward_requestor_data.clone().pop(),
                    Some(RewardRequestorData {
                        mining_eligibility_proxy_id: 0u64,
                        total_amt: 1000u64,
                        rewardee_count: 1u64,
                        member_kind: 1u32,
                        requested_date: 1616724600000u64,
                    })
                );
            } else {
                assert_eq!(false, true);
            }

            if let Some(reward_transfer_data) = MiningEligibilityProxyTestModule::reward_transfers(1u64) {
                // Check that data about the proxy claim reward transfer data has been stored.
                // Check latest transfer added to vector for transfer AccountId 0
                assert_eq!(
                    reward_transfer_data.clone().pop(),
                    Some(RewardTransferData {
                        mining_eligibility_proxy_id: 0u64,
                        is_sent: true,
                        total_amt: 1000u64,
                        rewardee_count: 1u64,
                        member_kind: 1u32,
                        requested_date: 1616724600000u64,
                    })
                );
            } else {
                assert_eq!(false, true);
            }

            // Add AccountId 2 to member list
            assert_ok!(MembershipSupernodesTestModule::add_member(Origin::root(), 2, 1));

            // Repeat with an additional claim
            assert_ok!(MiningEligibilityProxyTestModule::proxy_eligibility_claim(
                Origin::signed(2),
                3000, // _proxy_claim_total_reward_amount
                Some(proxy_claim_rewardees_data.clone()),
            ));

            if let Some(rewards_daily_data) = MiningEligibilityProxyTestModule::rewards_daily(
                // NaiveDate::from_ymd(2021, 03, 27).and_hms(0, 0, 0).timestamp(),
                1616811000000u64, // 27.3.2021 @ 2am

            ) {
                // Check that data about the proxy claim reward daily data has been stored.
                // Check latest transfer added to vector for requestor AccountId 0
                assert_eq!(
                    rewards_daily_data.clone().pop(),
                    // TODO - instead of using `RewardDailyData` from the implementation, consider
                    // creating a mock of it instead and decorate it with `Debug` and so forth
                    // like in the implementation. It doesn't cause any errors at the moment
                    // because `RewardDailyData` only uses generics in the implementation,
                    // but if it was defined with specific types then it would generate errors
                    Some(RewardDailyData {
                        mining_eligibility_proxy_id: 1u64,
                        total_amt: 3000u64,
                        proxy_claim_requestor_account_id: 2u64,
                        member_kind: 1u32,
                        // rewarded_date: NaiveDate::from_ymd(2021, 03, 27).and_hms(0, 0, 0).timestamp(),
                        rewarded_date: 1616811000000u64, // 27.3.2021 @ 2am
                    })
                );
            } else {
                assert_eq!(false, true);
            }

            // let unused_timstamp =  "2001-03-27".as_bytes().to_vec();
            // assert_eq!(
            //     MiningEligibilityProxyTestModule::block_rewarded_for_day(unused_timstamp),
            //     None,
            // );

            // let timstamp_26mar2021 = "2021-03-26".as_bytes().to_vec();
            // assert_eq!(
            //     MiningEligibilityProxyTestModule::block_rewarded_for_day(timstamp_26mar2021),
            //     None,
            // );

            // let timstamp_27mar2021 = "2021-03-27".as_bytes().to_vec();
            // assert_eq!(
            //     MiningEligibilityProxyTestModule::block_rewarded_for_day(timstamp_27mar2021),
            //     None,
            // );

            // // TODO - fix all the below

            // // it should only return a timestamp for the block number that corresponds to the
            // // start of the day for which the reward request was submitted.
            // // i.e. if it was made at 1400hr on 1st Apr, it'd return the block corresponding
            // // to 0000hr on 1st Apr
            // let valid_day_start = 1616713200000u64;
            // assert_eq!(
            //     MiningEligibilityProxyTestModule::block_rewarded_for_day(valid_day_start),
            //     Some(2),
            // );

            // assert_eq!(
            //     MiningEligibilityProxyTestModule::day_rewarded_for_block(2),
            //     Some(1616713200000u64),
            // );

            // If we reward them on 26th March 2021 @ 02:00 (1616724600000u64),
            // the reward gets inserted for start of that day at 26th Mar 2021 @ 0:00 (1616713200000u64)
            // according to https://currentmillis.com/, so that's the key we need to lookup results with
            assert_eq!(
                MiningEligibilityProxyTestModule::total_rewards_daily(
                    // NaiveDate::from_ymd(2021, 03, 26).and_hms(0, 0, 0).timestamp(),
                    1616724600000u64, // 26.3.2021 @ 2am
                ),
                Some(1000),
            );

            // If we reward them on 27th March 2021 @ 02:00 (1616811000000u64),
            // the reward gets inserted for start of that day at 26th Mar 2021 @ 0:00 (1616799600000u64)
            // according to https://currentmillis.com/, so that's the key we need to lookup results with
            assert_eq!(
                MiningEligibilityProxyTestModule::total_rewards_daily(
                    // NaiveDate::from_ymd(2021, 03, 27).and_hms(0, 0, 0).timestamp(),
                    1616811000000u64, // 27.3.2021 @ 2am
                ),
                Some(3000u64),
            );

            // TODO - add an extra test later on in the day on 26th Mar 2021 to check it gets added
            // to the total rewards for 26th Mar 2021

            // this should return None, since the timestamp was not used
            assert_eq!(
                MiningEligibilityProxyTestModule::total_rewards_daily(
                    // NaiveDate::from_ymd(2021, 01, 15).and_hms(0, 0, 0).timestamp(),
                    947890800000u64, // 15.1.2021 @ 2am
                ),
                None,
            );

            // TODO - add the below functionality to a custom RPC since we cannot return a value
            // from an extrinsic function
            //
            // Check the total sum of rewards sent for a given day
            // assert_eq!(
            //     MiningEligibilityProxyTestModule::calc_rewards_of_day(Origin::signed(0), Some(1u64)).unwrap(),
            //     Some(3000u64)
            // );
        });
    }
}
