#![cfg_attr(not(feature = "export-abi"), no_main, no_std)]
extern crate alloc;

use crate::erc20::{ERC20Params, ERC20};
use alloc::vec::Vec;
use alloy_primitives::{Address, B256, U256};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::{evm, msg, prelude::*};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod erc20;

struct ERC4626Params;

/// Immutable definitions
impl ERC20Params for ERC4626Params {
    const NAME: &'static str = "Vault";
    const SYMBOL: &'static str = "VLT";
    const DECIMALS: u8 = 18;
    const INITIAL_CHAIN_ID: u64 = 1;
    const INITIAL_DOMAIN_SEPARATOR: B256 = B256::ZERO;
}

// The contract
sol_storage! {
    #[entrypoint] // Makes ERC4626 the entrypoint
    struct ERC4626 {
        #[borrow] // Allows ERC20 to access ERC4626's storage and make calls
        ERC20<ERC4626Params> erc20;
        address asset;
        bool initialized;
    }
}

sol! {
    event Deposit(address indexed caller, address indexed owner, uint256 assets, uint256 shares);

    event Withdraw(
        address indexed caller,
        address indexed receiver,
        address indexed owner,
        uint256 assets,
        uint256 shares
    );

    error Unauthorized();
    error AlreadyInitialized();
    error InvalidInitialize();
    error ZeroShares();
}

/// Represents the ways methods may fail.
pub enum ERC4626Error {
    Unauthorized(Unauthorized),
    AlreadyInitialized(AlreadyInitialized),
    InvalidInitialize(InvalidInitialize),
    ZeroShares(ZeroShares),
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<ERC4626Error> for Vec<u8> {
    fn from(val: ERC4626Error) -> Self {
        match val {
            ERC4626Error::Unauthorized(err) => err.encode(),
            ERC4626Error::AlreadyInitialized(err) => err.encode(),
            ERC4626Error::InvalidInitialize(err) => err.encode(),
            ERC4626Error::ZeroShares(err) => err.encode(),
        }
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = ERC4626Error> = core::result::Result<T, E>;

impl ERC4626 {
    pub fn before_withdraw(&mut self, assets: U256, shares: U256) {}

    pub fn after_deposit(&mut self, assets: U256, shares: U256) {}
}

#[external]
#[inherit(ERC20<ERC4626Params>)]
impl ERC4626 {
    pub fn initialize(&mut self, _asset: Address) -> Result<()> {
        if self.initialized.get() {
            return Err(ERC4626Error::AlreadyInitialized(AlreadyInitialized {}));
        }

        if _asset.is_zero() {
            return Err(ERC4626Error::InvalidInitialize(InvalidInitialize {}));
        }

        self.asset.set(_asset);

        Ok(())
    }

    pub fn deposit(&mut self, assets: U256, receiver: Address) -> Result<U256> {
        let shares = self.preview_deposit(assets)?;

        if shares == U256::from(0) {
            return Err(ERC4626Error::ZeroShares(ZeroShares {}));
        }

        // TODO: Fix below.
        // call!(self.asset).safe_transfer_from(msg::sender(), address!(self), assets);

        self.erc20.mint(receiver, shares);

        evm::log(Deposit {
            caller: msg::sender(),
            owner: receiver,
            assets,
            shares,
        });

        self.after_deposit(assets, shares);

        Ok(shares)
    }

    pub fn mint(&mut self, shares: U256, receiver: Address) -> Result<U256> {
        let assets = self.preview_mint(shares)?;

        // TODO: Fix below.
        // call!(self.asset).safe_transfer_from(msg::sender(), address!(self), assets);

        self.erc20.mint(receiver, shares);

        evm::log(Deposit {
            caller: msg::sender(),
            owner: receiver,
            assets,
            shares,
        });

        self.after_deposit(assets, shares);

        Ok(shares)
    }

    pub fn withdraw(&mut self, assets: U256, receiver: Address, owner: Address) -> Result<U256> {
        let shares = self.preview_withdraw(assets)?;

        if msg::sender() != owner {
            let allowed = self.erc20.allowance.get(owner).get(msg::sender());

            if allowed != U256::MAX {
                self.erc20
                    .allowance
                    .setter(owner)
                    .insert(msg::sender(), allowed - shares);
            }
        }

        self.before_withdraw(assets, shares);

        self.erc20.burn(owner, shares);

        evm::log(Withdraw {
            caller: msg::sender(),
            receiver,
            owner,
            assets,
            shares,
        });

        // TODO: asset.safeTransfer(receiver, assets);

        Ok(shares)
    }

    pub fn redeem(&mut self, shares: U256, receiver: Address, owner: Address) -> Result<U256> {
        if msg::sender() != owner {
            let allowed = self.erc20.allowance.get(owner).get(msg::sender());

            if allowed != U256::MAX {
                self.erc20
                    .allowance
                    .setter(owner)
                    .insert(msg::sender(), allowed - shares);
            }
        }

        let assets = self.preview_redeem(shares)?;

        if assets == U256::from(0) {
            return Err(ERC4626Error::ZeroShares(ZeroShares {}));
        }

        self.before_withdraw(assets, shares);

        self.erc20.burn(owner, shares);

        evm::log(Withdraw {
            caller: msg::sender(),
            receiver,
            owner,
            assets,
            shares,
        });

        // TODO: asset.safeTransfer(receiver, assets);

        Ok(assets)
    }

    pub fn total_assets() -> Result<U256> {
        Ok(U256::from(0))
    }

    pub fn convert_to_shares(&mut self, assets: U256) -> Result<U256> {
        let supply = self.erc20.total_supply.get();

        if supply == U256::from(0) {
            Ok(assets)
        } else {
            Ok(ERC4626::total_assets()?)
            // TODO: Fix with return supply == 0 ? assets : assets.mulDivDown(supply, totalAssets());
        }
    }

    pub fn convert_to_assets(&mut self, shares: U256) -> Result<U256> {
        let supply = self.erc20.total_supply.get();

        if supply == U256::from(0) {
            Ok(shares)
        } else {
            Ok(ERC4626::total_assets()?)
            // TODO: Fix with return supply == 0 ? shares : shares.mulDivDown(totalAssets(), supply);
        }
    }

    pub fn preview_deposit(&mut self, assets: U256) -> Result<U256> {
        Ok(self.convert_to_shares(assets)?)
    }

    pub fn preview_mint(&mut self, shares: U256) -> Result<U256> {
        let supply = self.erc20.total_supply.get();

        if supply == U256::from(0) {
            Ok(shares)
        } else {
            Ok(ERC4626::total_assets()?)
            // TODO: Fix with shares.mulDivUp(totalAssets(), supply);
        }
    }

    pub fn preview_withdraw(&mut self, assets: U256) -> Result<U256> {
        let supply = self.erc20.total_supply.get();

        if supply == U256::from(0) {
            Ok(assets)
        } else {
            Ok(ERC4626::total_assets()?)
            // TODO: Fix with assets.mulDivUp(supply, totalAssets());
        }
    }

    pub fn preview_redeem(&mut self, shares: U256) -> Result<U256> {
        Ok(self.convert_to_assets(shares)?)
    }

    pub fn max_deposit(&mut self, _user: Address) -> Result<U256> {
        Ok(U256::MAX)
    }

    pub fn max_mint(&mut self, _user: Address) -> Result<U256> {
        Ok(U256::MAX)
    }

    pub fn max_withdraw(&mut self, owner: Address) -> Result<U256> {
        Ok(self.convert_to_assets(self.erc20.balance.get(owner))?)
    }

    pub fn max_redeem(&mut self, owner: Address) -> Result<U256> {
        Ok(self.erc20.balance.get(owner))
    }
}
