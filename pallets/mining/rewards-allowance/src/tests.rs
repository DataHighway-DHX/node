use crate::{mock::*, Error};
use crate::{BondedDHXForAccountData};
use frame_support::{assert_noop, assert_ok};
use codec::{Encode};

#[test]
fn it_sets_rewards_allowance_with_timestamp() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // 27th August 2021 @ ~7am is 1630049371000
        // where milliseconds/day         86400000
        // 27th August 2021 @ 12am is 1630022400000 (start of day)
        Timestamp::set_timestamp(1630049371000u64);

        assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_allowance_dhx_current(
            Origin::signed(0),
            5_000u64
        ));

        assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_allowance_dhx_for_date(
            Origin::signed(0),
            5_000u64,
            1630049371000
        ));

        assert_ok!(MiningRewardsAllowanceTestModule::set_bonded_dhx_of_account_for_date(
            Origin::signed(0),
            1
        ));

        // Verify Storage
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_current(), Some(5_000u128));

        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date(1630022400000), Some(5_000u64));

        assert_eq!(
            MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date(1630022400000),
            Some(BondedDHXForAccountData {
                account_id: 1,
                bonded_dhx_current: 1_000u64,
                requestor_account_id: 0,
            })
        );

        assert_ok!(MiningRewardsAllowanceTestModule::change_remaining_rewards_allowance_dhx_for_date(
            Origin::signed(0),
            500,
            1630049371000,
            0
        ));

        // reducing the remaining rewards for a specific date does not change the default rewards allowance
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_current(), Some(5_000u128));

        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date(1630022400000), Some(4_500u64));

        assert_ok!(MiningRewardsAllowanceTestModule::change_remaining_rewards_allowance_dhx_for_date(
            Origin::signed(0),
            2000,
            1630049371000,
            1
        ));

        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date(1630022400000), Some(6_500u64));

        assert_ok!(Scheduler::cancel_named(1u32.encode(), Vec(1)).unwrap());
    })
}
