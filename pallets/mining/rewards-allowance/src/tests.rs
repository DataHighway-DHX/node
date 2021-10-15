use super::{Call, Event, *};
use crate::{mock::*, Error};
pub use mock::{INIT_DAO_BALANCE_DHX, TOTAL_SUPPLY_DHX, TEN_DHX};
use codec::Encode;
use frame_support::{assert_noop, assert_ok,
    weights::{DispatchClass, DispatchInfo, GetDispatchInfo},
    traits::{OnFinalize, OnInitialize},
};
use frame_system::{self, AccountInfo, EventRecord, Phase, RawOrigin};
use pallet_balances::{self, BalanceLock, Reasons};
use pallet_democracy::{self, AccountVote, ReferendumStatus, Tally, VoteThreshold};
use sp_core::{
    H256,
    Hasher, // so we may use BlakeTwo256::hash
};
use sp_runtime::{
	traits::{BlakeTwo256},
};

const NORMAL_AMOUNT: u128 = 25_133_000_000_000_000_000_000u128; // 25,133 DHX
const LARGE_AMOUNT_DHX: u128 = 33_333_333_333_000_000_000_000_000u128; // 33,333,333.333 DHX
const FIVE_THOUSAND_DHX: u128 = 5_000_000_000_000_000_000_000_u128; // 5000
const TWO_THOUSAND_DHX: u128 = 2_000_000_000_000_000_000_000_u128; // 2,000
const FIVE_HUNDRED_DHX: u128 = 500_000_000_000_000_000_000_u128; // 500
const TWO_DHX: u128 = 2_000_000_000_000_000_000_u128; // 2

#[test]
// ignore this test until the FIXME is resolved
#[ignore]
fn it_sets_rewards_allowance_with_genesis_defaults_automatically_in_on_finalize_if_not_already_set_for_today() {
    new_test_ext().execute_with(|| {
        assert_ok!(MiningRewardsAllowanceTestModule::set_registered_dhx_miner(Origin::signed(1)));
        assert_ok!(MiningRewardsAllowanceTestModule::set_registered_dhx_miner(Origin::signed(2)));
        assert_ok!(MiningRewardsAllowanceTestModule::set_registered_dhx_miner(Origin::signed(3)));

        // 27th August 2021 @ ~7am is 1630049371000
        // where milliseconds/day         86400000
        // 27th August 2021 @ 12am is 1630022400000 (start of day)
        Timestamp::set_timestamp(1630049371000u64);

		MiningRewardsAllowanceTestModule::on_initialize(1);

        // FIXME - why doesn't this work and use the defaults that we have set in the genesis config?
        // i've had to add a function `set_rewards_allowance_dhx_daily` to set this instead
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_daily(), Some(FIVE_THOUSAND_DHX));
    })
}

#[test]
// Note: if we remove `cooling_off_period_days_remaining.0 != start_of_requested_date_millis.clone() &&`
// four times from the implementation, then all this happens on the same day so we'd need to use the
// same timestamp for all the blocks and tests below.
fn it_distributes_rewards_automatically_in_on_finalize_for_default_amount() {
    new_test_ext().execute_with(|| {
        setup_treasury_balance();

        distribute_rewards(NORMAL_AMOUNT.clone());
    })
}

#[test]
fn it_distributes_rewards_automatically_in_on_finalize_for_large_amount() {
    new_test_ext().execute_with(|| {
        // create a test that instead of using a hard-coded value for `locks_first_amount_as_u128`
        // that is in the implementation, it instead sets the locked value of each of them using frame_balances
        // for the 3x miners, since we can then store that with `set_bonded_dhx_of_account_for_date` and
        // then use that easier for the tests too for trying different values that they have bonded.
        //
        // in this test we'll test that it distributes rewards when each of their account balances are very large
        // (i.e. a third of the total supply) ONE_THIRD_OF_TOTAL_SUPPLY_DHX

        assert_ok!(Balances::set_balance(Origin::root(), 1, LARGE_AMOUNT_DHX, 0));
        assert_ok!(Balances::set_balance(Origin::root(), 2, LARGE_AMOUNT_DHX, 0));
        assert_ok!(Balances::set_balance(Origin::root(), 3, LARGE_AMOUNT_DHX, 0));

        assert_eq!(Balances::free_balance(&1), LARGE_AMOUNT_DHX);
        assert_eq!(Balances::free_balance(&2), LARGE_AMOUNT_DHX);
        assert_eq!(Balances::free_balance(&3), LARGE_AMOUNT_DHX);

        assert_eq!(Balances::reserved_balance(&1), 0);

        setup_treasury_balance();

        let pre_image_hash = BlakeTwo256::hash(b"test");
        let r = Democracy::inject_referendum(1, pre_image_hash.clone(), VoteThreshold::SuperMajorityApprove, 2);
        // lock the whole balance of account 1, 2, and 3 in voting
        let v1a1 = AccountVote::Standard { vote: AYE, balance: Balances::free_balance(1) };
        let v1a2 = AccountVote::Standard { vote: AYE, balance: Balances::free_balance(2) };
        let v1a3 = AccountVote::Standard { vote: AYE, balance: Balances::free_balance(3) };
        // vote on referenda using time-lock voting with a conviction to scale the vote power
        // note: second parameter is the referendum index being voted on
        assert_ok!(Democracy::vote(Origin::signed(1), r, v1a1));
        assert_ok!(Democracy::vote(Origin::signed(2), r, v1a2));
        assert_ok!(Democracy::vote(Origin::signed(3), r, v1a3));
        // let v2 = AccountVote::Split { aye: 1, nay: 2 };
        // assert_ok!(Democracy::vote(Origin::signed(1), r, v2));

        // TODO - use this later to simulate that unbonding works
        // // assert_ok!(Democracy::remove_vote(Origin::signed(1), r));
        // // assert_eq!(tally(r), Tally { ayes: 0, nays: 0, turnout: 0 });
        // // assert_ok!(Democracy::unlock(Origin::signed(1), 5));
        assert_eq!(Balances::locks(1)[0],
            BalanceLock {
                id: [100, 101, 109, 111, 99, 114, 97, 99],
                amount: LARGE_AMOUNT_DHX,
                reasons: Reasons::Misc
            }
        );

        // FIXME - why do we get the same result when using addition and multiplation?
        assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_multiplier_operation(
            Origin::signed(0),
            1u8,
        ));

        distribute_rewards(LARGE_AMOUNT_DHX);
    })
}

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
            FIVE_THOUSAND_DHX
        ));

        assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_allowance_dhx_for_date_remaining(
            Origin::signed(0),
            FIVE_THOUSAND_DHX,
            1630049371000
        ));

        // Verify Storage
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_daily(), Some(FIVE_THOUSAND_DHX));

        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1630022400000), Some(FIVE_THOUSAND_DHX));

        assert_ok!(MiningRewardsAllowanceTestModule::change_rewards_allowance_dhx_for_date_remaining(
            Origin::signed(0),
            FIVE_HUNDRED_DHX,
            1630049371000,
            0
        ));

        // reducing the remaining rewards for a specific date does not change the default rewards allowance
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_daily(), Some(FIVE_THOUSAND_DHX));

        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1630022400000), Some(4_500_000_000_000_000_000_000u128));

        assert_ok!(MiningRewardsAllowanceTestModule::change_rewards_allowance_dhx_for_date_remaining(
            Origin::signed(0),
            TWO_THOUSAND_DHX,
            1630049371000,
            1
        ));

        assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1630022400000), Some(6_500_000_000_000_000_000_000u128));
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
        assert_eq!(Balances::locks(1)[0],
            BalanceLock {
                id: [100, 101, 109, 111, 99, 114, 97, 99],
                amount: 300000000000000,
                reasons: Reasons::Misc
            }
        );
    });
}

#[test]
fn it_sets_min_mpower_daily() {
    new_test_ext().execute_with(|| {
        assert_ok!(MiningRewardsAllowanceTestModule::set_min_mpower_daily(
            Origin::signed(1),
            5u128,
        ));
    });
}

#[test]
#[ignore]
fn it_allows_us_to_retrieve_genesis_value_for_min_mpower_daily() {
    new_test_ext().execute_with(|| {
        // FIXME - why doesn't it set the values we added in the chain_spec.rs at genesis
        // https://matrix.to/#/!HzySYSaIhtyWrwiwEV:matrix.org/$163424903366086IiiUH:matrix.org?via=matrix.parity.io&via=corepaper.org&via=matrix.org
        MiningRewardsAllowanceTestModule::on_initialize(1);
        assert_eq!(MiningRewardsAllowanceTestModule::min_mpower_daily(), Some(5u128));
    });
}

fn distribute_rewards(amount_bonded_each_miner: u128) {
    assert_ok!(MiningRewardsAllowanceTestModule::set_registered_dhx_miner(Origin::signed(1)));
    assert_ok!(MiningRewardsAllowanceTestModule::set_registered_dhx_miner(Origin::signed(2)));
    assert_ok!(MiningRewardsAllowanceTestModule::set_registered_dhx_miner(Origin::signed(3)));

    assert_ok!(MiningRewardsAllowanceTestModule::set_min_bonded_dhx_daily(
        Origin::signed(1),
        TEN_DHX,
    ));
    assert_ok!(MiningRewardsAllowanceTestModule::set_cooling_off_period_days(
        Origin::signed(1),
        1_u32, // debug quickly for testing
    ));
    assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_allowance_dhx_daily(
        Origin::signed(1),
        FIVE_THOUSAND_DHX,
    ));

    assert_eq!(MiningRewardsAllowanceTestModule::registered_dhx_miners(), Some(vec![1, 2, 3]));
    assert_eq!(MiningRewardsAllowanceTestModule::min_bonded_dhx_daily(), Some(TEN_DHX));
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days(), Some(1));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_daily(), Some(FIVE_THOUSAND_DHX));

    // since the timestamp is 0 (corresponds to 1970-01-01) at block number #1, we early exit from on_initialize in
    // that block in the implementation and do not set any storage values associated with the date until block #2.
    // in the tests we could set the timestamp before we run on_initialize(1), but that wouldn't reflect reality.
    MiningRewardsAllowanceTestModule::on_initialize(1);

    // 27th August 2021 @ ~7am is 1630049371000
    // where milliseconds/day         86400000
    // 27th August 2021 @ 12am is 1630022400000 (start of day)
    Timestamp::set_timestamp(1630049371000u64);
    MiningRewardsAllowanceTestModule::on_initialize(2);
    // System::on_initialize(2);
    // System::on_finalize(2);
    // System::set_block_number(2);

    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1630022400000), Some(FIVE_THOUSAND_DHX));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining_distributed(1630022400000), Some(false));

    // https://www.epochconverter.com/
    // 28th August 2021 @ ~7am is 1635406274000
    // where milliseconds/day         86400000
    // 28th August 2021 @ 12am is 1635379200000 (start of day)
    Timestamp::set_timestamp(1635406274000u64);
    MiningRewardsAllowanceTestModule::on_initialize(3);

    // check that on_initialize has populated this storage value automatically for the start of the current date
    // still cooling off so no rewards distributed on this date
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1635379200000), Some(FIVE_THOUSAND_DHX));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining_distributed(1635379200000), Some(false));

    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1635379200000, 1)), Some(amount_bonded_each_miner));
    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1635379200000, 2)), Some(amount_bonded_each_miner));
    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1635379200000, 3)), Some(amount_bonded_each_miner));

    // 29th August 2021 @ ~7am is 1630220400000
    // 29th August 2021 @ 12am is 1630195200000 (start of day)
    Timestamp::set_timestamp(1630195200000u64);
    MiningRewardsAllowanceTestModule::on_initialize(4);

    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1630195200000, 1)), Some(amount_bonded_each_miner));
    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1630195200000, 2)), Some(amount_bonded_each_miner));
    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1630195200000, 3)), Some(amount_bonded_each_miner));

    // i.e. for example, if locked is 25_133_000_000_000_000_000_000u128 (NORMAL_AMOUNT), which is 25,133 DHX,
    // then with 10:1 each of the 3x accounts get 2513.3 DHX, which is ~7538.9 DHX combined
    // or 33_333_333_333_000_000_000_000_000u128 (LARGE_AMOUNT_DHX),
    // but the results are rounded to the nearest integer so it would be 2513 DHX, not 2513.3 DHX
    if amount_bonded_each_miner.clone() == NORMAL_AMOUNT {
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_aggregated_dhx_for_all_miners_for_date(1630195200000), Some(7_539_000_000_000_000_000_000u128));

        assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630195200000, 1)), Some(2_513_000_000_000_000_000_000u128));
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630195200000, 2)), Some(2_513_000_000_000_000_000_000u128));
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630195200000, 3)), Some(2_513_000_000_000_000_000_000u128));
    } else if amount_bonded_each_miner.clone() == LARGE_AMOUNT_DHX {
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_aggregated_dhx_for_all_miners_for_date(1630195200000), Some(9_999_999_000_000_000_000_000_000u128));

        assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630195200000, 1)), Some(3_333_333_000_000_000_000_000_000u128));
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630195200000, 2)), Some(3_333_333_000_000_000_000_000_000u128));
        assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630195200000, 3)), Some(3_333_333_000_000_000_000_000_000u128));
    }

    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1630195200000), Some(TWO_DHX));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining_distributed(1630195200000), Some(true));
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1630195200000, 0, 1)));
}

fn setup_treasury_balance() {
    // set the balance of the treasury so it distributes rewards
    Balances::set_balance(Origin::root(), Treasury::account_id(), INIT_DAO_BALANCE_DHX, 0);
    assert_eq!(Balances::usable_balance(&Treasury::account_id()), INIT_DAO_BALANCE_DHX);
}
