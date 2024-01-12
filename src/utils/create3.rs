//! Provides an implementation of the CREATE3 library.
//!
//! The eponymous [`CREATE3`] type provides all the standard methods,
//! and is intended to be inherited by other contract types.
//!
//! Note that this code is unaudited and not fit for production use.

use alloc::vec::Vec;
use alloy_primitives::{
    Address,
    B256,
    U256,
};
use alloy_sol_types::{
    sol,
    sol_data,
    SolError,
    SolType,
};
use core::marker::PhantomData;
use stylus_sdk::call::RawCall;
use stylus_sdk::contract;
use stylus_sdk::crypto;
use stylus_sdk::deploy::RawDeploy;
use stylus_sdk::{
    evm,
    prelude::*,
};

const KECCAK256_PROXY_CHILD_BYTECODE: [u8; 32] = [
    33, 195, 93, 190, 27, 52, 74, 36, 136, 207, 51, 33, 214, 206, 84, 47, 142, 159, 48, 85, 68,
    255, 9, 228, 153, 58, 98, 49, 154, 73, 124, 31,
];

pub trait CREATE3Params {}

sol_storage! {
    pub struct CREATE3<T: CREATE3Params> {
        PhantomData<T> phantom;
    }
}

sol! {
    error DeploymentFailed();
    error InitilizationFailed();
}

/// Represents the ways methods may fail.
pub enum CREATE3Error {
    DeploymentFailed(DeploymentFailed),
    InitilizationFailed(InitilizationFailed),
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<CREATE3Error> for Vec<u8> {
    fn from(val: CREATE3Error) -> Self {
        match val {
            CREATE3Error::DeploymentFailed(err) => err.encode(),
            CREATE3Error::InitilizationFailed(err) => err.encode(),
        }
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = CREATE3Error> = core::result::Result<T, E>;

impl<T: CREATE3Params> CREATE3<T> {
    pub fn deploy(salt: B256, creation_code: &[u8], value: U256) -> Result<Address> {
        if let Ok(proxy) = unsafe { RawDeploy::new().salt(salt).deploy(creation_code, value) } {
            let deployed = Self::get_deployed(salt)?;

            RawCall::new_static()
                .gas(evm::gas_left())
                .call(proxy, creation_code)
                .map(|ret| sol_data::Address::decode_single(ret.as_slice(), false).unwrap())
                .map_err(|_| CREATE3Error::InitilizationFailed(InitilizationFailed {}))?;

            Ok(deployed)
        } else {
            Err(CREATE3Error::DeploymentFailed(DeploymentFailed {}))
        }
    }

    pub fn get_deployed(salt: B256) -> Result<Address> {
        Self::get_deployed_with_creator(salt, contract::address())
    }

    pub fn get_deployed_with_creator(salt: B256, creator: Address) -> Result<Address> {
        let mut proxy_packed = [0u8; 1 + 20 + 32 + 32];
        proxy_packed[0] = 0xFF;
        proxy_packed[1..21].copy_from_slice(&creator[..]);
        proxy_packed[21..53].copy_from_slice(&salt[..]);
        proxy_packed[53..85].copy_from_slice(&KECCAK256_PROXY_CHILD_BYTECODE[..]);

        let proxy = Address::from_word(crypto::keccak(proxy_packed));

        let mut packed = [0u8; 1 + 1 + 20 + 1];
        packed[0] = 0xd6;
        packed[1] = 0x94;
        packed[2..22].copy_from_slice(&proxy[..]);
        packed[22] = 0x01;

        Ok(Address::from_word(crypto::keccak(packed)))
    }
}

#[external]
impl<T: CREATE3Params> CREATE3<T> {}
