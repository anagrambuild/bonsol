mkdir -p elf
pushd images/simple
cargo risczero build --manifest-path ./Cargo.toml && mv target/riscv-guest/riscv32im-risc0-zkvm-elf/docker/simple/simple ../../elf/
popd