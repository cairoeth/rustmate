//! Provides an implementation of an Owner contract.
//!
//! The eponymous [`Owner`] type provides all the standard methods,
//! and is intended to be inherited by other contract types.
//!
//! Note that this code is unaudited and not fit for production use.

use alloc::{vec::Vec};
use alloy_primitives::{Address};
use alloy_sol_types::{sol, SolError};
use core::{marker::PhantomData};
use stylus_sdk::{evm, msg, prelude::*};

pub trait OwnerParams {}

sol_storage! {
    /// Owner implements all ERC-6909 methods
    pub struct Owner<T: OwnerParams> {
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
pub enum OwnerError {
    Unauthorized(Unauthorized),
    AlreadyInitialized(AlreadyInitialized),
    InvalidInitialize(InvalidInitialize),
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<OwnerError> for Vec<u8> {
    fn from(val: OwnerError) -> Self {
        match val {
            OwnerError::Unauthorized(err) => err.encode(),
            OwnerError::AlreadyInitialized(err) => err.encode(),
            OwnerError::InvalidInitialize(err) => err.encode(),
        }
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = OwnerError> = core::result::Result<T, E>;

impl<T: OwnerParams> Owner<T> {
    pub fn only_owner(&mut self) -> Result<()> {
        if msg::sender() != self.owner.get() {
            return Err(OwnerError::Unauthorized(Unauthorized {}));
        }

        Ok(())
    }
}

#[external]
impl<T: OwnerParams> Owner<T> {
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
            return Err(OwnerError::AlreadyInitialized(AlreadyInitialized {}));
        }

        if _owner.is_zero() {
            return Err(OwnerError::InvalidInitialize(InvalidInitialize {}));
        }

        self.owner.set(_owner);

        Ok(())
    }
}
