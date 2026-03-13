# Build stage
FROM rust:1.82 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy all source files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust-serv /usr/local/bin/rust-serv

# Create directories and copy default config
RUN mkdir -p /var/www/html /etc/rust-serv
COPY docker/config.toml /etc/rust-serv/config.toml

EXPOSE 8080 8443

ENV RUST_LOG=info

ENTRYPOINT ["rust-serv"]
CMD ["--config", "/etc/rust-serv/config.toml"]
