use super::{Call, Event, *};
use crate::{mock::*, Error};
use crate::{BondedDHXForAccountData};
use codec::Encode;
use frame_support::{assert_noop, assert_ok,
    weights::{DispatchClass, DispatchInfo, GetDispatchInfo},
};
use frame_system::{self, AccountInfo, EventRecord, Phase};
use pallet_democracy::{self, AccountVote, ReferendumStatus, Tally, VoteThreshold};
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

        assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_allowance_dhx_daily(
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
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_daily(), Some(5_000u128));

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
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_daily(), Some(5_000u128));

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
            Err(x) => panic!("Democracy::note_preimage error {:?}", x),
        }
        System::set_block_number(1);

        let event: SysEvent = SysEvent::NewAccount(32).into();
        let event_record: EventRecord<(), H256> = EventRecord {
            phase: Phase::Initialization,
            // we cannot do `event.clone() or it gives error:
            //   `expected `()`, found enum `frame_system::Event``
            event: (),
            topics: vec![],
        };
        let hash = sp_core::H256::default();
        // PreimageNoted: proposal_hash, who, deposit
        let event2: DemocracyEvent = DemocracyEvent::PreimageNoted(hash.clone(), 1, 0);
        System::deposit_event(event.clone());
        System::deposit_event(event2.clone());
        System::finalize();
        assert_eq!(
            System::events(),
            vec![
                event_record.clone(),
                event_record.clone(),
            ]
        );
        // Note: We are just going to assume that the event `DemocracyEvent::PreimageNoted`
        // emits a pre-image hash that we may then use to create a proposal with.
        let pre_image_hash = BlakeTwo256::hash(b"test");
        // let pre_image = <Preimages<T>>::take(&proposal_hash);

        // add a public proposal with a proposal pre-image hash (with a required deposit)
        // to the proposal queue prior to it becoming a referendum
        //
        // since we've configured MinimumDeposit to be 100 * DOLLARS, which is 100_000_000_000_000,
        // we need to deposit at least that much otherwise we get error `ValueLow`
        match Democracy::propose(Origin::signed(1), pre_image_hash.clone(), 100_000_000_000_000) {
            Ok(_) => (),
            Err(x) => panic!(" Democracy::propose error {:?}", x),
        }

        System::set_block_number(2);
        Democracy::note_imminent_preimage(Origin::signed(1), encoded_proposal_preimage.clone());
        let public_prop_count = Democracy::public_prop_count();
        assert_eq!(public_prop_count, 1);
        // check if a lock exists on an account until a block in the future. there shouldn't be any yet
        let locks_until_block_for_account = Democracy::locks(1);
        assert_eq!(locks_until_block_for_account, None);
        // second the proposals
        assert_ok!(Democracy::second(Origin::signed(1), 0, u32::MAX));
        System::set_block_number(3);
        // check the deposits made on a proposal index
		let deposits = Democracy::deposit_of(0).ok_or("Proposal not created").unwrap();
		assert_eq!(deposits.0.len(), 2 as usize, "Seconds not recorded");

        // check for info about referendums. there shouldn't be any yet.
        let referendum_count = Democracy::referendum_count();
        assert_eq!(referendum_count, 0);
        // info about a referendum index
        let referendum_info_1 = Democracy::referendum_info(0);
        assert_eq!(referendum_info_1, None);

        // we have 4.32 seconds per block, with a launch period of 672 hours,
        // so there are 10450944 blocks in the launch period before the the
        // public and external proposals take turns becoming a referendum
        System::set_block_number(11_000_000);

        // Note: Unfortunately we cannot use `Democracy::referendum_status` since it's a
        // private function if don't fork Substrate and modify the Democracy module

        // in fork DataHighway-DHX/substrate, branch luke/democracy,
        // commit 527101517d0ad67780131def8d227de51e503a89
        // we made this `inject_referendum` a public function
        let r = Democracy::inject_referendum(
			11_000_020,
			pre_image_hash.clone(),
			VoteThreshold::SuperMajorityApprove,
			2,
		);

        assert!(Democracy::referendum_status(r).is_ok());

        // wait for referendums to be launched from the proposals after the launch period
		// external proposal becomes referendum first then public proposals
		assert_eq!(
            // in fork DataHighway-DHX/substrate, branch luke/democracy,
            // commit 527101517d0ad67780131def8d227de51e503a89
            // we made this `referendum_status` a public function
			Democracy::referendum_status(0),
			Ok(ReferendumStatus {
				end: 11_000_020, // block when voting on referendum ends
				proposal_hash: pre_image_hash.clone(),
				threshold: VoteThreshold::SuperMajorityApprove,
				delay: 2,
				tally: Tally { ayes: 0, nays: 0, turnout: 0 },
			})
		);

        System::set_block_number(11_000_001);
        // end of voting on referendum
        System::set_block_number(11_000_050);
        // vote on referenda using time-lock voting with a conviction to scale the vote power
        // note: second parameter is the referendum index being voted on
		assert_ok!(Democracy::vote(
            Origin::signed(1),
            0,
            // aye(1), // cannot use aye(..) from Substrate pallet_democracy
            // since functions used as tests cannot have any arguments
            AccountVote::Standard { vote: AYE, balance: Balances::free_balance(1) },
        ));
        assert_eq!(Democracy::referendum_status(r).unwrap().tally, Tally { ayes: 30000000000000, nays: 0, turnout: 300000000000000 });
	});
}
