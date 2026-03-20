FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
LABEL org.opencontainers.image.source="https://github.com/ubermetroid/cyber-snake"

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/cyber-snake /app/cyber-snake
COPY public ./public

EXPOSE 8300

CMD ["/app/cyber-snake"]
