#![cfg_attr(not(feature = "std"), no_std)]

use codec::{
    Decode,
    Encode,
};
use frame_support::traits::{
    Currency,
    ExistenceRequirement,
    Randomness,
};
/// A runtime module for managing non-fungible tokens
use frame_support::{
    decl_event,
    decl_module,
    decl_storage,
    ensure,
    Parameter,
};
use frame_system::{
    self as system,
    ensure_signed,
};
use sp_io::hashing::blake2_128;
use sp_runtime::{
    traits::{
        AtLeast32Bit,
        Bounded,
        Member,
        One,
    },
    DispatchError,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type RoamingOperatorIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type Currency: Currency<Self::AccountId>;
    type Randomness: Randomness<Self::Hash>;
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct RoamingOperator(pub [u8; 16]);

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::RoamingOperatorIndex,
        Balance = BalanceOf<T>,
    {
        /// A roaming operator is created. (owner, roaming_operator_id)
        Created(AccountId, RoamingOperatorIndex),
        /// A roaming operator is transferred. (from, to, roaming_operator_id)
        Transferred(AccountId, AccountId, RoamingOperatorIndex),
        /// A roaming operator is available for sale. (owner, roaming_operator_id, price)
        PriceSet(AccountId, RoamingOperatorIndex, Option<Balance>),
        /// A roaming operator is sold. (from, to, roaming_operator_id, price)
        Sold(AccountId, AccountId, RoamingOperatorIndex, Balance),
    }
);

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as RoamingOperators {
        /// Stores all the roaming operators, key is the roaming operator id / index
        pub RoamingOperators get(fn roaming_operator): map hasher(blake2_128_concat) T::RoamingOperatorIndex => Option<RoamingOperator>;

        /// Stores the total number of roaming operators. i.e. the next roaming operator index
        pub RoamingOperatorsCount get(fn roaming_operators_count): T::RoamingOperatorIndex;

        /// Get roaming operator owner
        pub RoamingOperatorOwners get(fn roaming_operator_owner): map hasher(blake2_128_concat) T::RoamingOperatorIndex => Option<T::AccountId>;

        /// Get roaming operator price. None means not for sale.
        pub RoamingOperatorPrices get(fn roaming_operator_price): map hasher(blake2_128_concat) T::RoamingOperatorIndex => Option<BalanceOf<T>>
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        /// Create a new roaming operator
        pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let roaming_operator_id = Self::next_roaming_operator_id()?;

            // Generate a random 128bit value
            let unique_id = Self::random_value(&sender);

            // Create and store roaming operator
            let roaming_operator = RoamingOperator(unique_id);
            Self::insert_roaming_operator(&sender, roaming_operator_id, roaming_operator);

            Self::deposit_event(RawEvent::Created(sender, roaming_operator_id));
        }

        /// Transfer a roaming operator to new owner
        pub fn transfer(origin, to: T::AccountId, roaming_operator_id: T::RoamingOperatorIndex) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_operator_owner(roaming_operator_id) == Some(sender.clone()), "Only owner can transfer roaming operator");

            Self::update_owner(&to, roaming_operator_id);

            Self::deposit_event(RawEvent::Transferred(sender, to, roaming_operator_id));
        }

        /// Set a price for a roaming operator for sale
        /// None to delist the roaming operator
        pub fn set_price(origin, roaming_operator_id: T::RoamingOperatorIndex, price: Option<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            ensure!(Self::roaming_operator_owner(roaming_operator_id) == Some(sender.clone()), "Only owner can set price for roaming operator");

            if let Some(ref price) = price {
                <RoamingOperatorPrices<T>>::insert(roaming_operator_id, price);
            } else {
                <RoamingOperatorPrices<T>>::remove(roaming_operator_id);
            }

            Self::deposit_event(RawEvent::PriceSet(sender, roaming_operator_id, price));
        }

        /// Buy a roaming operator with max price willing to pay
        pub fn buy(origin, roaming_operator_id: T::RoamingOperatorIndex, price: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            let owner = Self::roaming_operator_owner(roaming_operator_id);
            ensure!(owner.is_some(), "RoamingOperator owner does not exist");
            let owner = owner.unwrap();

            let roaming_operator_price = Self::roaming_operator_price(roaming_operator_id);
            ensure!(roaming_operator_price.is_some(), "RoamingOperator not for sale");

            let roaming_operator_price = roaming_operator_price.unwrap();
            ensure!(price >= roaming_operator_price, "Price is too low");

            T::Currency::transfer(&sender, &owner, roaming_operator_price, ExistenceRequirement::AllowDeath)?;

            <RoamingOperatorPrices<T>>::remove(roaming_operator_id);

            Self::update_owner(&sender, roaming_operator_id);

            Self::deposit_event(RawEvent::Sold(owner, sender, roaming_operator_id, roaming_operator_price));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_roaming_operator_owner(
        roaming_operator_id: T::RoamingOperatorIndex,
        sender: T::AccountId,
    ) -> Result<(), DispatchError> {
        ensure!(
            Self::roaming_operator_owner(&roaming_operator_id).map(|owner| owner == sender).unwrap_or(false),
            "Sender is not owner of RoamingOperator"
        );
        Ok(())
    }

    pub fn exists_roaming_operator(
        roaming_operator_id: T::RoamingOperatorIndex,
    ) -> Result<RoamingOperator, DispatchError> {
        match Self::roaming_operator(roaming_operator_id) {
            Some(roaming_operator) => Ok(roaming_operator),
            None => Err(DispatchError::Other("RoamingOperator does not exist")),
        }
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random(&[0]),
            sender,
            <frame_system::Module<T>>::extrinsic_index(),
            <frame_system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }

    fn next_roaming_operator_id() -> Result<T::RoamingOperatorIndex, DispatchError> {
        let roaming_operator_id = Self::roaming_operators_count();
        if roaming_operator_id == <T::RoamingOperatorIndex as Bounded>::max_value() {
            return Err(DispatchError::Other("RoamingOperators count overflow"));
        }
        Ok(roaming_operator_id)
    }

    fn insert_roaming_operator(
        owner: &T::AccountId,
        roaming_operator_id: T::RoamingOperatorIndex,
        roaming_operator: RoamingOperator,
    ) {
        // Create and store roaming operator
        <RoamingOperators<T>>::insert(roaming_operator_id, roaming_operator);
        <RoamingOperatorsCount<T>>::put(roaming_operator_id + One::one());
        <RoamingOperatorOwners<T>>::insert(roaming_operator_id, owner.clone());
    }

    fn update_owner(to: &T::AccountId, roaming_operator_id: T::RoamingOperatorIndex) {
        <RoamingOperatorOwners<T>>::insert(roaming_operator_id, to);
    }
}
