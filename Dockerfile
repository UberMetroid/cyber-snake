# -- BUILD STAGE --
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Install dependencies for downloading trunk
RUN apt-get update && apt-get install -y wget

# Install WebAssembly target
RUN rustup target add wasm32-unknown-unknown

# Download and install Trunk pre-compiled binary for fast builds
RUN wget -qO- https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /usr/local/bin

# Copy workspace
COPY Cargo.toml Cargo.lock* ./
COPY shared ./shared
COPY client ./client
COPY server ./server

# Build the frontend Wasm application
WORKDIR /app/client
RUN trunk build --release

# Build the backend server
WORKDIR /app
RUN cargo build --release -p server

# -- RUNTIME STAGE --
FROM debian:bookworm-slim
LABEL org.opencontainers.image.source="https://github.com/ubermetroid/cyber-snake"

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=builder /app/target/release/server /app/cyber-snake
# Copy frontend Wasm build
COPY --from=builder /app/client/dist ./client/dist

VOLUME /app/data
EXPOSE 8300

CMD ["/app/cyber-snake"]
