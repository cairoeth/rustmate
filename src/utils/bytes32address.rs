//! Provides an implementation of the Bytes32AddressLib library.
//!
//! The eponymous [`Bytes32AddressLib`] type provides all the standard methods,
//! and is intended to be inherited by other contract types.
//!
//! Note that this code is unaudited and not fit for production use.

use alloy_primitives::{Address, FixedBytes};
use core::marker::PhantomData;
use stylus_sdk::prelude::*;

pub trait Bytes32AddressLibParams {}

sol_storage! {
    pub struct Bytes32AddressLib<T: Bytes32AddressLibParams> {
        PhantomData<T> phantom;
    }
}

impl<T: Bytes32AddressLibParams> Bytes32AddressLib<T> {
    pub fn from_last_20_bytes(bytes_value: FixedBytes<32>) -> Address {
        Address::from_word(bytes_value)
    }

    pub fn fill_last_12_bytes(address_value: Address) -> FixedBytes<32> {
        address_value.into_word()
    }
}

#[external]
impl<T: Bytes32AddressLibParams> Bytes32AddressLib<T> {}
