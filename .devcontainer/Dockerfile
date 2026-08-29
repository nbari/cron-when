FROM mcr.microsoft.com/devcontainers/rust:2-1-bookworm

USER root

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        age \
        build-essential \
        ca-certificates \
        curl \
        git \
        jq \
        libssl-dev \
        musl-tools \
        netcat-openbsd \
        openssl \
        pkg-config \
        sudo \
        vim \
        zsh \
    && rm -rf /var/lib/apt/lists/*

# Pin developer tools so rebuilding the container remains reproducible.
RUN rustup component add clippy llvm-tools-preview rustfmt \
    && rustup target add x86_64-unknown-linux-musl \
    && cargo install --locked \
        cargo-audit@0.22.2 \
        cargo-deny@0.20.2 \
        cargo-edit@0.13.13 \
        cargo-expand@1.0.126 \
        cargo-llvm-cov@0.9.0 \
        cargo-outdated@0.19.0 \
        just@1.58.0

# cargo install runs as root so tools are globally available; restore the
# image's non-root ownership so normal Cargo downloads can update the cache.
RUN chown -R vscode:vscode /usr/local/cargo

USER vscode
WORKDIR /workspaces
