//! Provides an implementation of the ERC-1155 standard.
//!
//! The eponymous [`ERC1155`] type provides all the standard methods,
//! and is intended to be inherited by other contract types.
//!
//! You can configure the behavior of [`ERC1155`] via the [`ERC1155Params`] trait,
//! which allows specifying the name, symbol, and token uri.
//!
//! Note that this code is unaudited and not fit for production use.

use alloc::{string::String, vec::Vec};
use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolError};
use core::{borrow::BorrowMut, marker::PhantomData};
use stylus_sdk::{abi::Bytes, evm, msg, prelude::*};

pub trait ERC1155Params {
    fn uri(id: U256) -> String;
}

sol_storage! {
    /// ERC1155 implements all ERC-1155 methods
    pub struct ERC1155<T: ERC1155Params> {
        mapping(address => mapping(uint256 => uint256)) balance_of;
        mapping(address => mapping(address => bool)) is_approved_for_all;

        PhantomData<T> phantom;
    }
}

// Declare events and Solidity error types
sol! {
    event TransferSingle(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256 id,
        uint256 amount
    );
    event TransferBatch(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256[] ids,
        uint256[] amounts
    );
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    event URI(string value, uint256 indexed id);

    error NotAuthorized();
    error UnsafeRecipient();
}

/// Represents the ways methods may fail.
pub enum ERC1155Error {
    NotAuthorized(NotAuthorized),
    CallFailed(stylus_sdk::call::Error),
    UnsafeRecipient(UnsafeRecipient),
}

impl From<stylus_sdk::call::Error> for ERC1155Error {
    fn from(err: stylus_sdk::call::Error) -> Self {
        Self::CallFailed(err)
    }
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<ERC1155Error> for Vec<u8> {
    fn from(val: ERC1155Error) -> Self {
        match val {
            ERC1155Error::CallFailed(err) => err.into(),
            ERC1155Error::NotAuthorized(err) => err.encode(),
            ERC1155Error::UnsafeRecipient(err) => err.encode(),
        }
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = ERC1155Error> = core::result::Result<T, E>;

impl<T: ERC1155Params> ERC1155<T> {
    fn call_receiver<S: TopLevelStorage>(
        storage: &mut S,
        id: U256,
        from: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<()> {
        if to.has_code() {
            let receiver = IERC1155TokenReceiver::new(to);
            let received = receiver
                .on_erc_1155_received(&mut *storage, msg::sender(), from, id, value, data)?
                .0;

            // TODO: Update selector.
            // 0x150b7a02 = bytes4(keccak256("onERC1155Received(address,address,uint256,uint256,bytes)"))
            if u32::from_be_bytes(received) != 0x150b7a02 {
                return Err(ERC1155Error::UnsafeRecipient(UnsafeRecipient {}));
            }
        } else {
            if to == Address::ZERO {
                return Err(ERC1155Error::UnsafeRecipient(UnsafeRecipient {}));
            }
        }

        Ok(())
    }
}

#[external]
impl<T: ERC1155Params> ERC1155<T> {
    pub fn balance_of(&self, owner: Address, id: U256) -> Result<U256> {
        Ok(self.balance_of.getter(owner).get(id))
    }

    pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> Result<bool> {
        Ok(self.is_approved_for_all.getter(owner).get(operator))
    }

    #[selector(name = "uri")]
    pub fn uri(&self, id: U256) -> Result<String> {
        Ok(T::uri(id))
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

    pub fn safe_transfer_from<S: TopLevelStorage + BorrowMut<Self>>(
        storage: &mut S,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
        data: Bytes,
    ) -> Result<()> {
        if msg::sender() != from || !storage.borrow_mut().is_approved_for_all.getter(from).get(msg::sender()) {
            return Err(ERC1155Error::NotAuthorized(NotAuthorized {}));
        }

        let mut from_balance = storage.borrow_mut().balance_of.setter(from);
        let balance = from_balance.get(id) - amount;
        from_balance.insert(id, balance);

        let mut to_balance = storage.borrow_mut().balance_of.setter(to);
        let balance = to_balance.get(id) + amount;
        to_balance.insert(id, balance);

        evm::log(TransferSingle {
            operator: msg::sender(),
            from: from,
            to: to,
            id: id,
            amount: amount,
        });

        Self::call_receiver(storage, id, from, to, amount, data.0)
    }
}

sol_interface! {
    interface IERC1155TokenReceiver {
        function onERC1155Received(
            address,
            address,
            uint256,
            uint256,
            bytes calldata
        ) external returns (bytes4);
    
        function onERC1155BatchReceived(
            address,
            address,
            uint256[] calldata,
            uint256[] calldata,
            bytes calldata
        ) external returns (bytes4);
    }
}
