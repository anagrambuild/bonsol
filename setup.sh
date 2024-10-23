#!/usr/bin/env zsh
set -e
if [ ! -f "/stark/stark_verify" ]; then
    echo "Error: Bonsol requires 'stark_verify' to be located at /stark/stark_verify."
    echo "You can install 'stark_verify' from supported container builds of 'risc0-groth16-prover'"
    echo "or manually build it by following the instructions provided by 'risc0'."
    exit 1
fi

pnpx snarkjs zkey export verificationkey /stark/stark_verify_final.zkey verification_key.json
if [ ! -d vkey ]
then
    mkdir vkey
fi

cd vkey
pnpm i && pnpm run parse-vk ../verification_key.json ../onchain/bonsol/src && \
rm ../verification_key.json
