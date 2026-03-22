//! Magnet effect implementation.

use shared::{BonusFood, Food, Point, Snake};

const MAGNET_PULL_RADIUS: i32 = 5;

/// Applies magnet effect to pull nearby food toward the snake's head.
pub fn apply_magnet_effect(
    foods: &mut std::collections::HashMap<String, Food>,
    bonus_foods: &mut [BonusFood],
    head: Point,
    snake: &Snake,
    now: i64,
) {
    let magnet_active = snake
        .active_effects
        .magnet
        .map(|t| t > now)
        .unwrap_or(false)
        || snake
            .active_effects
            .super_mode
            .map(|t| t > now)
            .unwrap_or(false);

    if !magnet_active {
        return;
    }

    for food in foods.values_mut() {
        let dx = head.x - food.x;
        let dy = head.y - food.y;
        if dx.abs() + dy.abs() <= MAGNET_PULL_RADIUS && dx.abs() + dy.abs() > 0 {
            if dx.abs() > dy.abs() {
                food.x += dx.signum();
            } else {
                food.y += dy.signum();
            }
        }
    }

    for bf in bonus_foods.iter_mut() {
        let dx = head.x - bf.x;
        let dy = head.y - bf.y;
        if dx.abs() + dy.abs() <= MAGNET_PULL_RADIUS && dx.abs() + dy.abs() > 0 {
            if dx.abs() > dy.abs() {
                bf.x += dx.signum();
            } else {
                bf.y += dy.signum();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_magnet_effect_no_magnet() {
        let mut foods = std::collections::HashMap::new();
        foods.insert(
            "food1".to_string(),
            Food::new("owner".to_string(), "#ff0000".to_string(), 10, 10, false),
        );
        let mut bonus_foods = Vec::new();
        let head = Point::new(5, 5);
        let snake = Snake::new(
            "test".to_string(),
            "#ff0000".to_string(),
            "Test".to_string(),
            5,
            5,
        );
        let now = 0;

        apply_magnet_effect(&mut foods, &mut bonus_foods, head, &snake, now);

        let food = foods.get("food1").unwrap();
        assert_eq!(food.x, 10);
        assert_eq!(food.y, 10);
    }
}
