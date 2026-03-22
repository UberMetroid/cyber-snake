//! Food types for the game.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Food item dropped by snakes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Food {
    pub owner_id: String,
    pub x: i32,
    pub y: i32,
    pub color: String,
    pub is_super: bool,
    pub is_ring: bool,
    pub expires_at: i64,
}

impl Food {
    pub fn new(owner_id: String, color: String, x: i32, y: i32, is_super: bool) -> Self {
        Self {
            owner_id,
            x,
            y,
            color,
            is_super,
            is_ring: rand::thread_rng().gen_bool(0.5),
            expires_at: chrono::Utc::now().timestamp_millis() + 60000,
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp_millis() > self.expires_at
    }
}

/// Bonus food spawned from dead snake segments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BonusFood {
    pub x: i32,
    pub y: i32,
    pub color: String,
    pub is_ring: bool,
    pub expires_at: i64,
}

impl BonusFood {
    pub fn new(x: i32, y: i32, color: String, is_ring: bool) -> Self {
        Self {
            x,
            y,
            color,
            is_ring,
            expires_at: chrono::Utc::now().timestamp_millis() + 60000,
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp_millis() > self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_food_is_expired() {
        let mut food = Food::new("id".to_string(), "#ff0000".to_string(), 5, 5, false);
        food.expires_at = chrono::Utc::now().timestamp_millis() - 1;
        assert!(food.is_expired());
    }
}
