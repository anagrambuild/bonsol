#!/bin/zsh
set -e
NKP=node_keypair.json
if [ -f $RKP ]; then
    echo "Bonsol node keypair exists"
else
    solana-keygen new --outfile $NKP
fi
solana -u http://localhost:8899 airdrop 1 --keypair node_keypair.json
solana -u http://localhost:8899 airdrop 1
ulimit -s unlimited
(cd node;
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cargo run --release -p bonsol-node -- -f ./Node.toml
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "NOTE: MAC Arm cpus will not be able to run the stark to snark prover, this is a known issue"
    cargo run --release -p bonsol-node --features metal -- -f ./Node.toml
fi
)
