use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerupType {
    Speed,
    Shield,
    Bomb,
    Ghost,
    Magnet,
    Grow,
    Shrink,
}

impl PowerupType {
    pub fn random() -> Self {
        match rand::thread_rng().gen_range(0..7) {
            0 => Self::Speed,
            1 => Self::Shield,
            2 => Self::Bomb,
            3 => Self::Ghost,
            4 => Self::Magnet,
            5 => Self::Grow,
            _ => Self::Shrink,
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Self::Speed => "#ffff00",
            Self::Shield => "#00ffff",
            Self::Bomb => "#ff0000",
            Self::Ghost => "#9900ff",
            Self::Magnet => "#ff00ff",
            Self::Grow => "#00ff00",
            Self::Shrink => "#ff8800",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Powerup {
    pub x: i32,
    pub y: i32,
    pub powerup_type: String,
    pub color: String,
    pub expires_at: i64,
}

impl Powerup {
    pub fn new(x: i32, y: i32, powerup_type: PowerupType) -> Self {
        Self {
            x,
            y,
            powerup_type: format!("{:?}", powerup_type).to_uppercase(),
            color: powerup_type.color().to_string(),
            expires_at: Utc::now().timestamp_millis() + 30000,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp_millis() > self.expires_at
    }
}

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
            date: Utc::now().to_rfc3339(),
        }
    }
}
