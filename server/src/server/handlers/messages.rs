//! Client message handling and dispatching.
//! Routes incoming client messages to appropriate game actions.

use crate::game::Direction;
use crate::server::handlers::ws::AppState;
use serde::Serialize;
use std::sync::Arc;

/// Handles incoming client messages and updates game state accordingly.
pub async fn handle_client_message(socket_id: &str, msg: ClientMessage, state: &Arc<AppState>) {
    match msg {
        ClientMessage::Spawn => {
            handle_spawn(socket_id, state).await;
        }
        ClientMessage::Input { dir } => {
            handle_input(socket_id, &dir, state);
        }
        ClientMessage::ActivatePowerup => {
            handle_activate_powerup(socket_id, state);
        }
        ClientMessage::Respawn => {
            handle_respawn(socket_id, state).await;
        }
        ClientMessage::Ping => {
            handle_ping(socket_id, state);
        }
    }
}

async fn handle_spawn(socket_id: &str, state: &Arc<AppState>) {
    let mut game = state.game_state.0.write();
    let active_players = game
        .snakes
        .values()
        .filter(|s| s.alive && s.spawned)
        .count();

    if active_players >= crate::config::CONFIG.max_players {
        if let Some(sender) = state.player_sockets.0.read().get(socket_id) {
            if let Ok(err_msg) = rmp_serde::to_vec_named(&ErrMsg {
                t: "error".to_string(),
                message: "SERVER FULL - SPECTATING".to_string(),
            }) {
                let _ = sender.send(err_msg);
            }
        }
    } else {
        game.spawn_snake(socket_id);
    }
}

fn handle_input(socket_id: &str, dir: &str, state: &Arc<AppState>) {
    let mut game = state.game_state.0.write();
    if let Some(snake) = game.snakes.get_mut(socket_id) {
        if snake.alive && snake.spawned {
            let new_dir = match dir {
                "up" => Some(Direction::Up),
                "down" => Some(Direction::Down),
                "left" => Some(Direction::Left),
                "right" => Some(Direction::Right),
                _ => None,
            };

            if let Some(new_dir) = new_dir {
                let opposite = matches!(
                    (&new_dir, &snake.dir),
                    (Direction::Up, Direction::Down)
                        | (Direction::Down, Direction::Up)
                        | (Direction::Left, Direction::Right)
                        | (Direction::Right, Direction::Left)
                );

                if !opposite {
                    snake.next_dir = new_dir;
                }
            }
        }
    }
}

fn handle_activate_powerup(socket_id: &str, state: &Arc<AppState>) {
    let mut game = state.game_state.0.write();
    game.activate_powerup(socket_id);
}

async fn handle_respawn(socket_id: &str, state: &Arc<AppState>) {
    let mut game = state.game_state.0.write();
    let active_players = game
        .snakes
        .values()
        .filter(|s| s.alive && s.spawned)
        .count();

    if active_players >= crate::config::CONFIG.max_players {
        if let Some(sender) = state.player_sockets.0.read().get(socket_id) {
            if let Ok(err_msg) = rmp_serde::to_vec_named(&ErrMsg {
                t: "error".to_string(),
                message: "SERVER FULL - SPECTATING".to_string(),
            }) {
                let _ = sender.send(err_msg);
            }
        }
    } else {
        game.respawn_snake(socket_id);
    }
}

fn handle_ping(socket_id: &str, state: &Arc<AppState>) {
    #[derive(Serialize)]
    struct PongMsg {
        #[serde(rename = "type")]
        t: String,
    }
    if let Ok(msg) = rmp_serde::to_vec_named(&PongMsg {
        t: "pong".to_string(),
    }) {
        if let Some(sender) = state.player_sockets.0.read().get(socket_id) {
            let _ = sender.send(msg);
        }
    }
}

#[derive(Serialize)]
struct ErrMsg {
    #[serde(rename = "type")]
    t: String,
    message: String,
}

// Re-export ClientMessage from ws module for external use
pub use crate::server::handlers::ws::ClientMessage;
