//! Provides an implementation of the ERC-721 standard.
//!
//! The eponymous [`ERC721`] type provides all the standard methods,
//! and is intended to be inherited by other contract types.
//!
//! You can configure the behavior of [`ERC721`] via the [`ERC721Params`] trait,
//! which allows specifying the name, symbol, and token uri.
//!
//! Note that this code is unaudited and not fit for production use.

use alloc::{string::String, vec::Vec};
use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolError};
use core::{borrow::BorrowMut, marker::PhantomData};
use stylus_sdk::{abi::Bytes, evm, msg, prelude::*};

pub trait ERC721Params {
    const NAME: &'static str;

    const SYMBOL: &'static str;

    fn token_uri(token_id: U256) -> String;
}

sol_storage! {
    /// ERC721 implements all ERC-721 methods
    pub struct ERC721<T: ERC721Params> {
        mapping(uint256 => address) owner_of;
        mapping(address => uint256) balance_of;
        mapping(uint256 => address) get_approved;
        mapping(address => mapping(address => bool)) is_approved_for_all;
        PhantomData<T> phantom;
    }
}

// Declare events and Solidity error types
sol! {
    event Transfer(address indexed from, address indexed to, uint256 indexed id);
    event Approval(address indexed owner, address indexed spender, uint256 indexed id);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    error NotMinted();
    error ZeroAddress();
    error NotAuthorized();
    error WrongFrom();
    error InvalidRecipient();
    error UnsafeRecipient();
    error AlreadyMinted();
}

/// Represents the ways methods may fail.
pub enum ERC721Error {
    NotMinted(NotMinted),
    ZeroAddress(ZeroAddress),
    NotAuthorized(NotAuthorized),
    WrongFrom(WrongFrom),
    InvalidRecipient(InvalidRecipient),
    UnsafeRecipient(UnsafeRecipient),
    CallFailed(stylus_sdk::call::Error),
    AlreadyMinted(AlreadyMinted),
}

impl From<stylus_sdk::call::Error> for ERC721Error {
    fn from(err: stylus_sdk::call::Error) -> Self {
        Self::CallFailed(err)
    }
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<ERC721Error> for Vec<u8> {
    fn from(val: ERC721Error) -> Self {
        match val {
            ERC721Error::NotMinted(err) => err.encode(),
            ERC721Error::ZeroAddress(err) => err.encode(),
            ERC721Error::NotAuthorized(err) => err.encode(),
            ERC721Error::WrongFrom(err) => err.encode(),
            ERC721Error::InvalidRecipient(err) => err.encode(),
            ERC721Error::UnsafeRecipient(err) => err.encode(),
            ERC721Error::CallFailed(err) => err.into(),
            ERC721Error::AlreadyMinted(err) => err.encode(),
        }
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = ERC721Error> = core::result::Result<T, E>;

impl<T: ERC721Params> ERC721<T> {
    fn call_receiver<S: TopLevelStorage>(
        storage: &mut S,
        id: U256,
        from: Address,
        to: Address,
        data: Vec<u8>,
    ) -> Result<()> {
        if to.has_code() {
            let receiver = IERC721TokenReceiver::new(to);
            let received = receiver
                .on_erc_721_received(&mut *storage, msg::sender(), from, id, data)?
                .0;

            // 0x150b7a02 = bytes4(keccak256("onERC721Received(address,address,uint256,bytes)"))
            if u32::from_be_bytes(received) != 0x150b7a02 {
                return Err(ERC721Error::UnsafeRecipient(UnsafeRecipient {}));
            }
        }
        Ok(())
    }

    pub fn safe_transfer<S: TopLevelStorage + BorrowMut<Self>>(
        storage: &mut S,
        id: U256,
        from: Address,
        to: Address,
        data: Vec<u8>,
    ) -> Result<()> {
        storage.borrow_mut().transfer_from(from, to, id)?;
        Self::call_receiver(storage, id, from, to, data)
    }

    pub fn mint(&mut self, to: Address, id: U256) -> Result<()> {
        if to.is_zero() {
            return Err(ERC721Error::InvalidRecipient(InvalidRecipient {}));
        }

        if self.owner_of.get(id) != Address::ZERO {
            return Err(ERC721Error::AlreadyMinted(AlreadyMinted {}));
        }

        let mut to_balance = self.balance_of.setter(to);
        let balance = to_balance.get() + U256::from(1);
        to_balance.set(balance);

        self.owner_of.setter(id).set(to);

        evm::log(Transfer {
            from: Address::ZERO,
            to: to,
            id: id,
        });
    
        Ok(())
    }

    pub fn burn(&mut self, id: U256) -> Result<()> {
        let owner = self.owner_of.get(id);

        if owner.is_zero() {
            return Err(ERC721Error::NotMinted(NotMinted {}));
        }

        let mut owner_balance = self.balance_of.setter(owner);
        let balance = owner_balance.get() - U256::from(1);
        owner_balance.set(balance);

        self.owner_of.delete(id);

        self.get_approved.delete(id);

        evm::log(Transfer {
            from: owner,
            to: Address::ZERO,
            id: id,
        });

        Ok(())
    }

    pub fn safe_mint<S: TopLevelStorage>(&mut self, storage: &mut S, to: Address, id: U256) -> Result<()> {
        Self::mint(self, to, id)?;

        Self::call_receiver(storage, id, Address::ZERO, to, vec![])?;

        Ok(())
    }

    pub fn safe_mint_with_data<S: TopLevelStorage>(
        &mut self,
        storage: &mut S,
        to: Address,
        id: U256,
        data: Bytes,
    ) -> Result<()> {
        Self::mint(self, to, id)?;

        Self::call_receiver(storage, id, Address::ZERO, to, data.0)?;

        Ok(())
    }
}

#[external]
impl<T: ERC721Params> ERC721<T> {
    pub fn name() -> Result<String> {
        Ok(T::NAME.into())
    }

    pub fn symbol() -> Result<String> {
        Ok(T::SYMBOL.into())
    }

    pub fn owner_of(&self, id: U256) -> Result<Address> {
        let owner = self.owner_of.get(id);

        if owner.is_zero() {
            return Err(ERC721Error::NotMinted(NotMinted {}));
        }

        Ok(owner)
    }

    pub fn balance_of(&self, owner: Address) -> Result<U256> {
        if owner.is_zero() {
            return Err(ERC721Error::ZeroAddress(ZeroAddress {}));
        }

        Ok(self.balance_of.get(owner))
    }

    pub fn get_approved(&self, id: U256) -> Result<Address> {
        Ok(self.get_approved.get(id))
    }

    pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> Result<bool> {
        Ok(self.is_approved_for_all.getter(owner).get(operator))
    }
    
    #[selector(name = "tokenURI")]
    pub fn token_uri(&self, id: U256) -> Result<String> {
        Ok(T::token_uri(id))
    }

    pub fn approve(&mut self, spender: Address, id: U256) -> Result<()> {
        let owner = self.owner_of.get(id);

        if msg::sender() != owner || !self.is_approved_for_all.getter(owner).get(msg::sender()) {
            return Err(ERC721Error::NotAuthorized(NotAuthorized {}));
        }

        self.get_approved.setter(id).set(spender);

        evm::log(Approval {
            owner: owner,
            spender: spender,
            id: id,
        });

        Ok(())
    }

    pub fn set_approval_for_all(&mut self, operator: Address, approved: bool) -> Result<()> {
        self.is_approved_for_all.setter(msg::sender()).insert(operator, approved);

        evm::log(ApprovalForAll {
            owner: msg::sender(),
            operator: operator,
            approved: approved,
        });

        Ok(())
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, id: U256) -> Result<()> {
        if from != self.owner_of.get(id) {
            return Err(ERC721Error::WrongFrom(WrongFrom {}));
        }

        if to.is_zero() {
            return Err(ERC721Error::InvalidRecipient(InvalidRecipient {}));
        }

        if msg::sender() != from
            && !self.is_approved_for_all.getter(from).get(msg::sender())
            && msg::sender() != self.get_approved.get(id)
        {
            return Err(ERC721Error::NotAuthorized(NotAuthorized {}));
        }

        let mut from_balance = self.balance_of.setter(from);
        let balance = from_balance.get() - U256::from(1);
        from_balance.set(balance);

        let mut to_balance = self.balance_of.setter(to);
        let balance = to_balance.get() + U256::from(1);
        to_balance.set(balance);

        self.owner_of.setter(id).set(to);

        self.get_approved.delete(id);

        evm::log(Transfer {
            from: from,
            to: to,
            id: id,
        });

        Ok(())
    }

    pub fn safe_transfer_from<S: TopLevelStorage + BorrowMut<Self>>(
        storage: &mut S,
        from: Address,
        to: Address,
        id: U256,
    ) -> Result<()> {
        Self::safe_transfer_from_with_data(storage, from, to, id, Bytes(vec![]))
    }

    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from_with_data<S: TopLevelStorage + BorrowMut<Self>>(
        storage: &mut S,
        from: Address,
        to: Address,
        id: U256,
        data: Bytes,
    ) -> Result<()> {
        Self::safe_transfer(storage, id, from, to, data.0)
    }

    pub fn supports_interface(interface: [u8; 4]) -> Result<bool> {
        let supported = interface == 0x01ffc9a7u32.to_be_bytes() // ERC165 Interface ID for ERC165
            || interface == 0x80ac58cdu32.to_be_bytes() // ERC165 Interface ID for ERC721
            || interface == 0x780e9d63u32.to_be_bytes(); // ERC165 Interface ID for ERC721Metadata
        Ok(supported)
    }
}

sol_interface! {
    interface IERC721TokenReceiver {
        function onERC721Received(address operator, address from, uint256 token_id, bytes data) external returns(bytes4);
    }
}