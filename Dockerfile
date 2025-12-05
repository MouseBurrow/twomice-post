# Build stage
FROM rust:1.91 AS builder
WORKDIR /app

# Copy workspace definition first (better caching)
COPY Cargo.toml Cargo.lock ./
COPY migrator migrator
COPY services services
COPY shared shared
#COPY services/post services/post

# Build only this service
RUN cargo build --release -p post

# Runtime stage
FROM debian:bookworm-slim

# Install OpenSSL runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/post /app/post

#EXPOSE 8080
CMD ["/app/post"]
