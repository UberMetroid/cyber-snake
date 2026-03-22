//! Snake type definition and helpers.

use crate::point_direction::{ActiveEffects, Direction, Point};
use serde::{Deserialize, Serialize};

/// A snake entity in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snake {
    #[serde(skip_serializing)]
    #[serde(default)]
    pub id: String,
    pub segments: Vec<Point>,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub dir: Direction,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub next_dir: Direction,
    pub color: String,
    pub name: String,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub is_bot: bool,
    pub score: u32,
    pub alive: bool,
    pub spawned: bool,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub speed: u32,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub frame_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub death_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub held_powerup: Option<String>,
    pub active_effects: ActiveEffects,
    pub super_meter: u32,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub super_mode_start: Option<i64>,
    #[serde(skip_serializing)]
    #[serde(default)]
    pub own_food_count: u32,
}

impl Snake {
    pub fn new(id: String, color: String, name: String, x: i32, y: i32) -> Self {
        Self {
            id,
            segments: vec![Point::new(x, y)],
            dir: Direction::Up,
            next_dir: Direction::Up,
            color,
            name,
            is_bot: false,
            score: 0,
            alive: true,
            spawned: false,
            speed: 2,
            frame_count: 0,
            death_reason: None,
            held_powerup: None,
            active_effects: ActiveEffects::default(),
            super_meter: 0,
            super_mode_start: None,
            own_food_count: 0,
        }
    }

    pub fn head(&self) -> Point {
        self.segments[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_head() {
        let snake = Snake::new(
            "test".to_string(),
            "#ff0000".to_string(),
            "Test".to_string(),
            10,
            20,
        );
        assert_eq!(snake.head(), Point::new(10, 20));
    }
}
