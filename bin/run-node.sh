#!/usr/bin/env bash

set -e

NKP=node_keypair.json
USE_CUDA=false

while getopts "F:" opt; do
  case $opt in
    F)
      if [ "$OPTARG" = "cuda" ]; then
        USE_CUDA=true
      else
        echo "Error: Unknown feature flag: $OPTARG"
        exit 1
      fi
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      exit 1
      ;;
  esac
done

if [ -f $NKP ]; then
    echo "Bonsol node keypair exists"
else
    solana-keygen new --outfile $NKP
fi

solana -u http://localhost:8899 airdrop 1 --keypair node_keypair.json
solana -u http://localhost:8899 airdrop 1
ulimit -s unlimited

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if [ "$USE_CUDA" = true ]; then
        cargo run --release -p bonsol-node --features cuda -- -f ./Node.toml
    else
        cargo run --release -p bonsol-node -- -f ./Node.toml
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    if [ "$USE_CUDA" = true ]; then
        echo "Error: CUDA is not supported on macOS"
        exit 1
    else
        echo "NOTE: MAC Arm CPUs will not be able to run the stark to snark prover, this is a known issue"
        cargo run --release -p bonsol-node --features metal -- -f ./Node.toml
    fi
else
    echo "Unsupported operating system"
    exit 1
fi
