use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};

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
            expires_at: Utc::now().timestamp_millis() + 60000,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp_millis() > self.expires_at
    }
}

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
            expires_at: Utc::now().timestamp_millis() + 60000,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp_millis() > self.expires_at
    }
}

pub const BONUS_COLORS: [&str; 6] = [
    "#ffd700", "#ffffff", "#ff69b4", "#00ff99", "#ff6600", "#99ff00",
];

pub fn get_bonus_color(used_colors: &[String]) -> String {
    let available: Vec<&str> = BONUS_COLORS
        .iter()
        .copied()
        .filter(|c| !used_colors.contains(&c.to_string()))
        .collect();

    if available.is_empty() {
        BONUS_COLORS[rand::thread_rng().gen_range(0..BONUS_COLORS.len())].to_string()
    } else {
        available[rand::thread_rng().gen_range(0..available.len())].to_string()
    }
}
