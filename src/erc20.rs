//! Provides an implementation of the ERC-20 standard.
//!
//! The eponymous [`ERC20`] type provides all the standard methods,
//! and is intended to be inherited by other contract types.
//!
//! You can configure the behavior of [`ERC20`] via the [`ERC20Params`] trait,
//! which allows specifying the name, symbol, and token uri.
//!
//! Note that this code is unaudited and not fit for production use.

use alloc::{string::String, vec::Vec};
use alloy_primitives::{address, Address, B256, U256};
use alloy_sol_types::{sol, sol_data, SolError, SolType};
use core::{marker::PhantomData};
use stylus_sdk::{evm, contract, block, msg, prelude::*};
use stylus_sdk::crypto;
use stylus_sdk::call::RawCall;

pub trait ERC20Params {
    const NAME: &'static str;

    const SYMBOL: &'static str;

    // TODO: Immutable tag?
    const DECIMALS: u8;

    const INITIAL_CHAIN_ID: u64;

    const INITIAL_DOMAIN_SEPARATOR: B256;
}

sol_storage! {
    /// ERC20 implements all ERC-20 methods
    pub struct ERC20<T: ERC20Params> {
        uint256 total_supply;
        mapping(address => uint256) balance;
        mapping(address => mapping(address => uint256)) allowance;
        mapping(address => uint256) nonces;
        PhantomData<T> phantom;
    }
}

// Declare events and Solidity error types
sol! {
    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);

    error PermitDeadlineExpired();
    error InvalidSigner();
}

/// Represents the ways methods may fail.
pub enum ERC20Error {
    PermitDeadlineExpired(PermitDeadlineExpired),
    InvalidSigner(InvalidSigner),
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<ERC20Error> for Vec<u8> {
    fn from(val: ERC20Error) -> Self {
        match val {
            ERC20Error::PermitDeadlineExpired(err) => err.encode(),
            ERC20Error::InvalidSigner(err) => err.encode(),
        }
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = ERC20Error> = core::result::Result<T, E>;

impl<T: ERC20Params> ERC20<T> {
    pub fn compute_domain_separator() -> Result<B256> {
        let mut digest_input = [0u8; 32 + 32];
        digest_input[0..32].copy_from_slice(&crypto::keccak("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)".as_bytes())[..]);
        digest_input[32..64].copy_from_slice(&crypto::keccak(T::NAME.as_bytes())[..]);
        digest_input[64..96].copy_from_slice(&crypto::keccak("1".as_bytes())[..]);
        digest_input[96..128].copy_from_slice(&block::chainid().to_be_bytes()[..]);
        digest_input[128..160].copy_from_slice(&contract::address()[..]);

        Ok(crypto::keccak(digest_input))
    }

    pub fn mint(&mut self, to: Address, amount: U256) {
        self.total_supply.set(self.total_supply.get() + amount);

        let mut balance_setter = self.balance.setter(to);
        let balance = balance_setter.get();
        balance_setter.set(balance + amount);

        evm::log(Transfer {
            from: Address::ZERO,
            to: to,
            amount: amount,
        });
    }

    pub fn burn(&mut self, from: Address, amount: U256) {
        let mut balance_setter = self.balance.setter(from);
        let balance = balance_setter.get();
        balance_setter.set(balance - amount);

        self.total_supply.set(self.total_supply.get() - amount);

        evm::log(Transfer {
            from: from,
            to: Address::ZERO,
            amount: amount,
        });
    }
}

#[external]
impl<T: ERC20Params> ERC20<T> {
    pub fn name() -> Result<String> {
        Ok(T::NAME.into())
    }

    pub fn symbol() -> Result<String> {
        Ok(T::SYMBOL.into())
    }

    pub fn decimals() -> Result<u8> {
        Ok(T::DECIMALS)
    }

    pub fn total_supply(&self) -> Result<U256> {
        Ok(self.total_supply.get())
    }

    pub fn balance_of(&self, owner: Address) -> Result<U256> {
        Ok(U256::from(self.balance.get(owner)))
    }

    pub fn allowance(&mut self, owner: Address, spender: Address) -> Result<U256> {
        Ok(self.allowance.getter(owner).get(spender))
    }

    pub fn nonces(&self, owner: Address) -> Result<U256> {
        Ok(U256::from(self.nonces.get(owner)))
    }

    pub fn approve(&mut self, spender: Address, amount: U256) -> Result<bool> {
        self.allowance
            .setter(msg::sender())
            .insert(spender, amount);

        evm::log(Approval {
            owner: msg::sender(),
            spender,
            amount,
        });

        Ok(true)
    }

    pub fn transfer(&mut self, to: Address, amount: U256) -> Result<bool> {
        let mut from_setter = self.balance.setter(msg::sender());
        let from_balance = from_setter.get();
        from_setter.set(from_balance - amount);

        let mut to_setter = self.balance.setter(to);
        let to_balance = to_setter.get();
        to_setter.set(to_balance + amount);

        evm::log(Transfer {
            from: msg::sender(),
            to,
            amount,
        });
        
        Ok(true)
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, amount: U256) -> Result<bool> {
        let allowed = self.allowance.getter(from).get(msg::sender());

        if allowed != U256::MAX {
            self.allowance
            .setter(from)
            .insert(msg::sender(), allowed - amount);
        }

        let mut from_setter = self.balance.setter(from);
        let from_balance = from_setter.get();
        from_setter.set(from_balance - amount);

        let mut to_setter = self.balance.setter(to);
        let to_balance = to_setter.get();
        to_setter.set(to_balance + amount);

        evm::log(Transfer {
            from,
            to,
            amount,
        });
        
        Ok(true)
    }

    pub fn permit(&mut self, owner: Address, spender: Address, value: U256, deadline: U256, v: u8, r: U256, s: U256) -> Result<()> {
        if deadline < U256::from(block::timestamp()) {
            return Err(ERC20Error::PermitDeadlineExpired(PermitDeadlineExpired {}));
        }

        let mut nonce_setter = self.balance.setter(owner);
        let nonce = nonce_setter.get();
        nonce_setter.set(nonce + U256::from(1));

        let mut struct_hash = [0u8; 32 + 32];
        struct_hash[0..32].copy_from_slice(&crypto::keccak(b"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)")[..]);
        struct_hash[32..64].copy_from_slice(&owner[..]);
        struct_hash[64..96].copy_from_slice(&spender[..]);
        struct_hash[96..128].copy_from_slice(&value.to_be_bytes_vec()[..]);
        // TODO: Increase nonce
        struct_hash[128..160].copy_from_slice(&nonce.to_be_bytes_vec()[..]);
        struct_hash[160..192].copy_from_slice(&deadline.to_be_bytes_vec()[..]);

        let mut digest_input = [0u8; 2 + 32 + 32];
        digest_input[0] = 0x19;
        digest_input[1] = 0x01;
        digest_input[2..34].copy_from_slice(&self.domain_separator()?[..]);
        digest_input[34..66].copy_from_slice(&crypto::keccak(struct_hash)[..]);

        let data = <sol! { (bytes32, uint8, uint256, uint256) }>::encode(&(*crypto::keccak(digest_input), v, r, s));

        let recovered_address = RawCall::new_static()
        .gas(evm::gas_left())
        .call(address!("0000000000000000000000000000000000000001"),  &data)
        .map(|ret| sol_data::Address::decode_single(ret.as_slice(), false).unwrap()).map_err(|_| ERC20Error::InvalidSigner(InvalidSigner {}))?;

        if recovered_address.is_zero() || recovered_address != owner {
            return Err(ERC20Error::InvalidSigner(InvalidSigner {}));
        }

        self.allowance
            .setter(recovered_address)
            .insert(spender, value);
        
        evm::log(Approval {
            owner,
            spender,
            amount: value,
        });

        Ok(())
    }

    pub fn domain_separator(&mut self) -> Result<B256> {
        if block::chainid() == T::INITIAL_CHAIN_ID {
            Ok(T::INITIAL_DOMAIN_SEPARATOR.into())
        } else {
            Ok(ERC20::<T>::compute_domain_separator()?)
        }
    }
}