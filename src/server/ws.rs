use crate::config::CONFIG;
use crate::game::{SharedGameState, Snake};
use crate::server::SharedPlayerSockets;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{info, error};
use uuid::Uuid;

#[derive(Clone)]
pub struct SharedPlayerSockets(pub Arc<RwLock<HashMap<String, broadcast::Sender<String>>>);

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

pub async fn start_ws_server(state: SharedGameState, player_sockets: SharedPlayerSockets) {
    let addr = format!("0.0.0.0:{}", CONFIG.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("[SERVER] WebSocket server listening on {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let state = state.clone();
        let player_sockets = player_sockets.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, state, player_sockets).await {
                error!("[WS] Connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
    state: SharedGameState,
    player_sockets: SharedPlayerSockets,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();
    
    let socket_id = Uuid::new_v4().to_string();
    info!("[CONNECT] {} connected", socket_id);

    {
        let mut sockets = player_sockets.0.write();
        sockets.insert(socket_id.clone(), broadcast::channel(100).0);
    }

    let (tx, mut rx) = broadcast::channel::<String>(100);
    {
        let mut sockets = player_sockets.0.write();
        if let Some(sender) = sockets.get(&socket_id) {
            *sender = tx.clone();
        }
    }

    let state_clone = state.clone();
    let socket_id_clone = socket_id.clone();
    let write_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if write.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    {
        let mut game = state.0.write();
        let snake = game.create_snake(socket_id.clone());
        game.snakes.insert(socket_id.clone(), snake);
    }

    let welcome = {
        let game = state.0.read();
        if let Some(snake) = game.snakes.get(&socket_id) {
            serde_json::to_string(&WelcomeMessage {
                msg_type: "welcome".to_string(),
                id: socket_id.clone(),
                snake: SnakePreview {
                    color: snake.color.clone(),
                    name: snake.name.clone(),
                    score: snake.score,
                },
            }).unwrap()
        } else {
            return Ok(());
        }
    };

    write.send(Message::Text(welcome.into())).await?;

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    handle_client_message(
                        &socket_id,
                        client_msg,
                        &state,
                        &player_sockets,
                    ).await;
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                error!("[WS] Error reading message: {}", e);
                break;
            }
            _ => {}
        }
    }

    info!("[DISCONNECT] {}", socket_id);
    {
        let mut game = state.0.write();
        game.remove_snake(&socket_id);
    }
    {
        let mut sockets = player_sockets.0.write();
        sockets.remove(&socket_id);
    }

    Ok(())
}

async fn handle_client_message(
    socket_id: &str,
    msg: ClientMessage,
    state: &SharedGameState,
    player_sockets: &SharedPlayerSockets,
) {
    match msg {
        ClientMessage::Spawn => {
            let mut game = state.0.write();
            game.spawn_snake(socket_id);
        }
        ClientMessage::Input { dir } => {
            let mut game = state.0.write();
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
                        let opposite = match (&new_dir, &snake.dir) {
                            (crate::game::Direction::Up, crate::game::Direction::Down) => true,
                            (crate::game::Direction::Down, crate::game::Direction::Up) => true,
                            (crate::game::Direction::Left, crate::game::Direction::Right) => true,
                            (crate::game::Direction::Right, crate::game::Direction::Left) => true,
                            _ => false,
                        };
                        
                        if !opposite {
                            snake.next_dir = new_dir;
                        }
                    }
                }
            }
        }
        ClientMessage::ActivatePowerup => {
            let mut game = state.0.write();
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
            let mut game = state.0.write();
            game.respawn_snake(socket_id);
        }
        ClientMessage::Ping => {
            let msg = r#"{"type":"pong"}"#.to_string();
            if let Some(sender) = player_sockets.0.read().get(socket_id) {
                let _ = sender.send(msg);
            }
        }
    }
}
