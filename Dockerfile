FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY src ./src
COPY public ./public

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.19

RUN apk add --no-cache ca-certificates tini

WORKDIR /app

RUN addgroup -g 1000 app && adduser -u 1000 -G app -s /bin/sh -D app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cyber-snake /app/cyber-snake
COPY --from=builder /app/public ./public

RUN mkdir -p /app/data /app/logs && chown -R app:app /app

USER app

EXPOSE 3000 3001

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/cyber-snake"]
