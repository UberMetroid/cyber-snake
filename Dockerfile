FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl && rm -rf src

COPY src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.19
LABEL org.opencontainers.image.source="https://github.com/ubermetroid/cyber-snake"

RUN apk add --no-cache ca-certificates tini

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cyber-snake /app/cyber-snake
COPY public ./public

EXPOSE 8300

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/cyber-snake"]
