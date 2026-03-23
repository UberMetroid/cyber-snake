# Build stage
FROM rust:1.75-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY server ./server

# Create dummy source to cache dependencies
RUN mkdir -p server/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "pub fn main() {}" > shared/src/lib.rs

# Build dependencies (caching layer)
RUN cargo build --release -p server --locked 2>/dev/null || true

# Copy actual source
COPY shared/src shared/src
COPY server/src server/src

# Build release binary
RUN touch server/src/main.rs && \
    cargo build --release -p server --locked

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create data directory for game state persistence
RUN mkdir -p /app/data

# Copy binary from builder
COPY --from=builder /build/server/release/server /app/server
COPY --from=builder /build/server/data /app/data

# Create non-root user
RUN useradd -m -s /bin/bash appuser && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE 8300

ENV RUST_LOG=info
ENV PORT=8300
ENV TZ=UTC
ENV MAX_PLAYERS=50
ENV TICK_RATE=20
ENV COLS=100
ENV ROWS=100

ENTRYPOINT ["/app/server"]
