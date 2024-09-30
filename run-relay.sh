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
ulimit -s unlimited
(cd relay;
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cargo run --release -p relay -- -f ./Node.toml
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "NOTE: MAC Arm cpus will not be able to run the stark to snark prover, this is a known issue"
    cargo run --release -p relay --features metal -- -f ./Node.toml
fi
)