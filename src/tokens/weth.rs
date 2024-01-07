#![cfg_attr(not(feature = "export-abi"), no_main, no_std)]
extern crate alloc;

mod erc20;

use create::erc20::{ERC20Params, ERC20};
use alloc::vec::Vec;
use alloy_primitives::{B256, U256};
use alloy_sol_types::sol;
use stylus_sdk::{call, evm, msg, prelude::*};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct WETHParams;

/// Immutable definitions
impl ERC20Params for WETHParams {
    const NAME: &'static str = "Wrapped Ether";
    const SYMBOL: &'static str = "WETH";
    const DECIMALS: u8 = 18;
    const INITIAL_CHAIN_ID: u64 = 1;
    const INITIAL_DOMAIN_SEPARATOR: B256 = B256::ZERO;
}

// The contract
sol_storage! {
    #[entrypoint] // Makes WETH the entrypoint
    struct WETH {
        #[borrow] // Allows ERC20 to access WETH's storage and make calls
        ERC20<WETHParams> erc20;
    }
}

sol! {
    event Deposit(address indexed from, uint256 amount);
    event Withdrawal(address indexed to, uint256 amount);
}

#[external]
#[inherit(ERC20<WETHParams>)]
impl WETH {
    #[payable]
    pub fn deposit(&mut self) -> Result<(), Vec<u8>> {
        self.erc20.mint(msg::sender(), msg::value());

        evm::log(Deposit {
            from: msg::sender(),
            amount: msg::value(),
        });

        Ok(())
    }

    pub fn withdraw(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.erc20.burn(msg::sender(), amount);

        evm::log(Withdrawal {
            to: msg::sender(),
            amount: amount,
        });

        call::transfer_eth(msg::sender(), amount)
    }

    #[payable]
    pub fn receive(&mut self) -> Result<(), Vec<u8>> {
        self.deposit()
    }
}
