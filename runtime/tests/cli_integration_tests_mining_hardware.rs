// extern crate env as env;
extern crate mining_speed_boosts_claims_hardware_mining as mining_speed_boosts_claims_hardware_mining;
extern crate mining_speed_boosts_configuration_hardware_mining as mining_speed_boosts_configuration_hardware_mining;
extern crate mining_speed_boosts_eligibility_hardware_mining as mining_speed_boosts_eligibility_hardware_mining;
extern crate mining_speed_boosts_rates_hardware_mining as mining_speed_boosts_rates_hardware_mining;
extern crate mining_speed_boosts_sampling_hardware_mining as mining_speed_boosts_sampling_hardware_mining;
extern crate roaming_operators as roaming_operators;

#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_ok,
        impl_outer_origin,
        parameter_types,
        weights::Weight,
    };
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{
            BlakeTwo256,
            IdentityLookup,
        },
        Perbill,
    };
    // Import Trait for each runtime module being tested
    use mining_speed_boosts_claims_hardware_mining::{
        MiningSpeedBoostClaimsHardwareMining,
        MiningSpeedBoostClaimsHardwareMiningClaimResult,
        Module as MiningSpeedBoostClaimsHardwareMiningModule,
        Trait as MiningSpeedBoostClaimsHardwareMiningTrait,
    };
    use mining_speed_boosts_configuration_hardware_mining::{
        MiningSpeedBoostConfigurationHardwareMining,
        MiningSpeedBoostConfigurationHardwareMiningHardwareConfig,
        Module as MiningSpeedBoostConfigurationHardwareMiningModule,
        Trait as MiningSpeedBoostConfigurationHardwareMiningTrait,
    };
    use mining_speed_boosts_eligibility_hardware_mining::{
        MiningSpeedBoostEligibilityHardwareMining,
        MiningSpeedBoostEligibilityHardwareMiningEligibilityResult,
        Module as MiningSpeedBoostEligibilityHardwareMiningModule,
        Trait as MiningSpeedBoostEligibilityHardwareMiningTrait,
    };
    use mining_speed_boosts_rates_hardware_mining::{
        MiningSpeedBoostRatesHardwareMining,
        MiningSpeedBoostRatesHardwareMiningRatesConfig,
        Module as MiningSpeedBoostRatesHardwareMiningModule,
        Trait as MiningSpeedBoostRatesHardwareMiningTrait,
    };
    use mining_speed_boosts_sampling_hardware_mining::{
        MiningSpeedBoostSamplingHardwareMining,
        MiningSpeedBoostSamplingHardwareMiningSamplingConfig,
        Module as MiningSpeedBoostSamplingHardwareMiningModule,
        Trait as MiningSpeedBoostSamplingHardwareMiningTrait,
    };
    use roaming_operators;

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
        type AccountData = ();
        type AccountId = u64;
        type AvailableBlockRatio = AvailableBlockRatio;
        type BlockHashCount = BlockHashCount;
        type BlockNumber = u64;
        type Call = ();
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Header = Header;
        type Index = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type MaximumBlockLength = MaximumBlockLength;
        type MaximumBlockWeight = MaximumBlockWeight;
        type ModuleToIndex = ();
        type OnNewAccount = ();
        type OnReapAccount = ();
        type Origin = Origin;
        type Version = ();
    }
    impl balances::Trait for Test {
        type AccountData = ();
        type AccountStore = ();
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ();
        type OnNewAccount = ();
        type OnReapAccount = ();
    }
    impl transaction_payment::Trait for Test {
        type Currency = Balances;
        type FeeMultiplierUpdate = ();
        type OnTransactionPayment = ();
        type TransactionBaseFee = ();
        type TransactionByteFee = ();
        type WeightToFee = ();
    }
    // FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
    impl roaming_operators::Trait for Test {
        type Currency = Balances;
        type Event = ();
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl MiningSpeedBoostConfigurationHardwareMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostConfigurationHardwareMiningHardwareDevEUI = u64;
        // type MiningSpeedBoostConfigurationHardwareMiningHardwareType =
        // MiningSpeedBoostConfigurationHardwareMiningHardwareTypes;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareID = u64;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodEndDate = u64;
        type MiningSpeedBoostConfigurationHardwareMiningHardwareLockPeriodStartDate = u64;
        // Mining Speed Boost Hardware Mining Config
        type MiningSpeedBoostConfigurationHardwareMiningHardwareSecure = bool;
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningSpeedBoostConfigurationHardwareMiningHardwareType = Vec<u8>;
        // type Currency = Balances;
        // type Randomness = Randomness;
        type MiningSpeedBoostConfigurationHardwareMiningIndex = u64;
    }
    impl MiningSpeedBoostRatesHardwareMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostRatesHardwareMiningHardwareInsecure = u32;
        // Mining Speed Boost Rate
        type MiningSpeedBoostRatesHardwareMiningHardwareSecure = u32;
        type MiningSpeedBoostRatesHardwareMiningIndex = u64;
        // Mining Speed Boost Max Rates
        type MiningSpeedBoostRatesHardwareMiningMaxHardware = u32;
    }
    impl MiningSpeedBoostSamplingHardwareMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostSamplingHardwareMiningIndex = u64;
        type MiningSpeedBoostSamplingHardwareMiningSampleDate = u64;
        type MiningSpeedBoostSamplingHardwareMiningSampleHardwareOnline = u64;
    }
    impl MiningSpeedBoostEligibilityHardwareMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostEligibilityHardwareMiningCalculatedEligibility = u64;
        type MiningSpeedBoostEligibilityHardwareMiningHardwareUptimePercentage = u32;
        type MiningSpeedBoostEligibilityHardwareMiningIndex = u64;
        // type MiningSpeedBoostEligibilityHardwareMiningDateAudited = u64;
        // type MiningSpeedBoostEligibilityHardwareMiningAuditorAccountID = u64;
    }
    impl MiningSpeedBoostClaimsHardwareMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostClaimsHardwareMiningClaimAmount = u64;
        type MiningSpeedBoostClaimsHardwareMiningClaimDateRedeemed = u64;
        type MiningSpeedBoostClaimsHardwareMiningIndex = u64;
    }

    // type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type MiningSpeedBoostConfigurationHardwareMiningTestModule =
        MiningSpeedBoostConfigurationHardwareMiningModule<Test>;
    type MiningSpeedBoostRatesHardwareMiningTestModule = MiningSpeedBoostRatesHardwareMiningModule<Test>;
    type MiningSpeedBoostSamplingHardwareMiningTestModule = MiningSpeedBoostSamplingHardwareMiningModule<Test>;
    type MiningSpeedBoostEligibilityHardwareMiningTestModule = MiningSpeedBoostEligibilityHardwareMiningModule<Test>;
    type MiningSpeedBoostClaimsHardwareMiningTestModule = MiningSpeedBoostClaimsHardwareMiningModule<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
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
              MiningSpeedBoostRatesHardwareMiningTestModule::set_mining_speed_boosts_rates_hardware_mining_rates_config(
                Origin::signed(0),
                0, // mining_speed_boosts_rates_hardware_mining_id
                // FIXME - convert all below types to Vec<u8> since float values? i.e. b"1.025".to_vec()
                Some(1), // hardware_hardware_secure
                Some(1), // hardware_hardware_insecure
                Some(1), // hardware_max_hardware
              )
            );

            // Verify Storage
            assert_eq!(MiningSpeedBoostRatesHardwareMiningTestModule::mining_speed_boosts_rates_hardware_mining_count(), 1);
            assert!(MiningSpeedBoostRatesHardwareMiningTestModule::mining_speed_boosts_rates_hardware_mining(0).is_some());
            assert_eq!(MiningSpeedBoostRatesHardwareMiningTestModule::mining_speed_boosts_rates_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostRatesHardwareMiningTestModule::mining_speed_boosts_rates_hardware_mining_rates_configs(0),
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
              MiningSpeedBoostConfigurationHardwareMiningTestModule::set_mining_speed_boosts_configuration_hardware_mining_hardware_config(
                Origin::signed(0),
                0, // mining_speed_boosts_hardware_mining_id
                Some(true), // hardware_secure
                Some(b"gateway".to_vec()), // hardware_type
                Some(1), // hardware_id
                Some(12345), // hardware_dev_eui
                Some(23456), // hardware_lock_period_start_date
                Some(34567), // hardware_lock_period_end_date
              )
            );

            // Verify Storage
            assert_eq!(MiningSpeedBoostConfigurationHardwareMiningTestModule::mining_speed_boosts_configuration_hardware_mining_count(), 1);
            assert!(MiningSpeedBoostConfigurationHardwareMiningTestModule::mining_speed_boosts_configuration_hardware_mining(0).is_some());
            assert_eq!(MiningSpeedBoostConfigurationHardwareMiningTestModule::mining_speed_boosts_configuration_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostConfigurationHardwareMiningTestModule::mining_speed_boosts_configuration_hardware_mining_hardware_configs(0),
                Some(MiningSpeedBoostConfigurationHardwareMiningHardwareConfig {
                    hardware_secure: true,
                    hardware_type: b"gateway".to_vec(),
                    hardware_id: 1,
                    hardware_dev_eui: 12345,
                    hardware_lock_period_start_date: 23456,
                    hardware_lock_period_end_date: 34567,
                })
            );

            // Create Mining Speed Boost Sampling Hardware Mining

            // Call Functions
            assert_ok!(MiningSpeedBoostSamplingHardwareMiningTestModule::create(Origin::signed(0)));
            assert_ok!(
                MiningSpeedBoostSamplingHardwareMiningTestModule::set_mining_speed_boosts_samplings_hardware_mining_samplings_config(
                    Origin::signed(0),
                    0, // mining_speed_boosts_sampling_hardware_mining_id
                    0, // mining_speed_boosts_sampling_hardware_mining_sample_id
                    Some(23456), // hardware_sample_date
                    Some(1), // hardware_sample_hardware_online
                )
            );
            assert_ok!(MiningSpeedBoostSamplingHardwareMiningTestModule::assign_sampling_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningSpeedBoostSamplingHardwareMiningTestModule::mining_speed_boosts_samplings_hardware_mining_count(), 1);
            assert!(MiningSpeedBoostSamplingHardwareMiningTestModule::mining_speed_boosts_samplings_hardware_mining(0).is_some());
            assert_eq!(MiningSpeedBoostSamplingHardwareMiningTestModule::mining_speed_boosts_samplings_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostSamplingHardwareMiningTestModule::mining_speed_boosts_samplings_hardware_mining_samplings_configs((0, 0)),
                Some(MiningSpeedBoostSamplingHardwareMiningSamplingConfig {
                    hardware_sample_date: 23456, // hardware_sample_date
                    hardware_sample_hardware_online: 1 // hardware_sample_hardware_online
                })
            );

            // Create Mining Speed Boost Eligibility Hardware Mining

            // Call Functions
            assert_ok!(MiningSpeedBoostEligibilityHardwareMiningTestModule::create(Origin::signed(0)));
            // assert_eq!(
            //     MiningSpeedBoostEligibilityTestModule::calculate_mining_speed_boosts_eligibility_hardware_mining_result(
            //         Origin::signed(0),
            //         0, // mining_speed_boosts_configuration_hardware_mining_id
            //         0, // mining_speed_boosts_eligibility_hardware_mining_id
            //     ),
            //     Some(
            //         MiningSpeedBoostEligibilityHardwareMiningEligibilityResult {
            //             eligibility_hardware_mining_calculated_eligibility: 1.1
            //             // to determine eligibility for proportion (incase user moves funds around during lock period)
            //             eligibility_hardware_mining_hardware_uptime_percentage: 0.3,
            //             // eligibility_hardware_mining_date_audited: 123,
            //             // eligibility_hardware_mining_auditor_account_id: 123
            //         }
            //     )
            // ))

            // Override by DAO if necessary
            assert_ok!(
                MiningSpeedBoostEligibilityHardwareMiningTestModule::set_mining_speed_boosts_eligibility_hardware_mining_eligibility_result(
                    Origin::signed(0),
                    0, // mining_speed_boosts_configuration_hardware_mining_id
                    0, // mining_speed_boosts_eligibility_hardware_mining_id
                    Some(1), // mining_speed_boosts_eligibility_hardware_mining_calculated_eligibility
                    Some(1), // mining_speed_boosts_eligibility_hardware_mining_hardware_uptime_percentage
                    // 123, // mining_speed_boosts_eligibility_hardware_mining_date_audited
                    // 123, // mining_speed_boosts_eligibility_hardware_mining_auditor_account_id
                    // Some({
                    //     MiningSpeedBoostEligibilityHardwareMiningEligibilityResult {
                    //         eligibility_hardware_mining_calculated_eligibility: 1,
                    //         // to determine eligibility for proportion (incase user moves funds around during lock period)
                    //         eligibility_hardware_mining_hardware_uptime_percentage: 1,
                    //         // eligibility_hardware_mining_date_audited: 123,
                    //         // eligibility_hardware_mining_auditor_account_id: 123
                    //     }
                    // }),
                )
            );
            assert_ok!(MiningSpeedBoostEligibilityHardwareMiningTestModule::assign_eligibility_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningSpeedBoostEligibilityHardwareMiningTestModule::mining_speed_boosts_eligibility_hardware_mining_count(), 1);
            assert!(MiningSpeedBoostEligibilityHardwareMiningTestModule::mining_speed_boosts_eligibility_hardware_mining(0).is_some());
            assert_eq!(MiningSpeedBoostEligibilityHardwareMiningTestModule::mining_speed_boosts_eligibility_hardware_mining_owner(0), Some(0));
            assert_eq!(
                MiningSpeedBoostEligibilityHardwareMiningTestModule::mining_speed_boosts_eligibility_hardware_mining_eligibility_results((0, 0)),
                Some(MiningSpeedBoostEligibilityHardwareMiningEligibilityResult {
                    eligibility_hardware_mining_calculated_eligibility: 1,
                    // to determine eligibility for proportion (incase user moves funds around during lock period)
                    eligibility_hardware_mining_hardware_uptime_percentage: 1,
                    // eligibility_hardware_mining_date_audited: 123,
                    // eligibility_hardware_mining_auditor_account_id: 123
                })
            );

            // Create Mining Speed Boost Claims Hardware Mining

            // // Call Functions
            assert_ok!(MiningSpeedBoostClaimsHardwareMiningTestModule::create(Origin::signed(0)));
            assert_ok!(MiningSpeedBoostClaimsHardwareMiningTestModule::assign_claim_to_configuration(Origin::signed(0), 0, 0));
            // assert_ok!(
            //     MiningSpeedBoostClaimsHardwareMiningTestModule::claim(
            //         Origin::signed(0),
            //         0, // mining_speed_boosts_configuration_hardware_mining_id
            //         0, // mining_speed_boosts_eligibility_hardware_mining_id
            //         0, // mining_speed_boosts_claims_hardware_mining_id
            //     )
            // )
            // Override by DAO if necessary
            assert_ok!(
                MiningSpeedBoostClaimsHardwareMiningTestModule::set_mining_speed_boosts_claims_hardware_mining_claims_result(
                    Origin::signed(0),
                    0, // mining_speed_boosts_configuration_hardware_mining_id
                    0, // mining_speed_boosts_eligibility_hardware_mining_id
                    0, // mining_speed_boosts_claims_hardware_mining_id
                    Some(1), // hardware_claim_amount
                    Some(34567), // hardware_claim_date_redeemed
                )
            );

            // Verify Storage
            assert_eq!(MiningSpeedBoostClaimsHardwareMiningTestModule::mining_speed_boosts_claims_hardware_mining_count(), 1);
            assert!(MiningSpeedBoostClaimsHardwareMiningTestModule::mining_speed_boosts_claims_hardware_mining(0).is_some());
            assert_eq!(MiningSpeedBoostClaimsHardwareMiningTestModule::mining_speed_boosts_claims_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostClaimsHardwareMiningTestModule::mining_speed_boosts_claims_hardware_mining_claims_results((0, 0)),
                Some(MiningSpeedBoostClaimsHardwareMiningClaimResult {
                    hardware_claim_amount: 1,
                    hardware_claim_date_redeemed: 34567,
                })
            );
        });
    }
}
