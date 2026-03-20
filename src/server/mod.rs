use crate::config::CONFIG;
use crate::game::SharedGameState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use futures_util::{stream::StreamExt, SinkExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
pub struct SharedPlayerSockets(pub Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>);

impl SharedPlayerSockets {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn broadcast(&self, message: &str) {
        for sender in self.0.read().values() {
            let _ = sender.send(message.to_string());
        }
    }
}

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

#[derive(Serialize)]
struct WelcomeMessage {
    #[serde(rename = "type")]
    msg_type: String,
    id: String,
    snake: SnakePreview,
}

#[derive(Serialize)]
struct SnakePreview {
    color: String,
    name: String,
    score: u32,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
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

pub async fn start_server(state: SharedGameState, player_sockets: SharedPlayerSockets) {
    let app_state = AppState {
        game_state: state,
        player_sockets,
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler))
        .route("/admin/highscores", get(highscores_handler))
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new("public"))
        .with_state(Arc::new(app_state));

    let addr = format!("0.0.0.0:{}", CONFIG.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("[SERVER] HTTP and WebSocket server listening on {}", addr);

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
        players_online: game
            .snakes
            .values()
            .filter(|s| s.alive && s.spawned)
            .count(),
    })
}

async fn stats_handler(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
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

async fn highscores_handler(State(state): State<Arc<AppState>>) -> Json<HighScoresResponse> {
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

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let socket_id = Uuid::new_v4().to_string();
    info!("[CONNECT] {} connected", socket_id);

    let (tx, mut rx) = broadcast::channel::<String>(100);
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
            serde_json::to_string(&WelcomeMessage {
                msg_type: "welcome".to_string(),
                id: socket_id.clone(),
                snake: SnakePreview {
                    color: snake.color.clone(),
                    name: snake.name.clone(),
                    score: snake.score,
                },
            })
            .unwrap()
        } else {
            return;
        }
    };

    if sender.send(Message::Text(welcome)).await.is_err() {
        return;
    }

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let socket_id_clone = socket_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    handle_client_message(&socket_id_clone, client_msg, &state_clone).await;
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("[DISCONNECT] {}", socket_id);
    {
        let mut game = state.game_state.0.write();
        game.remove_snake(&socket_id);
    }
    {
        let mut sockets = state.player_sockets.0.write();
        sockets.remove(&socket_id);
    }
}

async fn handle_client_message(socket_id: &str, msg: ClientMessage, state: &Arc<AppState>) {
    match msg {
        ClientMessage::Spawn => {
            let mut game = state.game_state.0.write();
            game.spawn_snake(socket_id);
        }
        ClientMessage::Input { dir } => {
            let mut game = state.game_state.0.write();
            if let Some(snake) = game.snakes.get_mut(socket_id) {
                if snake.alive && snake.spawned {
                    let new_dir = match dir.as_str() {
                        "up" => Some(crate::game::Direction::Up),
                        "down" => Some(crate::game::Direction::Down),
                        "left" => Some(crate::game::Direction::Left),
                        "right" => Some(crate::game::Direction::Right),
                        _ => None,
                    };

                    if let Some(new_dir) = new_dir {
                        let opposite = matches!(
                            (&new_dir, &snake.dir),
                            (crate::game::Direction::Up, crate::game::Direction::Down)
                                | (crate::game::Direction::Down, crate::game::Direction::Up)
                                | (crate::game::Direction::Left, crate::game::Direction::Right)
                                | (crate::game::Direction::Right, crate::game::Direction::Left)
                        );

                        if !opposite {
                            snake.next_dir = new_dir;
                        }
                    }
                }
            }
        }
        ClientMessage::ActivatePowerup => {
            let mut game = state.game_state.0.write();
            if let Some(snake) = game.snakes.get_mut(socket_id) {
                if snake.alive && snake.spawned {
                    if let Some(powerup_type) = &snake.held_powerup {
                        info!("[POWERUP] {} activated {}", snake.color, powerup_type);
                        snake.held_powerup = None;
                    }
                }
            }
        }
        ClientMessage::Respawn => {
            let mut game = state.game_state.0.write();
            game.respawn_snake(socket_id);
        }
        ClientMessage::Ping => {
            let msg = r#"{"type":"pong"}"#.to_string();
            if let Some(sender) = state.player_sockets.0.read().get(socket_id) {
                let _ = sender.send(msg);
            }
        }
    }
}
