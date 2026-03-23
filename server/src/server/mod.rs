//! HTTP and WebSocket server module.
//! Handles routing, player socket management, graceful shutdown, and server startup.

pub mod handlers;

use crate::config::CONFIG;
use crate::game::SharedGameState;
use crate::server::handlers::{
    health_handler, highscores_handler, stats_handler, ws_handler, AppState,
};
use axum::routing::get;
use axum::Router;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tracing::info;

pub type BroadcastSender = broadcast::Sender<Vec<u8>>;

#[derive(Clone)]
pub struct SharedPlayerSockets(pub Arc<RwLock<HashMap<String, BroadcastSender>>>);

impl Default for SharedPlayerSockets {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedPlayerSockets {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn broadcast(&self, message: Vec<u8>) {
        for sender in self.0.read().values() {
            let _ = sender.send(message.clone());
        }
    }
}

pub async fn start_server(
    state: SharedGameState,
    player_sockets: SharedPlayerSockets,
    mut shutdown: broadcast::Receiver<()>,
) {
    let app_state = AppState {
        game_state: state,
        player_sockets: player_sockets.clone(),
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler))
        .route("/admin/highscores", get(highscores_handler))
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("client/dist"))
        .with_state(Arc::new(app_state));

    let addr = format!("0.0.0.0:{}", CONFIG.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("[SERVER] HTTP and WebSocket server listening on {}", addr);

    let server = axum::serve(listener, app);

    tokio::select! {
        result = server => {
            if let Err(e) = result {
                tracing::error!("[SERVER] Axum server error: {}", e);
            }
        }
        _ = shutdown.recv() => {
            info!("[SERVER] Shutdown signal received, stopping server...");
        }
    }
}
