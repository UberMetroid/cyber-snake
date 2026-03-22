//! Pickup handling for food, bonus food, and powerups.

use crate::config::{
    BONUS_FOOD_POINTS_MULTIPLIER, FOOD_POINTS_MULTIPLIER, MAX_BONUS_FOOD_COUNT,
    SUPER_METER_INCREMENT, SUPER_MODE_DURATION_MS,
};
use crate::game::spawner;
use shared::{BonusFood, Food, Point, Snake};

/// Handles food consumption and scoring.
#[allow(clippy::ptr_arg)]
pub fn handle_food_pickup(
    _socket_id: &str,
    snake: &mut Snake,
    foods: &mut std::collections::HashMap<String, Food>,
    snakes: &std::collections::HashMap<String, Snake>,
    head: Point,
    now: i64,
    shield_active: bool,
) {
    let mut eaten_food_id = None;
    for (food_id, food) in foods.iter() {
        if head.x == food.x && head.y == food.y {
            eaten_food_id = Some(food_id.clone());
            break;
        }
    }

    if let Some(fid) = eaten_food_id {
        if shield_active {
            tracing::info!("[SHIELD] {} deflected food", snake.color);
        } else {
            let food = match foods.remove(&fid) {
                Some(f) => f,
                None => return,
            };
            let is_own_food = food.color == snake.color;

            if food.is_ring {
                tracing::info!("[FOOD] {} ate a ring (+length)", snake.color);
            } else {
                snake.speed = (snake.speed + 1).min(4);
                tracing::info!("[FOOD] {} ate a circle (+speed)", snake.color);
            }

            if is_own_food {
                if food.is_super || snake.super_meter >= 100 {
                    tracing::info!(
                        "[SUPER] {} {} activated SUPER MODE!",
                        snake.color,
                        snake.name
                    );
                    snake.active_effects.super_mode = Some(now + SUPER_MODE_DURATION_MS);
                    snake.active_effects.speed_boost = Some(now + SUPER_MODE_DURATION_MS);
                    snake.active_effects.shield = Some(now + SUPER_MODE_DURATION_MS);
                    snake.active_effects.magnet = Some(now + SUPER_MODE_DURATION_MS);
                    snake.super_meter = 100;
                    snake.super_mode_start = Some(now);
                    snake.own_food_count = 0;
                } else {
                    snake.super_meter = (snake.super_meter + SUPER_METER_INCREMENT).min(100);
                    snake.own_food_count += 1;
                    tracing::info!(
                        "[FOOD] {} ate OWN food (meter: {}%)",
                        snake.color,
                        snake.super_meter
                    );
                }
            } else {
                let points = FOOD_POINTS_MULTIPLIER * snake.segments.len() as u32;
                snake.score += points;
                tracing::info!("[FOOD] {} ate ENEMY food (+{} points)", snake.color, points);
            }

            let owner_color = snakes
                .get(&fid)
                .map(|s| s.color.clone())
                .unwrap_or(food.color.clone());
            let new_food_super = is_own_food && snake.own_food_count >= 5;

            if snakes.contains_key(&fid) {
                if let Some(new_food) =
                    spawner::spawn_food(&fid, &owner_color, new_food_super, snakes, foods, &[], &[])
                {
                    foods.insert(fid, new_food);
                }
            }
        }
    }
}

/// Handles bonus food consumption.
pub fn handle_bonus_food_pickup(snake: &mut Snake, bonus_foods: &mut Vec<BonusFood>, head: Point) {
    let mut remove_idx = None;
    for (i, bf) in bonus_foods.iter().enumerate() {
        if head.x == bf.x && head.y == bf.y {
            let points = BONUS_FOOD_POINTS_MULTIPLIER * snake.segments.len() as u32;
            snake.score += points;
            snake.super_meter = (snake.super_meter + SUPER_METER_INCREMENT / 2).min(100);
            tracing::info!(
                "[BONUS] {} ate BONUS food (+{} points)",
                snake.color,
                points
            );
            remove_idx = Some(i);
            break;
        }
    }

    if let Some(i) = remove_idx {
        bonus_foods.remove(i);
        if bonus_foods.len() < MAX_BONUS_FOOD_COUNT {
            spawner::spawn_bonus_food(
                &std::collections::HashMap::new(),
                &std::collections::HashMap::new(),
                &[],
                bonus_foods,
            );
        }
    }
}

/// Handles powerup pickup.
pub fn handle_powerup_pickup(
    snake: &mut Snake,
    powerups: &mut Vec<shared::Powerup>,
    head: Point,
    shield_active: bool,
) {
    let mut remove_idx = None;
    for (i, pu) in powerups.iter().enumerate() {
        if head.x == pu.x && head.y == pu.y {
            if shield_active {
                tracing::info!("[SHIELD] {} deflected powerup", snake.color);
            } else if snake.held_powerup.is_none() {
                snake.held_powerup = Some(pu.powerup_type.clone());
                tracing::info!("[POWERUP] {} picked up {}", snake.color, pu.powerup_type);
                remove_idx = Some(i);
            }
            break;
        }
    }

    if let Some(i) = remove_idx {
        powerups.remove(i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_bonus_food_pickup_no_collision() {
        let mut snake = Snake::new(
            "test".to_string(),
            "#ff0000".to_string(),
            "Test".to_string(),
            5,
            5,
        );
        let mut bonus_foods = Vec::new();
        bonus_foods.push(BonusFood::new(10, 10, "#ffffff".to_string(), false));
        let head = Point::new(5, 5);

        let initial_score = snake.score;
        handle_bonus_food_pickup(&mut snake, &mut bonus_foods, head);

        assert_eq!(snake.score, initial_score);
        assert_eq!(bonus_foods.len(), 1);
    }

    #[test]
    fn test_handle_powerup_pickup_no_collision() {
        let mut snake = Snake::new(
            "test".to_string(),
            "#ff0000".to_string(),
            "Test".to_string(),
            5,
            5,
        );
        let mut powerups = Vec::new();
        powerups.push(shared::Powerup::new(10, 10, shared::PowerupType::Speed));
        let head = Point::new(5, 5);

        handle_powerup_pickup(&mut snake, &mut powerups, head, false);

        assert!(snake.held_powerup.is_none());
        assert_eq!(powerups.len(), 1);
    }

    #[test]
    fn test_handle_powerup_pickup_with_shield_deflects() {
        let mut snake = Snake::new(
            "test".to_string(),
            "#ff0000".to_string(),
            "Test".to_string(),
            5,
            5,
        );
        let mut powerups = Vec::new();
        powerups.push(shared::Powerup::new(5, 5, shared::PowerupType::Speed));
        let head = Point::new(5, 5);

        handle_powerup_pickup(&mut snake, &mut powerups, head, true);

        assert!(snake.held_powerup.is_none());
        assert_eq!(powerups.len(), 1);
    }
}
