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
        assert_eq!(RoamingSessionModule::roaming_sessions_count(), 0);
        assert!(RoamingSessionModule::roaming_session(0).is_none());
        assert_eq!(RoamingSessionModule::roaming_session_owner(0), None);
        assert_eq!(Balances::free_balance(1), 10);
        assert_eq!(Balances::free_balance(2), 20);
    });
}

#[test]
fn create_works() {
    new_test_ext().execute_with(|| {
        // Call Functions
        assert_ok!(RoamingSessionModule::create(Origin::signed(1)));
        // Verify Storage
        assert_eq!(RoamingSessionModule::roaming_sessions_count(), 1);
        assert!(RoamingSessionModule::roaming_session(0).is_some());
        assert_eq!(RoamingSessionModule::roaming_session_owner(0), Some(1));
    });
}

#[test]
fn create_handles_basic_errors() {
    new_test_ext().execute_with(|| {
        // Setup
        <RoamingSessionsCount<Test>>::put(u64::max_value());
        // Call Functions
        assert_noop!(RoamingSessionModule::create(Origin::signed(1)), "RoamingSessions count overflow");
        // Verify Storage
        assert_eq!(RoamingSessionModule::roaming_sessions_count(), u64::max_value());
        assert!(RoamingSessionModule::roaming_session(0).is_none());
        assert_eq!(RoamingSessionModule::roaming_session_owner(0), None);
    });
}

#[test]
fn transfer_works() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(RoamingSessionModule::create(Origin::signed(1)));
        // Call Functions
        assert_ok!(RoamingSessionModule::transfer(Origin::signed(1), 2, 0));
        // Verify Storage
        assert_eq!(RoamingSessionModule::roaming_sessions_count(), 1);
        assert!(RoamingSessionModule::roaming_session(0).is_some());
        assert_eq!(RoamingSessionModule::roaming_session_owner(0), Some(2));
    });
}

#[test]
fn transfer_handles_basic_errors() {
    new_test_ext().execute_with(|| {
        // Setup
        assert_ok!(RoamingSessionModule::create(Origin::signed(1)));
        // Call Functions
        assert_noop!(
            RoamingSessionModule::transfer(Origin::signed(2), 2, 0),
            "Only owner can transfer roaming session"
        );
        assert_noop!(
            RoamingSessionModule::transfer(Origin::signed(1), 2, 1),
            "Only owner can transfer roaming session"
        );
        // Verify Storage
        assert_eq!(RoamingSessionModule::roaming_sessions_count(), 1);
        assert!(RoamingSessionModule::roaming_session(0).is_some());
        assert_eq!(RoamingSessionModule::roaming_session_owner(0), Some(1));
    });
}
