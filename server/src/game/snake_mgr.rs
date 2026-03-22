//! Snake management operations: create, spawn, respawn, remove.
//! Handles snake lifecycle and bonus food drops on death.

use crate::config::CONFIG;
use rand::Rng;
use shared::{BonusFood, Direction, Food, Snake};

/// Creates a new snake with a unique color and random name at a random position.
pub fn create_snake(snakes: &std::collections::HashMap<String, Snake>, socket_id: String) -> Snake {
    let used_colors: Vec<String> = snakes.values().map(|s| s.color.clone()).collect();
    let color = crate::game::get_next_color(&used_colors);
    let name = crate::game::random_name();

    let mut rng = rand::thread_rng();
    let cols = CONFIG.cols as i32;
    let rows = CONFIG.rows as i32;
    let x = rng.gen_range(5..(cols - 5).max(6));
    let y = rng.gen_range(5..(rows - 5).max(6));
    let dirs = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];
    let dir = dirs[rng.gen_range(0..4)];

    let mut snake = Snake::new(socket_id, color, name, x, y);
    snake.dir = dir;
    snake.next_dir = dir;
    snake
}

/// Spawns a snake at its starting position and creates its initial food.
pub fn spawn_snake(
    snakes: &mut std::collections::HashMap<String, Snake>,
    foods: &mut std::collections::HashMap<String, Food>,
    socket_id: &str,
) {
    if let Some(snake) = snakes.get_mut(socket_id) {
        if !snake.spawned {
            snake.spawned = true;
            let food = Food::new(
                socket_id.to_string(),
                snake.color.clone(),
                snake.head().x,
                snake.head().y,
                false,
            );
            foods.insert(socket_id.to_string(), food);
            tracing::info!("[SPAWN] {} {} SPAWNED", snake.color, snake.name);
        }
    }
}

/// Respawns a dead snake with its original color and name.
pub fn respawn_snake(
    snakes: &mut std::collections::HashMap<String, Snake>,
    foods: &mut std::collections::HashMap<String, Food>,
    socket_id: &str,
) {
    let (old_color, old_name) = if let Some(old_snake) = snakes.get(socket_id) {
        (old_snake.color.clone(), old_snake.name.clone())
    } else {
        return;
    };

    let mut new_snake = create_snake(snakes, socket_id.to_string());
    new_snake.color = old_color;
    new_snake.name = old_name;
    new_snake.score = 0;
    new_snake.spawned = true;

    snakes.insert(socket_id.to_string(), new_snake.clone());
    let food = Food::new(
        socket_id.to_string(),
        new_snake.color.clone(),
        new_snake.head().x,
        new_snake.head().y,
        false,
    );
    foods.insert(socket_id.to_string(), food);
    tracing::info!("[RESPAWN] {} {} respawned", new_snake.color, new_snake.name);
}

/// Removes a snake and drops bonus food from its segments.
pub fn remove_snake(
    snakes: &mut std::collections::HashMap<String, Snake>,
    foods: &mut std::collections::HashMap<String, Food>,
    bonus_foods: &mut Vec<BonusFood>,
    powerups: &mut Vec<shared::Powerup>,
    socket_id: &str,
) {
    if let Some(snake) = snakes.get(socket_id) {
        if snake.alive && snake.spawned {
            let segments = snake.segments.clone();
            drop_powerup(snakes, powerups, socket_id);
            for seg in segments.iter().skip(1).step_by(2) {
                if bonus_foods.len() < 10 {
                    bonus_foods.push(BonusFood::new(seg.x, seg.y, "#ffffff".to_string(), false));
                }
            }
        }
    }
    snakes.remove(socket_id);
    foods.remove(socket_id);
}

/// Drops a random powerup at the snake's head position.
fn drop_powerup(
    snakes: &std::collections::HashMap<String, Snake>,
    powerups: &mut Vec<shared::Powerup>,
    socket_id: &str,
) {
    if powerups.len() >= 3 {
        return;
    }
    if let Some(snake) = snakes.get(socket_id) {
        let powerup_type = shared::PowerupType::random();
        powerups.push(shared::Powerup::new(
            snake.head().x,
            snake.head().y,
            powerup_type,
        ));
        tracing::info!(
            "[POWERUP] {} dropped at {},{}",
            powerup_type.color(),
            snake.head().x,
            snake.head().y
        );
    }
}

/// Removes bots that have died.
pub fn remove_dead_bots(
    snakes: &mut std::collections::HashMap<String, Snake>,
    foods: &mut std::collections::HashMap<String, Food>,
    bonus_foods: &mut Vec<BonusFood>,
    powerups: &mut Vec<shared::Powerup>,
) {
    let mut dead_bots = Vec::new();
    for (id, snake) in snakes.iter() {
        if snake.is_bot && !snake.alive {
            dead_bots.push(id.clone());
        }
    }
    for id in dead_bots {
        remove_snake(snakes, foods, bonus_foods, powerups, &id);
    }
}

/// Spawns new bots if below the minimum count and server has capacity.
pub fn spawn_bots_if_needed(
    snakes: &mut std::collections::HashMap<String, Snake>,
    foods: &mut std::collections::HashMap<String, Food>,
) {
    use crate::config::MIN_BOT_COUNT;

    let active_bots = snakes.values().filter(|s| s.is_bot && s.alive).count();
    let total_players = snakes.len();

    if active_bots < MIN_BOT_COUNT && total_players < CONFIG.max_players {
        let bots_to_spawn = (MIN_BOT_COUNT - active_bots).min(CONFIG.max_players - total_players);
        for _ in 0..bots_to_spawn {
            let id = format!("bot_{}", uuid::Uuid::new_v4());
            let mut bot = create_snake(snakes, id.clone());
            bot.is_bot = true;
            snakes.insert(id.clone(), bot);
            spawn_snake(snakes, foods, &id);
        }
    }
}
