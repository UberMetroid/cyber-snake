//! Wall collision detection helpers.

use crate::config::CONFIG;
use shared::Direction;

/// Checks if a position collides with walls.
pub fn check_wall_collision(x: i32, y: i32) -> bool {
    x < 0 || x >= CONFIG.cols as i32 || y < 0 || y >= CONFIG.rows as i32
}

/// Calculates the safe direction when hitting a wall with shield active.
pub fn get_shield_safe_direction(x: i32, y: i32, cols: i32, _rows: i32) -> Direction {
    if x < 0 {
        Direction::Right
    } else if x >= cols {
        Direction::Left
    } else if y < 0 {
        Direction::Down
    } else {
        Direction::Up
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_shield_safe_direction_left_wall() {
        let dir = get_shield_safe_direction(-1, 50, 100, 100);
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn test_get_shield_safe_direction_right_wall() {
        let dir = get_shield_safe_direction(100, 50, 100, 100);
        assert_eq!(dir, Direction::Left);
    }

    #[test]
    fn test_get_shield_safe_direction_top_wall() {
        let dir = get_shield_safe_direction(50, -1, 100, 100);
        assert_eq!(dir, Direction::Down);
    }

    #[test]
    fn test_get_shield_safe_direction_bottom_wall() {
        let dir = get_shield_safe_direction(50, 100, 100, 100);
        assert_eq!(dir, Direction::Up);
    }
}
