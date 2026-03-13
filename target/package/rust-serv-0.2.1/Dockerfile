# Build stage
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

COPY --from=builder /app/target/release/rust_serv /usr/local/bin/rust-serv

# Create default web root
RUN mkdir -p /var/www/html

EXPOSE 8080 8443

ENV RUST_LOG=info

ENTRYPOINT ["rust-serv"]
CMD ["--config", "/etc/rust-serv/config.toml"]
