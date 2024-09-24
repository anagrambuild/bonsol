# syntax = docker/dockerfile:1.2
ARG RUST_VERSION=1.80.0
FROM rust:${RUST_VERSION} as chef
RUN cargo install cargo-chef

FROM chef as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --bin relay --recipe-path recipe.json

FROM chef as builder
ARG FLAVOR=standard

RUN apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get update --fix-missing && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    software-properties-common \
    build-essential

WORKDIR /app/
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --bin relay --release --recipe-path recipe.json
COPY . . 
RUN if [ "$FLAVOR" = "cuda" ]; then \
        wget https://developer.download.nvidia.com/compute/cuda/repos/debian12/x86_64/cuda-keyring_1.1-1_all.deb && \
        dpkg -i cuda-keyring_1.1-1_all.deb && \
        add-apt-repository contrib && \
        apt-get update && \
        apt-get -y install cuda-toolkit-12-5; \
        export FEATURES="--features=cuda"; \
    fi
ENV FEATURES=${FEATURES}
RUN cargo build ${FEATURES} --release
FROM rust:${RUST_VERSION}-slim
LABEL org.opencontainers.image.source=https://github.com/anagrambuild/bonsol
LABEL org.opencontainers.image.title="bonsol-relay"
LABEL org.opencontainers.image.description="A bonsol proving node"
ARG FLAVOR=standard  
RUN mkdir -p /usr/opt/bonsol/stark
RUN apt-get update && apt-get install -y --no-install-recommends software-properties-common wget ca-certificates
COPY --from=builder /app/target/release/relay /usr/opt/bonsol
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /app/stark_verify /usr/opt/bonsol/stark/stark_verify
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /app/stark_verify.dat /usr/opt/bonsol/stark/stark_verify.dat
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /app/stark_verify_final.zkey /usr/opt/bonsol/stark/stark_verify_final.zkey
COPY --from=risczero/risc0-groth16-prover:v2024-05-17.1 /usr/local/sbin/rapidsnark /usr/opt/bonsol/stark/rapidsnark
RUN if [ "$FLAVOR" = "cuda" ]; then \
        wget https://developer.download.nvidia.com/compute/cuda/repos/debian12/x86_64/cuda-keyring_1.1-1_all.deb && \
        dpkg -i cuda-keyring_1.1-1_all.deb && \
        add-apt-repository contrib && \
        apt-get update && \
        apt-get -y install cuda-toolkit-12-5; \
    fi
WORKDIR /usr/opt/bonsol
EXPOSE 9000
ENTRYPOINT ["/usr/opt/bonsol/relay"]




