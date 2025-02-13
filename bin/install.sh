#!/usr/bin/env bash
set -e

cargo install cargo-binstall
cargo binstall cargo-risczero
cargo risczero install

# check os linux or mac
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # check if nvidia-smi exists
    if ! command -v nvidia-smi &> /dev/null
    then
        echo "installing without cuda support, proving will be slower"
        cargo install bonsol-cli --git https://github.com/anagrambuild/bonsol 
    else
        echo "installing with cuda support"
        cargo install bonsol-cli --git https://github.com/anagrambuild/bonsol --features linux
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    cargo install bonsol-cli --git https://github.com/anagrambuild/bonsol --features mac
fi
