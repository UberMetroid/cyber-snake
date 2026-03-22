//! Bot AI pathfinding and decision making.
//! Bots find nearest targets, avoid hazards, and select optimal directions.

use crate::config::CONFIG;
use rand::Rng;
use shared::{Direction, Point, Snake};

/// Bot configuration constants
const BOT_POWERUP_ACTIVATE_CHANCE: f64 = 0.02;
const BOT_TARGET_FOOD_WEIGHT_DIVISOR: i32 = 2;

/// Represents the computed direction decision for a bot.
pub struct BotDecision {
    pub next_direction: Direction,
    pub should_activate_powerup: bool,
}

/// Computes the next direction for a bot based on nearest targets and hazard avoidance.
pub fn compute_bot_direction(
    bot: &Snake,
    snakes: &std::collections::HashMap<String, Snake>,
    foods: &std::collections::HashMap<String, shared::Food>,
    bonus_foods: &[shared::BonusFood],
    powerups: &[shared::Powerup],
) -> BotDecision {
    let head = bot.head();
    let current_dir = bot.dir;
    let has_powerup = bot.held_powerup.is_some();

    // Determine if bot should activate its powerup (2% chance per tick)
    let should_activate = if has_powerup {
        let mut rng = rand::thread_rng();
        rng.gen_bool(BOT_POWERUP_ACTIVATE_CHANCE)
    } else {
        false
    };

    // Find nearest target (food, bonus food, or powerup)
    let nearest_target = find_nearest_target(head, &bot.color, foods, bonus_foods, powerups);

    // Get valid directions (not leading into walls or other snakes)
    let valid_dirs = get_valid_directions(head, current_dir, snakes);

    // Select best direction
    let best_dir = select_best_direction(head, current_dir, valid_dirs, nearest_target);

    BotDecision {
        next_direction: best_dir,
        should_activate_powerup: should_activate,
    }
}

/// Finds the nearest target (food, bonus food, or powerup) for a bot.
fn find_nearest_target(
    head: Point,
    bot_color: &str,
    foods: &std::collections::HashMap<String, shared::Food>,
    bonus_foods: &[shared::BonusFood],
    powerups: &[shared::Powerup],
) -> Option<Point> {
    let mut nearest_target = None;
    let mut min_dist = i32::MAX;

    // Check regular foods
    for food in foods.values() {
        let dist = (head.x - food.x).abs() + (head.y - food.y).abs();
        let is_own = food.color == bot_color;
        // Own food is weighted closer (bot prioritizes its own)
        let weighted_dist = if is_own {
            dist / BOT_TARGET_FOOD_WEIGHT_DIVISOR
        } else {
            dist
        };
        if weighted_dist < min_dist {
            min_dist = weighted_dist;
            nearest_target = Some(Point::new(food.x, food.y));
        }
    }

    // Check bonus foods
    for bf in bonus_foods {
        let dist = (head.x - bf.x).abs() + (head.y - bf.y).abs();
        if dist < min_dist {
            min_dist = dist;
            nearest_target = Some(Point::new(bf.x, bf.y));
        }
    }

    // Check powerups
    for pu in powerups {
        let dist = (head.x - pu.x).abs() + (head.y - pu.y).abs();
        if dist < min_dist {
            min_dist = dist;
            nearest_target = Some(Point::new(pu.x, pu.y));
        }
    }

    nearest_target
}

/// Returns directions that won't lead into walls or snake bodies.
fn get_valid_directions(
    head: Point,
    current_dir: Direction,
    snakes: &std::collections::HashMap<String, Snake>,
) -> Vec<Direction> {
    let possible_dirs = match current_dir {
        Direction::Up => vec![Direction::Up, Direction::Left, Direction::Right],
        Direction::Down => vec![Direction::Down, Direction::Left, Direction::Right],
        Direction::Left => vec![Direction::Left, Direction::Up, Direction::Down],
        Direction::Right => vec![Direction::Right, Direction::Up, Direction::Down],
    };

    let mut valid_dirs = Vec::new();

    for dir in possible_dirs {
        let dir_point = dir.to_point();
        let next_x = head.x + dir_point.x;
        let next_y = head.y + dir_point.y;

        let mut is_hazard = false;

        // Check wall collision
        if next_x < 0 || next_x >= CONFIG.cols as i32 || next_y < 0 || next_y >= CONFIG.rows as i32
        {
            is_hazard = true;
        } else {
            // Check collision with all snake segments (including dead snakes)
            for other in snakes.values() {
                for seg in &other.segments {
                    if seg.x == next_x && seg.y == next_y {
                        is_hazard = true;
                        break;
                    }
                }
                if is_hazard {
                    break;
                }
            }
        }

        if !is_hazard {
            valid_dirs.push(dir);
        }
    }

    valid_dirs
}

/// Selects the best direction to move toward the target or escape hazard.
fn select_best_direction(
    head: Point,
    current_dir: Direction,
    valid_dirs: Vec<Direction>,
    target: Option<Point>,
) -> Direction {
    let mut best_dir = current_dir;

    // Default to any safe turn if current path is blocked
    if !valid_dirs.contains(&current_dir) && !valid_dirs.is_empty() {
        best_dir = valid_dirs[0];
    }

    // Pathfind to target if possible and safe
    if !valid_dirs.is_empty() {
        if let Some(target_pos) = target {
            let mut best_dist = i32::MAX;
            for dir in valid_dirs.iter() {
                let dp = dir.to_point();
                let nx = head.x + dp.x;
                let ny = head.y + dp.y;
                let dist = (nx - target_pos.x).abs() + (ny - target_pos.y).abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_dir = *dir;
                }
            }
        } else if !valid_dirs.contains(&current_dir) {
            // Random valid turn if no target
            let mut rng = rand::thread_rng();
            best_dir = valid_dirs[rng.gen_range(0..valid_dirs.len())];
        }
    }

    best_dir
}
