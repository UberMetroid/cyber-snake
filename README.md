# Cyber-Snake

A multiplayer, real-time Snake game written in Rust. Features a highly optimized backend using `tokio` and `axum`, packed into a lightweight Docker container.

The entire application—serving the static HTML/JS frontend, the HTTP API, and the real-time WebSocket game loop—is hosted on a **single port (8300)**, making it perfect for hosting behind reverse proxies or Cloudflare Tunnels.

## Features

- **Real-time Multiplayer:** Play against others on a shared grid with powerups and dynamic scoring.
- **High Performance:** Written in Rust, running via a multi-threaded async Tokio runtime.
- **Single Port Architecture:** Frontend and WebSockets are unified via Axum.
- **Docker Ready:** Multi-stage `alpine` Docker build with aggressive layer caching.
- **Security:** Rate limiting, message size validation, and input sanitization.
- **Accessible:** Keyboard navigation with ARIA labels for screen readers.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Client (WASM)                       │
│   ┌─────────┐  ┌─────────┐  ┌─────────┐              │
│   │ Network  │  │  Input  │  │ Render  │              │
│   └────┬────┘  └────┬────┘  └────┬────┘              │
└────────┼─────────────┼─────────────┼────────────────────┘
         │             │             │
         │ WebSocket   │             │
         ▼             ▼             ▼
┌─────────────────────────────────────────────────────────┐
│                   Server (Rust)                          │
│   ┌─────────────────────────────────────────────────┐   │
│   │              Game State (Tick Loop)              │   │
│   │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐        │   │
│   │  │Snakes│ │ Food │ │Powerup│ │Collision│       │   │
│   └─────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────┐   │
│   │        HTTP Server (Axum)                        │   │
│   │   /health  /stats  /admin/highscores  /ws      │   │
│   └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Game Protocol

### WebSocket Messages

**Client → Server:**

| Type | Payload | Description |
|------|---------|-------------|
| `spawn` | `{}` | Spawn a new snake |
| `input` | `{"dir": "up\|down\|left\|right"}` | Change direction |
| `activatePowerup` | `{}` | Use held powerup |
| `respawn` | `{}` | Respawn after death |
| `ping` | `{}` | Keep-alive ping |

**Server → Client:**

| Type | Payload | Description |
|------|---------|-------------|
| `welcome` | `{id, snake, tickRate, cols, rows}` | Initial game state |
| `broadcast` | `{snakes, foods, bonusFoods, powerups, explosions, tick}` | Game state update |

### REST Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Server health and uptime |
| `/stats` | GET | Current game statistics |
| `/admin/highscores` | GET | Top 10 high scores |

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `PORT` | 8300 | Server port |
| `TZ` | UTC | Timezone |
| `MAX_PLAYERS` | 50 | Maximum concurrent players |
| `TICK_RATE` | 20 | Game ticks per second |
| `COLS` | 100 | Grid width |
| `ROWS` | 100 | Grid height |

## Running

### Docker Compose

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
      - TICK_RATE=${TICK_RATE:-20}
      - COLS=${COLS:-30}
      - ROWS=${ROWS:-30}
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost:8300/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 15s
```

### Cloudflare Tunnels

Because `cyber-snake` serves everything over a single port, configuring it with Cloudflare Tunnels (Zero Trust) is incredibly simple:

1. In your Cloudflare Zero Trust dashboard, create a Public Hostname.
2. Set the Service Type to `HTTP`.
3. Set the URL to `your_docker_host_ip:8300` (e.g., `192.168.1.100:8300`).
4. The game UI and WebSocket connection will automatically route through the tunnel perfectly.

### Building from Source

```bash
# Build the WASM client
cd client && trunk build && cd ..

# Build the server
cargo build --release

# Run
RUST_LOG=info ./target/release/server
```

## Development

```bash
# Run tests
cargo test

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Lint
cargo clippy
```

## Troubleshooting

### GitHub Container Registry (GHCR)

If you get a `Head "https://ghcr.io/v2/..."` error when pulling the image for the first time:

1. Go to the [Cyber-snake Packages page](https://github.com/ubermetroid/cyber-snake/pkgs/container/cyber-snake) on GitHub.
2. Go to **Package Settings** (bottom right).
3. Under **Danger Zone**, click **Change visibility** and set it to **Public**.

## License

MIT License - see LICENSE file for details.
