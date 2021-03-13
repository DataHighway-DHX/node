#![cfg_attr(not(feature = "std"), no_std)]

//! A pallet that funds the pallet_treasury's account_id in the genesis block

use frame_support::{
    debug,
    decl_error,
    decl_event,
    decl_module,
    decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{
        Currency,
        ExistenceRequirement,
    },
};
use frame_system::{
    self as system,
    ensure_signed,
};
use hex_literal::hex;
use sp_core::crypto::UncheckedFrom;
use sp_std::prelude::*;

pub trait Trait:
    frame_system::Trait + roaming_operators::Trait + pallet_treasury::Trait + pallet_balances::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
    // type AccountId: UncheckedFrom<<Self as frame_system::Config>::Hash> + AsRef<[u8]>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        BalanceOf = BalanceOf<T>,
    {
        TreasuryFundedWithUnlockedReserves(AccountId, AccountId, BalanceOf),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        fn on_finalize(current_block_number: T::BlockNumber) {
            debug::info!("treasury-dao - on_finalize");
            debug::info!("treasury-dao - current block number {:#?}", current_block_number);

            if <frame_system::Module<T>>::block_number() == 0.into() {
                debug::info!("treasury-dao - on_finalize: Genesis block");
                let treasury_account_id: T::AccountId = <pallet_treasury::Module<T>>::account_id();
                // FIXME - why does this give error:
                // `the trait Wraps is not implemented for <T as frame_system::Trait>::AccountId`
                // let endowed_account_id = UncheckedFrom::unchecked_from(hex!("6d6f646c70792f74727372790000000000000000000000000000000000000000").into());
                // let balance_to_deposit = <T as Trait>::Currency::free_balance(&endowed_account_id);

                // if balance_to_deposit > 0.into() {
                //     <T as Trait>::Currency::transfer(
                //         &endowed_account_id,
                //         &treasury_account_id,
                //         balance_to_deposit,
                //         ExistenceRequirement::KeepAlive
                //     );
                // }

                // // Emit event since treasury funded with unlocked reserves from endowed account
                // Self::deposit_event(RawEvent::TreasuryFundedWithUnlockedReserves(
                //     endowed_account_id,
                //     treasury_account_id,
                //     balance_to_deposit
                // ));
            } else {
                debug::info!("treasury-dao - on_finalize: Not genesis block");
            }
        }
    }
}
