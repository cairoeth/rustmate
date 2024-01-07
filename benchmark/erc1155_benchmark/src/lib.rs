// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use rustmate::tokens::erc1155::{ERC1155Params, ERC1155};
use stylus_sdk::{alloy_primitives::U256, prelude::*};

pub struct SampleParams;

/// Immutable definitions
impl ERC1155Params for SampleParams {
    fn uri(id: U256) -> String {
        format!("ipfs://hash/{}", id)
    }
}

// The contract
sol_storage! {
    #[entrypoint] // Makes SampleNFT the entrypoint
    pub struct SampleNFT {
        #[borrow] // Allows erc1155 to access SampleNFT's storage and make calls
        ERC1155<SampleParams> erc1155;
    }
}

#[external]
#[inherit(ERC1155<SampleParams>)]
impl SampleNFT {}
