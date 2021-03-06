// extern crate env as env;
extern crate mining_claims_hardware as mining_claims_hardware;
extern crate mining_config_hardware as mining_config_hardware;
extern crate mining_eligibility_hardware as mining_eligibility_hardware;
extern crate mining_rates_hardware as mining_rates_hardware;
extern crate mining_sampling_hardware as mining_sampling_hardware;
extern crate roaming_operators as roaming_operators;

#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_ok,
        impl_outer_origin,
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
            Zero,
        },
        DispatchResult,
        Perbill,
        Permill,
    };
    // Import Trait for each runtime module being tested
    use mining_claims_hardware::{
        MiningClaimsHardwareClaimResult,
        Module as MiningClaimsHardwareModule,
        Trait as MiningClaimsHardwareTrait,
    };
    use mining_config_hardware::{
        MiningConfigHardwareConfig,
        Module as MiningConfigHardwareModule,
        Trait as MiningConfigHardwareTrait,
    };
    use mining_eligibility_hardware::{
        MiningEligibilityHardwareResult,
        Module as MiningEligibilityHardwareModule,
        Trait as MiningEligibilityHardwareTrait,
    };
    use mining_rates_hardware::{
        MiningRatesHardwareConfig,
        Module as MiningRatesHardwareModule,
        Trait as MiningRatesHardwareTrait,
    };
    use mining_sampling_hardware::{
        MiningSamplingHardwareConfig,
        Module as MiningSamplingHardwareModule,
        Trait as MiningSamplingHardwareTrait,
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
    // FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
    impl roaming_operators::Trait for Test {
        type Currency = Balances;
        type Event = ();
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl MiningConfigHardwareTrait for Test {
        type Event = ();
        type MiningConfigHardwareDevEUI = u64;
        // type MiningConfigHardwareType =
        // MiningConfigHardwareTypes;
        type MiningConfigHardwareID = u64;
        // type Currency = Balances;
        // type Randomness = Randomness;
        type MiningConfigHardwareIndex = u64;
        // Mining Speed Boost Hardware Mining Config
        type MiningConfigHardwareSecure = bool;
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningConfigHardwareType = Vec<u8>;
    }
    impl MiningRatesHardwareTrait for Test {
        type Event = ();
        type MiningRatesHardwareCategory1MaxTokenBonusPerGateway = u32;
        type MiningRatesHardwareCategory2MaxTokenBonusPerGateway = u32;
        type MiningRatesHardwareCategory3MaxTokenBonusPerGateway = u32;
        type MiningRatesHardwareIndex = u64;
        type MiningRatesHardwareInsecure = u32;
        // Mining Speed Boost Max Rates
        type MiningRatesHardwareMaxHardware = u32;
        // Mining Speed Boost Rate
        type MiningRatesHardwareSecure = u32;
    }
    impl MiningSamplingHardwareTrait for Test {
        type Event = ();
        type MiningSamplingHardwareIndex = u64;
        type MiningSamplingHardwareSampleHardwareOnline = u64;
    }
    impl MiningEligibilityHardwareTrait for Test {
        type Event = ();
        type MiningEligibilityHardwareCalculatedEligibility = u64;
        type MiningEligibilityHardwareIndex = u64;
        type MiningEligibilityHardwareUptimePercentage = u32;
        // type MiningEligibilityHardwareAuditorAccountID = u64;
    }
    impl MiningClaimsHardwareTrait for Test {
        type Event = ();
        type MiningClaimsHardwareClaimAmount = u64;
        type MiningClaimsHardwareIndex = u64;
    }

    type System = frame_system::Module<Test>;
    pub type Balances = pallet_balances::Module<Test>;
    pub type MiningConfigHardwareTestModule = MiningConfigHardwareModule<Test>;
    pub type MiningRatesHardwareTestModule = MiningRatesHardwareModule<Test>;
    pub type MiningSamplingHardwareTestModule = MiningSamplingHardwareModule<Test>;
    pub type MiningEligibilityHardwareTestModule = MiningEligibilityHardwareModule<Test>;
    pub type MiningClaimsHardwareTestModule = MiningClaimsHardwareModule<Test>;
    pub type Randomness = pallet_randomness_collective_flip::Module<Test>;

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

            // Create Mining Speed Boost Rates Hardware Mining

            // Call Functions
            assert_ok!(MiningRatesHardwareTestModule::create(Origin::signed(0)));
            assert_ok!(MiningRatesHardwareTestModule::set_mining_rates_hardware_rates_config(
                Origin::signed(0),
                0, // mining_rates_hardware_id
                // FIXME - convert all below types to Vec<u8> since float values? i.e. b"1.025".to_vec()
                Some(1), // hardware_hardware_secure
                Some(1), // hardware_hardware_insecure
                Some(1), // hardware_max_hardware
                Some(1000000),
                Some(500000),
                Some(250000)
            ));

            // Verify Storage
            assert_eq!(MiningRatesHardwareTestModule::mining_rates_hardware_count(), 1);
            assert!(MiningRatesHardwareTestModule::mining_rates_hardware(0).is_some());
            assert_eq!(MiningRatesHardwareTestModule::mining_rates_hardware_owner(0), Some(0));
            assert_eq!(
                MiningRatesHardwareTestModule::mining_rates_hardware_rates_configs(0),
                Some(MiningRatesHardwareConfig {
                    hardware_hardware_secure: 1,
                    hardware_hardware_insecure: 1,
                    hardware_max_hardware: 1,
                    hardware_category_1_max_token_bonus_per_gateway: 1000000,
                    hardware_category_2_max_token_bonus_per_gateway: 500000,
                    hardware_category_3_max_token_bonus_per_gateway: 250000
                })
            );

            // Create Mining Speed Boost Configuration Hardware Mining

            // Call Functions
            assert_ok!(MiningConfigHardwareTestModule::create(Origin::signed(0)));
            assert_ok!(MiningConfigHardwareTestModule::set_mining_config_hardware_hardware_config(
                Origin::signed(0),
                0,                         // mining_hardware_id
                Some(true),                // hardware_secure
                Some(b"gateway".to_vec()), // hardware_type
                Some(1),                   // hardware_id
                Some(12345),               // hardware_dev_eui
                Some(23456),               // hardware_lock_start_block
                Some(34567),               // hardware_lock_interval_blocks
            ));

            // Verify Storage
            assert_eq!(MiningConfigHardwareTestModule::mining_config_hardware_count(), 1);
            assert!(MiningConfigHardwareTestModule::mining_config_hardware(0).is_some());
            assert_eq!(MiningConfigHardwareTestModule::mining_config_hardware_owner(0), Some(0));
            assert_eq!(
                MiningConfigHardwareTestModule::mining_config_hardware_hardware_configs(0),
                Some(MiningConfigHardwareConfig {
                    hardware_secure: true,
                    hardware_type: b"gateway".to_vec(),
                    hardware_id: 1,
                    hardware_dev_eui: 12345,
                    hardware_lock_start_block: 23456,
                    hardware_lock_interval_blocks: 34567,
                })
            );

            // Create Mining Speed Boost Sampling Hardware Mining

            // Call Functions
            assert_ok!(MiningSamplingHardwareTestModule::create(Origin::signed(0)));
            assert_ok!(MiningSamplingHardwareTestModule::set_mining_samplings_hardware_samplings_config(
                Origin::signed(0),
                0,           // mining_sampling_hardware_id
                0,           // mining_sampling_hardware_sample_id
                Some(23456), // hardware_sample_block
                Some(1),     // hardware_sample_hardware_online
            ));
            assert_ok!(MiningSamplingHardwareTestModule::assign_sampling_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningSamplingHardwareTestModule::mining_samplings_hardware_count(), 1);
            assert!(MiningSamplingHardwareTestModule::mining_samplings_hardware(0).is_some());
            assert_eq!(MiningSamplingHardwareTestModule::mining_samplings_hardware_owner(0), Some(0));
            assert_eq!(
                MiningSamplingHardwareTestModule::mining_samplings_hardware_samplings_configs((0, 0)),
                Some(MiningSamplingHardwareConfig {
                    hardware_sample_block: 23456,       // hardware_sample_block
                    hardware_sample_hardware_online: 1  // hardware_sample_hardware_online
                })
            );

            // Create Mining Speed Boost Eligibility Hardware Mining

            // Call Functions
            assert_ok!(MiningEligibilityHardwareTestModule::create(Origin::signed(0)));
            // assert_eq!(
            //     MiningEligibilityTestModule::calculate_mining_eligibility_hardware_result(
            //         Origin::signed(0),
            //         0, // mining_config_hardware_id
            //         0, // mining_eligibility_hardware_id
            //     ),
            //     Some(
            //         MiningEligibilityHardwareResult {
            //             hardware_calculated_eligibility: 1.1
            //             // to determine eligibility for proportion (incase user moves funds around during lock
            // period)             hardware_uptime_percentage: 0.3,
            //             // hardware_block_audited: 123,
            //             // hardware_auditor_account_id: 123
            //         }
            //     )
            // ))

            // Override by DAO if necessary
            assert_ok!(MiningEligibilityHardwareTestModule::set_mining_eligibility_hardware_eligibility_result(
                Origin::signed(0),
                0,       // mining_config_hardware_id
                0,       // mining_eligibility_hardware_id
                Some(1), // mining_hardware_calculated_eligibility
                Some(1), /* mining_hardware_uptime_percentage
                          * 123, // mining_hardware_block_audited
                          * 123, // mining_hardware_auditor_account_id
                          * Some({
                          *     MiningEligibilityHardwareResult {
                          *         hardware_calculated_eligibility: 1,
                          *         // to determine eligibility for proportion (incase user moves funds around
                          * during lock period)         hardware_uptime_percentage: 1,
                          *         // hardware_block_audited: 123,
                          *         // hardware_auditor_account_id: 123
                          *     }
                          * }), */
            ));
            assert_ok!(MiningEligibilityHardwareTestModule::assign_eligibility_to_configuration(
                Origin::signed(0),
                0,
                0
            ));

            // Verify Storage
            assert_eq!(MiningEligibilityHardwareTestModule::mining_eligibility_hardware_count(), 1);
            assert!(MiningEligibilityHardwareTestModule::mining_eligibility_hardware(0).is_some());
            assert_eq!(MiningEligibilityHardwareTestModule::mining_eligibility_hardware_owner(0), Some(0));
            assert_eq!(
                MiningEligibilityHardwareTestModule::mining_eligibility_hardware_eligibility_results((0, 0)),
                Some(MiningEligibilityHardwareResult {
                    hardware_calculated_eligibility: 1,
                    // to determine eligibility for proportion (incase user moves funds around during lock period)
                    hardware_uptime_percentage: 1,
                    /* hardware_block_audited: 123,
                     * hardware_auditor_account_id: 123 */
                })
            );

            // Create Mining Speed Boost Claims Hardware Mining

            // // Call Functions
            assert_ok!(MiningClaimsHardwareTestModule::create(Origin::signed(0)));
            assert_ok!(MiningClaimsHardwareTestModule::assign_claim_to_configuration(Origin::signed(0), 0, 0));
            // assert_ok!(
            //     MiningClaimsHardwareTestModule::claim(
            //         Origin::signed(0),
            //         0, // mining_config_hardware_id
            //         0, // mining_eligibility_hardware_id
            //         0, // mining_claims_hardware_id
            //     )
            // )
            // Override by DAO if necessary
            assert_ok!(MiningClaimsHardwareTestModule::set_mining_claims_hardware_claims_result(
                Origin::signed(0),
                0,           // mining_config_hardware_id
                0,           // mining_eligibility_hardware_id
                0,           // mining_claims_hardware_id
                Some(1),     // hardware_claim_amount
                Some(34567), // hardware_claim_block_redeemed
            ));

            // Verify Storage
            assert_eq!(MiningClaimsHardwareTestModule::mining_claims_hardware_count(), 1);
            assert!(MiningClaimsHardwareTestModule::mining_claims_hardware(0).is_some());
            assert_eq!(MiningClaimsHardwareTestModule::mining_claims_hardware_owner(0), Some(0));
            assert_eq!(
                MiningClaimsHardwareTestModule::mining_claims_hardware_claims_results((0, 0)),
                Some(MiningClaimsHardwareClaimResult {
                    hardware_claim_amount: 1,
                    hardware_claim_block_redeemed: 34567,
                })
            );
        });
    }
}
