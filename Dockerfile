# Build stage
FROM rust:1.82 AS builder

# Install ALL build dependencies that might be needed
RUN apt-get update && \
    apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    clang \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage  
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust-serv /usr/local/bin/rust-serv

RUN mkdir -p /var/www/html /etc/rust-serv
COPY docker/config.toml /etc/rust-serv/config.toml

EXPOSE 8080 8443

ENV RUST_LOG=info

ENTRYPOINT ["rust-serv"]
CMD ["--config", "/etc/rust-serv/config.toml"]
