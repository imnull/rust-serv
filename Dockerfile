# Build stage - use full Debian image with all build tools
FROM rust:1.82

WORKDIR /app

# Copy all source files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=0 /app/target/release/rust-serv /usr/local/bin/rust-serv

RUN mkdir -p /var/www/html /etc/rust-serv
COPY docker/config.toml /etc/rust-serv/config.toml

EXPOSE 8080 8443

ENV RUST_LOG=info

ENTRYPOINT ["rust-serv"]
CMD ["--config", "/etc/rust-serv/config.toml"]
