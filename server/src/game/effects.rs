//! Powerup effect activation and application.
//! Handles all 6 powerup types: Speed, Shield, Bomb, Ghost, Magnet, Grow.

use crate::config::{
    BOMB_EXPLOSION_DURATION_MS, BOMB_RADIUS, GHOST_DURATION_MS, GROW_BONUS_SCORE, GROW_SEGMENTS,
    MAGNET_DURATION_MS, SHIELD_DURATION_MS, SPEED_BOOST_DURATION_MS,
};
use chrono::Utc;
use shared::{Explosion, Snake};

/// Applies a timed effect to a snake.
fn apply_timed_effect(effect: &mut Option<i64>, duration_ms: i64, now: i64) {
    *effect = Some(now + duration_ms);
}

/// Activates a held powerup for the given snake.
pub fn activate_powerup(
    snakes: &mut std::collections::HashMap<String, Snake>,
    powerups: &mut Vec<shared::Powerup>,
    explosions: &mut Vec<Explosion>,
    socket_id: &str,
) {
    let now = Utc::now().timestamp_millis();
    let (head, powerup_type) = {
        let snake = match snakes.get_mut(socket_id) {
            Some(s) if s.alive && s.spawned => s,
            _ => return,
        };
        let p_type = match snake.held_powerup.take() {
            Some(p) => p,
            None => return,
        };
        (snake.head(), p_type)
    };

    tracing::info!("[POWERUP] Snake {} activated {}", socket_id, powerup_type);

    match powerup_type.as_str() {
        "SPEED" => {
            if let Some(snake) = snakes.get_mut(socket_id) {
                apply_timed_effect(
                    &mut snake.active_effects.speed_boost,
                    SPEED_BOOST_DURATION_MS,
                    now,
                );
            }
        }
        "SHIELD" => {
            if let Some(snake) = snakes.get_mut(socket_id) {
                apply_timed_effect(&mut snake.active_effects.shield, SHIELD_DURATION_MS, now);
            }
        }
        "GHOST" => {
            if let Some(snake) = snakes.get_mut(socket_id) {
                apply_timed_effect(&mut snake.active_effects.ghost, GHOST_DURATION_MS, now);
            }
        }
        "MAGNET" => {
            if let Some(snake) = snakes.get_mut(socket_id) {
                apply_timed_effect(&mut snake.active_effects.magnet, MAGNET_DURATION_MS, now);
            }
        }
        "GROW" => {
            if let Some(snake) = snakes.get_mut(socket_id) {
                if let Some(&last) = snake.segments.last() {
                    for _ in 0..GROW_SEGMENTS {
                        snake.segments.push(last);
                    }
                }
                snake.score += GROW_BONUS_SCORE;
            }
        }
        "BOMB" => {
            let mut to_kill = Vec::new();
            let killer_name = snakes
                .get(socket_id)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| "UNKNOWN".to_string());
            for (other_id, other_snake) in snakes.iter() {
                if other_id != socket_id && other_snake.alive && other_snake.spawned {
                    let other_head = other_snake.head();
                    let dist = (head.x - other_head.x).abs() + (head.y - other_head.y).abs();
                    if dist <= BOMB_RADIUS {
                        to_kill.push(other_id.clone());
                    }
                }
            }
            for id in &to_kill {
                kill_snake(snakes, id, &format!("BOMBED BY {}", killer_name));
                drop_powerup_on_death(snakes, powerups, id);
            }
            explosions.push(Explosion {
                x: head.x,
                y: head.y,
                radius: BOMB_RADIUS,
                color: "#ff0000".to_string(),
                expires_at: now + BOMB_EXPLOSION_DURATION_MS,
            });
        }
        _ => {}
    }
}

/// Sets a snake to dead status with a reason.
pub fn kill_snake(
    snakes: &mut std::collections::HashMap<String, Snake>,
    socket_id: &str,
    reason: &str,
) {
    if let Some(snake) = snakes.get_mut(socket_id) {
        snake.alive = false;
        snake.death_reason = Some(reason.to_string());
        tracing::info!("[DEATH] {} {} died: {}", snake.color, snake.name, reason);
    }
}

/// Drops a random powerup at the snake's head position when it dies.
pub fn drop_powerup_on_death(
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snake(id: &str) -> Snake {
        Snake::new(
            id.to_string(),
            "#ff0000".to_string(),
            "Test".to_string(),
            5,
            5,
        )
    }

    #[test]
    fn test_kill_snake() {
        let mut snakes = std::collections::HashMap::new();
        let mut snake = make_snake("test");
        snake.alive = true;
        snakes.insert("test".to_string(), snake);
        kill_snake(&mut snakes, "test", "TEST_KILL");
        let snake = snakes.get("test").unwrap();
        assert!(!snake.alive);
        assert_eq!(snake.death_reason, Some("TEST_KILL".to_string()));
    }

    #[test]
    fn test_activate_powerup_no_held_powerup() {
        let mut snakes = std::collections::HashMap::new();
        snakes.insert("test".to_string(), make_snake("test"));
        let (powerups, explosions) = (Vec::new(), Vec::new());
        activate_powerup(
            &mut snakes,
            &mut powerups.clone(),
            &mut explosions.clone(),
            "test",
        );
        assert!(explosions.is_empty());
    }

    #[test]
    fn test_activate_powerup_speed() {
        let mut snakes = std::collections::HashMap::new();
        let mut snake = make_snake("test");
        snake.held_powerup = Some("SPEED".to_string());
        snake.alive = true;
        snake.spawned = true;
        snakes.insert("test".to_string(), snake);
        let (mut powerups, mut explosions) = (Vec::new(), Vec::new());
        activate_powerup(&mut snakes, &mut powerups, &mut explosions, "test");
        assert!(snakes
            .get("test")
            .unwrap()
            .active_effects
            .speed_boost
            .is_some());
    }

    #[test]
    fn test_activate_powerup_shield() {
        let mut snakes = std::collections::HashMap::new();
        let mut snake = make_snake("test");
        snake.held_powerup = Some("SHIELD".to_string());
        snake.alive = true;
        snake.spawned = true;
        snakes.insert("test".to_string(), snake);
        let (mut powerups, mut explosions) = (Vec::new(), Vec::new());
        activate_powerup(&mut snakes, &mut powerups, &mut explosions, "test");
        assert!(snakes.get("test").unwrap().active_effects.shield.is_some());
    }

    #[test]
    fn test_activate_powerup_bomb() {
        let mut snakes = std::collections::HashMap::new();
        let mut snake1 = make_snake("snake1");
        snake1.held_powerup = Some("BOMB".to_string());
        snake1.alive = true;
        snake1.spawned = true;
        snakes.insert("snake1".to_string(), snake1);
        let mut snake2 = make_snake("snake2");
        snake2.alive = true;
        snake2.spawned = true;
        snakes.insert("snake2".to_string(), snake2);
        let (mut powerups, mut explosions) = (Vec::new(), Vec::new());
        activate_powerup(&mut snakes, &mut powerups, &mut explosions, "snake1");
        assert!(!snakes.get("snake2").unwrap().alive);
        assert!(!explosions.is_empty());
    }

    #[test]
    fn test_drop_powerup_on_death_respects_max() {
        let snakes: std::collections::HashMap<String, Snake> = std::collections::HashMap::new();
        let mut powerups = vec![
            shared::Powerup::new(0, 0, shared::PowerupType::Speed),
            shared::Powerup::new(1, 1, shared::PowerupType::Shield),
            shared::Powerup::new(2, 2, shared::PowerupType::Bomb),
        ];
        drop_powerup_on_death(&snakes, &mut powerups, "any_id");
        assert_eq!(powerups.len(), 3);
    }
}
