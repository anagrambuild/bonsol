#!/bin/zsh
cargo --version
cargo build-sbf && solana-test-validator --bind-address 0.0.0.0 --rpc-pubsub-enable-block-subscription --bpf-program BoNsHRcyLLNdtnoDf8hiCNZpyehMC4FDMxs6NTxFi3ew target/deploy/anagram_bonsol_channel.so  -r