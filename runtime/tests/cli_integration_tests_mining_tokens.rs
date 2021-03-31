// extern crate env as env;
extern crate membership_supernodes as membership_supernodes;
extern crate mining_claims_token as mining_claims_token;
extern crate mining_setting_token as mining_setting_token;
extern crate mining_eligibility_token as mining_eligibility_token;
extern crate mining_execution_token as mining_execution_token;
extern crate mining_rates_token as mining_rates_token;
extern crate mining_sampling_token as mining_sampling_token;
extern crate roaming_operators as roaming_operators;

#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_err,
        assert_ok,
        parameter_types,
        weights::{
            IdentityFee,
            Weight,
        },
    };

    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{
            BlakeTwo256,
            IdentityLookup,

        },
        DispatchError,
        DispatchResult,
        Perbill,
        Permill,
    };
    pub use pallet_transaction_payment::{
        CurrencyAdapter,
    };
    use membership_supernodes::{
        Module as MembershipSupernodesModule,
        Trait as MembershipSupernodesTrait,
    };
    // Import Trait for each runtime module being tested
    use mining_claims_token::{
        MiningClaimsTokenClaimResult,
        Module as MiningClaimsTokenModule,
        Config as MiningClaimsTokenConfig,
    };
    use mining_setting_token::{
        MiningSettingTokenSetting,
        MiningSettingTokenRequirementsSetting,
        Module as MiningSettingTokenModule,
        Config as MiningSettingTokenConfig,
    };
    use mining_eligibility_token::{
        MiningEligibilityTokenResult,
        Module as MiningEligibilityTokenModule,
        Config as MiningEligibilityTokenConfig,
    };
    use mining_execution_token::{
        MiningExecutionTokenExecutionResult,
        Module as MiningExecutionTokenModule,
        Config as MiningExecutionTokenConfig,
    };
    use mining_rates_token::{
        MiningRatesTokenSetting,
        Module as MiningRatesTokenModule,
        Config as MiningRatesTokenConfig,
    };
    use mining_sampling_token::{
        MiningSamplingTokenSetting,
        Module as MiningSamplingTokenModule,
        Config as MiningSamplingTokenConfig,
    };
    use roaming_operators;

    // pub fn origin_of(who: &AccountId) -> <Runtime as frame_system::Config>::Origin {
    // 	<Runtime as frame_system::Config>::Origin::signed((*who).clone())
    // }
    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
    type Block = frame_system::mocking::MockBlock<Test>;

    frame_support::construct_runtime!(
        pub enum Test where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system::{Module, Call, Config, Storage, Event<T>},
            Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
            RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
            TransactionPayment: pallet_transaction_payment::{Module, Storage},
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const SS58Prefix: u8 = 33;
    }
    impl frame_system::Config for Test {
        type AccountData = pallet_balances::AccountData<u64>;
        type AccountId = u64;
        type BaseCallFilter = ();
        type BlockHashCount = BlockHashCount;
        type BlockLength = ();
        type BlockNumber = u64;
        type BlockWeights = ();
        type Call = Call;
        type DbWeight = ();
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Header = Header;
        type Index = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type OnKilledAccount = ();
        type OnNewAccount = ();
        type Origin = Origin;
        type PalletInfo = PalletInfo;
        type SS58Prefix = SS58Prefix;
        type SystemWeightInfo = ();
        type Version = ();
    }
    parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
    }
    impl pallet_balances::Config for Test {
        type AccountStore = System;
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type MaxLocks = ();
        type WeightInfo = ();
    }
    impl pallet_transaction_payment::Config for Test {
        type FeeMultiplierUpdate = ();
        type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
        type TransactionByteFee = ();
        type WeightToFee = IdentityFee<u64>;
    }
    // FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
    impl roaming_operators::Config for Test {
        type Currency = Balances;
        type Event = ();
        type Randomness = RandomnessCollectiveFlip;
        type RoamingOperatorIndex = u64;
    }
    impl MiningSettingTokenConfig for Test {
        type Event = ();
        // type Currency = Balances;
        // type Randomness = Randomness;
        type MiningSettingTokenIndex = u64;
        type MiningSettingTokenLockAmount = u64;
        // Mining Speed Boost Token Mining Config
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningSettingTokenType = Vec<u8>;
    }
    impl MiningRatesTokenConfig for Test {
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
    impl MiningSamplingTokenConfig for Test {
        type Event = ();
        type MiningSamplingTokenIndex = u64;
        type MiningSamplingTokenSampleLockedAmount = u64;
    }
    impl MiningEligibilityTokenConfig for Test {
        type Event = ();
        type MiningEligibilityTokenCalculatedEligibility = u64;
        type MiningEligibilityTokenIndex = u64;
        type MiningEligibilityTokenLockedPercentage = u32;
        // type MiningEligibilityTokenAuditorAccountID = u64;
    }
    impl MiningClaimsTokenConfig for Test {
        type Event = ();
        type MiningClaimsTokenClaimAmount = u64;
        type MiningClaimsTokenIndex = u64;
    }
    impl MiningExecutionTokenConfig for Test {
        type Event = ();
        type MiningExecutionTokenIndex = u64;
    }
    impl MembershipSupernodesTrait for Test {
        type Event = ();
    }

    pub type MiningSettingTokenTestModule = MiningSettingTokenModule<Test>;
    pub type MiningRatesTokenTestModule = MiningRatesTokenModule<Test>;
    pub type MiningSamplingTokenTestModule = MiningSamplingTokenModule<Test>;
    pub type MiningEligibilityTokenTestModule = MiningEligibilityTokenModule<Test>;
    pub type MiningClaimsTokenTestModule = MiningClaimsTokenModule<Test>;
    pub type MiningExecutionTokenTestModule = MiningExecutionTokenModule<Test>;
    pub type MembershipSupernodesTestModule = MembershipSupernodesModule<Test>;
    type Randomness = pallet_randomness_collective_flip::Module<Test>;
    type MembershipSupernodes = membership_supernodes::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    pub fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30)],
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
                Some(MiningRatesTokenSetting {
                    token_token_mxc: 1,
                    token_token_iota: 1,
                    token_token_dot: 1,
                    token_max_token: 1,
                    token_max_loyalty: 1,
                })
            );

            // Create Mining Speed Boost Configuration & Cooldown Configuration Token Mining

            // Call Functions
            assert_ok!(MiningSettingTokenTestModule::create(Origin::signed(0)));
            assert_ok!(MiningSettingTokenTestModule::set_mining_setting_token_token_cooldown_config(
                Origin::signed(0),
                0,                     // mining_token_id
                Some(b"DHX".to_vec()), // token_type
                Some(10),              // token_lock_min_amount
                Some(7),               // token_lock_min_blocks
            ));
            assert_ok!(MiningSettingTokenTestModule::set_mining_setting_token_token_setting(
                Origin::signed(0),
                0,                     // mining_token_id
                Some(b"MXC".to_vec()), // token_type
                Some(100),             // token_lock_amount
                Some(12345),           // token_lock_start_block
                Some(23456),           // token_lock_interval_blocks
            ));

            // Verify Storage
            assert_eq!(MiningSettingTokenTestModule::mining_setting_token_count(), 1);
            assert!(MiningSettingTokenTestModule::mining_setting_token(0).is_some());
            assert_eq!(MiningSettingTokenTestModule::mining_setting_token_owner(0), Some(0));
            assert_eq!(
                MiningSettingTokenTestModule::mining_setting_token_token_cooldown_configs(0),
                Some(MiningSettingTokenRequirementsSetting {
                    token_type: b"DHX".to_vec(), // token_type
                    token_lock_min_amount: 10,   // token_lock_min_amount
                    token_lock_min_blocks: 7,    // token_lock_min_blocks
                })
            );
            assert_eq!(
                MiningSettingTokenTestModule::mining_setting_token_token_settings(0),
                Some(MiningSettingTokenSetting {
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
                Some(MiningSamplingTokenSetting {
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
            // mining_setting_token_id

            // Call Functions
            assert_ok!(MiningEligibilityTokenTestModule::create(Origin::signed(0)));
            // assert_eq!(
            //   MiningEligibilityTokenTestModule::calculate_mining_eligibility_token_result(
            //       Origin::signed(0),
            //       0, // mining_setting_token_id
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
                0,       // mining_setting_token_id
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
                0, // mining_setting_token_id
                0, // mining_eligibility_token_id
                0, // mining_claims_token_id
            ));
            // Override by DAO if necessary
            assert_ok!(MiningClaimsTokenTestModule::set_mining_claims_token_claims_result(
                Origin::signed(0),
                0,           // mining_setting_token_id
                0,           // mining_eligibility_token_id
                0,           // mining_claims_token_id
                Some(1),     // token_claim_amount
                Some(34567)  // token_claim_block_redeemed
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
                0,           // mining_setting_token_id
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

            // Only the Sudo root account may add and remove any account's membership of Member Supernodes, but in
            // practice only the Supernode Centre's account would be added with membership of Member Supernodes.

            // The implementation uses ensure_root, so only the root origin may add and remove members (i.e. not account 1)
            assert_ok!(MembershipSupernodesTestModule::add_member(Origin::root(), 0));
            assert_err!(MembershipSupernodesTestModule::add_member(Origin::signed(0), 0), DispatchError::BadOrigin);

            assert_ok!(MembershipSupernodesTestModule::remove_member(Origin::root(), 0));
            assert_err!(MembershipSupernodesTestModule::remove_member(Origin::signed(0), 0), DispatchError::BadOrigin);
        });
    }
}
