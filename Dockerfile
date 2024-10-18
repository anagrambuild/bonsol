# Bonsol Development Container
FROM ghcr.io/anagrambuild/risczero:latest

ENV USER=solana
ARG SOLANA=1.18.22
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH=${PATH}:/usr/local/cargo/bin:/go/bin:/home/solana/.local/share/solana/install/releases/${SOLANA}/bin
USER solana

# Set user and working directory
ARG PACKAGE=bonsol
WORKDIR /workspaces/${PACKAGE}

# Install Rust components
RUN rustup component add \
    rustfmt \
    clippy \
    rust-analyzer


# Risk0 Groth16 Prover
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /app/stark_verify /stark/stark_verify
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /app/stark_verify.dat /stark/stark_verify.dat
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /app/stark_verify_final.zkey /stark/stark_verify_final.zkey
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /usr/local/sbin/rapidsnark /stark/rapidsnark

LABEL \
    org.label-schema.name="bonsol" \
    org.label-schema.description="Bonsol Development Container" \
    org.label-schema.url="https://github.com/anagrambuild/bonsol" \
    org.label-schema.vcs-url="git@github.com/anagrambuild/bonsol.git" \
    org.label-schema.vendor="anagram.xyz" \
    org.label-schema.schema-version="1.0" \
    org.opencontainers.image.description="Bonsol Development Container"
