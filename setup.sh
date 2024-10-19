#!/bin/zsh
set -e
if [ ! -f "/stark/stark_verify" ]; then
    echo "stark_verify not found, please copy it to /stark/stark_verify"
    exit 1
fi

pnpx snarkjs zkey export verificationkey node/stark/stark_verify_final.zkey verification_key.json
cd vkey
pnpm i && pnpm run parse-vk ../verification_key.json ../onchain/channel/src/ && \
rm ../verification_key.json
