use crate::{mock::*, Error};
use crate::{BondedDHXForAccountData};
use codec::Encode;
use frame_support::{assert_noop, assert_ok};
use sp_core::{
    H256,
    Hasher, // so we may use BlakeTwo256::hash
};
use sp_runtime::{
	traits::{BlakeTwo256},
};

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
    })
}

#[test]
fn setup_preimage() {
	new_test_ext().execute_with(|| {
        System::set_block_number(0);
        // pre-image byte deposit with reasonable fee value of 1
        // PREIMAGE_BYTE_DEPOSIT.with(|v| *v.borrow_mut() = 1);

        // increase the balance of the account that is creating the pre-image hash
        // so it may afford the minimum deposit and pre-image byte deposit
        Balances::set_balance(Origin::root(), 1, 1_000_000_000_000_000, 0);
        assert_eq!(Balances::free_balance(1), 1_000_000_000_000_000);
        // register pre-image for upcoming proposal
        let encoded_proposal_preimage = vec![0; 500];
        match Democracy::note_preimage(Origin::signed(1), encoded_proposal_preimage.clone()) {
            Ok(_) => (),
            Err(x) if x == Error::<Test>::DuplicatePreimage.into() => (),
            Err(x) => panic!("note_preimage error {:?}", x),
        }
        System::set_block_number(1);
        // add a public proposal with a pre-image hash (with a required deposit)
        // to the proposal queue prior to it becoming a referendum
        let p1 = Democracy::propose(Origin::signed(1), BlakeTwo256::hash(b"test"), 1);
        // add a single proposal with a pre-image hash (no deposit required)
        // that originates from an external origin (i.e. collective group)
        // to the external queue prior to it becoming a referendum
        let p2 = Democracy::external_propose(Origin::signed(1), BlakeTwo256::hash(b"test"));
        System::set_block_number(2);
        Democracy::note_imminent_preimage(Origin::signed(1), encoded_proposal_preimage.clone());
        // second the proposals
        assert_ok!(Democracy::second(Origin::signed(1), 0, u32::MAX));

        // thread 'tests::setup_preimage' panicked at 'Expected Ok(_). Got Err(
        //     Module {
        //         index: 5,
        //         error: 1,
        //         message: Some(
        //             "ProposalMissing",
        //         ),
        //     },
        // )', pallets/mining/rewards-allowance/src/tests.rs:105:9

        assert_ok!(Democracy::second(Origin::signed(1), 1, u32::MAX));
        // we have 4.32 seconds per block, with a launch period of 672 hours,
        // so there are 10450944 blocks in the launch period before the the
        // public and external proposals take turns becoming a referendum
        System::set_block_number(11_000_000);
        // wait for referendums to be launched from the proposals after the launch period
		// external proposal becomes referendum first
		assert_eq!(
			Democracy::referendum_status(0),
			Ok(ReferendumStatus {
				end: 11_000_020, // block when voting on referendum ends
				proposal_hash: BlakeTwo256::hash(b"test"),
				threshold: VoteThreshold::SuperMajorityApprove,
				delay: 2,
				tally: Tally { ayes: 0, nays: 0, turnout: 0 },
			})
		);
		// public proposal's turn to become a referendum
		assert_eq!(
			Democracy::referendum_status(1),
			Ok(ReferendumStatus {
				end: 11_000_020,
				proposal_hash: BlakeTwo256::hash(b"test"),
				threshold: VoteThreshold::SuperMajorityApprove,
				delay: 2,
				tally: Tally { ayes: 0, nays: 0, turnout: 0 },
			})
		);
        System::set_block_number(11_000_001);
        // end of voting on referendum
        System::set_block_number(11_000_050);
        // vote on referenda using time-lock voting with a conviction to scale the vote power
        // note: second parameter is the referendum index
		assert_ok!(Democracy::vote(Origin::signed(1), 0, aye(1)));
        assert_eq!(tally(0), Tally { ayes: 1, nays: 0, turnout: 10 });
		assert_ok!(Democracy::vote(Origin::signed(1), 1, aye(1)));
        assert_eq!(tally(1), Tally { ayes: 1, nays: 0, turnout: 10 });
	});
}
