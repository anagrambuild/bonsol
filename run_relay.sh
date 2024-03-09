#!/bin/zsh
set -e
RKP=relaykp.json
if [ -f $RKP ]; then
    echo "Relay keypair exists"
else
    solana-keygen new --outfile $RKP
fi
solana -u http://localhost:8899 airdrop 1 --keypair relaykp.json
(cd relay;
cargo run --release -- -k ../$RKP --risc0-image-folder ../elf start-with-rpc --wss-rpc-url ws://localhost:8900/ --rpc-url http://localhost:8899 --bonsol-program BoNSrwTtTM4PRkbbPvehk1XzHC65cKfdNSod9FyTejRn
)