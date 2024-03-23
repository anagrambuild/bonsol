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
cargo run --release -- -f ./Node.toml
)