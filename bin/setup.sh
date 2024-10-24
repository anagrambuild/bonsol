#!/usr/bin/env zsh

set -e

INSTALL_PREFIX="/opt/risc0-prover"

if [ ! -f "${INSTALL_PREFIX}/stark/stark_verify" ]; then
    echo "Error: Bonsol default to checking for the groth16 compression tool to be located at ${INSTALL_PREFIX}/stark"
    echo "You can install these tools by running bin/install-prover.sh courtesy of the bonsol team."
    exit 1
fi

pnpx snarkjs zkey export verificationkey ${INSTALL_PREFIX}/stark/stark_verify_final.zkey verification_key.json
if [ ! -d vkey ]
then
    mkdir vkey
fi

cd vkey
pnpm i && pnpm run parse-vk ../verification_key.json ../onchain/bonsol/src && \
rm ../verification_key.json
