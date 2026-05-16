FROM rust:1.91 AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y gcc-aarch64-linux-gnu && rm -rf /var/lib/apt/lists/*
RUN rustup target add aarch64-unknown-linux-gnu

RUN mkdir -p .cargo && printf '[target.aarch64-unknown-linux-gnu]\nlinker = "aarch64-linux-gnu-gcc"\n' > .cargo/config.toml

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    cargo fetch
COPY src/ src/
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target \
    cargo build --release --target aarch64-unknown-linux-gnu --target-dir /app/target && \
    cp /app/target/aarch64-unknown-linux-gnu/release/post /app/service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/service /app/service
CMD ["/app/service"]
