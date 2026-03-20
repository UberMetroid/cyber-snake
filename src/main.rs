mod config;
mod game;
mod server;

use crate::config::CONFIG;
use crate::game::SharedGameState;
use crate::server::{start_server, SharedPlayerSockets};
use std::fs;
use tokio::time::{interval, Duration};
use tracing::info;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn setup_logging() {
    fs::create_dir_all(&CONFIG.log_dir).ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, &CONFIG.log_dir, "game.log");

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .init();
}

fn setup_data_dir() {
    fs::create_dir_all(&CONFIG.data_dir).ok();
}

#[tokio::main]
async fn main() {
    setup_logging();
    setup_data_dir();

    info!("[SERVER] Starting CYBER_SNAKE v1.0");
    info!("[CONFIG] Port: {}", CONFIG.port);
    info!("[CONFIG] TZ: {}", CONFIG.timezone);
    info!("[CONFIG] Max players: {}", CONFIG.max_players);
    info!("[CONFIG] Tick rate: {}", CONFIG.tick_rate);
    info!("[CONFIG] Grid: {}x{}", CONFIG.cols, CONFIG.rows);

    let state = SharedGameState::new();
    let player_sockets = SharedPlayerSockets::new();

    let state_clone = state.clone();
    let sockets_clone = player_sockets.clone();
    tokio::spawn(async move {
        start_server(state_clone, sockets_clone).await;
    });

    let mut ticker = interval(Duration::from_millis(1000 / CONFIG.tick_rate));

    loop {
        ticker.tick().await;
        let mut game = state.0.write();
        game.tick();

        let broadcast = game.broadcast_state();
        if let Ok(json) = serde_json::to_string(&broadcast) {
            player_sockets.broadcast(&json);
        }
    }
}
