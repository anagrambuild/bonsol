# Bonsol Development Container

# Stage 1: Node setup
FROM debian:stable-slim AS node-slim
RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install -y -q --no-install-recommends \
    ca-certificates \
    curl \
    git \
    gnupg2 \
    && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

ENV NODE_VERSION=v22.3.0
ENV NVM_DIR=/usr/local/nvm

RUN mkdir -p ${NVM_DIR}
ADD https://raw.githubusercontent.com/creationix/nvm/master/install.sh /usr/local/etc/nvm/install.sh
RUN bash /usr/local/etc/nvm/install.sh

# Stage 2: flatc build
FROM debian:stable-slim AS flatc-build
RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install -y -q --no-install-recommends \
    build-essential \
    cmake \
    ca-certificates \
    curl \
    git \
    gnupg2 \
    && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# flatc
WORKDIR /flatbuffers
ARG FLATC_VERSION=24.3.25
ADD https://github.com/google/flatbuffers/archive/refs/tags/v${FLATC_VERSION}.tar.gz v${FLATC_VERSION}.tar.gz
RUN tar -zxvf v${FLATC_VERSION}.tar.gz || { echo "Failed to extract tarball"; exit 1; }
WORKDIR /flatbuffers/flatbuffers-${FLATC_VERSION}
RUN cmake -G "Unix Makefiles" && make -j && make install
RUN strip /usr/local/bin/flatc

# Stage 2: Bonsol Dev
FROM ghcr.io/anagrambuild/risczero:latest

# flatbuffers
COPY --from=flatc-build /usr/local/bin/flatc /usr/local/bin/flatc
COPY --from=flatc-build /usr/local/include/flatbuffers /usr/local/include/flatbuffers
COPY --from=flatc-build /usr/local/lib/libflatbuffers.a /usr/local/lib/libflatbuffers.a
COPY --from=flatc-build /usr/local/lib/cmake/flatbuffers /usr/local/lib/cmake/flatbuffers
COPY --from=flatc-build /usr/local/lib/pkgconfig/flatbuffers.pc /usr/local/lib/pkgconfig/flatbuffers.pc

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
    clippy \
    rust-analyzer

RUN rustup toolchain install nightly  && \
    rustup component add rustfmt --toolchain nightly

# Install Node
ENV NODE_VERSION=v22.3.0
ENV NVM_DIR=/usr/local/nvm
ENV NVM_NODE_PATH ${NVM_DIR}/versions/node/${NODE_VERSION}
ENV NODE_PATH ${NVM_NODE_PATH}/lib/node_modules
ENV PATH      ${NVM_NODE_PATH}/bin:$PATH
COPY --from=node-slim --chown=${USER}:${USER} /usr/local/nvm /usr/local/nvm
RUN bash -c ". $NVM_DIR/nvm.sh && nvm install $NODE_VERSION && nvm alias default $NODE_VERSION && nvm use default"


RUN npm install npm
RUN npm install yarn -g

# Install PNPM
ENV PNPM_HOME=/home/solana/.local/share
RUN curl -fsSL https://get.pnpm.io/install.sh | \
    bash -

ENV PATH=${PATH}:/home/solana/.local/share/pnpm

LABEL \
    org.label-schema.name="bonsol" \
    org.label-schema.description="Bonsol Development Container" \
    org.label-schema.url="https://github.com/anagrambuild/bonsol" \
    org.label-schema.vcs-url="git@github.com/anagrambuild/bonsol.git" \
    org.label-schema.vendor="anagram.xyz" \
    org.label-schema.schema-version="1.0" \
    org.opencontainers.image.description="Bonsol Development Container"
