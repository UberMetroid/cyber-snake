//! Snake collision detection helpers.
//! Re-exports from dominance.rs and snakes_collision.rs.

use shared::Point;

#[allow(unused_imports)]
pub use crate::game::dominance::{determine_dominance, DominanceContext};
#[allow(unused_imports)]
pub use crate::game::snakes_collision::{
    check_head_collision, check_snake_collisions, SnakeCollisionStatus,
};
#[allow(unused_imports)]
pub use crate::game::wall::{check_wall_collision, get_shield_safe_direction};

/// Checks if a position collides with the snake's own body.
pub fn check_self_collision(head: Point, segments: &[Point]) -> bool {
    for seg in segments.iter().skip(1) {
        if head.x == seg.x && head.y == seg.y {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::Point;

    #[test]
    fn test_check_self_collision_no_collision() {
        let head = Point::new(5, 5);
        let segments = vec![Point::new(5, 5), Point::new(5, 6), Point::new(5, 7)];
        assert!(!check_self_collision(head, &segments));
    }

    #[test]
    fn test_check_self_collision_with_collision() {
        let head = Point::new(5, 6);
        let segments = vec![Point::new(5, 5), Point::new(5, 6), Point::new(5, 7)];
        assert!(check_self_collision(head, &segments));
    }
}
