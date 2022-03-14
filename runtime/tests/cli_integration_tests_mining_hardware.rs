// extern crate env as env;
extern crate mining_claims_hardware as mining_claims_hardware;
extern crate mining_setting_hardware as mining_setting_hardware;
extern crate mining_eligibility_hardware as mining_eligibility_hardware;
extern crate mining_rates_hardware as mining_rates_hardware;
extern crate mining_sampling_hardware as mining_sampling_hardware;
extern crate roaming_operators as roaming_operators;

#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_ok,
        parameter_types,
        traits::{
            ConstU8,
            ConstU16,
            ConstU32,
            ConstU64,
            ConstU128,
        },
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
    };
    pub use pallet_transaction_payment::{
        CurrencyAdapter,
    };
    // Import Config for each runtime module being tested
    use mining_claims_hardware::{
        MiningClaimsHardwareClaimResult,
        Module as MiningClaimsHardwareModule,
        Config as MiningClaimsHardwareConfig,
    };
    use mining_setting_hardware::{
        MiningSettingHardwareSetting,
        Module as MiningSettingHardwareModule,
        Config as MiningSettingHardwareConfig,
    };
    use mining_eligibility_hardware::{
        MiningEligibilityHardwareResult,
        Module as MiningEligibilityHardwareModule,
        Config as MiningEligibilityHardwareConfig,
    };
    use mining_rates_hardware::{
        MiningRatesHardwareSetting,
        Module as MiningRatesHardwareModule,
        Config as MiningRatesHardwareConfig,
    };
    use mining_sampling_hardware::{
        MiningSamplingHardwareSetting,
        Module as MiningSamplingHardwareModule,
        Config as MiningSamplingHardwareConfig,
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
            System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
            Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
            RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
            TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
        }
    );

    parameter_types! {
        pub const BlockHashCount: u32 = 250;
        pub const SS58Prefix: u16 = 33;
    }
    impl frame_system::Config for Test {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type Origin = Origin;
        type Call = Call;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u128; // u64 is not enough to hold bytes used to generate bounty account
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = ();
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = pallet_balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
    	type OnSetCode = ();
	    type MaxConsumers = frame_support::traits::ConstU32<16>;
    }
    impl pallet_randomness_collective_flip::Config for Test {}
    pub const EXISTENTIAL_DEPOSIT_AS_CONST: u64 = 1;
    parameter_types! {
        pub const ExistentialDeposit: u64 = EXISTENTIAL_DEPOSIT_AS_CONST;
    }
    impl pallet_balances::Config for Test {
        type MaxLocks = ();
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT_AS_CONST>;
        type AccountStore = System;
        type WeightInfo = ();
    }
    impl pallet_transaction_payment::Config for Test {
        type FeeMultiplierUpdate = ();
        type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
        type TransactionByteFee = ();
        type OperationalFeeMultiplier = ();
        type WeightToFee = IdentityFee<u64>;
    }
    // FIXME - remove this when figure out how to use these types within mining-speed-boost runtime module itself
    impl roaming_operators::Config for Test {
        type Currency = Balances;
        type Event = ();
        type Randomness = RandomnessCollectiveFlip;
        type RoamingOperatorIndex = u64;
    }
    impl MiningSettingHardwareConfig for Test {
        type Event = ();
        type MiningSettingHardwareDevEUI = u64;
        // type MiningSettingHardwareType =
        // MiningSettingHardwareTypes;
        type MiningSettingHardwareID = u64;
        // type Currency = Balances;
        // type Randomness = Randomness;
        type MiningSettingHardwareIndex = u64;
        // Mining Speed Boost Hardware Mining Config
        type MiningSettingHardwareSecure = bool;
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningSettingHardwareType = Vec<u8>;
    }
    impl MiningRatesHardwareConfig for Test {
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
    impl MiningSamplingHardwareConfig for Test {
        type Event = ();
        type MiningSamplingHardwareIndex = u64;
        type MiningSamplingHardwareSampleHardwareOnline = u64;
    }
    impl MiningEligibilityHardwareConfig for Test {
        type Event = ();
        type MiningEligibilityHardwareCalculatedEligibility = u64;
        type MiningEligibilityHardwareIndex = u64;
        type MiningEligibilityHardwareUptimePercentage = u32;
        // type MiningEligibilityHardwareAuditorAccountID = u64;
    }
    impl MiningClaimsHardwareConfig for Test {
        type Event = ();
        type MiningClaimsHardwareClaimAmount = u64;
        type MiningClaimsHardwareIndex = u64;
    }

    pub type MiningSettingHardwareTestModule = MiningSettingHardwareModule<Test>;
    pub type MiningRatesHardwareTestModule = MiningRatesHardwareModule<Test>;
    pub type MiningSamplingHardwareTestModule = MiningSamplingHardwareModule<Test>;
    pub type MiningEligibilityHardwareTestModule = MiningEligibilityHardwareModule<Test>;
    pub type MiningClaimsHardwareTestModule = MiningClaimsHardwareModule<Test>;
    pub type Randomness = pallet_randomness_collective_flip::Pallet<Test>;

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
                Some(MiningRatesHardwareSetting {
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
            assert_ok!(MiningSettingHardwareTestModule::create(Origin::signed(0)));
            assert_ok!(MiningSettingHardwareTestModule::set_mining_setting_hardware_hardware_config(
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
            assert_eq!(MiningSettingHardwareTestModule::mining_setting_hardware_count(), 1);
            assert!(MiningSettingHardwareTestModule::mining_setting_hardware(0).is_some());
            assert_eq!(MiningSettingHardwareTestModule::mining_setting_hardware_owner(0), Some(0));
            assert_eq!(
                MiningSettingHardwareTestModule::mining_setting_hardware_hardware_configs(0),
                Some(MiningSettingHardwareSetting {
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
                Some(MiningSamplingHardwareSetting {
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
            //         0, // mining_setting_hardware_id
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
                0,       // mining_setting_hardware_id
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
            //         0, // mining_setting_hardware_id
            //         0, // mining_eligibility_hardware_id
            //         0, // mining_claims_hardware_id
            //     )
            // )
            // Override by DAO if necessary
            assert_ok!(MiningClaimsHardwareTestModule::set_mining_claims_hardware_claims_result(
                Origin::signed(0),
                0,           // mining_setting_hardware_id
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
