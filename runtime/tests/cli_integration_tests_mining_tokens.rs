// extern crate env as env;
extern crate mining_speed_boosts_configuration_token_mining as mining_speed_boosts_configuration_token_mining;
extern crate mining_speed_boosts_eligibility_token_mining as mining_speed_boosts_eligibility_token_mining;
extern crate mining_speed_boosts_lodgements_token_mining as mining_speed_boosts_lodgements_token_mining;
extern crate mining_speed_boosts_rates_token_mining as mining_speed_boosts_rates_token_mining;
extern crate mining_speed_boosts_sampling_token_mining as mining_speed_boosts_sampling_token_mining;
extern crate roaming_operators as roaming_operators;

#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{
        assert_noop,
        assert_ok,
        impl_outer_origin,
        parameter_types,
        weights::Weight,
    };
    use frame_system::{self as system,};
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{
            BlakeTwo256,
            IdentityLookup,
            OnFinalize,
            Zero,
        },
        DispatchResult,
        Perbill,
        Permill,
    };
    // Import Trait for each runtime module being tested
    use mining_speed_boosts_configuration_token_mining::{
        MiningSpeedBoostConfigurationTokenMining,
        MiningSpeedBoostConfigurationTokenMiningTokenConfig,
        Module as MiningSpeedBoostConfigurationTokenMiningModule,
        Trait as MiningSpeedBoostConfigurationTokenMiningTrait,
    };
    use mining_speed_boosts_eligibility_token_mining::{
        MiningSpeedBoostEligibilityTokenMining,
        MiningSpeedBoostEligibilityTokenMiningEligibilityResult,
        Module as MiningSpeedBoostEligibilityTokenMiningModule,
        Trait as MiningSpeedBoostEligibilityTokenMiningTrait,
    };
    use mining_speed_boosts_lodgements_token_mining::{
        MiningSpeedBoostLodgementsTokenMining,
        MiningSpeedBoostLodgementsTokenMiningLodgementResult,
        Module as MiningSpeedBoostLodgementsTokenMiningModule,
        Trait as MiningSpeedBoostLodgementsTokenMiningTrait,
    };
    use mining_speed_boosts_rates_token_mining::{
        MiningSpeedBoostRatesTokenMining,
        MiningSpeedBoostRatesTokenMiningRatesConfig,
        Module as MiningSpeedBoostRatesTokenMiningModule,
        Trait as MiningSpeedBoostRatesTokenMiningTrait,
    };
    use mining_speed_boosts_sampling_token_mining::{
        MiningSpeedBoostSamplingTokenMining,
        MiningSpeedBoostSamplingTokenMiningSamplingConfig,
        Module as MiningSpeedBoostSamplingTokenMiningModule,
        Trait as MiningSpeedBoostSamplingTokenMiningTrait,
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
        type OnKilledAccount = ();
        type OnNewAccount = ();
        type Origin = Origin;
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
    }
    impl pallet_transaction_payment::Trait for Test {
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
    impl MiningSpeedBoostConfigurationTokenMiningTrait for Test {
        type Event = ();
        // type Currency = Balances;
        // type Randomness = Randomness;
        type MiningSpeedBoostConfigurationTokenMiningIndex = u64;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriod = u32;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodEndDate = u64;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockPeriodStartDate = u64;
        // type MiningSpeedBoostConfigurationTokenMiningTokenType = MiningSpeedBoostConfigurationTokenMiningTokenTypes;
        type MiningSpeedBoostConfigurationTokenMiningTokenLockedAmount = u64;
        // Mining Speed Boost Token Mining Config
        // FIXME - how to use this enum from std? (including importing `use std::str::FromStr;`)
        type MiningSpeedBoostConfigurationTokenMiningTokenType = Vec<u8>;
    }
    impl MiningSpeedBoostRatesTokenMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostRatesTokenMiningIndex = u64;
        type MiningSpeedBoostRatesTokenMiningMaxLoyalty = u32;
        // Mining Speed Boost Max Rates
        type MiningSpeedBoostRatesTokenMiningMaxToken = u32;
        type MiningSpeedBoostRatesTokenMiningTokenDOT = u32;
        type MiningSpeedBoostRatesTokenMiningTokenIOTA = u32;
        // Mining Speed Boost Rate
        type MiningSpeedBoostRatesTokenMiningTokenMXC = u32;
    }
    impl MiningSpeedBoostSamplingTokenMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostSamplingTokenMiningIndex = u64;
        type MiningSpeedBoostSamplingTokenMiningSampleDate = u64;
        type MiningSpeedBoostSamplingTokenMiningSampleTokensLocked = u64;
    }
    impl MiningSpeedBoostEligibilityTokenMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostEligibilityTokenMiningCalculatedEligibility = u64;
        type MiningSpeedBoostEligibilityTokenMiningIndex = u64;
        type MiningSpeedBoostEligibilityTokenMiningTokenLockedPercentage = u32;
        // type MiningSpeedBoostEligibilityTokenMiningDateAudited = u64;
        // type MiningSpeedBoostEligibilityTokenMiningAuditorAccountID = u64;
    }
    impl MiningSpeedBoostLodgementsTokenMiningTrait for Test {
        type Event = ();
        type MiningSpeedBoostLodgementsTokenMiningIndex = u64;
        type MiningSpeedBoostLodgementsTokenMiningLodgementAmount = u64;
        type MiningSpeedBoostLodgementsTokenMiningLodgementDateRedeemed = u64;
    }

    type System = frame_system::Module<Test>;
    pub type Balances = pallet_balances::Module<Test>;
    pub type MiningSpeedBoostConfigurationTokenMiningTestModule = MiningSpeedBoostConfigurationTokenMiningModule<Test>;
    pub type MiningSpeedBoostRatesTokenMiningTestModule = MiningSpeedBoostRatesTokenMiningModule<Test>;
    pub type MiningSpeedBoostSamplingTokenMiningTestModule = MiningSpeedBoostSamplingTokenMiningModule<Test>;
    pub type MiningSpeedBoostEligibilityTokenMiningTestModule = MiningSpeedBoostEligibilityTokenMiningModule<Test>;
    pub type MiningSpeedBoostLodgementsTokenMiningTestModule = MiningSpeedBoostLodgementsTokenMiningModule<Test>;
    type Randomness = pallet_randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    pub fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        pallet_balances::GenesisConfig::<Test> {
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

            // Create Mining Speed Boost Rates Token Mining

            // Call Functions
            assert_ok!(MiningSpeedBoostRatesTokenMiningTestModule::create(Origin::signed(0)));
            assert_ok!(
              MiningSpeedBoostRatesTokenMiningTestModule::set_mining_speed_boosts_rates_token_mining_rates_config(
                Origin::signed(0),
                0, // mining_speed_boosts_rates_token_mining_id
                // FIXME - convert all below types to Vec<u8> since float values? i.e. b"1.025".to_vec()
                Some(1), // token_token_mxc
                Some(1), // token_token_iota
                Some(1), // token_token_dot
                Some(1), // token_max_token
                Some(1), // token_max_loyalty
              )
            );

            // Verify Storage
            assert_eq!(MiningSpeedBoostRatesTokenMiningTestModule::mining_speed_boosts_rates_token_mining_count(), 1);
            assert!(MiningSpeedBoostRatesTokenMiningTestModule::mining_speed_boosts_rates_token_mining(0).is_some());
            assert_eq!(MiningSpeedBoostRatesTokenMiningTestModule::mining_speed_boosts_rates_token_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostRatesTokenMiningTestModule::mining_speed_boosts_rates_token_mining_rates_configs(0),
                Some(MiningSpeedBoostRatesTokenMiningRatesConfig {
                    token_token_mxc: 1,
                    token_token_iota: 1,
                    token_token_dot: 1,
                    token_max_token: 1,
                    token_max_loyalty: 1,
                })
            );

            // Create Mining Speed Boost Configuration Token Mining

            // Call Functions
            assert_ok!(MiningSpeedBoostConfigurationTokenMiningTestModule::create(Origin::signed(0)));
            assert_ok!(
              MiningSpeedBoostConfigurationTokenMiningTestModule::set_mining_speed_boosts_configuration_token_mining_token_config(
                Origin::signed(0),
                0, // mining_speed_boosts_token_mining_id
                Some(b"MXC".to_vec()), // token_type
                Some(100), // token_locked_amount
                Some(15), // token_lock_period
                Some(12345), // token_lock_period_start_date
                Some(23456), // token_lock_period_end_date
              )
            );

            // Verify Storage
            assert_eq!(MiningSpeedBoostConfigurationTokenMiningTestModule::mining_speed_boosts_configuration_token_mining_count(), 1);
            assert!(MiningSpeedBoostConfigurationTokenMiningTestModule::mining_speed_boosts_configuration_token_mining(0).is_some());
            assert_eq!(MiningSpeedBoostConfigurationTokenMiningTestModule::mining_speed_boosts_configuration_token_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostConfigurationTokenMiningTestModule::mining_speed_boosts_configuration_token_mining_token_configs(0),
                Some(MiningSpeedBoostConfigurationTokenMiningTokenConfig {
                    token_type: b"MXC".to_vec(), // token_type
                    token_locked_amount: 100, // token_locked_amount
                    token_lock_period: 15, // token_lock_period
                    token_lock_period_start_date: 12345, // token_lock_period_start_date
                    token_lock_period_end_date: 23456, // token_lock_period_end_date
                })
            );

            // Create Mining Speed Boost Sampling Token Mining

            // Call Functions
            assert_ok!(MiningSpeedBoostSamplingTokenMiningTestModule::create(Origin::signed(0)));
            assert_ok!(
              MiningSpeedBoostSamplingTokenMiningTestModule::set_mining_speed_boosts_samplings_token_mining_samplings_config(
                Origin::signed(0),
                0, // mining_speed_boosts_token_mining_id
                0, // mining_speed_boosts_token_mining_sample_id
                Some(23456), // token_sample_date
                Some(100), // token_sample_tokens_locked
              )
            );
            assert_ok!(MiningSpeedBoostSamplingTokenMiningTestModule::assign_sampling_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningSpeedBoostSamplingTokenMiningTestModule::mining_speed_boosts_samplings_token_mining_count(), 1);
            assert!(MiningSpeedBoostSamplingTokenMiningTestModule::mining_speed_boosts_samplings_token_mining(0).is_some());
            assert_eq!(MiningSpeedBoostSamplingTokenMiningTestModule::mining_speed_boosts_samplings_token_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostSamplingTokenMiningTestModule::mining_speed_boosts_samplings_token_mining_samplings_configs((0, 0)),
                Some(MiningSpeedBoostSamplingTokenMiningSamplingConfig {
                    token_sample_date: 23456, // token_sample_date
                    token_sample_tokens_locked: 100 // token_sample_tokens_locked
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
            // mining_speed_boosts_configuration_token_mining_id

            // Call Functions
            assert_ok!(MiningSpeedBoostEligibilityTokenMiningTestModule::create(Origin::signed(0)));
            // assert_eq!(
            //   MiningSpeedBoostEligibilityTokenMiningTestModule::calculate_mining_speed_boosts_eligibility_token_mining_result(
            //       Origin::signed(0),
            //       0, // mining_speed_boosts_configuration_token_mining_id
            //       0, // mining_speed_boosts_eligibility_token_mining_id
            //   ),
            //   Some(
            //     MiningSpeedBoostEligibilityTokenMiningEligibilityResult {
            //       eligibility_token_mining_calculated_eligibility: 1.1
            //       // to determine eligibility for proportion (incase user hardware is not online around during the whole lock period)
            //       eligibility_token_mining_token_locked_percentage: 0.7, // i.e. 70%
            //       // eligibility_token_mining_date_audited: 123,
            //       // eligibility_token_mining_auditor_account_id: 123
            //     }
            //   )
            // ))

            // Override by DAO if necessary
            assert_ok!(
              MiningSpeedBoostEligibilityTokenMiningTestModule::set_mining_speed_boosts_eligibility_token_mining_eligibility_result(
                Origin::signed(0),
                0, // mining_speed_boosts_configuration_token_mining_id
                0, // mining_speed_boosts_eligibility_token_mining_id
                Some(1), // mining_speed_boosts_eligibility_token_mining_calculated_eligibility
                Some(1), // mining_speed_boosts_eligibility_token_mining_token_locked_percentage
                // 123, // mining_speed_boosts_eligibility_token_mining_date_audited
                // 123, // mining_speed_boosts_eligibility_token_mining_auditor_account_id
                // Some({
                //   MiningSpeedBoostEligibilityTokenMiningEligibilityResult {
                //     eligibility_token_mining_calculated_eligibility: 1, // i.e. 1.1
                //     // to determine eligibility for proportion (incase user hardware is not online around during the whole lock period)
                //     eligibility_token_mining_token_locked_percentage: 1, // i.e. 0.7 for 70%
                //     // eligibility_token_mining_date_audited: 123,
                //     // eligibility_token_mining_auditor_account_id: 123
                //   }
                // }),
              )
            );
            assert_ok!(MiningSpeedBoostEligibilityTokenMiningTestModule::assign_eligibility_to_configuration(Origin::signed(0), 0, 0));

            // Verify Storage
            assert_eq!(MiningSpeedBoostEligibilityTokenMiningTestModule::mining_speed_boosts_eligibility_token_mining_count(), 1);
            assert!(MiningSpeedBoostEligibilityTokenMiningTestModule::mining_speed_boosts_eligibility_token_mining(0).is_some());
            assert_eq!(MiningSpeedBoostEligibilityTokenMiningTestModule::mining_speed_boosts_eligibility_token_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostEligibilityTokenMiningTestModule::mining_speed_boosts_eligibility_token_mining_eligibility_results((0, 0)),
                Some(MiningSpeedBoostEligibilityTokenMiningEligibilityResult {
                    eligibility_token_mining_calculated_eligibility: 1,
                    // to determine eligibility for proportion (incase user hardware is not online around during the whole lock period)
                    eligibility_token_mining_token_locked_percentage: 1, // i.e. 70%
                    // eligibility_token_mining_date_audited: 123,
                    // eligibility_token_mining_auditor_account_id: 123
                })
            );

            // Create Mining Speed Boost Lodgements Token Mining

            // Call Functions
            assert_ok!(MiningSpeedBoostLodgementsTokenMiningTestModule::create(Origin::signed(0)));
            assert_ok!(MiningSpeedBoostLodgementsTokenMiningTestModule::assign_claim_to_configuration(Origin::signed(0), 0, 0));
            assert_ok!(
                MiningSpeedBoostLodgementsTokenMiningTestModule::claim(
                    Origin::signed(0),
                    0, // mining_speed_boosts_configuration_token_mining_id
                    0, // mining_speed_boosts_eligibility_token_mining_id
                    0, // mining_speed_boosts_lodgements_token_mining_id
                )
            );
            // Override by DAO if necessary
            assert_ok!(
              MiningSpeedBoostLodgementsTokenMiningTestModule::set_mining_speed_boosts_lodgements_token_mining_lodgements_result(
                  Origin::signed(0),
                  0, // mining_speed_boosts_configuration_token_mining_id
                  0, // mining_speed_boosts_eligibility_token_mining_id
                  0, // mining_speed_boosts_lodgements_token_mining_id
                  Some(1), // hardware_claim_amount
                  Some(34567) // hardware_claim_date_redeemed
              )
            );

            // Verify Storage
            assert_eq!(MiningSpeedBoostLodgementsTokenMiningTestModule::mining_speed_boosts_lodgements_token_mining_count(), 1);
            assert!(MiningSpeedBoostLodgementsTokenMiningTestModule::mining_speed_boosts_lodgements_token_mining(0).is_some());
            assert_eq!(MiningSpeedBoostLodgementsTokenMiningTestModule::mining_speed_boosts_lodgements_token_mining_owner(0), Some(0));
            assert_eq!(
              MiningSpeedBoostLodgementsTokenMiningTestModule::mining_speed_boosts_lodgements_token_mining_lodgements_results((0, 0)),
                Some(MiningSpeedBoostLodgementsTokenMiningLodgementResult {
                    token_claim_amount: 1,
                    token_claim_date_redeemed: 34567,
                })
            );
        });
    }
}
