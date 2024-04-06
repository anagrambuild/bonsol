#!/bin/zsh
cargo --version
cd onchain
cargo build-sbf
cd ..
solana-test-validator --rpc-pubsub-enable-block-subscription --bpf-program BoNSrwTtTM4PRkbbPvehk1XzHC65cKfdNSod9FyTejRn onchain/target/deploy/anagram_bonsol_channel.so  -r