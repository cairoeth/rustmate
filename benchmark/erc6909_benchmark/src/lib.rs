// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use rustmate::tokens::erc6909::{ERC6909Params, ERC6909};
use stylus_sdk::prelude::*;

pub struct SampleParams;

/// Immutable definitions
impl ERC6909Params for SampleParams {}

// The contract
sol_storage! {
    #[entrypoint] // Makes MyToken the entrypoint
    pub struct MyToken {
        #[borrow] // Allows ERC6909 to access MyToken's storage and make calls
        ERC6909<SampleParams> erc6909;
    }
}

#[external]
#[inherit(ERC6909<SampleParams>)]
impl MyToken {}
