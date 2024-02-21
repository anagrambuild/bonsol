#!/bin/bash
cargo build-sbf && solana-test-validator --rpc-pubsub-enable-block-subscription --bpf-program BoNSrwTtTM4PRkbbPvehk1XzHC65cKfdNSod9FyTejRn target/deploy/anagram_bonsol_channel.so  -r