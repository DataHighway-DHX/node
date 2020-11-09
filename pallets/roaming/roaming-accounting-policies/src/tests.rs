// Tests to be written here

use super::*;
use crate::{
    mock::*,
};
use frame_support::{
    assert_noop,
    assert_ok,
};

#[test]
fn basic_setup_works() {
    new_test_ext().execute_with(|| {
        // Verify Initial Storage
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), 0);
        assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_none());
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), None);
        assert_eq!(Balances::free_balance(1), 10);
        assert_eq!(Balances::free_balance(2), 20);
    });
}

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        // Call Functions
        assert_ok!(RoamingAccountingPolicyModule::create(Origin::signed(1)));
        // Verify Storage
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), 1);
        assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_some());
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), Some(1));
    });
}

#[test]
fn create_handles_basic_errors() {
    new_test_ext().execute_with(|| {
        // Setup
        <RoamingAccountingPoliciesCount<Test>>::put(u64::max_value());
        // Call Functions
        assert_noop!(
            RoamingAccountingPolicyModule::create(Origin::signed(1)),
            "RoamingAccountingPolicies count overflow"
        );
        // Verify Storage
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), u64::max_value());
        assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_none());
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), None);
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(RoamingAccountingPolicyModule::create(Origin::signed(1)));
        // Call Functions
        assert_ok!(RoamingAccountingPolicyModule::transfer(Origin::signed(1), 2, 0));
        // Verify Storage
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), 1);
        assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_some());
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), Some(2));
    });
}

#[test]
fn transfer_handles_basic_errors() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(RoamingAccountingPolicyModule::create(Origin::signed(1)));
        // Call Functions
        assert_noop!(
            RoamingAccountingPolicyModule::transfer(Origin::signed(2), 2, 0),
            "Only owner can transfer roaming accounting_policy"
        );
        assert_noop!(
            RoamingAccountingPolicyModule::transfer(Origin::signed(1), 2, 1),
            "Only owner can transfer roaming accounting_policy"
        );
        // Verify Storage
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policies_count(), 1);
        assert!(RoamingAccountingPolicyModule::roaming_accounting_policy(0).is_some());
        assert_eq!(RoamingAccountingPolicyModule::roaming_accounting_policy_owner(0), Some(1));
    });
}
