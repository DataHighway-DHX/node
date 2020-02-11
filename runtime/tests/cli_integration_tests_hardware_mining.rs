// extern crate env as env;
extern crate roaming_operators as roaming_operators;
extern crate mining_speed_boost_configuration_hardware_mining as mining_speed_boost_configuration_hardware_mining;
extern crate mining_speed_boost_rates_hardware_mining as mining_speed_boost_rates_hardware_mining;
extern crate mining_speed_boost_sampling_token_mining as mining_speed_boost_sampling_token_mining;

#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{H256};
    use sr_primitives::Perbill;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{assert_noop, assert_ok, impl_outer_origin, parameter_types, weights::Weight};
    // Import Trait for each runtime module being tested
    use roaming_operators;
    use mining_speed_boost_configuration_hardware_mining::{
        Module as MiningSpeedBoostConfigurationHardwareMiningModule,
        MiningSpeedBoostConfigurationHardwareMining,
        MiningSpeedBoostConfigurationHardwareMiningHardwareConfig,
        // MiningSpeedBoostConfigurationHardwareMiningHardwareTypes,
        Trait as MiningSpeedBoostConfigurationHardwareMiningTrait,
    };
    use mining_speed_boost_rates_hardware_mining::{
        Module as MiningSpeedBoostRatesHardwareMiningModule,
        MiningSpeedBoostRatesHardwareMining,
        Trait as MiningSpeedBoostRatesHardwareMiningTrait,
    };
    use mining_speed_boost_sampling_hardware_mining::{
        Module as MiningSpeedBoostSamplingHardwareMiningModule,
        MiningSpeedBoostSamplingHardwareMining,
        Trait as MiningSpeedBoostSamplingHardwareMiningTrait,
    };
    // use mining_speed_boost_eligibilities::{
    //     Module as MiningSpeedBoostEligibilitiesModule,
    //     MiningSpeedBoostEligibility,
    //     Trait as MiningSpeedBoostEligibilitiesTrait,
    // };
    // use mining_speed_boost_rewards::{
    //     Module as MiningSpeedBoostRewardsModule,
    //     MiningSpeedBoostReward,
    //     Trait as MiningSpeedBoostRewardsTrait,
    // };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
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
    impl MiningSpeedBoostConfigurationHardwareMiningTrait for Test {
        type Event = ();
        // type Currency = Balances;
        // type Randomness = Randomness;
        type MiningSpeedBoostConfigurationHardwareMiningIndex = u64;

        // Mining Speed Boost Hardware Mining Config
        type MiningSpeedBoostConfigurationHardwareMiningHardwareSecure = bool;
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningSpeedBoostConfigurationHardwareMiningHardwareType = Vec<u8>;
        // type MiningSpeedBoostConfigurationHardwareMiningHardwareType = MiningSpeedBoostConfigurationHardwareMiningHardwareTypes;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareID = u64;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI = u64;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate = u64;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate = u64;
    }
    impl MiningSpeedBoostRatesHardwareMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostRatesHardwareMiningIndex = u64;
        // Mining Speed Boost Rate
        type MiningSpeedBoostRatesHardwareMiningHardwareSecure = u32;
        type MiningSpeedBoostRatesHardwareMiningHardwareInsecure = u32;
        // Mining Speed Boost Max Rates
        type MiningSpeedBoostRatesHardwareMiningMaxHardware = u32;
    }
    impl MiningSpeedBoostSamplingHardwareMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostSamplingHardwareMiningIndex = u64;
        type MiningSpeedBoostSamplingHardwareMiningSampleDate = u64;
        type MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline = u64;
    }
    // impl MiningSpeedBoostEligibilitiesTrait for Test {
    //     type Event = ();
    //     type MiningSpeedBoostEligibilitiesIndex = u64;
    //     // Mining Speed Boost Eligibility
    //     type MiningSpeedBoostEligibilityCalculated = u64;
    //     type MiningSpeedBoostEligibilityTokenLockedPercentage = u32;
    //     type MiningSpeedBoostEligibilityHardwareUptimePercentage = u32;
    //     type MiningSpeedBoostEligibilityDateAudited = u64;
    //     type MiningSpeedBoostEligibilityAuditorAccountID = u64;
    // }   
    // impl MiningSpeedBoostRewardsTrait for Test {
    //     type Event = ();
    //     type MiningSpeedBoostRewardsIndex = u64;
    //     // Mining Speed Boost Reward
    //     type MiningSpeedBoostRewardAmount = u64;
    //     type MiningSpeedBoostRewardDateRedeemed = u64;
    // }   

    //type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type MiningSpeedBoostConfigurationHardwareMiningTestModule = MiningSpeedBoostConfigurationHardwareMiningModule<Test>;
    type MiningSpeedBoostRatesHardwareMiningTestModule = MiningSpeedBoostRatesHardwareMiningModule<Test>;
    type MiningSpeedBoostSamplingHardwareMiningTestModule = MiningSpeedBoostSamplingHardwareMiningModule<Test>;
    // type MiningSpeedBoostEligibilitiesTestModule = MiningSpeedBoostEligibilitiesModule<Test>;
    // type MiningSpeedBoostRewardsTestModule = MiningSpeedBoostRewardsModule<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        runtime_io::TestExternalities::new(t)
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

            // Create Mining Speed Boost Rates Hardware Mining

            // Call Functions
            assert_ok!(MiningSpeedBoostRatesHardwareMiningTestModule::create(Origin::signed(0)));
            assert_ok!(
              MiningSpeedBoostRatesHardwareMiningTestModule::set_mining_speed_boost_rates_hardware_mining_rates_config(
                Origin::signed(0),
                0, // mining_speed_boost_rates_hardware_mining_id
                // FIXME - convert all below types to Vec<u8> since float values? i.e. "1.025".as_bytes().to_vec()
                Some(1), // hardware_hardware_secure
                Some(1), // hardware_hardware_insecure
                Some(1), // hardware_max_hardware
              )
            );

            // Verify Storage
            assert_eq!(MiningSpeedBoostRatesHardwareMiningTestModule::mining_speed_boost_rates_hardware_mining_count(), 1);
            assert!(MiningSpeedBoostRatesHardwareMiningTestModule::mining_speed_boost_rates_hardware_mining(0).is_some());
            assert_eq!(MiningSpeedBoostRatesHardwareMiningTestModule::mining_speed_boost_rates_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostRatesTokenMiningTestModule::mining_speed_boost_rates_hardware_mining_rates_configs(0),
                Some(MiningSpeedBoostRatesHardwareMiningRatesConfig {
                    hardware_hardware_secure: 1,
                    hardware_hardware_insecure: 1,
                    hardware_max_hardware: 1,
                })
            );

            // Create Mining Speed Boost Configuration Hardware Mining

            // Call Functions
            assert_ok!(MiningSpeedBoostConfigurationHardwareMiningTestModule::create(Origin::signed(0)));
            assert_ok!(
              MiningSpeedBoostConfigurationHardwareMiningTestModule::set_mining_speed_boost_configuration_hardware_mining_hardware_config(
                Origin::signed(0),
                0, // mining_speed_boost_hardware_mining_id
                Some(true), // hardware_secure
                Some("gateway".as_bytes().to_vec()), // hardware_type
                Some(1), // hardware_id
                Some(12345), // hardware_dev_eui
                Some(23456), // hardware_lock_period_start_date
                Some(34567), // hardware_lock_period_end_date
              )
            );

            // Verify Storage
            assert_eq!(MiningSpeedBoostConfigurationHardwareMiningTestModule::mining_speed_boost_configuration_hardware_mining_count(), 1);
            assert!(MiningSpeedBoostConfigurationHardwareMiningTestModule::mining_speed_boost_configuration_hardware_mining(0).is_some());
            assert_eq!(MiningSpeedBoostConfigurationHardwareMiningTestModule::mining_speed_boost_configuration_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostConfigurationHardwareMiningTestModule::mining_speed_boost_configuration_hardware_mining_hardware_configs(0),
                Some(MiningSpeedBoostConfigurationHardwareMiningHardwareConfig {
                    hardware_secure: true,
                    hardware_type: "gateway".as_bytes().to_vec(),
                    hardware_id: 1,
                    hardware_dev_eui: 12345,
                    hardware_lock_period_start_date: 23456,
                    hardware_lock_period_end_date: 34567,
                })
            );

            // Create Mining Speed Boost Sampling Hardware Mining

            // Call Functions
            assert_ok!(
                MiningSpeedBoostSamplingHardwareMiningTestModule::set_mining_speed_boost_sampling_hardware_mining_sampling_configs(
                    Origin::signed(0),
                    0, // mining_speed_boost_sampling_hardware_mining_id
                    0, // mining_speed_boost_sampling_hardware_mining_sample_id
                    Some({
                        MiningSpeedBoostSamplingHardwareMiningSamplingConfig {
                            hardware_sample_date: Some(23456), // hardware_sample_date
                            hardware_sample_hardware_online: Some(1) // hardware_sample_hardware_online
                        }
                    }),
                )
            );
            assert_ok!(MiningSpeedBoostSamplingHardwareMiningTestModule::assign_sampling_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningSpeedBoostSamplingHardwareMiningTestModule::mining_speed_boost_sampling_hardware_mining_count(), 1);
            assert!(MiningSpeedBoostSamplingHardwareMiningTestModule::mining_speed_boost_sampling_hardware_mining(0).is_some());
            assert_eq!(MiningSpeedBoostSamplingHardwareMiningTestModule::mining_speed_boost_sampling_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostSamplingHardwareMiningTestModule::mining_speed_boost_sampling_hardware_mining_sampling_configs(0),
                Some(MiningSpeedBoostSamplingHardwareMiningSamplingConfig {
                    hardware_sample_date: Some(23456), // hardware_sample_date
                    hardware_sample_hardware_online: Some(1) // hardware_sample_hardware_online
                })
            );

            // // Eligibilities

            // assert_ok!(MiningSpeedBoostEligibilitiesTestModule::set_random_sample(
            //     Origin::signed(0),
            //     0, // mining_speed_boost_id
            //     11111, // sample_hash
            //     {
            //         sample_date: Some(23456) // sample_date
            //         // sample_tokens_locked: Some(70) // sample_tokens_locked
            //     }
            // ))
            // // Note: On the random sampling dates an oracle service audits and publishes logs
            // // of how many tokens were locked. The log should include the account id
            // // of the auditor.
            // // Store the amount of tokens locked that is published in the logs for the sample_hash
            // assert_ok!(MiningSpeedBoostEligibilitiesTestModule::set_random_sample(
            //     Origin::signed(0),
            //     0, // mining_speed_boost_id
            //     11111, // sample_hash
            //     {
            //         // sample_date: Some(23456) // sample_date
            //         sample_tokens_locked: Some(70) // sample_tokens_locked
            //     }
            // ))
            // assert_eq!(
            //     MiningSpeedBoostEligibilitiesTestModule::get_random_sample(
            //         Origin::signed(0),
            //         0, // mining_speed_boost_id,
            //         11111 // sample_hash
            //     ),
            //     Some(MiningSpeedBoostSample {
            //         sample_date: Some(23456) // sample_date
            //         sample_tokens_locked: Some(70) // sample_tokens_locked
            //     })
            // ]))

            // // Search through the emitted logs for each `sample_hash`
            // // and aggregate the results to determine their eligibility
            // assert_eq!(
            //     MiningSpeedBoostEligibilitiesTestModule::check_eligibility(
            //         Origin::signed(0),
            //         0, // mining_speed_boost_id
            //         12345, // token_lock_period_start_date
            //         23456, // token_lock_period_end_date
            //     ),
            //     Some(MiningSpeedBoostEligibility {
            //         eligibility_calculated: 1.1
            //         // to determine eligibility for proportion (incase user moves funds around during lock period)
            //         eligibility_token_locked_percentage: 0.7, // i.e. 70%
            //         eligibility_hardware_uptime_percentage: 0
            //     })
            // ))

            // // Rewards

            // assert_ok!(
            //     MiningSpeedBoostRewardsTestModule::reward(
            //         Origin:signed(0),
            //         0,
            //         12345, // token_lock_period_start_date
            //         23456, // token_lock_period_end_date
            //     ),
            //     Some(MiningSpeedBoostReward {
            //         reward_hash: Some(22222), // reward_hash
            //         reward_amount: 1.1,
            //         reward_date_redeemed: 34567,
            //     })
            // )
        });
    }
}
