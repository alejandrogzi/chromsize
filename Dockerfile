# ---------- Build Stage ----------
FROM rust:1.93-bullseye AS builder

WORKDIR /build

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    cmake \
    pkg-config \
    build-essential \
    binutils \
    && rm -rf /var/lib/apt/lists/*

COPY chromsize/Cargo.toml chromsize/Cargo.lock ./
COPY chromsize/src ./src

RUN cargo build --release --locked --bin chromsize && \
    strip target/release/chromsize

# ---------- Runtime Stage ----------
FROM debian:bookworm-slim

# Install minimal runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /build/target/release/chromsize /usr/local/bin/chromsize

# Set up non-root user
RUN useradd -m -u 1000 user && \
    chmod +x /usr/local/bin/chromsize

USER user
WORKDIR /data

# Test that it works
RUN chromsize --help
