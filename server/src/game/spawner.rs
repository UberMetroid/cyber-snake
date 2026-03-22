//! Entity spawning logic for food, bonus food, and powerups.
//! Handles random position selection and occupancy checks.

use crate::config::CONFIG;
use rand::Rng;
use shared::{BonusFood, Food, Powerup, PowerupType};

/// Checks if a position is occupied by any entity (snake, food, bonus food, powerup).
fn is_occupied(
    x: i32,
    y: i32,
    snakes: &std::collections::HashMap<String, shared::Snake>,
    foods: &std::collections::HashMap<String, Food>,
    bonus_foods: &[BonusFood],
    powerups: &[Powerup],
) -> bool {
    for snake in snakes.values() {
        for seg in &snake.segments {
            if seg.x == x && seg.y == y {
                return true;
            }
        }
    }
    for food in foods.values() {
        if food.x == x && food.y == y {
            return true;
        }
    }
    for bf in bonus_foods {
        if bf.x == x && bf.y == y {
            return true;
        }
    }
    for pu in powerups {
        if pu.x == x && pu.y == y {
            return true;
        }
    }
    false
}

/// Spawns food at a random unoccupied position for the given owner.
pub fn spawn_food(
    owner_id: &str,
    color: &str,
    is_super: bool,
    snakes: &std::collections::HashMap<String, shared::Snake>,
    foods: &std::collections::HashMap<String, Food>,
    bonus_foods: &[BonusFood],
    powerups: &[Powerup],
) -> Option<Food> {
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let x = rng.gen_range(0..CONFIG.cols as i32);
        let y = rng.gen_range(0..CONFIG.rows as i32);
        if !is_occupied(x, y, snakes, foods, bonus_foods, powerups) {
            return Some(Food::new(
                owner_id.to_string(),
                color.to_string(),
                x,
                y,
                is_super,
            ));
        }
    }
    None
}

/// Spawns a bonus food at a random unoccupied position.
pub fn spawn_bonus_food(
    snakes: &std::collections::HashMap<String, shared::Snake>,
    foods: &std::collections::HashMap<String, Food>,
    powerups: &[Powerup],
    bonus_foods_list: &mut Vec<BonusFood>,
) {
    if bonus_foods_list.len() >= 3 {
        return;
    }

    let used_colors: Vec<String> = snakes.values().map(|s| s.color.clone()).collect();
    let color = crate::game::get_bonus_color(&used_colors);
    let is_ring = rand::thread_rng().gen_bool(0.5);
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let x = rng.gen_range(0..CONFIG.cols as i32);
        let y = rng.gen_range(0..CONFIG.rows as i32);
        if !is_occupied(x, y, snakes, foods, bonus_foods_list, powerups) {
            bonus_foods_list.push(BonusFood::new(x, y, color, is_ring));
            tracing::info!("[BONUS] spawned at {},{}", x, y);
            return;
        }
    }
}

/// Spawns a random powerup at a random unoccupied position.
pub fn spawn_powerup(
    snakes: &std::collections::HashMap<String, shared::Snake>,
    foods: &std::collections::HashMap<String, Food>,
    bonus_foods: &[BonusFood],
    powerups_list: &mut Vec<Powerup>,
) {
    if powerups_list.len() >= 3 {
        return;
    }

    let powerup_type = PowerupType::random();
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let x = rng.gen_range(0..CONFIG.cols as i32);
        let y = rng.gen_range(0..CONFIG.rows as i32);
        if !is_occupied(x, y, snakes, foods, bonus_foods, powerups_list) {
            powerups_list.push(Powerup::new(x, y, powerup_type));
            tracing::info!("[POWERUP] {} spawned at {},{}", powerup_type.color(), x, y);
            return;
        }
    }
}
