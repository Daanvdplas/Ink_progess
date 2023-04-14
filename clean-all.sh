#!/usr/bin/env bash

set -eu

cd my_contracts
cargo clean --manifest-path caller/Cargo.toml
cargo clean --manifest-path dao/Cargo.toml
cargo clean --manifest-path dao/erc20/Cargo.toml
cargo clean --manifest-path delegator/Cargo.toml
cargo clean --manifest-path delegator/adder/Cargo.toml
cargo clean --manifest-path delegator/subber/Cargo.toml
cargo clean --manifest-path delegator/accumulator/Cargo.toml
cargo clean --manifest-path erc_1155/Cargo.toml
cargo clean --manifest-path flip_trait/Cargo.toml
cargo clean --manifest-path flipper/Cargo.toml
cargo clean --manifest-path foobar/Cargo.toml
cargo clean --manifest-path incrementer/Cargo.toml
cargo clean --manifest-path multisig/Cargo.toml
cargo clean --manifest-path safe/Cargo.toml
cargo clean --manifest-path token/Cargo.toml
cargo clean --manifest-path tokenlock/Cargo.toml
