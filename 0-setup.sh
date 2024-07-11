#!/bin/zsh
set -e
docker build -f setup.dockerfile -o relay .
pnpx snarkjs zkey export verificationkey relay/stark/stark_verify_final.zkey verification_key.json
cd vkey
pnpm i && pnpm run parse-vk ../verification_key.json ../onchain/channel/src/ && \
rm ../verification_key.json
