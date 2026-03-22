//! HTTP and WebSocket server module.
//! Handles routing, player socket management, and server startup.

pub mod handlers;

use crate::config::CONFIG;
use crate::game::SharedGameState;
use crate::server::handlers::{
    health_handler, highscores_handler, stats_handler, ws_handler, AppState,
};
use axum::routing::get;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tracing::info;

/// Broadcast sender type for player communication.
pub type BroadcastSender = broadcast::Sender<Vec<u8>>;

/// Manages all connected player WebSocket senders.
/// Thread-safe container for broadcast channels.
#[derive(Clone)]
pub struct SharedPlayerSockets(pub Arc<RwLock<HashMap<String, BroadcastSender>>>);

impl Default for SharedPlayerSockets {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedPlayerSockets {
    /// Creates a new empty player sockets manager.
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    /// Broadcasts a message to all connected players.
    pub fn broadcast(&self, message: Vec<u8>) {
        for sender in self.0.read().values() {
            let _ = sender.send(message.clone());
        }
    }
}

/// Starts the HTTP and WebSocket server.
pub async fn start_server(state: SharedGameState, player_sockets: SharedPlayerSockets) {
    let app_state = AppState {
        game_state: state,
        player_sockets,
    };

    let app = axum::Router::new()
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler))
        .route("/admin/highscores", get(highscores_handler))
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("client/dist"))
        .with_state(Arc::new(app_state));

    let addr = format!("0.0.0.0:{}", CONFIG.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("[SERVER] HTTP and WebSocket server listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
