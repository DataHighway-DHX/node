#![cfg_attr(not(feature = "std"), no_std)]

//! A pallet that implements a storage set on top of a sorted vec and demonstrates performance
//! tradeoffs when using map sets.

use account_set::AccountSet;
use frame_support::{
    decl_error,
    decl_event,
    decl_module,
    decl_storage,
    dispatch::DispatchResult,
    ensure,
};
use frame_system::{
    self as system,
    ensure_root,
};
use sp_std::{
    collections::btree_set::BTreeSet,
    prelude::*,
};

#[cfg(test)]
mod tests;

/// A maximum number of members. When membership reaches this number, no new members may join.
pub const MAX_MEMBERS: usize = 16;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as VecSet {
        // The set of all members. Stored as a single vec
        Members get(fn members): Vec<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        /// Added a member
        MemberAdded(AccountId),
        /// Removed a member
        MemberRemoved(AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Cannot join as a member because you are already a member
        AlreadyMember,
        /// Cannot give up membership because you are not currently a member
        NotMember,
        /// Cannot add another member because the limit is already reached
        MembershipLimitReached,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        type Error = Error<T>;

        /// Adds a member to the membership set unless the max is reached
        #[weight = 10_000]
        pub fn add_member(
            origin,
            new_member: T::AccountId,
        ) -> DispatchResult {
            // let _sender = ensure_root(origin)?;
            let _sender = ensure_root(origin)?;

            let mut members = Members::<T>::get();
            ensure!(members.len() < MAX_MEMBERS, Error::<T>::MembershipLimitReached);

            // We don't want to add duplicate members, so we check whether the potential new
            // member is already present in the list. Because the list is always ordered, we can
            // leverage the binary search which makes this check O(log n).
            match members.binary_search(&new_member) {
                // If the search succeeds, the caller is already a member, so just return
                Ok(_) => Err(Error::<T>::AlreadyMember.into()),
                // If the search fails, the caller is not a member and we learned the index where
                // they should be inserted
                Err(index) => {
                    members.insert(index, new_member.clone());
                    Members::<T>::put(members);
                    Self::deposit_event(RawEvent::MemberAdded(new_member));
                    Ok(())
                }
            }
        }

        /// Removes a member.
        #[weight = 10_000]
        pub fn remove_member(
            origin,
            old_member: T::AccountId,
        ) -> DispatchResult {
            // let _sender = ensure_root(origin)?;
            let _sender = ensure_root(origin)?;

            let mut members = Members::<T>::get();

            // We have to find out if the member exists in the sorted vec, and, if so, where.
            match members.binary_search(&old_member) {
                // If the search succeeds, the caller is a member, so remove her
                Ok(index) => {
                    members.remove(index);
                    Members::<T>::put(members);
                    Self::deposit_event(RawEvent::MemberRemoved(old_member));
                    Ok(())
                },
                // If the search fails, the caller is not a member, so just return
                Err(_) => Err(Error::<T>::NotMember.into()),
            }
        }

        // also see `append_or_insert`, `append_or_put` in pallet-elections/phragmen, democracy
    }
}

impl<T: Trait> AccountSet for Module<T> {
    type AccountId = T::AccountId;

    fn accounts() -> BTreeSet<T::AccountId> {
        Self::members().into_iter().collect::<BTreeSet<_>>()
    }
}
