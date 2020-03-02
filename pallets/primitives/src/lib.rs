#![cfg_attr(not(feature = "std"), no_std)]

use codec::{
    Decode,
    Encode,
    Error,
    Input,
};
use sp_runtime::{
    traits::Convert,
    RuntimeDebug,
};
use sp_std::{
    prelude::*,
    vec,
};

#[macro_use]
extern crate bitmask;

#[cfg(feature = "std")]
use serde::{
    Deserialize,
    Serialize,
};

pub use orml_prices::Price;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
  DHX = 0,
  AUSD,
}

pub type Balance = u128;

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn my_test() {
    //   // Add test
    // }
}
