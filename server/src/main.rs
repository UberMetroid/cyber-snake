//! Cyber-Snake game server entry point.
//! Handles startup, graceful shutdown, and game tick loop.

mod config;
mod game;
mod server;

use crate::config::CONFIG;
use crate::game::SharedGameState;
use crate::server::{start_server, SharedPlayerSockets};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

pub fn is_shutting_down() -> bool {
    SHUTTING_DOWN.load(Ordering::SeqCst)
}

fn setup_logging() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .init();
}

#[tokio::main]
async fn main() {
    setup_logging();

    info!("[SERVER] Starting CYBER_SNAKE v1.0");
    info!("[CONFIG] Port: {}", CONFIG.port);
    info!("[CONFIG] TZ: {}", CONFIG.timezone);
    info!("[CONFIG] Max players: {}", CONFIG.max_players);
    info!("[CONFIG] Tick rate: {}", CONFIG.tick_rate);
    info!("[CONFIG] Grid: {}x{}", CONFIG.cols, CONFIG.rows);

    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let state = SharedGameState::new();
    let player_sockets = SharedPlayerSockets::new();

    let state_clone = state.clone();
    let sockets_clone = player_sockets.clone();
    let shutdown_rx = shutdown_tx.subscribe();

    let server_handle = tokio::spawn(async move {
        start_server(state_clone, sockets_clone, shutdown_rx).await;
    });

    let shutdown_tx_clone = shutdown_tx.clone();
    let shutdown_signal = async {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("[SHUTDOWN] Received SIGINT (Ctrl+C)");
            }
            Err(e) => {
                error!("[SHUTDOWN] Failed to listen for Ctrl+C: {}", e);
            }
        }
        SHUTTING_DOWN.store(true, Ordering::SeqCst);
        let _ = shutdown_tx_clone.send(());
    };

    let mut ticker = interval(Duration::from_millis(1000 / CONFIG.tick_rate));
    let mut shutdown_rx = shutdown_tx.subscribe();

    tokio::select! {
        _ = shutdown_signal => {
            info!("[SHUTDOWN] Initiating graceful shutdown...");
        }
        _ = ticker.tick() => {
            let mut game = state.0.write();
            game.tick();
            drop(game);

            let msgpack = {
                let game = state.0.read();
                rmp_serde::to_vec_named(&game.broadcast_state())
            };
            if let Ok(msgpack) = msgpack {
                player_sockets.broadcast(msgpack);
            }
        }
        _ = shutdown_rx.recv() => {
            info!("[SHUTDOWN] Shutdown signal received");
        }
    }

    info!("[SHUTDOWN] Saving game state...");
    {
        let game = state.0.read();
        game.save_state();
    }

    info!("[SHUTDOWN] Stopping server...");
    server_handle.abort();

    info!("[SHUTDOWN] Cleanup complete. Goodbye!");
}
