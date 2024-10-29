#!/usr/bin/env zsh

set -e

INSTALL_PREFIX=
DEFAULT_INSTALL_PREFIX="/opt/risc0-prover"

function parse_arguments() {
    # Initialize variables with default values
    PROVER_PROVIDER_URL="${DEFAULT_PROVER_PROVIDER_URL}"
    INSTALL_PREFIX="${DEFAULT_INSTALL_PREFIX}"
    JOB_TIMEOUT="${DEFAULT_JOB_TIMEOUT}"
    PROVER_VERSION="${DEFAULT_VERSION}"

    # Loop through all arguments
    while [[ "$#" -gt 0 ]]; do
        case "$1" in
            --help)
                echo "Usage: $0 [--prefix <install location>] [--prover-provider-url <prover provider URL>]"
                echo "Options:"
                echo "  --prefix                Specify the install location."
                echo "                          Default: $DEFAULT_INSTALL_PREFIX"
                exit 0
                ;;
            --prefix)
                shift
                if [[ -z "$1" ]]; then
                    echo "Error: --prefix requires a non-empty argument."
                    exit 1
                fi
                INSTALL_PREFIX="$1"
                ;;
            *)
                echo "Error: Unknown option '$1'"
                echo "Use --help to see the usage."
                exit 1
                ;;
        esac
        shift
    done

    # Output the parsed values for confirmation
    echo "INSTALL_PREFIX is set to '$INSTALL_PREFIX'"
}


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
