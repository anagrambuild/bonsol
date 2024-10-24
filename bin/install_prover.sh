#!/usr/bin/env bash

set -e

DEFAULT_PROVER_PROVIDER_URL="http://risc0-prover-us-east-1-041119533185.s3-website-us-east-1.amazonaws.com"
DEFAULT_INSTALL_PREFIX="."
DEFAULT_JOB_TIMEOUT=3600
DEFAULT_VERSION="v2024-05-17.1"

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
                echo "  --prover-provider-url   URL of the prover provider to install."
                echo "                          Default: $DEFAULT_PROVER_PROVIDER_URL"
                echo "  --job-timeout           Timeout for the job in seconds."
                echo "                          Default: $DEFAULT_JOB_TIMEOUT"
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
            --prover-provider-url)
                shift
                if [[ -z "$1" ]]; then
                    echo "Error: --prover-provider-url requires a non-empty argument."
                    exit 1
                fi
                PROVER_PROVIDER_URL="$1"
                ;;
            --job-timeout)
                shift
                if [[ -z "$1" ]]; then
                    echo "Error: --job-timeout requires a non-empty argument."
                    exit 1
                fi
                JOB_TIMEOUT="$1"
                ;;
            --version)
                shift
                if [[ -z "$1" ]]; then
                    echo "Error: --version requires a non-empty argument."
                    exit 1
                fi
                PROVER_VERSION="$1"
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
    echo "PROVER_PROVIDER_URL is set to '$PROVER_PROVIDER_URL'"
    echo "INSTALL_PREFIX is set to '$INSTALL_PREFIX'"
    echo "JOB_TIMEOUT is set to '$JOB_TIMEOUT'"
}

if [ ! -x $(which curl) ]; then 
    echo "Error: curl is required to download risc0-prover."
    exit 1
fi

parse_arguments "$@"

mkdir -p "${INSTALL_PREFIX}"/stark
for stark_tech in stark/rapidsnark stark/stark_verify stark/stark_verify_final.zkey stark/stark_verify.dat; do
    if [ ! -f "${INSTALL_PREFIX}/${stark_tech}" ]; then
        echo "Downloading ${stark_tech} from ${PROVER_PROVIDER_URL}/${PROVER_VERSION}"
        curl --max-time ${JOB_TIMEOUT} -o "${INSTALL_PREFIX}/${stark_tech}" "$PROVER_PROVIDER_URL/${PROVER_VERSION}/${stark_tech}"
        if [ -x $(which sha256sum) ]; then
            echo "Verifying the integrity of ${stark_tech}"
            sha256sum -c "${INSTALL_PREFIX}/${stark_tech}.sha256"
            cat "${INSTALL_PREFIX}/${stark_tech}.sha256"
        fi
    else 
        echo "${INSTALL_PREFIX}/${stark_tech} already exists. Skipping download."
    fi
done
chmod +x "${INSTALL_PREFIX}/stark/rapidsnark"
chmod +x "${INSTALL_PREFIX}/stark/stark_verify"

