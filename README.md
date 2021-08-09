# Solana Test

This repo implemented a test escrow Solana program based on [tutorial](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/) provided by Paulx. I found this tutorial is a very useful guide to start developing program (smart contract) on Solana blockchain.

On top of that, I am implementing test cases on the escrow program. I am using Solana Test Validator suggested by this [template repo](https://github.com/mvines/solana-bpf-program-template) instead of the [program test crate](https://docs.solana.com/developing/on-chain-programs/developing-rust#how-to-test) in Solana official docs. The solana test validator offers more flexibility on conducting end-to-end test as well as similarity to writing smart contract test on mocha in Ethereum contract development.

## Environment Setup
1. Install dependencies
```
# On ubuntu
$ sudo apt update
$ sudo apt-get install libssl-dev libudev-dev pkg-config zlib1g-dev llvm clang make -y
```
2. Install Rust and Cargo
```
$ curl https://sh.rustup.rs -sSf | bash -s -- -y
$ echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
$ source $HOME/.cargo/env
$ rustup component add rustfmt
$ rustup update
```
3. Install Solana v1.7.4
```
$ sh -c "$(curl -sSfL https://release.solana.com/v1.7.4/install)"
```

## Build and test the program compiled for BPF
```
$ cargo build-bpf
$ cargo test-bpf
$ cargo test-bpf -- --nocapture ## To display the logs during test
```
