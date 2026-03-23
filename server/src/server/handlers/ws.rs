//! WebSocket handling for client connections.
//! Includes rate limiting and message size validation for security.

use crate::server::handlers::rate_limiter::RateLimiter;
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
use tracing::info;
use uuid::Uuid;

/// Maximum message size in bytes (64KB)
const MAX_MESSAGE_SIZE: usize = 64 * 1024;

#[derive(Clone)]
pub struct AppState {
    pub game_state: SharedGameState,
    pub player_sockets: SharedPlayerSockets,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let client_ip = extract_ip_from_headers(&headers);
    ws.on_upgrade(move |socket| handle_socket(socket, state, client_ip))
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    client_ip: Option<String>,
) {
    let (mut sender, mut receiver) = socket.split();
    let socket_id = Uuid::new_v4().to_string();
    let ip_str = client_ip.as_deref().unwrap_or("unknown");
    info!("[CONNECT] {} connected from {}", socket_id, ip_str);

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

    let (tx, mut rx) = broadcast::channel::<Vec<u8>>(100);
    {
        let mut sockets = state.player_sockets.0.write();
        sockets.insert(socket_id.clone(), tx.clone());
    }

    {
        let mut game = state.game_state.0.write();
        let snake = game.create_snake(socket_id.clone());
        game.snakes.insert(socket_id.clone(), snake);
    }

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

    let mut rate_limiter = RateLimiter::new();

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

    let state_clone = state.clone();
    let socket_id_clone = socket_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if !rate_limiter.try_consume() {
                info!("[RATE_LIMIT] {} exceeded rate limit", socket_id_clone);
                continue;
            }

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

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    info!("[DISCONNECT] {} (from {})", socket_id, ip_str);

    {
        let mut game = state.game_state.0.write();
        game.remove_snake(&socket_id);
    }
    {
        let mut sockets = state.player_sockets.0.write();
        sockets.remove(&socket_id);
    }
}

fn extract_ip_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(cf_ip) = headers
        .get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok())
    {
        return Some(cf_ip.to_string());
    }

    if let Some(xff) = headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(ip) = xff.split(',').next() {
            let ip = ip.trim();
            if !ip.is_empty() && ip != "127.0.0.1" {
                return Some(ip.to_string());
            }
        }
    }

    if let Some(real_ip) = headers
        .get("X-Real-IP")
        .and_then(|v| v.to_str().ok())
    {
        return Some(real_ip.to_string());
    }

    None
}

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
