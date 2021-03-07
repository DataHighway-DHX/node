// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Supernode Membership Module (Modification of Substrate's Membership Module)
//!
//! Allows control of membership of a set of `AccountId`s that represent Supernodes by the
//! Sudo `AccountId`. A prime member may be set.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    debug::native::debug,
    decl_error,
    decl_event,
    decl_module,
    decl_storage,
    ensure,
    traits::{
        ChangeMembers,
        Contains,
        EnsureOrigin,
        InitializeMembers,
    },
};
use frame_system::ensure_signed;
use sp_runtime::DispatchError;
use sp_std::prelude::*;

pub trait Trait<I = DefaultInstance>: frame_system::Trait + pallet_sudo::Trait {
    /// The overarching event type.
    type Event: From<Event<Self, I>> + Into<<Self as frame_system::Trait>::Event>;

    /// Required origin for adding a member (though can always be Root).
    type AddOrigin: EnsureOrigin<Self::Origin>;

    /// Required origin for removing a member (though can always be Root).
    type RemoveOrigin: EnsureOrigin<Self::Origin>;

    /// Required origin for adding and removing a member in a single action.
    type SwapOrigin: EnsureOrigin<Self::Origin>;

    /// Required origin for resetting membership.
    type ResetOrigin: EnsureOrigin<Self::Origin>;

    /// Required origin for setting or resetting the prime member.
    type PrimeOrigin: EnsureOrigin<Self::Origin>;

    /// The receiver of the signal for when the membership has been initialized. This happens pre-
    /// genesis and will usually be the same as `MembershipChanged`. If you need to do something
    /// different on initialization, then you can change this accordingly.
    type MembershipInitialized: InitializeMembers<Self::AccountId>;

    /// The receiver of the signal for when the membership has changed.
    type MembershipChanged: ChangeMembers<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as Membership {
        /// The current membership, stored as an ordered Vec.
        Members get(fn members): Vec<T::AccountId>;

        /// The current prime member, if one exists.
        Prime get(fn prime): Option<T::AccountId>;
    }
    add_extra_genesis {
        config(members): Vec<T::AccountId>;
        config(phantom): sp_std::marker::PhantomData<I>;
        build(|config: &Self| {
            let mut members = config.members.clone();
            members.sort();
            T::MembershipInitialized::initialize_members(&members);
            <Members<T, I>>::put(members);
        })
    }
}

decl_event!(
    pub enum Event<T, I=DefaultInstance> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait<I>>::Event,
    {
        /// The given member was added; see the transaction for who.
        MemberAdded,
        /// The given member was removed; see the transaction for who.
        MemberRemoved,
        /// Two members were swapped; see the transaction for who.
        MembersSwapped,
        /// The membership was reset; see the transaction for who the new set is.
        MembersReset,
        /// One of the members' keys changed.
        KeyChanged,
        /// Phantom member, never used.
        Dummy(sp_std::marker::PhantomData<(AccountId, Event)>),
    }
);

decl_error! {
    /// Error for the nicks module.
    pub enum Error for Module<T: Trait<I>, I: Instance> {
        /// Already a member.
        AlreadyMember,
        /// Not a member.
        NotMember,
    }
}

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance=DefaultInstance>
        for enum Call
        where origin: T::Origin
    {
        fn deposit_event() = default;

        /// Add a member `who` to the set.
        ///
        /// May only be called from `T::AddOrigin`.
        #[weight = 50_000_000]
        pub fn add_member(origin, who: T::AccountId) {
            T::AddOrigin::ensure_origin(origin.clone())?;
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_sudo(sender).is_ok(), "Only sudo is authorised");

            let mut members = <Members<T, I>>::get();
            let location = members.binary_search(&who).err().ok_or(Error::<T, I>::AlreadyMember)?;
            members.insert(location, who.clone());
            <Members<T, I>>::put(&members);

            T::MembershipChanged::change_members_sorted(&[who], &[], &members[..]);

            Self::deposit_event(RawEvent::MemberAdded);
        }

        /// Remove a member `who` from the set.
        ///
        /// May only be called from `T::RemoveOrigin`.
        #[weight = 50_000_000]
        pub fn remove_member(origin, who: T::AccountId) {
            T::RemoveOrigin::ensure_origin(origin.clone())?;
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_sudo(sender).is_ok(), "Only sudo is authorised");

            let mut members = <Members<T, I>>::get();
            let location = members.binary_search(&who).ok().ok_or(Error::<T, I>::NotMember)?;
            members.remove(location);
            <Members<T, I>>::put(&members);

            T::MembershipChanged::change_members_sorted(&[], &[who], &members[..]);
            Self::rejig_prime(&members);

            Self::deposit_event(RawEvent::MemberRemoved);
        }

        /// Swap out one member `remove` for another `add`.
        ///
        /// May only be called from `T::SwapOrigin`.
        ///
        /// Prime membership is *not* passed from `remove` to `add`, if extant.
        #[weight = 50_000_000]
        pub fn swap_member(origin, remove: T::AccountId, add: T::AccountId) {
            T::SwapOrigin::ensure_origin(origin.clone())?;
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_sudo(sender).is_ok(), "Only sudo is authorised");

            if remove == add { return Ok(()) }

            let mut members = <Members<T, I>>::get();
            let location = members.binary_search(&remove).ok().ok_or(Error::<T, I>::NotMember)?;
            let _ = members.binary_search(&add).err().ok_or(Error::<T, I>::AlreadyMember)?;
            members[location] = add.clone();
            members.sort();
            <Members<T, I>>::put(&members);

            T::MembershipChanged::change_members_sorted(
                &[add],
                &[remove],
                &members[..],
            );
            Self::rejig_prime(&members);

            Self::deposit_event(RawEvent::MembersSwapped);
        }

        /// Change the membership to a new set, disregarding the existing membership. Be nice and
        /// pass `members` pre-sorted.
        ///
        /// May only be called from `T::ResetOrigin`.
        #[weight = 50_000_000]
        pub fn reset_members(origin, members: Vec<T::AccountId>) {
            T::ResetOrigin::ensure_origin(origin.clone())?;
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_sudo(sender).is_ok(), "Only sudo is authorised");

            let mut members = members;
            members.sort();
            <Members<T, I>>::mutate(|m| {
                T::MembershipChanged::set_members_sorted(&members[..], m);
                Self::rejig_prime(&members);
                *m = members;
            });


            Self::deposit_event(RawEvent::MembersReset);
        }

        /// Swap out the sending member for some other key `new`.
        ///
        /// May only be called from `Signed` origin of a current member.
        ///
        /// Prime membership is passed from the origin account to `new`, if extant.
        #[weight = 50_000_000]
        pub fn change_key(origin, new: T::AccountId) {
            let remove = ensure_signed(origin.clone())?;
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_sudo(sender).is_ok(), "Only sudo is authorised");

            if remove != new {
                let mut members = <Members<T, I>>::get();
                let location = members.binary_search(&remove).ok().ok_or(Error::<T, I>::NotMember)?;
                let _ = members.binary_search(&new).err().ok_or(Error::<T, I>::AlreadyMember)?;
                members[location] = new.clone();
                members.sort();
                <Members<T, I>>::put(&members);

                T::MembershipChanged::change_members_sorted(
                    &[new.clone()],
                    &[remove.clone()],
                    &members[..],
                );

                if Prime::<T, I>::get() == Some(remove) {
                    Prime::<T, I>::put(&new);
                    T::MembershipChanged::set_prime(Some(new));
                }
            }

            Self::deposit_event(RawEvent::KeyChanged);
        }

        /// Set the prime member. Must be a current member.
        ///
        /// May only be called from `T::PrimeOrigin`.
        #[weight = 50_000_000]
        pub fn set_prime(origin, who: T::AccountId) {
            T::PrimeOrigin::ensure_origin(origin.clone())?;
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_sudo(sender).is_ok(), "Only sudo is authorised");

            Self::members().binary_search(&who).ok().ok_or(Error::<T, I>::NotMember)?;
            Prime::<T, I>::put(&who);
            T::MembershipChanged::set_prime(Some(who));
        }

        /// Remove the prime member if it exists.
        ///
        /// May only be called from `T::PrimeOrigin`.
        #[weight = 50_000_000]
        pub fn clear_prime(origin) {
            T::PrimeOrigin::ensure_origin(origin.clone())?;
            let sender = ensure_signed(origin)?;
            ensure!(Self::is_sudo(sender).is_ok(), "Only sudo is authorised");

            Prime::<T, I>::kill();
            T::MembershipChanged::set_prime(None);
        }
    }
}

impl<T: Trait<I>, I: Instance> Module<T, I> {
    fn is_sudo(sender: T::AccountId) -> Result<(), DispatchError> {
        debug!(target: "runtime", "Sender: {:#?}", sender);
        debug!(target: "runtime", "Sudo key: {:#?}", <pallet_sudo::Module<T>>::key());
        if sender == <pallet_sudo::Module<T>>::key() {
            Ok(())
        } else {
            Err(DispatchError::Other("Only sudo is authorised"))
        }
    }

    fn rejig_prime(members: &[T::AccountId]) {
        if let Some(prime) = Prime::<T, I>::get() {
            match members.binary_search(&prime) {
                Ok(_) => T::MembershipChanged::set_prime(Some(prime)),
                Err(_) => Prime::<T, I>::kill(),
            }
        }
    }
}

impl<T: Trait<I>, I: Instance> Contains<T::AccountId> for Module<T, I> {
    fn sorted_members() -> Vec<T::AccountId> {
        Self::members()
    }

    fn count() -> usize {
        Members::<T, I>::decode_len().unwrap_or(0)
    }
}
