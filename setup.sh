#!/bin/zsh
set -e
docker build -f setup.dockerfile -o node .
pnpx snarkjs zkey export verificationkey node/stark/stark_verify_final.zkey verification_key.json
cd vkey
pnpm i && pnpm run parse-vk ../verification_key.json ../onchain/bonsol/src/ && \
rm ../verification_key.json
