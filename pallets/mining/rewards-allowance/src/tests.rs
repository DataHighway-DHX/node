use super::{Call, Event, *};
use crate::{mock::*, Error};
pub use mock::{INIT_DAO_BALANCE_DHX, TOTAL_SUPPLY_DHX, TEN_DHX, FIVE_THOUSAND_DHX};
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
const TWO_THOUSAND_DHX: u128 = 2_000_000_000_000_000_000_000_u128; // 2,000
const FIVE_HUNDRED_DHX: u128 = 500_000_000_000_000_000_000_u128; // 500
const THIRTY_DHX: u128 = 30_000_000_000_000_000_000_u128; // 30
const TWENTY_DHX: u128 = 20_000_000_000_000_000_000_u128; // 20
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
        let amount_mpower_each_miner = 5u128;
        let min_mpower_daily = 5u128;

        setup_min_mpower_daily(min_mpower_daily);

        let r = setup_bonding(NORMAL_AMOUNT, TEN_DHX);

        setup_treasury_balance();

        setup_multiplier();

        distribute_rewards(NORMAL_AMOUNT, amount_mpower_each_miner, r);
    })
}

#[test]
fn it_distributes_rewards_automatically_in_on_finalize_for_large_amount() {
    new_test_ext().execute_with(|| {
        let amount_mpower_each_miner = 5u128;
        let min_mpower_daily = 5u128;

        setup_min_mpower_daily(min_mpower_daily);

        let r = setup_bonding(LARGE_AMOUNT_DHX, TEN_DHX);

        setup_treasury_balance();

        setup_multiplier();

        distribute_rewards(LARGE_AMOUNT_DHX, amount_mpower_each_miner, r);
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

#[test]
fn it_converts_vec_u8_to_u128() {
    new_test_ext().execute_with(|| {
        // my snippet: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=69915086c8faa9de69301ee86e914bed
        let hex_literal = vec![48, 51, 48, 48, 49, 50, 48, 51, 57, 48, 48];
        assert_eq!(MiningRewardsAllowanceTestModule::convert_vec_u8_to_u128(&hex_literal), Ok(3001203900u128));
    });
}

fn distribute_rewards(amount_bonded_each_miner: u128, amount_mpower_each_miner: u128, referendum_index: u32) {
    assert_ok!(MiningRewardsAllowanceTestModule::set_registered_dhx_miner(Origin::signed(1)));
    assert_ok!(MiningRewardsAllowanceTestModule::set_registered_dhx_miner(Origin::signed(2)));
    assert_ok!(MiningRewardsAllowanceTestModule::set_registered_dhx_miner(Origin::signed(3)));

    assert_ok!(MiningRewardsAllowanceTestModule::set_cooling_off_period_days(
        Origin::signed(1),
        1_u32, // debug quickly for testing
    ));
    assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_allowance_dhx_daily(
        Origin::signed(1),
        FIVE_THOUSAND_DHX,
    ));

    assert_eq!(MiningRewardsAllowanceTestModule::registered_dhx_miners(), Some(vec![1, 2, 3]));
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days(), Some(1));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_daily(), Some(FIVE_THOUSAND_DHX));

    check_eligible_for_rewards_after_cooling_off_period_if_suffient_bonded(amount_bonded_each_miner.clone(), amount_mpower_each_miner.clone());

    // check that rewards multiplier increases by multiplier every period days and that days total and remaining are reset
    check_rewards_double_each_multiplier_period(amount_mpower_each_miner.clone());

    // check that after the multiplier doubles, they are no longer eligible to receive the rewards
    // if they have the same amount bonded (since they’d then need twice the amount bonded as ratio changes from 10:1 to 20:1),
    // even if they have sufficient mpower
    check_ineligible_for_rewards_and_cooling_down_period_starts_if_insufficient_bonded(amount_bonded_each_miner.clone(), amount_mpower_each_miner.clone(), referendum_index.clone());
}

fn setup_min_mpower_daily(min_mpower_daily: u128) {
    assert_ok!(MiningRewardsAllowanceTestModule::set_min_mpower_daily(
        Origin::signed(1),
        min_mpower_daily.clone(),
    ));
    assert_eq!(MiningRewardsAllowanceTestModule::min_mpower_daily(), Some(min_mpower_daily.clone()));
}

// we have to get their mpower the day before we check if they are eligible incase there are delays in getting the off-chain data
fn change_mpower_for_each_miner(amount_mpower_each_miner: u128, next_start_date: i64) {
    let account_1_public_key = vec![1];
    let account_2_public_key = vec![2];
    let account_3_public_key = vec![3];
    // https://aws1.discourse-cdn.com/business5/uploads/rust_lang/original/3X/9/0/909baa7e3d9569489b07c791ca76f2223bd7bac2.webp
    assert_ok!(MiningRewardsAllowanceTestModule::set_mpower_of_account_for_date(account_1_public_key, amount_mpower_each_miner.clone(), next_start_date));
    assert_ok!(MiningRewardsAllowanceTestModule::set_mpower_of_account_for_date(account_2_public_key, amount_mpower_each_miner.clone(), next_start_date));
    assert_ok!(MiningRewardsAllowanceTestModule::set_mpower_of_account_for_date(account_3_public_key, amount_mpower_each_miner.clone(), next_start_date));
    assert_eq!(
        MiningRewardsAllowanceTestModule::mpower_of_account_for_date((next_start_date, account_1_public_key)),
        Some(amount_mpower_each_miner.clone())
    );
    assert_eq!(
        MiningRewardsAllowanceTestModule::mpower_of_account_for_date((next_start_date, account_2_public_key)),
        Some(amount_mpower_each_miner.clone())
    );
    assert_eq!(
        MiningRewardsAllowanceTestModule::mpower_of_account_for_date((next_start_date, account_3_public_key)),
        Some(amount_mpower_each_miner.clone())
    );
}

fn setup_bonding(amount_bonded_each_miner: u128, min_bonding_dhx_daily: u128) -> u32 {
    assert_ok!(MiningRewardsAllowanceTestModule::set_min_bonded_dhx_daily(
        Origin::signed(1),
        min_bonding_dhx_daily.clone(),
    ));
    assert_eq!(MiningRewardsAllowanceTestModule::min_bonded_dhx_daily(), Some(min_bonding_dhx_daily.clone()));

    // create a test that instead of using a hard-coded value for `locks_first_amount_as_u128`
    // that is in the implementation, it instead sets the locked value of each of them using frame_balances
    // for the 3x miners, since we can then store that with `set_bonded_dhx_of_account_for_date` and
    // then use that easier for the tests too for trying different values that they have bonded.
    //
    // in this test we'll test that it distributes rewards when each of their account balances are very large
    // (i.e. a third of the total supply) ONE_THIRD_OF_TOTAL_SUPPLY_DHX

    assert_ok!(Balances::set_balance(Origin::root(), 1, amount_bonded_each_miner, 0));
    assert_ok!(Balances::set_balance(Origin::root(), 2, amount_bonded_each_miner, 0));
    assert_ok!(Balances::set_balance(Origin::root(), 3, amount_bonded_each_miner, 0));

    assert_eq!(Balances::free_balance(&1), amount_bonded_each_miner);
    assert_eq!(Balances::free_balance(&2), amount_bonded_each_miner);
    assert_eq!(Balances::free_balance(&3), amount_bonded_each_miner);

    assert_eq!(Balances::reserved_balance(&1), 0);

    let pre_image_hash = BlakeTwo256::hash(b"test");
    // params: end block, proposal hash, threshold, delay
    let r = Democracy::inject_referendum(1, pre_image_hash.clone(), VoteThreshold::SuperMajorityApprove, 2);

    bond_each_miner_by_voting_for_referendum(amount_bonded_each_miner, r);

    return r;
}

fn setup_multiplier() {
    assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_multiplier_operation(
        Origin::signed(0),
        1u8,
    ));

    // in the tests we want the period between each 10:1, 20:1 cycle to be just 2 days instead of 90 days
    // since we don't want to wait so long to check that it changes each cycle in the tests
    assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_multiplier_default_period_days(
        Origin::signed(0),
        2u32,
    ));

    assert_ok!(MiningRewardsAllowanceTestModule::set_rewards_multiplier_next_period_days(
        Origin::signed(0),
        2u32,
    ));
}

fn setup_treasury_balance() {
    // set the balance of the treasury so it distributes rewards
    Balances::set_balance(Origin::root(), Treasury::account_id(), INIT_DAO_BALANCE_DHX, 0);
    assert_eq!(Balances::usable_balance(&Treasury::account_id()), INIT_DAO_BALANCE_DHX);
}

fn bond_each_miner_by_voting_for_referendum(amount_bonded_each_miner: u128, referendum_index: u32) {
    // we're actually bonding with their entire account balance
    let b1 = Balances::free_balance(&1);
    let b2 = Balances::free_balance(&2);
    let b3 = Balances::free_balance(&3);

    // lock the whole balance of account 1, 2, and 3 in voting
    let v1a1 = AccountVote::Standard { vote: AYE, balance: b1.clone() };
    let v1a2 = AccountVote::Standard { vote: AYE, balance: b2.clone() };
    let v1a3 = AccountVote::Standard { vote: AYE, balance: b3.clone() };
    // vote on referenda using time-lock voting with a conviction to scale the vote power
    // note: second parameter is the referendum index being voted on
    assert_ok!(Democracy::vote(Origin::signed(1), referendum_index, v1a1));
    assert_ok!(Democracy::vote(Origin::signed(2), referendum_index, v1a2));
    assert_ok!(Democracy::vote(Origin::signed(3), referendum_index, v1a3));

    assert_eq!(Balances::locks(1)[0],
        BalanceLock {
            id: [100, 101, 109, 111, 99, 114, 97, 99],
            amount: b1.clone(),
            reasons: Reasons::Misc
        }
    );
    assert_eq!(Balances::locks(2)[0],
        BalanceLock {
            id: [100, 101, 109, 111, 99, 114, 97, 99],
            amount: b2.clone(),
            reasons: Reasons::Misc
        }
    );
    assert_eq!(Balances::locks(3)[0],
        BalanceLock {
            id: [100, 101, 109, 111, 99, 114, 97, 99],
            amount: b3.clone(),
            reasons: Reasons::Misc
        }
    );
}

fn unbond_each_miner_by_removing_their_referendum_vote(referendum_index: u32) {
        // remove the votes and then unlock for each account
    // note: `remove_vote` must be done before `unlock`
    assert_ok!(Democracy::remove_vote(Origin::signed(1), referendum_index));
    assert_ok!(Democracy::remove_vote(Origin::signed(2), referendum_index));
    assert_ok!(Democracy::remove_vote(Origin::signed(3), referendum_index));
    // we removed their votes
    assert_eq!(Democracy::referendum_status(referendum_index).unwrap().tally, Tally { ayes: 0, nays: 0, turnout: 0 });
    assert_ok!(Democracy::unlock(Origin::signed(1), 1));
    assert_ok!(Democracy::unlock(Origin::signed(2), 2));
    assert_ok!(Democracy::unlock(Origin::signed(3), 3));

    // check that all accounts are unlocked
    assert_eq!(Balances::locks(1), vec![]);
    assert_eq!(Balances::locks(2), vec![]);
    assert_eq!(Balances::locks(3), vec![]);
}

fn check_eligible_for_rewards_after_cooling_off_period_if_suffient_bonded(amount_bonded_each_miner: u128, amount_mpower_each_miner: u128) {
    // since the timestamp is 0 (corresponds to 1970-01-01) at block number #1, we early exit from on_initialize in
    // that block in the implementation and do not set any storage values associated with the date until block #2.
    // in the tests we could set the timestamp before we run on_initialize(1), but that wouldn't reflect reality.
    MiningRewardsAllowanceTestModule::on_initialize(1);

    // IMPORTANT: if we don't set the mpower for each miner for the current date beforehand, we won't be able to accumulate their rewards
    // let's assume that we receive sufficient mpower and it's above the min. mpower,
    // from off-chain on-time early enough on the same day so we have all other info we need
    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1630022400000i64);

    // 27th August 2021 @ ~7am is 1630049371000
    // where milliseconds/day         86400000
    // 27th August 2021 @ 12am is 1630022400000 (start of day)
    Timestamp::set_timestamp(1630049371000u64);
    MiningRewardsAllowanceTestModule::on_initialize(2);
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1630022400000, 1630022400000, 2u32, 2u32)));
    // System::on_initialize(2);
    // System::on_finalize(2);
    // System::set_block_number(2);

    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1630022400000), Some(FIVE_THOUSAND_DHX));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining_distributed(1630022400000), Some(false));

    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1635379200000i64);

    // https://www.epochconverter.com/
    // 28th August 2021 @ ~7am is 1635406274000
    // where milliseconds/day         86400000
    // 28th August 2021 @ 12am is 1635379200000 (start of day)
    Timestamp::set_timestamp(1635406274000u64);
    MiningRewardsAllowanceTestModule::on_initialize(3);
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1630022400000, 1635379200000, 2u32, 1u32)));

    // check that on_initialize has populated this storage value automatically for the start of the current date
    // still cooling off so no rewards distributed on this date
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1635379200000), Some(FIVE_THOUSAND_DHX));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining_distributed(1635379200000), Some(false));

    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1635379200000, 1)), Some(amount_bonded_each_miner));
    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1635379200000, 2)), Some(amount_bonded_each_miner));
    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1635379200000, 3)), Some(amount_bonded_each_miner));

    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1630195200000i64);

    // 29th August 2021 @ ~7am is 1630220400000
    // 29th August 2021 @ 12am is 1630195200000 (start of day)
    Timestamp::set_timestamp(1630195200000u64);
    MiningRewardsAllowanceTestModule::on_initialize(4);

    // a day before we start the new multiplier period and change from 10:1 to 20:1 since no more days remaining
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1630022400000, 1630195200000, 2u32, 0u32)));

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

    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1630281600000i64);

    // 30th August 2021 @ ~7am is 1630306800000
    // 30th August 2021 @ 12am is 1630281600000 (start of day)
    Timestamp::set_timestamp(1630306800000u64);
    MiningRewardsAllowanceTestModule::on_initialize(5);

    // we have finished the cooling off period and should now be distributing rewards each day unless they reduce their bonded
    // amount below the min. bonded DHX daily amount
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1630281600000, 0, 1)));
    // check that the min_bonded_dhx_daily doubled after 3 months from 10 DHX to 20 DHX
    assert_eq!(MiningRewardsAllowanceTestModule::min_bonded_dhx_daily(), Some(TWENTY_DHX));
    // the change between each multiplier period is 10 unless a user sets it to a different value
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_change(), Some(10u32));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_next_change(), Some(10u32));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_next_period_days(), Some(2u32));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_total(), Some(2u32));
    // start of new multiplier period
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1630281600000, 1630281600000, 2u32, 2u32)));

    // Note - these are just notes. no further action required
    // Note - why is this 2u128 instead of reset back to say 5000u128 DHX (unless set do different value??
    // this should be reset after rewards aggregated/accumulated each day
    // since distribution/claiming may not be done by a user each day
    // Update: it gets reset but difficult to add a test, have to run the logs with only one test running to see it gets accumulated/aggregated
    // to all miners each day over a few days
    // assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1630281600000), Some(FIVE_THOUSAND_DHX));
    // Note - why is this 'true' when the day just started and nothing has been distributed?
    // Update: difficult to add a test, check it works using the logs. they will be claiming rewards anyway, so we need separate tests for that
    // assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining_distributed(1630281600000), Some(false));
    // Note - why haven't these been reset to 0u128 for the next day?
    // Update: difficult to add a test to check they are reset before next day, just check it works using the logs.
    // assert_eq!(MiningRewardsAllowanceTestModule::rewards_aggregated_dhx_for_all_miners_for_date(1630281600000), Some(0u128));
    // assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630281600000, 1)), Some(0u128));
}

fn check_rewards_double_each_multiplier_period(amount_mpower_each_miner: u128) {
    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1630368000000i64);

    // 31th August 2021 @ ~7am is 1630393200000
    // 31th August 2021 @ 12am is 1630368000000 (start of day)
    Timestamp::set_timestamp(1630393200000u64);
    MiningRewardsAllowanceTestModule::on_initialize(6);
    // cooling off period doesn't change again unless they unbond
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1630368000000, 0, 1)));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1630281600000, 1630368000000, 2u32, 1u32)));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_change(), Some(10u32));

    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1630454400000i64);

    // 1st Sept 2021 @ ~7am is 1630479600000
    // 1st Sept 2021 @ 12am is 1630454400000 (start of day)
    Timestamp::set_timestamp(1630479600000u64);
    MiningRewardsAllowanceTestModule::on_initialize(7);
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1630454400000, 0, 1)));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1630281600000, 1630454400000, 2u32, 0u32)));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_change(), Some(10u32));

    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1630540800000i64);

    // 2nd Sept 2021 @ ~7am is 1630566000000
    // 2nd Sept 2021 @ 12am is 1630540800000 (start of day)
    Timestamp::set_timestamp(1630566000000u64);
    MiningRewardsAllowanceTestModule::on_initialize(7);
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1630540800000, 0, 1)));
    // start of new multiplier period
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1630540800000, 1630540800000, 2u32, 2u32)));
    // check that the min_bonded_dhx_daily doubled after 3 months (we're only doing it after 2 days in the tests though) from 20 DHX to 30 DHX
    assert_eq!(MiningRewardsAllowanceTestModule::min_bonded_dhx_daily(), Some(THIRTY_DHX));
}

fn check_ineligible_for_rewards_and_cooling_down_period_starts_if_insufficient_bonded(amount_bonded_each_miner: u128, amount_mpower_each_miner: u128, referendum_index: u32) {
    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1630627200000i64);

    // 3rd Sept 2021 @ ~7am is 1630652400000
    // 3rd Sept 2021 @ 12am is 1630627200000 (start of day)
    Timestamp::set_timestamp(1630652400000u64);
    MiningRewardsAllowanceTestModule::on_initialize(8);

    // the below works to unbond each of the accounts

    // check that the referendum that we created earlier still exists
    assert_eq!(Democracy::referendum_count(), 1, "referenda not created");

    unbond_each_miner_by_removing_their_referendum_vote(referendum_index.clone());

    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1630713600000i64);

    // now wait for the next day when we iterate through the miner accounts and they should have no locks
    // 4th Sept 2021 @ ~7am is 1630738800000
    // 4th Sept 2021 @ 12am is 1630713600000 (start of day)
    Timestamp::set_timestamp(1630738800000u64);
    MiningRewardsAllowanceTestModule::on_initialize(9);

    // IMPORTANT NOTE: The min. DHX bonded has increased from 10 (10:1) to 20 (20:1) in order to be eligible
    // for rewards, however none of the miner's increased their bonded DHX amount proportionally to still remain
    // eligible for rewards, so since having insufficient bonded DHX is the same as unbonding, we expect the
    // cooling off period days remaining to change so they are now going through the unbonding cool down period,
    // (which we also count using `cooling_off_period_days_remaining`)
    // where they aren't eligble for rewards until they bond the new min. DHX so cooling off period starts and
    // then they'd be eligible for rewards after waiting that period, but also note that if they don't bond the new min.
    // DHX and wait until the end of the cool down period then they'll be able to withdraw the amount they had bonded.
    //
    // but in the tests the initial bonded amounts were much more than the min. DHX bonded, so even after it increases
    // from 10 (10:1) to 20 (20:1) they are still eligible for rewards.
    // so in the tests we've just decided to remove their vote and `unlock` their bonded DHX so they don't have a lock
    // and so don't satisfy the min. DHX bonded

    // params: start of date, days remaining, bonding status
    // note: since they don't have the min. DHX bonded their bonding status changes to `2`, which is unbonding
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1630713600000, 1, 2)));

    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1630713600000, 1)), Some(0u128));
    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1630713600000, 2)), Some(0u128));
    assert_eq!(MiningRewardsAllowanceTestModule::bonded_dhx_of_account_for_date((1630713600000, 3)), Some(0u128));

    // check they are not eligible for rewards due to insufficient bonded amount
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_aggregated_dhx_for_all_miners_for_date(1630713600000), None);

    assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630713600000, 1)), None);
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630713600000, 2)), None);
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_accumulated_dhx_for_miner_for_date((1630713600000, 3)), None);

    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining(1630713600000), Some(FIVE_THOUSAND_DHX));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_allowance_dhx_for_date_remaining_distributed(1630713600000), Some(false));

    check_cooling_off_period_starts_again_if_sufficient_bonded_again(amount_bonded_each_miner.clone(), amount_mpower_each_miner.clone(), referendum_index.clone());
}

fn check_cooling_off_period_starts_again_if_sufficient_bonded_again(amount_bonded_each_miner: u128, amount_mpower_each_miner: u128, referendum_index: u32) {

    bond_each_miner_by_voting_for_referendum(amount_bonded_each_miner, referendum_index);

    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1630800000000i64);

    // now wait for the next day when we iterate through the miner accounts and they should have no locks
    // 5th Sept 2021 @ ~7am is 1630825200000
    // 5th Sept 2021 @ 12am is 1630800000000 (start of day)
    Timestamp::set_timestamp(1630825200000u64);
    MiningRewardsAllowanceTestModule::on_initialize(10);

    // params: start of date, days remaining, bonding status
    // note: since they have the min. DHX bonded again their bonding status changes to `1`, which is bonding
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1630800000000, 0, 1)));

    check_ineligible_for_rewards_and_cooling_down_period_starts_if_insufficient_mpower(amount_bonded_each_miner.clone(), amount_mpower_each_miner.clone(), referendum_index.clone());
}

fn check_ineligible_for_rewards_and_cooling_down_period_starts_if_insufficient_mpower(amount_bonded_each_miner: u128, amount_mpower_each_miner: u128, referendum_index: u32) {
    // no mpower to check they'll be ineligible for rewards
    change_mpower_for_each_miner(0u128, 1630886400000i64);

    // 6th Sept 2021 @ ~7am is 1630911600000
    // 6th Sept 2021 @ 12am is 1630886400000 (start of day)
    Timestamp::set_timestamp(1630911600000u64);
    MiningRewardsAllowanceTestModule::on_initialize(11);

    // no mpower to check they'll be ineligible for rewards
    change_mpower_for_each_miner(0u128, 1630972800000i64);

    // 7th Sept 2021 @ ~7am is 1630998000000
    // 7th Sept 2021 @ 12am is 1630972800000 (start of day)
    Timestamp::set_timestamp(1630998000000u64);
    MiningRewardsAllowanceTestModule::on_initialize(12);

    // params: start of date, days remaining, bonding status
    // note: since they don't have min. mPower their bonding status changes to `2`, which is unbonding
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1630972800000, 0, 2)));

    check_cooling_off_period_starts_again_if_sufficient_mpower_again(amount_bonded_each_miner.clone(), amount_mpower_each_miner.clone(), referendum_index.clone());
}

fn check_cooling_off_period_starts_again_if_sufficient_mpower_again(amount_bonded_each_miner: u128, amount_mpower_each_miner: u128, referendum_index: u32) {
    // reset mpower to what it was
    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1631059200000i64);

    // 8th Sept 2021 @ ~7am is 1631084400000
    // 8th Sept 2021 @ 12am is 1631059200000 (start of day)
    Timestamp::set_timestamp(1631084400000u64);
    MiningRewardsAllowanceTestModule::on_initialize(13);

    // params: start of date, days remaining, bonding status
    // note: they have min. mPower again so their bonding status changes to `0`, which is unbonded
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1631059200000, 0, 0)));

    // use original mpower
    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1631145600000i64);

    // 9th Sept 2021 @ ~7am is 1631170800000
    // 9th Sept 2021 @ 12am is 1631145600000 (start of day)
    Timestamp::set_timestamp(1631170800000u64);
    MiningRewardsAllowanceTestModule::on_initialize(14);

    // params: start of date, days remaining, bonding status
    // note: they have min. mPower again so their bonding status changes to `1`, which means they are bonded again
    assert_eq!(MiningRewardsAllowanceTestModule::cooling_off_period_days_remaining(1), Some((1631145600000, 1, 1)));

    // params: total days, days remaining
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1631059200000, 1631145600000, 2u32, 1u32)));

    // we're up to 50:1 now
    assert_eq!(MiningRewardsAllowanceTestModule::min_bonded_dhx_daily(), Some(50_000_000_000_000_000_000_u128));

    check_pause_and_reset_rewards_multiplier_works(amount_bonded_each_miner.clone(), amount_mpower_each_miner.clone(), referendum_index.clone());
}

fn check_pause_and_reset_rewards_multiplier_works(amount_bonded_each_miner: u128, amount_mpower_each_miner: u128, referendum_index: u32) {
    // we want to check if pausing will prevent it from changing from 50:1 to 60:1 when the rewards multiplier for current period in days remaining ends

    assert_ok!(MiningRewardsAllowanceTestModule::change_rewards_multiplier_paused_status(true));

    // use original mpower
    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1631232000000i64);

    // 10th Sept 2021 @ ~7am is 1631257200000
    // 10th Sept 2021 @ 12am is 1631232000000 (start of day)
    Timestamp::set_timestamp(1631257200000u64);
    MiningRewardsAllowanceTestModule::on_initialize(15);

    // use original mpower
    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1631318400000i64);

    // 11th Sept 2021 @ ~7am is 1631343600000
    // 11th Sept 2021 @ 12am is 1631318400000 (start of day)
    Timestamp::set_timestamp(1631343600000u64);
    MiningRewardsAllowanceTestModule::on_initialize(16);

    // params: total days, days remaining
    // assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1631318400000, 1631318400000, 2u32, 2u32)));
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1631059200000, 1631145600000, 2u32, 1u32)));

    // we've paused it, so it should still be 50:10 (if we didn't pause it, it would have increased to 60:1 since we were at end of a reward multiplier period)
    assert_eq!(MiningRewardsAllowanceTestModule::min_bonded_dhx_daily(), Some(50_000_000_000_000_000_000_u128));

    // unpause again
    assert_ok!(MiningRewardsAllowanceTestModule::change_rewards_multiplier_paused_status(false));

    // reset - reset to change back to 10:1 instead of 50:1
    assert_ok!(MiningRewardsAllowanceTestModule::change_rewards_multiplier_reset_status(true));

    // use original mpower
    change_mpower_for_each_miner(amount_mpower_each_miner.clone(), 1631404800000i64);

    // 12th Sept 2021 @ ~7am is 1631430000000
    // 12th Sept 2021 @ 12am is 1631404800000 (start of day)
    Timestamp::set_timestamp(1631430000000u64);
    MiningRewardsAllowanceTestModule::on_initialize(17);

    // this starts reducing again since we unpaused it
    assert_eq!(MiningRewardsAllowanceTestModule::rewards_multiplier_current_period_days_remaining(), Some((1631059200000, 1631404800000, 2u32, 0u32)));

    // check that reset worked
    assert_eq!(MiningRewardsAllowanceTestModule::min_bonded_dhx_daily(), Some(TEN_DHX));
}
