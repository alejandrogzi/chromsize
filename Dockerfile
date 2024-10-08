# syntax=docker/dockerfile:1

FROM rust:1.74.0-bullseye AS builder

COPY .git /app/.git
COPY Cargo.lock Cargo.toml /app/
COPY src/ /app/src/

RUN cd chromsize && cargo build --release --manifest-path /app/Cargo.toml

FROM debian:bullseye

COPY --from=builder /app/target/release/chromsize /usr/local/bin/

ENTRYPOINT ["/usr/local/bin/chromsize"]
