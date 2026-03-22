//! Speed calculation helpers for snake movement.
//! Contains effective speed calculation considering active effects.

use shared::Snake;

/// Calculates effective speed considering speed boost and slowed effects.
pub fn calculate_effective_speed(snake: &Snake, now: i64) -> u32 {
    let speed_boost = snake
        .active_effects
        .speed_boost
        .map(|t| t > now)
        .unwrap_or(false);
    let slowed = snake
        .active_effects
        .slowed
        .map(|t| t > now)
        .unwrap_or(false);

    let mut speed = if speed_boost {
        snake.speed + 4
    } else {
        snake.speed
    };

    if slowed {
        speed = (speed as f32 * 0.5).max(1.0) as u32;
    }

    speed
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{Direction, Point};

    fn create_test_snake() -> Snake {
        Snake::new(
            "test".to_string(),
            "#ff0000".to_string(),
            "Test".to_string(),
            5,
            5,
        )
    }

    #[test]
    fn test_calculate_effective_speed_no_effects() {
        let snake = create_test_snake();
        let speed = calculate_effective_speed(&snake, 0);
        assert_eq!(speed, 2);
    }

    #[test]
    fn test_calculate_effective_speed_speed_boost() {
        let mut snake = create_test_snake();
        snake.speed = 2;
        let now = 1000;
        snake.active_effects.speed_boost = Some(now + 5000);
        let speed = calculate_effective_speed(&snake, now);
        assert_eq!(speed, 6);
    }

    #[test]
    fn test_calculate_effective_speed_slowed() {
        let mut snake = create_test_snake();
        snake.speed = 4;
        let now = 1000;
        snake.active_effects.slowed = Some(now + 5000);
        let speed = calculate_effective_speed(&snake, now);
        assert_eq!(speed, 2);
    }
}
