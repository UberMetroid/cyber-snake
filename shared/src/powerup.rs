//! Powerup and explosion types.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Type of powerup that can be collected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerupType {
    Speed,
    Shield,
    Bomb,
    Ghost,
    Magnet,
    Grow,
}

impl PowerupType {
    pub fn random() -> Self {
        match rand::thread_rng().gen_range(0..6) {
            0 => Self::Speed,
            1 => Self::Shield,
            2 => Self::Bomb,
            3 => Self::Ghost,
            4 => Self::Magnet,
            _ => Self::Grow,
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Speed => "#ffff00",
            Self::Shield => "#00ffff",
            Self::Bomb => "#ff0000",
            Self::Ghost => "#9900ff",
            Self::Magnet => "#ff00ff",
            Self::Grow => "#00ff00",
        }
    }
}

/// A powerup item on the game grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Powerup {
    pub x: i32,
    pub y: i32,
    pub powerup_type: String,
    pub color: String,
    pub expires_at: i64,
}

impl Powerup {
    pub fn new(x: i32, y: i32, powerup_type: PowerupType) -> Self {
        let type_str = match powerup_type {
            PowerupType::Speed => "SPEED",
            PowerupType::Shield => "SHIELD",
            PowerupType::Bomb => "BOMB",
            PowerupType::Ghost => "GHOST",
            PowerupType::Magnet => "MAGNET",
            PowerupType::Grow => "GROW",
        };
        Self {
            x,
            y,
            powerup_type: type_str.to_string(),
            color: powerup_type.color().to_string(),
            expires_at: chrono::Utc::now().timestamp_millis() + 30000,
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp_millis() > self.expires_at
    }
}

/// Explosion effect from bomb powerup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Explosion {
    pub x: i32,
    pub y: i32,
    pub radius: i32,
    pub color: String,
    pub expires_at: i64,
}

/// High score entry for leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighScore {
    pub name: String,
    pub score: u32,
    pub date: String,
}

impl HighScore {
    pub fn new(name: String, score: u32) -> Self {
        Self {
            name,
            score,
            date: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powerup_type_colors() {
        assert_eq!(PowerupType::Speed.color(), "#ffff00");
        assert_eq!(PowerupType::Shield.color(), "#00ffff");
        assert_eq!(PowerupType::Bomb.color(), "#ff0000");
        assert_eq!(PowerupType::Ghost.color(), "#9900ff");
        assert_eq!(PowerupType::Magnet.color(), "#ff00ff");
        assert_eq!(PowerupType::Grow.color(), "#00ff00");
    }

    #[test]
    fn test_high_score_new() {
        let hs = HighScore::new("Player".to_string(), 1000);
        assert_eq!(hs.name, "Player");
        assert_eq!(hs.score, 1000);
        assert!(!hs.date.is_empty());
    }
}
