
# Build stage
FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN mkdir -p /app/uploads \
    && chown 65532:65532 /app/uploads

COPY --from=builder \
    /app/target/release/rust-worker \
    /usr/local/bin/rust-worker

USER 65532:65532

CMD ["/usr/local/bin/rust-worker"]
