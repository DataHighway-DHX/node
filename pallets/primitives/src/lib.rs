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

pub type Balance = u128;

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn my_test() {
    //   // Add test
    // }
}
