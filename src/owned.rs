//! Provides an implementation of an Owned contract.
//!
//! The eponymous [`Owned`] type provides all the standard methods,
//! and is intended to be inherited by other contract types.
//!
//! Note that this code is unaudited and not fit for production use.

use alloc::{vec::Vec};
use alloy_primitives::{Address};
use alloy_sol_types::{sol, SolError};
use core::{marker::PhantomData};
use stylus_sdk::{evm, msg, prelude::*};

pub trait OwnedParams {}

sol_storage! {
    pub struct Owned<T: OwnedParams> {
        address owner;
        bool initialized;
        PhantomData<T> phantom;
    }
}

// Declare events and Solidity error types
sol! {
    event OwnershipTransferred(address indexed user, address indexed newOwner);

    error Unauthorized();
    error AlreadyInitialized();
    error InvalidInitialize();
}

/// Represents the ways methods may fail.
pub enum OwnedError {
    Unauthorized(Unauthorized),
    AlreadyInitialized(AlreadyInitialized),
    InvalidInitialize(InvalidInitialize),
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<OwnedError> for Vec<u8> {
    fn from(val: OwnedError) -> Self {
        match val {
            OwnedError::Unauthorized(err) => err.encode(),
            OwnedError::AlreadyInitialized(err) => err.encode(),
            OwnedError::InvalidInitialize(err) => err.encode(),
        }
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = OwnedError> = core::result::Result<T, E>;

impl<T: OwnedParams> Owned<T> {
    pub fn only_owner(&mut self) -> Result<()> {
        if msg::sender() != self.owner.get() {
            return Err(OwnedError::Unauthorized(Unauthorized {}));
        }

        Ok(())
    }
}

#[external]
impl<T: OwnedParams> Owned<T> {
    pub fn transfer_ownership(&mut self, newOwner: Address) -> Result<()> {
        self.only_owner()?;

        self.owner.set(newOwner);

        evm::log(OwnershipTransferred {
            user: msg::sender(),
            newOwner: newOwner,
        });

        Ok(())
    }

    pub fn initialize(&mut self, _owner: Address) -> Result<()> {
        if self.initialized.get() {
            return Err(OwnedError::AlreadyInitialized(AlreadyInitialized {}));
        }

        if _owner.is_zero() {
            return Err(OwnedError::InvalidInitialize(InvalidInitialize {}));
        }

        self.owner.set(_owner);

        Ok(())
    }
}
