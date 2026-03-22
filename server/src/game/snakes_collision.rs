//! Snake-to-snakes collision detection.

use shared::{Point, Snake};

use crate::game::dominance::{determine_dominance, DominanceContext};

/// Status context for snake collision checks.
pub struct SnakeCollisionStatus {
    pub effective_speed: u32,
    pub shield_active: bool,
    pub super_active: bool,
    pub now: i64,
}

/// Checks if a head position collides with any segment of any other snake.
pub fn check_head_collision(
    head: Point,
    socket_id: &str,
    snakes: &std::collections::HashMap<String, Snake>,
) -> Option<(String, String)> {
    for (other_id, other) in snakes.iter() {
        if other_id == socket_id || !other.alive {
            continue;
        }
        for seg in &other.segments {
            if head.x == seg.x && head.y == seg.y {
                return Some((other_id.clone(), other.name.clone()));
            }
        }
    }
    None
}

/// Checks for collisions with other snakes and determines dominance outcome.
pub fn check_snake_collisions(
    snakes: &std::collections::HashMap<String, Snake>,
    socket_id: &str,
    new_head: Point,
    snake: &Snake,
    status: SnakeCollisionStatus,
) -> (Option<(String, String)>, Option<String>) {
    if snake
        .active_effects
        .ghost
        .map(|t| t > status.now)
        .unwrap_or(false)
    {
        return (None, None);
    }

    if let Some((other_id, other_name)) = check_head_collision(new_head, socket_id, snakes) {
        let other = snakes.get(&other_id).expect("snake exists in map");
        let other_speed_boost = other
            .active_effects
            .speed_boost
            .map(|t| t > status.now)
            .unwrap_or(false);
        let other_slowed = other
            .active_effects
            .slowed
            .map(|t| t > status.now)
            .unwrap_or(false);
        let mut other_speed = if other_speed_boost {
            other.speed + 4
        } else {
            other.speed
        };
        if other_slowed {
            other_speed = (other_speed as f32 * 0.5).max(1.0) as u32;
        }

        let (should_kill, was_killed) = determine_dominance(
            snake,
            other,
            DominanceContext::new(
                status.effective_speed,
                other_speed,
                status.shield_active,
                other
                    .active_effects
                    .shield
                    .map(|t| t > status.now)
                    .unwrap_or(false),
                status.super_active,
                other
                    .active_effects
                    .super_mode
                    .map(|t| t > status.now)
                    .unwrap_or(false),
            ),
        );

        if should_kill {
            return (Some((other_id, other_name)), None);
        } else if was_killed {
            return (None, Some(other_name));
        }
    }

    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{Direction, Point, Snake};

    fn create_test_snake(id: &str, segments: Vec<Point>) -> Snake {
        Snake {
            id: id.to_string(),
            segments,
            dir: Direction::Up,
            next_dir: Direction::Up,
            color: "#ff0000".to_string(),
            name: id.to_string(),
            is_bot: false,
            score: 0,
            alive: true,
            spawned: true,
            speed: 2,
            frame_count: 0,
            death_reason: None,
            held_powerup: None,
            active_effects: shared::ActiveEffects::default(),
            super_meter: 0,
            super_mode_start: None,
            own_food_count: 0,
        }
    }

    #[test]
    fn test_check_head_collision_no_collision() {
        let mut snakes = std::collections::HashMap::new();
        snakes.insert(
            "snake1".to_string(),
            create_test_snake("snake1", vec![Point::new(5, 5), Point::new(5, 6)]),
        );

        let head = Point::new(10, 10);
        let result = check_head_collision(head, "snake1", &snakes);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_head_collision_with_collision() {
        let mut snakes = std::collections::HashMap::new();
        snakes.insert(
            "snake1".to_string(),
            create_test_snake("snake1", vec![Point::new(5, 5), Point::new(5, 6)]),
        );
        snakes.insert(
            "snake2".to_string(),
            create_test_snake("snake2", vec![Point::new(10, 10), Point::new(10, 11)]),
        );

        let head = Point::new(5, 6);
        let result = check_head_collision(head, "snake2", &snakes);
        assert!(result.is_some());
    }

    #[test]
    fn test_check_snake_collisions_ghost_mode_no_collision() {
        let mut snakes = std::collections::HashMap::new();
        let mut snake = create_test_snake("snake1", vec![Point::new(5, 5)]);
        snake.active_effects.ghost = Some(9999999999999);
        snakes.insert("snake1".to_string(), snake);

        let result = check_snake_collisions(
            &snakes,
            "snake1",
            Point::new(10, 10),
            snakes.get("snake1").unwrap(),
            SnakeCollisionStatus {
                effective_speed: 2,
                shield_active: false,
                super_active: false,
                now: 0,
            },
        );

        assert_eq!(result, (None, None));
    }
}
