//! Point and Direction types.

use serde::{Deserialize, Serialize};

/// A 2D point representing a position on the game grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (self.x, self.y).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (x, y) = <(i32, i32)>::deserialize(deserializer)?;
        Ok(Point { x, y })
    }
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Direction of snake movement.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_point(self) -> Point {
        match self {
            Direction::Up => Point::new(0, -1),
            Direction::Down => Point::new(0, 1),
            Direction::Left => Point::new(-1, 0),
            Direction::Right => Point::new(1, 0),
        }
    }
}

/// Active effects currently applied to a snake.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveEffects {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_boost: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shield: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magnet: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub super_mode: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slowed: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_new() {
        let p = Point::new(5, 10);
        assert_eq!(p.x, 5);
        assert_eq!(p.y, 10);
    }

    #[test]
    fn test_direction_to_point() {
        assert_eq!(Direction::Up.to_point(), Point::new(0, -1));
        assert_eq!(Direction::Down.to_point(), Point::new(0, 1));
        assert_eq!(Direction::Left.to_point(), Point::new(-1, 0));
        assert_eq!(Direction::Right.to_point(), Point::new(1, 0));
    }
}
