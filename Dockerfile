FROM rust:1.91 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    cargo fetch
COPY src/ src/
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target \
    cargo build --release --target-dir /app/target && \
    cp /app/target/release/post /app/service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/service /app/service
CMD ["/app/service"]
