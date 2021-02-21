// extern crate env as env;
extern crate mining_config_hardware_mining as mining_config_hardware_mining;
extern crate mining_eligibility_hardware_mining as mining_eligibility_hardware_mining;
extern crate mining_claims_hardware_mining as mining_claims_hardware_mining;
extern crate mining_rates_hardware_mining as mining_rates_hardware_mining;
extern crate mining_sampling_hardware_mining as mining_sampling_hardware_mining;
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
    use mining_config_hardware_mining::{
        MiningConfigHardwareMiningHardwareConfig,
        Module as MiningConfigHardwareMiningModule,
        Trait as MiningConfigHardwareMiningTrait,
    };
    use mining_eligibility_hardware_mining::{
        MiningEligibilityHardwareMiningEligibilityResult,
        Module as MiningEligibilityHardwareMiningModule,
        Trait as MiningEligibilityHardwareMiningTrait,
    };
    use mining_claims_hardware_mining::{
        MiningClaimsHardwareMiningClaimResult,
        Module as MiningClaimsHardwareMiningModule,
        Trait as MiningClaimsHardwareMiningTrait,
    };
    use mining_rates_hardware_mining::{
        MiningRatesHardwareMiningRatesConfig,
        Module as MiningRatesHardwareMiningModule,
        Trait as MiningRatesHardwareMiningTrait,
    };
    use mining_sampling_hardware_mining::{
        MiningSamplingHardwareMiningSamplingConfig,
        Module as MiningSamplingHardwareMiningModule,
        Trait as MiningSamplingHardwareMiningTrait,
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
    impl MiningConfigHardwareMiningTrait for Test {
        type Event = ();
        type MiningConfigHardwareMiningHardwareDevEUI = u64;
        // type MiningConfigHardwareMiningHardwareType =
        // MiningConfigHardwareMiningHardwareTypes;
        type MiningConfigHardwareMiningHardwareID = u64;
        // Mining Speed Boost Hardware Mining Config
        type MiningConfigHardwareMiningHardwareSecure = bool;
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningConfigHardwareMiningHardwareType = Vec<u8>;
        // type Currency = Balances;
        // type Randomness = Randomness;
        type MiningConfigHardwareMiningIndex = u64;
    }
    impl MiningRatesHardwareMiningTrait for Test {
        type Event = ();
        type MiningRatesHardwareMiningCategory1MaxTokenBonusPerGateway = u32;
        type MiningRatesHardwareMiningCategory2MaxTokenBonusPerGateway = u32;
        type MiningRatesHardwareMiningCategory3MaxTokenBonusPerGateway = u32;
        type MiningRatesHardwareMiningHardwareInsecure = u32;
        // Mining Speed Boost Rate
        type MiningRatesHardwareMiningHardwareSecure = u32;
        type MiningRatesHardwareMiningIndex = u64;
        // Mining Speed Boost Max Rates
        type MiningRatesHardwareMiningMaxHardware = u32;
    }
    impl MiningSamplingHardwareMiningTrait for Test {
        type Event = ();
        type MiningSamplingHardwareMiningIndex = u64;
        type MiningSamplingHardwareMiningSampleHardwareOnline = u64;
    }
    impl MiningEligibilityHardwareMiningTrait for Test {
        type Event = ();
        type MiningEligibilityHardwareMiningCalculatedEligibility = u64;
        type MiningEligibilityHardwareMiningHardwareUptimePercentage = u32;
        type MiningEligibilityHardwareMiningIndex = u64;
        // type MiningEligibilityHardwareMiningAuditorAccountID = u64;
    }
    impl MiningClaimsHardwareMiningTrait for Test {
        type Event = ();
        type MiningClaimsHardwareMiningIndex = u64;
        type MiningClaimsHardwareMiningClaimAmount = u64;
    }

    type System = frame_system::Module<Test>;
    pub type Balances = pallet_balances::Module<Test>;
    pub type MiningConfigHardwareMiningTestModule =
        MiningConfigHardwareMiningModule<Test>;
    pub type MiningRatesHardwareMiningTestModule = MiningRatesHardwareMiningModule<Test>;
    pub type MiningSamplingHardwareMiningTestModule = MiningSamplingHardwareMiningModule<Test>;
    pub type MiningEligibilityHardwareMiningTestModule =
        MiningEligibilityHardwareMiningModule<Test>;
    pub type MiningClaimsHardwareMiningTestModule = MiningClaimsHardwareMiningModule<Test>;
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
            assert_ok!(MiningRatesHardwareMiningTestModule::create(Origin::signed(0)));
            assert_ok!(
              MiningRatesHardwareMiningTestModule::set_mining_rates_hardware_mining_rates_config(
                Origin::signed(0),
                0, // mining_rates_hardware_mining_id
                // FIXME - convert all below types to Vec<u8> since float values? i.e. b"1.025".to_vec()
                Some(1), // hardware_hardware_secure
                Some(1), // hardware_hardware_insecure
                Some(1), // hardware_max_hardware
                Some(1000000),
                Some(500000),
                Some(250000)
              )
            );

            // Verify Storage
            assert_eq!(MiningRatesHardwareMiningTestModule::mining_rates_hardware_mining_count(), 1);
            assert!(MiningRatesHardwareMiningTestModule::mining_rates_hardware_mining(0).is_some());
            assert_eq!(MiningRatesHardwareMiningTestModule::mining_rates_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningRatesHardwareMiningTestModule::mining_rates_hardware_mining_rates_configs(0),
                Some(MiningRatesHardwareMiningRatesConfig {
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
            assert_ok!(MiningConfigHardwareMiningTestModule::create(Origin::signed(0)));
            assert_ok!(
              MiningConfigHardwareMiningTestModule::set_mining_config_hardware_mining_hardware_config(
                Origin::signed(0),
                0, // mining_hardware_mining_id
                Some(true), // hardware_secure
                Some(b"gateway".to_vec()), // hardware_type
                Some(1), // hardware_id
                Some(12345), // hardware_dev_eui
                Some(23456), // hardware_lock_start_block
                Some(34567), // hardware_lock_interval_blocks
              )
            );

            // Verify Storage
            assert_eq!(MiningConfigHardwareMiningTestModule::mining_config_hardware_mining_count(), 1);
            assert!(MiningConfigHardwareMiningTestModule::mining_config_hardware_mining(0).is_some());
            assert_eq!(MiningConfigHardwareMiningTestModule::mining_config_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningConfigHardwareMiningTestModule::mining_config_hardware_mining_hardware_configs(0),
                Some(MiningConfigHardwareMiningHardwareConfig {
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
            assert_ok!(MiningSamplingHardwareMiningTestModule::create(Origin::signed(0)));
            assert_ok!(
                MiningSamplingHardwareMiningTestModule::set_mining_samplings_hardware_mining_samplings_config(
                    Origin::signed(0),
                    0, // mining_sampling_hardware_mining_id
                    0, // mining_sampling_hardware_mining_sample_id
                    Some(23456), // hardware_sample_block
                    Some(1), // hardware_sample_hardware_online
                )
            );
            assert_ok!(MiningSamplingHardwareMiningTestModule::assign_sampling_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningSamplingHardwareMiningTestModule::mining_samplings_hardware_mining_count(), 1);
            assert!(MiningSamplingHardwareMiningTestModule::mining_samplings_hardware_mining(0).is_some());
            assert_eq!(MiningSamplingHardwareMiningTestModule::mining_samplings_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningSamplingHardwareMiningTestModule::mining_samplings_hardware_mining_samplings_configs((0, 0)),
                Some(MiningSamplingHardwareMiningSamplingConfig {
                    hardware_sample_block: 23456, // hardware_sample_block
                    hardware_sample_hardware_online: 1 // hardware_sample_hardware_online
                })
            );

            // Create Mining Speed Boost Eligibility Hardware Mining

            // Call Functions
            assert_ok!(MiningEligibilityHardwareMiningTestModule::create(Origin::signed(0)));
            // assert_eq!(
            //     MiningEligibilityTestModule::calculate_mining_eligibility_hardware_mining_result(
            //         Origin::signed(0),
            //         0, // mining_config_hardware_mining_id
            //         0, // mining_eligibility_hardware_mining_id
            //     ),
            //     Some(
            //         MiningEligibilityHardwareMiningEligibilityResult {
            //             hardware_calculated_eligibility: 1.1
            //             // to determine eligibility for proportion (incase user moves funds around during lock period)
            //             hardware_uptime_percentage: 0.3,
            //             // hardware_block_audited: 123,
            //             // hardware_auditor_account_id: 123
            //         }
            //     )
            // ))

            // Override by DAO if necessary
            assert_ok!(
                MiningEligibilityHardwareMiningTestModule::set_mining_eligibility_hardware_mining_eligibility_result(
                    Origin::signed(0),
                    0, // mining_config_hardware_mining_id
                    0, // mining_eligibility_hardware_mining_id
                    Some(1), // mining_hardware_calculated_eligibility
                    Some(1), // mining_hardware_uptime_percentage
                    // 123, // mining_hardware_block_audited
                    // 123, // mining_hardware_auditor_account_id
                    // Some({
                    //     MiningEligibilityHardwareMiningEligibilityResult {
                    //         hardware_calculated_eligibility: 1,
                    //         // to determine eligibility for proportion (incase user moves funds around during lock period)
                    //         hardware_uptime_percentage: 1,
                    //         // hardware_block_audited: 123,
                    //         // hardware_auditor_account_id: 123
                    //     }
                    // }),
                )
            );
            assert_ok!(MiningEligibilityHardwareMiningTestModule::assign_eligibility_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningEligibilityHardwareMiningTestModule::mining_eligibility_hardware_mining_count(), 1);
            assert!(MiningEligibilityHardwareMiningTestModule::mining_eligibility_hardware_mining(0).is_some());
            assert_eq!(MiningEligibilityHardwareMiningTestModule::mining_eligibility_hardware_mining_owner(0), Some(0));
            assert_eq!(
                MiningEligibilityHardwareMiningTestModule::mining_eligibility_hardware_mining_eligibility_results((0, 0)),
                Some(MiningEligibilityHardwareMiningEligibilityResult {
                    hardware_calculated_eligibility: 1,
                    // to determine eligibility for proportion (incase user moves funds around during lock period)
                    hardware_uptime_percentage: 1,
                    // hardware_block_audited: 123,
                    // hardware_auditor_account_id: 123
                })
            );

            // Create Mining Speed Boost Claims Hardware Mining

            // // Call Functions
            assert_ok!(MiningClaimsHardwareMiningTestModule::create(Origin::signed(0)));
            assert_ok!(MiningClaimsHardwareMiningTestModule::assign_claim_to_configuration(Origin::signed(0), 0, 0));
            // assert_ok!(
            //     MiningClaimsHardwareMiningTestModule::claim(
            //         Origin::signed(0),
            //         0, // mining_config_hardware_mining_id
            //         0, // mining_eligibility_hardware_mining_id
            //         0, // mining_claims_hardware_mining_id
            //     )
            // )
            // Override by DAO if necessary
            assert_ok!(
                MiningClaimsHardwareMiningTestModule::set_mining_claims_hardware_mining_claims_result(
                    Origin::signed(0),
                    0, // mining_config_hardware_mining_id
                    0, // mining_eligibility_hardware_mining_id
                    0, // mining_claims_hardware_mining_id
                    Some(1), // hardware_claim_amount
                    Some(34567), // hardware_claim_block_redeemed
                )
            );

            // Verify Storage
            assert_eq!(MiningClaimsHardwareMiningTestModule::mining_claims_hardware_mining_count(), 1);
            assert!(MiningClaimsHardwareMiningTestModule::mining_claims_hardware_mining(0).is_some());
            assert_eq!(MiningClaimsHardwareMiningTestModule::mining_claims_hardware_mining_owner(0), Some(0));
            assert_eq!(
              MiningClaimsHardwareMiningTestModule::mining_claims_hardware_mining_claims_results((0, 0)),
                Some(MiningClaimsHardwareMiningClaimResult {
                    hardware_claim_amount: 1,
                    hardware_claim_block_redeemed: 34567,
                })
            );
        });
    }
}
