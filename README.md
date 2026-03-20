# Cyber-Snake

A multiplayer, real-time Snake game written in Rust. Features a WebSocket game server and an HTTP API, packed into a lightweight, highly-optimized Docker container.

## Features
- **Real-time Multiplayer:** Play against others on a shared grid.
- **High Performance:** Written in Rust using `tokio`, `axum`, and `tungstenite`.
- **Docker Ready:** Multi-stage `alpine` Docker build with musl caching.

## Running on Synology (Container Manager)

To run this easily on a Synology NAS using Container Manager, use the following `docker-compose.yml`:

```yaml
version: '3.8'

services:
  cyber-snake:
    image: ghcr.io/ubermetroid/cyber-snake:latest
    container_name: cyber-snake
    ports:
      - "${PORT:-8300}:8300"
    environment:
      - TZ=${TZ:-UTC}
      - RUST_LOG=${RUST_LOG:-info}
      - MAX_PLAYERS=${MAX_PLAYERS:-10}
      - TICK_RATE=${TICK_RATE:-60}
      - COLS=${COLS:-30}
      - ROWS=${ROWS:-30}
      - DATA_DIR=/app/data
      - LOG_DIR=/app/logs
    volumes:
      # Use absolute paths to your shared folders for easy backup/access
      - /volume1/docker/cyber-snake/data:/app/data
      - /volume1/docker/cyber-snake/logs:/app/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost:8300/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 15s
```

### Important: GitHub Container Registry (GHCR) Visibility
If you get a `Head "https://ghcr.io/v2/..."` error when pulling the image:
1. Go to the [Cyber-snake Packages page](https://github.com/ubermetroid/cyber-snake/pkgs/container/cyber-snake) on GitHub.
2. Go to **Package Settings**.
3. Under **Danger Zone**, click **Change visibility** and set it to **Public**.
