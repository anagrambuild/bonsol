#!/bin/zsh
set -e
if [ ! -d "/stark/stark_verify" ]; then
    docker build -f setup.dockerfile -o node .  
fi

pnpx snarkjs zkey export verificationkey node/stark/stark_verify_final.zkey verification_key.json
cd vkey
pnpm i && pnpm run parse-vk ../verification_key.json ../onchain/channel/src/ && \
rm ../verification_key.json
