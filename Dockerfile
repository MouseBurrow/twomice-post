# Build stage
FROM rust:1.91 AS builder
WORKDIR /app

# Copy manifests and shared crates
COPY Cargo.toml Cargo.lock ./
COPY services/post/ services/post/
COPY shared/ shared/

# Build using cached target, copy binary to real FS
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target \
    cargo build --release -p post && \
    cp target/release/post /app/post

# Runtime stage
FROM debian:bookworm-slim

# Install OpenSSL runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/post /app/post

CMD ["/app/post"]
