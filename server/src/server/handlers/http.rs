//! HTTP request handlers for health, stats, and highscores endpoints.

use super::ws::AppState;
use crate::config::CONFIG;
use axum::{extract::State, response::Json};
use serde::Serialize;
use std::sync::Arc;

/// Health check response containing server status.
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    uptime_seconds: i64,
    players_online: usize,
}

/// Server statistics response.
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

/// High score entry.
#[derive(Serialize)]
pub struct HighScoreEntry {
    name: String,
    score: u32,
    date: String,
}

/// High scores response.
#[derive(Serialize)]
pub struct HighScoresResponse {
    highscores: Vec<HighScoreEntry>,
}

/// Handler for GET /health - returns server health status.
pub async fn health_handler(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let game = state.game_state.0.read();
    let start_time = game.start_time;
    let now = chrono::Utc::now().timestamp_millis();
    let uptime = (now - start_time) / 1000;

    Json(HealthResponse {
        status: "ok".to_string(),
        uptime_seconds: uptime,
        players_online: game
            .snakes
            .values()
            .filter(|s| s.alive && s.spawned)
            .count(),
    })
}

/// Handler for GET /stats - returns detailed server statistics.
pub async fn stats_handler(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    let game = state.game_state.0.read();
    let start_time = game.start_time;
    let now = chrono::Utc::now().timestamp_millis();
    let uptime = (now - start_time) / 1000;

    let alive_players = game
        .snakes
        .values()
        .filter(|s| s.alive && s.spawned)
        .count();

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

/// Handler for GET /admin/highscores - returns top 10 high scores.
pub async fn highscores_handler(State(state): State<Arc<AppState>>) -> Json<HighScoresResponse> {
    let mut game = state.game_state.0.write();
    game.update_high_scores();

    let highscores: Vec<HighScoreEntry> = game
        .high_scores
        .iter()
        .map(|hs| HighScoreEntry {
            name: hs.name.clone(),
            score: hs.score,
            date: hs.date.clone(),
        })
        .collect();

    Json(HighScoresResponse { highscores })
}
