#!/bin/zsh
set -e
RKP=relaykp.json
if [ -f $RKP ]; then
    echo "Relay keypair exists"
else
    solana-keygen new --outfile $RKP
fi
solana -u http://localhost:8899 airdrop 1 --keypair relaykp.json
solana -u http://localhost:8899 airdrop 1
(cd relay;
ulimit -s unlimited
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cargo run --release --features cuda -- -f ./Node.toml
elif [[ "$OSTYPE" == "darwin"* ]]; then
    cargo run --release --features metal -- -f ./Node.toml
fi
)