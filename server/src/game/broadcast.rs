//! Broadcast structures for network transmission.
//! Contains GameBroadcast struct sent to all clients each tick.

use serde::Serialize;
use shared::{BonusFood, Explosion, Food, Powerup, Snake};
use std::collections::HashMap;

/// Game state broadcast sent to all clients each tick.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameBroadcast<'a> {
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    #[serde(borrow)]
    pub snakes: &'a HashMap<String, Snake>,
    #[serde(borrow)]
    pub foods: &'a HashMap<String, Food>,
    #[serde(borrow, skip_serializing_if = "Vec::is_empty")]
    pub bonus_foods: &'a Vec<BonusFood>,
    #[serde(borrow, skip_serializing_if = "Vec::is_empty")]
    pub powerups: &'a Vec<Powerup>,
    #[serde(borrow, skip_serializing_if = "Vec::is_empty")]
    pub explosions: &'a Vec<Explosion>,
    pub tick: u64,
}

impl<'a> GameBroadcast<'a> {
    /// Creates a new game broadcast from the current game state.
    pub fn new(
        snakes: &'a HashMap<String, Snake>,
        foods: &'a HashMap<String, Food>,
        bonus_foods: &'a Vec<BonusFood>,
        powerups: &'a Vec<Powerup>,
        explosions: &'a Vec<Explosion>,
        tick: u64,
    ) -> Self {
        Self {
            msg_type: "gameState",
            snakes,
            foods,
            bonus_foods,
            powerups,
            explosions,
            tick,
        }
    }
}
