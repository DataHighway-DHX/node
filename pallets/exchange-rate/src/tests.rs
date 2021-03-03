// Tests to be written here

use super::*;
use crate::mock::*;
use frame_support::{
    assert_noop,
    assert_ok,
};

#[test]
fn basic_setup_works() {
    new_test_ext().execute_with(|| {
        // Verify Initial Storage
        assert!(ExchangeRateTestModule::exchange_rates(0).is_none());
        assert_eq!(ExchangeRateTestModule::exchange_rate_owner(0), None);
        assert_eq!(ExchangeRateTestModule::exchange_rate_count(), 0);
        assert!(ExchangeRateTestModule::exchange_rate_configs(0).is_none());
    });
}

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        // Call Functions
        assert_ok!(ExchangeRateTestModule::create(Origin::signed(1)));
        // Verify Storage
        assert_eq!(ExchangeRateTestModule::exchange_rate_count(), 1);
        assert!(ExchangeRateTestModule::exchange_rates(0).is_some());
        assert_eq!(ExchangeRateTestModule::exchange_rate_owner(0), Some(1));
    });
}

#[test]
fn create_handles_basic_errors() {
    new_test_ext().execute_with(|| {
        // Setup
        <ExchangeRateCount<Test>>::put(u64::max_value());
        // Call Functions
        assert_noop!(ExchangeRateTestModule::create(Origin::signed(1)), "ExchangeRate count overflow");
        // Verify Storage
        assert_eq!(ExchangeRateTestModule::exchange_rate_count(), u64::max_value());
        assert!(ExchangeRateTestModule::exchange_rates(0).is_none());
        assert_eq!(ExchangeRateTestModule::exchange_rate_owner(0), None);
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(ExchangeRateTestModule::create(Origin::signed(1)));
        // Call Functions
        assert_ok!(ExchangeRateTestModule::transfer(Origin::signed(1), 2, 0));
        // Verify Storage
        assert_eq!(ExchangeRateTestModule::exchange_rate_count(), 1);
        assert!(ExchangeRateTestModule::exchange_rates(0).is_some());
        assert_eq!(ExchangeRateTestModule::exchange_rate_owner(0), Some(2));
    });
}

#[test]
fn transfer_handles_basic_errors() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(ExchangeRateTestModule::create(Origin::signed(1)));
        // Call Functions
        assert_noop!(
            ExchangeRateTestModule::transfer(Origin::signed(2), 2, 0),
            "Only owner can transfer exchange_rate"
        );
        assert_noop!(
            ExchangeRateTestModule::transfer(Origin::signed(1), 2, 1),
            "Only owner can transfer exchange_rate"
        );
        // Verify Storage
        assert_eq!(ExchangeRateTestModule::exchange_rate_count(), 1);
        assert!(ExchangeRateTestModule::exchange_rates(0).is_some());
        assert_eq!(ExchangeRateTestModule::exchange_rate_owner(0), Some(1));
    });
}

#[test]
fn set_config_defaults_works() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(ExchangeRateTestModule::create(Origin::signed(1)));
        // Call Functions
        assert_ok!(ExchangeRateTestModule::set_config(Origin::signed(1), 0, None, None, None, None, None));
        // Verify Storage
        assert_eq!(ExchangeRateTestModule::exchange_rate_count(), 1);
        assert!(ExchangeRateTestModule::exchange_rates(0).is_some());
        assert_eq!(ExchangeRateTestModule::exchange_rate_owner(0), Some(1));

        assert!(ExchangeRateTestModule::exchange_rate_configs(0).is_some());
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().hbtc, 200000);
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().dot, 100);
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().iota, 5);
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().fil, 200);
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().decimals_after_point, 2);
    });
}

#[test]
fn set_config_works() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(ExchangeRateTestModule::create(Origin::signed(1)));
        // Call Functions
        assert_ok!(ExchangeRateTestModule::set_config(
            Origin::signed(1),
            0,
            Some(777),
            Some(778),
            None,
            Some(779),
            Some(3)
        ));
        // Verify Storage
        assert_eq!(ExchangeRateTestModule::exchange_rate_count(), 1);
        assert!(ExchangeRateTestModule::exchange_rates(0).is_some());
        assert_eq!(ExchangeRateTestModule::exchange_rate_owner(0), Some(1));

        assert!(ExchangeRateTestModule::exchange_rate_configs(0).is_some());
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().hbtc, 777);
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().dot, 778);
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().iota, 5);
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().fil, 779);
        assert_eq!(ExchangeRateTestModule::exchange_rate_configs(0).unwrap().decimals_after_point, 3);
    });
}

#[test]
fn et_config_basic_errors() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(ExchangeRateTestModule::create(Origin::signed(1)));
        // Call Functions
        assert_noop!(
            ExchangeRateTestModule::set_config(Origin::signed(1), 1, None, None, None, None, None),
            "ExchangeRates does not exist"
        );
        assert_noop!(
            ExchangeRateTestModule::set_config(Origin::signed(2), 0, None, None, None, None, None),
            "Only owner can set exchange_rate_config"
        );
        // Verify Storage
        assert_eq!(ExchangeRateTestModule::exchange_rate_count(), 1);
        assert!(ExchangeRateTestModule::exchange_rates(0).is_some());
        assert_eq!(ExchangeRateTestModule::exchange_rate_owner(0), Some(1));
    });
}
