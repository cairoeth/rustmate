# 🦀 rustmate

[![Check smart contracts](https://github.com/cairoeth/rustmate/actions/workflows/stylus.yml/badge.svg)](https://github.com/cairoeth/rustmate/actions/workflows/stylus.yml)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

**Blazing fast**, **modern**, and **optimized** Rust building blocks for smart contract development using Stylus. 

> This is **experimental software** and is provided on an "as is" and "as available" basis. We **do not give any warranties** and **will not be liable for any losses** incurred through any use of this code base.

## 📜 Contracts

```ml
auth
├─ Owned — "Simple single owner authorization"
├─ Auth — "Flexible and updatable auth pattern"
mixins
├─ ERC4626 — "Minimal ERC4626 tokenized Vault implementation"
tokens
├─ WETH — "Minimalist and modern Wrapped Ether implementation"
├─ ERC20 — "Modern and gas efficient ERC20 + EIP-2612 implementation"
├─ ERC721 — "Modern, minimalist, and gas efficient ERC721 implementation"
├─ ERC1155 — "Minimalist and gas efficient standard ERC1155 implementation"
├─ ERC6909 — "Minimalist and gas efficient standard ERC6909 implementation"
utils
├─ CREATE3 — "Deploy to deterministic addresses without an initcode factor"
├─ Bytes32Address — "Library for converting between addresses and bytes32 values"
```

## 🔧 How to use

First, install the [Stylus SDK for Rust](https://docs.arbitrum.io/stylus/stylus-quickstart) and create a new project:
    
```bash
cargo stylus new my-project --minimal
```

Then, install `rustmate`:

```bash
cargo install rustmate
```

Import the contracts you want to use:

```rust
use rustmate::tokens::erc20::{ERC20, ERC20Params};
```

## ✅ Benchmarks

### 🧪 Results

<details><summary>ERC20</summary>

| Function | Rustmate | Solmate | OpenZeppelin Contracts
| -------- | -------- | -------- | -------- |
| name()   | TBD    | TBD   | TBD    |
| symbol()   | TBD   | TBD   | TBD    |
| decimals()   | TBD   | TBD   | TBD    |
| totalSupply()   | TBD   | TBD   | TBD    |
| balanceOf(address)   | TBD   | TBD   | TBD    |
| allowance(address,address)   | TBD   | TBD   | TBD    |
| nonces(address)   | TBD   | TBD   | TBD    |
| approve(address,uint256)   | TBD   | TBD   | TBD    |
| transfer(address,uint256)   | TBD   | TBD   | TBD    |
| transferFrom(address,address,uint256)   | TBD   | TBD   | TBD    |

</details>

### 👷 How to replicate

Install [Python](https://www.python.org/downloads/) and [Rust](https://www.rust-lang.org/tools/install), and then install the Stylus CLI tool with Cargo:

```bash
RUSTFLAGS="-C link-args=-rdynamic" cargo install --force cargo-stylus
```

Add the `wasm32-unknown-unknown` build target to your Rust compiler:

```bash
rustup target add wasm32-unknown-unknown
```

Then, clone the repository:

```bash
git clone https://github.com/cairoeth/rustmate && cd rustmate
```

Clone Arbitrum Nitro node that supports Stylus:

```bash
git clone -b stylus --recurse-submodules https://github.com/OffchainLabs/nitro-testnode.git && cd nitro-testnode
```

Run the node and wait for it to start up:

```bash
./test-node.bash --init --detach
```

Open another terminal window and fund the deployer address:

```bash
./test-node.bash script send-l2 --to address_0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266 --ethamount 100
```

Navigate back to `rustmate` and select a benchmark to run. For example, ERC20:

```bash
cd ../benchmark/erc20_benchmark && python snapshot.py
```

Check the results in the terminal or in the `.gas-snapshot` file.

## 🙏🏼 Acknowledgements

This repository is inspired by or directly modified from many sources, primarily:

- [solmate](https://github.com/transmissions11/solmate)
- [OpenZeppelin Contracts](https://github.com/OpenZeppelin/openzeppelin-contracts)
- [snekmate](https://github.com/pcaversaccio/snekmate)
- [stylus-sdk-rs](https://github.com/OffchainLabs/stylus-sdk-rs)

## 🫡 Contributing

Check out the [Contribution Guidelines](./CONTRIBUTING.md)!
