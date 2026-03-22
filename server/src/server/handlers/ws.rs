//! WebSocket handling for client connections.
//! Includes rate limiting and message size validation for security.

use crate::config::CONFIG;
use crate::game::SharedGameState;
use crate::server::handlers::messages::handle_client_message;
use crate::server::SharedPlayerSockets;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use bytes::Bytes;
use futures_util::{stream::StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::Instant;
use tracing::info;
use uuid::Uuid;

/// Maximum message size in bytes (64KB)
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

/// Maximum messages per second per client (rate limiting)
const MAX_MESSAGES_PER_SEC: usize = 100;

/// Token bucket for rate limiting (async-friendly)
struct RateLimiter {
    tokens: usize,
    last_refill: Instant,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            tokens: MAX_MESSAGES_PER_SEC,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        // Refill tokens: 1 token per 10ms
        let new_tokens = elapsed.as_millis() as usize / 10;
        if new_tokens > 0 {
            self.tokens = (self.tokens + new_tokens).min(MAX_MESSAGES_PER_SEC);
            self.last_refill = now;
        }

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }
}

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub game_state: SharedGameState,
    pub player_sockets: SharedPlayerSockets,
}

/// Handler for WebSocket upgrade at GET /ws.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handles an individual WebSocket client connection.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let socket_id = Uuid::new_v4().to_string();
    info!("[CONNECT] {} connected", socket_id);

    // Check if server is at capacity
    let is_full = {
        let sockets = state.player_sockets.0.read();
        sockets.len() >= 100
    };

    if is_full {
        info!("[REJECT] {} rejected: Server Full", socket_id);
        let _ = sender
            .send(Message::Text(
                r#"{"type":"error","message":"Server full"}"#.into(),
            ))
            .await;
        return;
    }

    // Create broadcast channel for this client
    let (tx, mut rx) = broadcast::channel::<Vec<u8>>(100);
    {
        let mut sockets = state.player_sockets.0.write();
        sockets.insert(socket_id.clone(), tx.clone());
    }

    // Create snake for this player
    {
        let mut game = state.game_state.0.write();
        let snake = game.create_snake(socket_id.clone());
        game.snakes.insert(socket_id.clone(), snake);
    }

    // Send welcome message with snake info
    let welcome = {
        let game = state.game_state.0.read();
        if let Some(snake) = game.snakes.get(&socket_id) {
            if let Ok(welcome) = rmp_serde::to_vec_named(&WelcomeMessage {
                msg_type: "welcome".to_string(),
                id: socket_id.clone(),
                snake: SnakePreview {
                    color: snake.color.clone(),
                    name: snake.name.clone(),
                    score: snake.score,
                },
                tick_rate: CONFIG.tick_rate,
                cols: CONFIG.cols,
                rows: CONFIG.rows,
            }) {
                welcome
            } else {
                return;
            }
        } else {
            return;
        }
    };

    if sender
        .send(Message::Binary(Bytes::from(welcome)))
        .await
        .is_err()
    {
        return;
    }

    // Rate limiter for this connection
    let mut rate_limiter = RateLimiter::new();

    // Spawn task for sending messages to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender
                .send(Message::Binary(Bytes::from(msg)))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Spawn task for receiving messages from client
    let state_clone = state.clone();
    let socket_id_clone = socket_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // Check rate limit
            if !rate_limiter.try_consume() {
                info!("[RATE_LIMIT] {} exceeded rate limit", socket_id_clone);
                continue;
            }

            // Check message size to prevent DoS
            let msg_bytes = match &msg {
                Message::Binary(b) => b.len(),
                Message::Text(t) => t.len(),
                _ => 0,
            };

            if msg_bytes > MAX_MESSAGE_SIZE {
                info!(
                    "[REJECT] {} sent oversized message: {} bytes",
                    socket_id_clone, msg_bytes
                );
                continue;
            }

            match msg {
                Message::Binary(bin) => {
                    if let Ok(client_msg) = rmp_serde::from_slice::<ClientMessage>(&bin) {
                        handle_client_message(&socket_id_clone, client_msg, &state_clone).await;
                    }
                }
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        handle_client_message(&socket_id_clone, client_msg, &state_clone).await;
                    }
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    info!("[DISCONNECT] {}", socket_id);

    // Cleanup: remove snake and socket
    {
        let mut game = state.game_state.0.write();
        game.remove_snake(&socket_id);
    }
    {
        let mut sockets = state.player_sockets.0.write();
        sockets.remove(&socket_id);
    }
}

// Message types for client communication
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "spawn")]
    Spawn,
    #[serde(rename = "input")]
    Input { dir: String },
    #[serde(rename = "activatePowerup")]
    ActivatePowerup,
    #[serde(rename = "respawn")]
    Respawn,
    #[serde(rename = "ping")]
    Ping,
}

#[derive(serde::Serialize)]
struct WelcomeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    id: String,
    snake: SnakePreview,
    #[serde(rename = "tickRate")]
    tick_rate: u64,
    cols: u32,
    rows: u32,
}

#[derive(serde::Serialize)]
struct SnakePreview {
    color: String,
    name: String,
    score: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_burst() {
        let mut limiter = RateLimiter::new();
        // Should allow up to MAX_MESSAGES_PER_SEC immediately
        for _ in 0..MAX_MESSAGES_PER_SEC {
            assert!(limiter.try_consume());
        }
        // Should be exhausted now
        assert!(!limiter.try_consume());
    }

    #[test]
    fn test_rate_limiter_refills() {
        let mut limiter = RateLimiter::new();
        // Exhaust the limiter
        for _ in 0..MAX_MESSAGES_PER_SEC {
            limiter.try_consume();
        }
        assert!(!limiter.try_consume());

        // Wait 100ms (should refill ~10 tokens)
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Should be able to consume again
        assert!(limiter.try_consume());
    }
}
