//! Network message types for client-server communication.
//! Contains ClientMessage, WelcomeMessage, GameBroadcast, and related types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{BonusFood, Explosion, Food, Powerup, Snake};

/// Preview of snake info sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnakePreview {
    pub color: String,
    pub name: String,
    pub score: u32,
}

/// Welcome message sent to new clients upon connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub id: String,
    pub snake: SnakePreview,
    #[serde(rename = "tickRate")]
    pub tick_rate: u64,
    pub cols: u32,
    pub rows: u32,
}

/// Message sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Game state broadcast sent to all clients each tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameBroadcast {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub snakes: HashMap<String, Snake>,
    pub foods: HashMap<String, Food>,
    #[serde(default)]
    pub bonus_foods: Vec<BonusFood>,
    #[serde(default)]
    pub powerups: Vec<Powerup>,
    #[serde(default)]
    pub explosions: Vec<Explosion>,
    pub tick: u64,
}

impl GameBroadcast {
    pub fn new(
        snakes: HashMap<String, Snake>,
        foods: HashMap<String, Food>,
        bonus_foods: Vec<BonusFood>,
        powerups: Vec<Powerup>,
        explosions: Vec<Explosion>,
        tick: u64,
    ) -> Self {
        Self {
            msg_type: "gameState".to_string(),
            snakes,
            foods,
            bonus_foods,
            powerups,
            explosions,
            tick,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_spawn() {
        let json = r#"{"type":"spawn"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Spawn));
    }

    #[test]
    fn test_client_message_input() {
        let json = r#"{"type":"input","dir":"up"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Input { dir } if dir == "up"));
    }

    #[test]
    fn test_game_broadcast_new() {
        let broadcast = GameBroadcast::new(
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            100,
        );
        assert_eq!(broadcast.msg_type, "gameState");
        assert_eq!(broadcast.tick, 100);
    }
}
