// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use rustmate::erc721::{ERC721Params, ERC721};
use stylus_sdk::{alloy_primitives::U256, prelude::*};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct SampleParams;

/// Immutable definitions
impl ERC721Params for SampleParams {
    const NAME: &'static str = "MyNFT";
    const SYMBOL: &'static str = "NFT";

    fn token_uri(id: U256) -> String {
        format!("ipfs://hash/{}", id)
    }
}

// The contract
sol_storage! {
    #[entrypoint] // Makes SampleNFT the entrypoint
    pub struct SampleNFT {
        #[borrow] // Allows erc721 to access SampleNFT's storage and make calls
        ERC721<SampleParams> erc721;
    }
}

#[external]
#[inherit(ERC721<SampleParams>)]
impl SampleNFT {}
