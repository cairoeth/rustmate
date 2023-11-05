//! Provides an implementation of the ERC-6909 standard.
//!
//! The eponymous [`ERC6909`] type provides all the standard methods,
//! and is intended to be inherited by other contract types.
//!
//! You can configure the behavior of [`ERC6909`] via the [`ERC6909Params`] trait,
//! which allows specifying the name, symbol, and token uri.
//!
//! Note that this code is unaudited and not fit for production use.

use alloc::{vec::Vec};
use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol};
use core::marker::PhantomData;
use stylus_sdk::{evm, msg, prelude::*};

pub trait ERC6909Params {}

sol_storage! {
    /// ERC6909 implements all ERC-6909 methods
    pub struct ERC6909<T: ERC6909Params> {
        mapping(uint256 => uint256) total_supply;
        mapping(address => mapping(address => bool)) is_operator;
        mapping(address => mapping(uint256 => uint256)) balance_of;
        mapping(address => mapping(address => mapping(uint256 => uint256))) allowance;
        PhantomData<T> phantom;
    }
}

// Declare events and Solidity error types
sol! {
    event OperatorSet(address indexed owner, address indexed operator, bool approved);
    event Approval(address indexed owner, address indexed spender, uint256 indexed id, uint256 amount);
    event Transfer(address caller, address indexed from, address indexed to, uint256 indexed id, uint256 amount);
}

/// Represents the ways methods may fail.
pub enum ERC6909Error {}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<ERC6909Error> for Vec<u8> {
    fn from(val: ERC6909Error) -> Self {
        match val {}
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = ERC6909Error> = core::result::Result<T, E>;

impl<T: ERC6909Params> ERC6909<T> {
    pub fn mint(&mut self, receiver: Address, id: U256, amount: U256) {
        let mut total_supply = self.total_supply.setter(id);
        let supply = total_supply.get() + amount;
        total_supply.set(supply);

        let mut to_balance = self.balance_of.setter(receiver);
        let balance = to_balance.get(id) + amount;
        to_balance.insert(id, balance);

        evm::log(Transfer {
            caller: msg::sender(),
            from: Address::ZERO,
            to: receiver,
            id: id,
            amount: amount,
        });
    }

    pub fn burn(&mut self, sender: Address, id: U256, amount: U256) {
        let mut from_balance = self.balance_of.setter(sender);
        let balance = from_balance.get(id) - amount;
        from_balance.insert(id, balance);

        let mut total_supply = self.total_supply.setter(id);
        let supply = total_supply.get() - amount;
        total_supply.set(supply);

        evm::log(Transfer {
            caller: msg::sender(),
            from: sender,
            to: Address::ZERO,
            id: id,
            amount: amount,
        });
    }
}

#[external]
impl<T: ERC6909Params> ERC6909<T> {
    pub fn transfer(&mut self, receiver: Address, id: U256, amount: U256) -> Result<bool> {
        let mut from_balance = self.balance_of.setter(msg::sender());
        let balance = from_balance.get(id) - amount;
        from_balance.insert(id, balance);

        let mut to_balance = self.balance_of.setter(receiver);
        let balance = to_balance.get(id) + amount;
        to_balance.insert(id, balance);

        evm::log(Transfer {
            caller: msg::sender(),
            from: msg::sender(),
            to: receiver,
            id: id,
            amount: amount,
        });

        Ok(true)
    }

    pub fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool> {
        if msg::sender() != sender && !self.is_operator.getter(sender).get(msg::sender()) {
            let allowed = self.allowance.getter(sender).getter(msg::sender()).get(id);
            if allowed != U256::MAX {
                self.allowance
                    .setter(sender)
                    .setter(msg::sender())
                    .insert(id, allowed - amount);
            }
        }

        let mut from_balance = self.balance_of.setter(sender);
        let balance = from_balance.get(id) - amount;
        from_balance.insert(id, balance);

        let mut to_balance = self.balance_of.setter(receiver);
        let balance = to_balance.get(id) + amount;
        to_balance.insert(id, balance);

        evm::log(Transfer {
            caller: msg::sender(),
            from: sender,
            to: receiver,
            id: id,
            amount: amount,
        });

        Ok(true)
    }

    pub fn approve(&mut self, spender: Address, id: U256, amount: U256) -> Result<bool> {
        self.allowance
            .setter(msg::sender())
            .setter(spender)
            .insert(id, amount);

        evm::log(Approval {
            owner: msg::sender(),
            spender: spender,
            id: id,
            amount: amount,
        });

        Ok(true)
    }

    pub fn set_operator(&mut self, operator: Address, approved: bool) -> Result<bool> {
        self.is_operator
            .setter(msg::sender())
            .insert(operator, approved);

        evm::log(OperatorSet {
            owner: msg::sender(),
            operator: operator,
            approved: approved,
        });

        Ok(true)
    }

    pub fn supports_interface(interface: [u8; 4]) -> Result<bool> {
        let supported = interface == 0x01ffc9a7u32.to_be_bytes() // ERC165 Interface ID for ERC165
            || interface == 0xb2e69f8au32.to_be_bytes(); // ERC165 Interface ID for ERC6909
        Ok(supported)
    }
}
