# Development Container
# Stage 1: Build yamlfmt
FROM golang:1 AS go-builder
# defined from build kit
# DOCKER_BUILDKIT=1 docker build . -t ...
ARG TARGETARCH

# Install yamlfmt
WORKDIR /yamlfmt
RUN go install github.com/google/yamlfmt/cmd/yamlfmt@latest && \
    strip $(which yamlfmt) && \
    yamlfmt --version

# Stage 2: Rust Development Container
FROM rust:1-slim
ARG TARGETARCH

# Install packages
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    binutils \
    ca-certificates \
    clang \
    cmake \ 
    curl \
    git \
    gnupg2 \
    libssl-dev \
    make \
    ninja-build \ 
    perl \ 
    pkg-config \
    protobuf-c-compiler \
    python3 \
    python3-pip \
    ripgrep \
    sudo \
    valgrind \
    && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN echo "building platform $(uname -m)"

# create dev user
RUN useradd --create-home --shell /bin/bash bonsol
RUN usermod -a -G sudo bonsol
RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

## Rust
ENV USER=bonsol
COPY --chown=${USER}:${USER} --from=go-builder /go/bin/yamlfmt /go/bin/yamlfmt
USER bonsol
ENV PATH=${PATH}:/go/bin

# Set user and working directory
ARG PACKAGE=bonsol
USER bonsol
WORKDIR /workspaces/${PACKAGE}

# Install Rust components
RUN rustup component add \
    rustfmt \
    clippy \
    rust-analyzer

RUN cargo install cargo-binstall
RUN yes | cargo binstall cargo-risczero
RUN cargo risczero build-toolchain

# Clean up
RUN rm -rf /home/bonsol/.cargo/registry /home/bonsol/.cargo/git

ENV PATH=${PATH}:/home/bonsol/.cargo/bin

LABEL \
    org.label-schema.name="bonsol" \
    org.label-schema.description="Bonsol Development Container" \
    org.label-schema.url="https://github.com/anagrambuild/bonsol" \
    org.label-schema.vcs-url="git@github.com/anagrambuild/bonsol.git" \
    org.label-schema.vendor="anagram.xyz" \
    org.label-schema.schema-version="1.0" \
    org.opencontainers.image.description="Bonsol Development Container"
