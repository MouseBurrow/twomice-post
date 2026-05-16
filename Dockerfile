FROM rust:1.91 AS builder
WORKDIR /app

COPY services/post/ services/post/
COPY libs/ libs/

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target \
    cargo build --release --manifest-path services/post/Cargo.toml --target-dir /app/target && \
    cp /app/target/release/post /app/post

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/post /app/post
CMD ["/app/post"]
