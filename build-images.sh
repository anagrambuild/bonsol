#!/bin/zsh
mkdir -p elf
pushd images/simple
cargo clean
cargo risczero build --manifest-path ./Cargo.toml && mv target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/simple/simple ../../elf/
popd

pushd images/range
cargo clean
cargo risczero build --manifest-path ./Cargo.toml && mv target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/range/range ../../elf/
popd
