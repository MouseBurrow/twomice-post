# ---- Build stage ----
FROM rust:1.80-slim AS builder
WORKDIR /app

# Only copy manifests first to leverage caching
COPY Cargo.toml Cargo.lock ./
COPY ../.. ./

RUN cargo build --release -p post

# ---- Runtime stage ----
FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/post /app/post

# ENV variables loaded by docker-compose later
CMD ["/app/post"]