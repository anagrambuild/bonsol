# syntax = docker/dockerfile:1.2
ARG RUST_VERSION=1.77.0 
FROM rust:${RUST_VERSION} as chef
RUN cargo install cargo-chef

FROM chef as planner
WORKDIR /app
COPY . .
WORKDIR /app/relay
RUN cargo chef prepare --recipe-path recipe.json
COPY ./iop/ /app/iop
COPY ./schemas-rust/ /app/schemas-rust
COPY ./onchain/channel-utils /app/onchain/channel-utils
COPY ./Cargo.toml /app/Cargo.toml
COPY ./Cargo.lock /app/Cargo.lock
ARG FLAVOR=standard  
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential

FROM chef as builder
COPY ./iop/ /app/iop
COPY ./schemas-rust/ /app/schemas-rust
COPY ./onchain/channel-utils /app/onchain/channel-utils
COPY ./Cargo.toml /app/Cargo.toml
COPY ./Cargo.lock /app/Cargo.lock
WORKDIR /app/relay
COPY --from=planner /app/relay/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./relay/ . 
RUN if [ "$FLAVOR" = "cuda" ]; then \
        apt-get install -y --no-install-recommends \
            nvidia-cuda-toolkit \
        export FEATURES="--features=cuda"; \
    fi
RUN cargo build ${FEATURES} --release

FROM debian:stable-slim
COPY --from=builder /app/target/release/relay /usr/opt/bonsol
COPY --from=risczero/risc0-groth16-prover:v2024-04-03.2 /app/stark_verify /usr/opt/bonsol/stark/stark_verify
COPY --from=risczero/risc0-groth16-prover:v2024-04-03.2 /app/stark_verify.dat /usr/opt/bonsol/stark/stark_verify.dat
COPY --from=risczero/risc0-groth16-prover:v2024-04-03.2 /app/stark_verify_final.zkey /usr/opt/bonsol/stark/stark_verify_final.zkey
COPY --from=risczero/risc0-groth16-prover:v2024-04-03.2 /usr/local/sbin/rapidsnark /usr/opt/bonsol/stark/rapidsnark
WORKDIR /usr/opt/bonsol
ENTRYPOINT ["relay"]




