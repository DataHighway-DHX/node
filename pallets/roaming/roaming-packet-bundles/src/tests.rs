// Tests to be written here

use super::*;
use crate::mock;
use crate::mock::*;
use frame_support::{
    assert_noop,
    assert_ok,
};

#[test]
fn basic_setup_works() {
	new_test_ext().execute_with(|| {
		// Verify Initial Storage
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundles_count(), 0);
		assert!(RoamingPacketBundleModule::roaming_packet_bundle(0).is_none());
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_owner(0), None);
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_price(0), None);
		assert_eq!(Balances::free_balance(1), 10);
		assert_eq!(Balances::free_balance(2), 20);
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		// Call Functions
		assert_ok!(RoamingPacketBundleModule::create(Origin::signed(1)));
		// Verify Storage
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundles_count(), 1);
		assert!(RoamingPacketBundleModule::roaming_packet_bundle(0).is_some());
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_owner(0), Some(1));
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_price(0), None);
	});
}

#[test]
fn create_handles_basic_errors() {
	new_test_ext().execute_with(|| {
		// Setup
		<RoamingPacketBundlesCount<Test>>::put(u64::max_value());
		// Call Functions
		assert_noop!(RoamingPacketBundleModule::create(Origin::signed(1)), "RoamingPacketBundles count overflow");
		// Verify Storage
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundles_count(), u64::max_value());
		assert!(RoamingPacketBundleModule::roaming_packet_bundle(0).is_none());
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_owner(0), None);
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_price(0), None);
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		// Setup
		assert_ok!(RoamingPacketBundleModule::create(Origin::signed(1)));
		// Call Functions
		assert_ok!(RoamingPacketBundleModule::transfer(Origin::signed(1), 2, 0));
		// Verify Storage
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundles_count(), 1);
		assert!(RoamingPacketBundleModule::roaming_packet_bundle(0).is_some());
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_owner(0), Some(2));
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_price(0), None);
	});
}

#[test]
fn transfer_handles_basic_errors() {
	new_test_ext().execute_with(|| {
		// Setup
		assert_ok!(RoamingPacketBundleModule::create(Origin::signed(1)));
		// Call Functions
		assert_noop!(
			RoamingPacketBundleModule::transfer(Origin::signed(2), 2, 0),
			"Only owner can transfer roaming packet_bundle"
		);
		assert_noop!(
			RoamingPacketBundleModule::transfer(Origin::signed(1), 2, 1),
			"Only owner can transfer roaming packet_bundle"
		);
		// Verify Storage
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundles_count(), 1);
		assert!(RoamingPacketBundleModule::roaming_packet_bundle(0).is_some());
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_owner(0), Some(1));
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_price(0), None);
	});
}

#[test]
fn set_price_works() {
	new_test_ext().execute_with(|| {
		// Setup
		assert_ok!(RoamingPacketBundleModule::create(Origin::signed(1)));
		// Call Functions
		assert_ok!(RoamingPacketBundleModule::set_price(Origin::signed(1), 0, Some(10)));
		// Verify Storage
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundles_count(), 1);
		assert!(RoamingPacketBundleModule::roaming_packet_bundle(0).is_some());
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_owner(0), Some(1));
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_price(0), Some(10));
	});
}

#[test]
fn buy_works() {
	new_test_ext().execute_with(|| {
		// Setup
		assert_ok!(RoamingPacketBundleModule::create(Origin::signed(1)));
		assert_ok!(RoamingPacketBundleModule::set_price(Origin::signed(1), 0, Some(10)));
		// Call Functions
		assert_ok!(RoamingPacketBundleModule::buy(Origin::signed(2), 0, 10));
		// Verify Storage
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundles_count(), 1);
		assert!(RoamingPacketBundleModule::roaming_packet_bundle(0).is_some());
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_owner(0), Some(2));
		assert_eq!(RoamingPacketBundleModule::roaming_packet_bundle_price(0), None);
		assert_eq!(Balances::free_balance(1), 20);
		assert_eq!(Balances::free_balance(2), 10);
	});
}
