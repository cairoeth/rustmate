//! Provides an implementation of the Auth library.
//!
//! The eponymous [`Auth`] type provides all the standard methods,
//! and is intended to be inherited by other contract types.
//!
//! Note that this code is unaudited and not fit for production use.

use alloy_primitives::{Address, FixedBytes};
use alloy_sol_types::{sol, SolError};
use core::{borrow::BorrowMut, marker::PhantomData};
use stylus_sdk::{contract, evm, msg, prelude::*};

pub trait AuthParams {}

sol_storage! {
    pub struct Auth<T: AuthParams> {
        address owner;
        address authority;
        bool initialized;
        PhantomData<T> phantom;
    }
}

// Declare events
sol! {
    event OwnershipTransferred(address indexed user, address indexed newOwner);
    event AuthorityUpdated(address indexed user, address indexed newAuthority);

    error Unauthorized();
    error AlreadyInitialized();
    error InvalidInitialize();
}

/// Represents the ways methods may fail.
pub enum AuthError {
    Unauthorized(Unauthorized),
    CallFailed(stylus_sdk::call::Error),
    AlreadyInitialized(AlreadyInitialized),
    InvalidInitialize(InvalidInitialize),
}

impl From<stylus_sdk::call::Error> for AuthError {
    fn from(err: stylus_sdk::call::Error) -> Self {
        Self::CallFailed(err)
    }
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<AuthError> for Vec<u8> {
    fn from(val: AuthError) -> Self {
        match val {
            AuthError::Unauthorized(err) => err.encode(),
            AuthError::CallFailed(err) => err.into(),
            AuthError::AlreadyInitialized(err) => err.encode(),
            AuthError::InvalidInitialize(err) => err.encode(),
        }
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = AuthError> = core::result::Result<T, E>;

impl<T: AuthParams> Auth<T> {
    fn can_call<S: TopLevelStorage>(
        storage: &mut S,
        authority: Address,
        user: Address,
        target: Address,
        sig: FixedBytes<4>,
    ) -> Result<bool> {
        let authority_given = Authority::new(authority);
        let status = authority_given.can_call(&mut *storage, user, target, sig)?;

        return Ok(status);
    }

    fn is_authorized<S: TopLevelStorage + BorrowMut<Self>>(
        storage: &mut S,
        user: Address,
        function_sig: FixedBytes<4>,
    ) -> Result<bool> {
        let authority = storage.borrow_mut().authority.get();

        return Ok(authority != Address::ZERO
            && Self::can_call(storage, authority, user, contract::address(), function_sig)?
            || user == storage.borrow_mut().owner.get());
    }
}

#[external]
impl<T: AuthParams> Auth<T> {
    pub fn owner(&self) -> Result<Address> {
        Ok(Address::from(self.owner.get()))
    }

    pub fn authority(&self) -> Result<Authority> {
        Ok(Authority::new(self.authority.get()))
    }

    pub fn set_authority<S: TopLevelStorage + BorrowMut<Self>>(
        storage: &mut S,
        new_authority: Address,
    ) -> Result<()> {
        let authority = storage.borrow_mut().authority.get();

        if msg::sender() != storage.borrow_mut().owner.get()
            || !(Self::can_call(
                storage,
                authority,
                msg::sender(),
                contract::address(),
                FixedBytes(contract::args(4).try_into().unwrap()),
            )?)
        {
            return Err(AuthError::Unauthorized(Unauthorized {}));
        }

        storage.borrow_mut().authority.set(new_authority);

        evm::log(AuthorityUpdated {
            user: msg::sender(),
            newAuthority: new_authority,
        });

        Ok(())
    }

    pub fn transfer_ownership<S: TopLevelStorage + BorrowMut<Self>>(
        storage: &mut S,
        new_owner: Address,
    ) -> Result<()> {
        if !Self::is_authorized(
            storage,
            msg::sender(),
            FixedBytes(contract::args(4).try_into().unwrap()),
        )? {
            return Err(AuthError::Unauthorized(Unauthorized {}));
        }

        storage.borrow_mut().owner.set(new_owner);

        evm::log(OwnershipTransferred {
            user: msg::sender(),
            newOwner: new_owner,
        });

        Ok(())
    }

    pub fn initialize(&mut self, _owner: Address, _authority: Address) -> Result<()> {
        if self.initialized.get() {
            return Err(AuthError::AlreadyInitialized(AlreadyInitialized {}));
        }

        if _owner.is_zero() || _authority.is_zero() {
            return Err(AuthError::InvalidInitialize(InvalidInitialize {}));
        }

        self.owner.set(_owner);
        self.authority.set(_authority);

        Ok(())
    }
}

sol_interface! {
    interface Authority {
        function canCall(
            address,
            address,
            bytes4
        ) external returns (bool);
    }
}
