# 🦀 rustmate

[![Check smart contracts](https://github.com/cairoeth/rustmate/actions/workflows/stylus.yml/badge.svg)](https://github.com/cairoeth/rustmate/actions/workflows/stylus.yml)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

**Blazing fast**, **modern**, **opinionated**, and **extremely optimized** Rust building blocks for smart contract development. 

> This is **experimental software** and is provided on an "as is" and "as available" basis. We **do not give any warranties** and **will not be liable for any losses** incurred through any use of this code base.

## 📜 Contracts

```ml
src
├─ ERC20 — "Modern and Gas-Efficient ERC-20 + EIP-2612 Implementation"
```

## 🎛 Installation

Install [Rust](https://www.rust-lang.org/tools/install), and then install the Stylus CLI tool with Cargo

```bash
RUSTFLAGS="-C link-args=-rdynamic" cargo install --force cargo-stylus
```

Add the `wasm32-unknown-unknown` build target to your Rust compiler:

```
rustup target add wasm32-unknown-unknown
```

Then, clone the repository:

```
git clone https://github.com/cairoeth/rustmate && cd rustmate
```

## 🔧 How to use

To be implemented.

## ✅ Tests

To be implemented.

## 🙏🏼 Acknowledgements

This repository is inspired by or directly modified from many sources, primarily:

- [solmate](https://github.com/transmissions11/solmate)
- [OpenZeppelin Contracts](https://github.com/OpenZeppelin/openzeppelin-contracts)
- [snekmate](https://github.com/pcaversaccio/snekmate)
- [stylus-sdk-rs](https://github.com/OffchainLabs/stylus-sdk-rs)
- [stylus-hello-world](https://github.com/OffchainLabs/stylus-hello-world)

## 🫡 Contributing

Check out the [Contribution Guidelines](./CONTRIBUTING.md)!
