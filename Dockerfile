# syntax = docker/dockerfile:1.2
ARG RUST_VERSION=1.77.0 
FROM rust:${RUST_VERSION} as chef
RUN cargo install cargo-chef

FROM chef as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --bin relay --recipe-path recipe.json

FROM chef as builder
ARG FLAVOR=standard  
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential

WORKDIR /app/
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --bin relay --release --recipe-path recipe.json
COPY . . 
RUN if [ "$FLAVOR" = "cuda" ]; then \
        apt-get install -y --no-install-recommends \
            nvidia-cuda-toolkit \
        export FEATURES="--features=cuda"; \
    fi
RUN cargo build ${FEATURES} --release

FROM debian:stable-slim
RUN mkdir -p /usr/opt/bonsol/stark
COPY --from=builder /app/target/release/relay /usr/opt/bonsol
COPY --from=risczero/risc0-groth16-prover:v2024-04-03.2 /app/stark_verify /usr/opt/bonsol/stark/stark_verify
COPY --from=risczero/risc0-groth16-prover:v2024-04-03.2 /app/stark_verify.dat /usr/opt/bonsol/stark/stark_verify.dat
COPY --from=risczero/risc0-groth16-prover:v2024-04-03.2 /app/stark_verify_final.zkey /usr/opt/bonsol/stark/stark_verify_final.zkey
COPY --from=risczero/risc0-groth16-prover:v2024-04-03.2 /usr/local/sbin/rapidsnark /app/stark/rapidsnark
WORKDIR /usr/opt/bonsol
ENTRYPOINT ["relay"]

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
&&  rm -rf /var/lib/apt/lists/*

