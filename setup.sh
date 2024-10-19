#!/bin/zsh
set -e
if [ ! -f "/stark/stark_verify" ]; then
    echo "stark_verify not found, please copy it to /stark/stark_verify"
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
