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
