#!/bin/zsh
set -x
export COPYFILE_DISABLE=1
cargo --version
cargo build-sbf && solana-test-validator \
   --limit-ledger-size 0 \
   --bind-address 0.0.0.0 \
   --rpc-pubsub-enable-block-subscription \
    --bpf-program BoNsHRcyLLNdtnoDf8hiCNZpyehMC4FDMxs6NTxFi3ew target/deploy/bonsol.so  \
    --bpf-program exay1T7QqsJPNcwzMiWubR6vZnqrgM16jZRraHgqBGG target/deploy/callback_example.so \
    -r
