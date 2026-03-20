use crate::config::CONFIG;
use crate::game::SharedGameState;
use crate::server::SharedPlayerSockets;
use axum::{
    extract::{State, Path},
    response::Json,
    routing::get,
    Router,
};
use parking_lot::RwLock;
use serde::Serialize;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub game_state: SharedGameState,
    pub player_sockets: SharedPlayerSockets,
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    uptime_seconds: i64,
    players_online: usize,
}

#[derive(Serialize)]
pub struct StatsResponse {
    players_online: usize,
    max_players: usize,
    tick: u64,
    uptime_seconds: i64,
    foods: usize,
    bonus_foods: usize,
    powerups: usize,
}

#[derive(Serialize)]
pub struct HighScoresResponse {
    highscores: Vec<HighScoreEntry>,
}

#[derive(Serialize)]
pub struct HighScoreEntry {
    name: String,
    score: u32,
    date: String,
}

pub async fn start_http_server(state: SharedGameState, player_sockets: SharedPlayerSockets) {
    let app_state = AppState {
        game_state: state,
        player_sockets,
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler))
        .route("/admin/highscores", get(highscores_handler))
        .with_state(Arc::new(app_state));

    let addr = format!("0.0.0.0:{}", CONFIG.port + 1);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("[SERVER] HTTP API server listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn health_handler(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let game = state.game_state.0.read();
    let start_time = game.start_time;
    let now = chrono::Utc::now().timestamp_millis();
    let uptime = (now - start_time) / 1000;
    
    Json(HealthResponse {
        status: "ok".to_string(),
        uptime_seconds: uptime,
        players_online: game.snakes.values().filter(|s| s.alive && s.spawned).count(),
    })
}

async fn stats_handler(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    let game = state.game_state.0.read();
    let start_time = game.start_time;
    let now = chrono::Utc::now().timestamp_millis();
    let uptime = (now - start_time) / 1000;
    
    let alive_players = game.snakes.values().filter(|s| s.alive && s.spawned).count();
    
    Json(StatsResponse {
        players_online: alive_players,
        max_players: CONFIG.max_players,
        tick: game.tick,
        uptime_seconds: uptime,
        foods: game.foods.len(),
        bonus_foods: game.bonus_foods.len(),
        powerups: game.powerups.len(),
    })
}

async fn highscores_handler(State(state): State<Arc<AppState>>) -> Json<HighScoresResponse> {
    let mut game = state.game_state.0.write();
    game.update_high_scores();
    
    let highscores: Vec<HighScoreEntry> = game.high_scores.iter().map(|hs| {
        HighScoreEntry {
            name: hs.name.clone(),
            score: hs.score,
            date: hs.date.clone(),
        }
    }).collect();
    
    Json(HighScoresResponse { highscores })
}
