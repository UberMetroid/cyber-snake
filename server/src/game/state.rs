//! Core game state management.
//! Contains GameState struct, tick loop, and snake update logic.

use crate::config::CONFIG;
use crate::game::broadcast::GameBroadcast;
use crate::game::handlers;
use crate::game::{collision, effects, highscores, snake_mgr, tick};
use chrono::Utc;
use shared::{Food, Point, Snake};

/// Main game state container holding all entities and game tick counter.
pub struct GameState {
    pub snakes: std::collections::HashMap<String, Snake>,
    pub foods: std::collections::HashMap<String, Food>,
    pub bonus_foods: Vec<shared::BonusFood>,
    pub powerups: Vec<shared::Powerup>,
    pub explosions: Vec<shared::Explosion>,
    pub high_scores: Vec<shared::HighScore>,
    pub tick: u64,
    pub start_time: i64,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        std::fs::create_dir_all("data")
            .unwrap_or_else(|e| tracing::warn!("Failed to create data dir: {}", e));
        let high_scores = highscores::load_highscores();

        Self {
            snakes: std::collections::HashMap::new(),
            foods: std::collections::HashMap::new(),
            bonus_foods: Vec::new(),
            powerups: Vec::new(),
            explosions: Vec::new(),
            high_scores,
            tick: 0,
            start_time: Utc::now().timestamp_millis(),
        }
    }

    pub fn create_snake(&mut self, socket_id: String) -> Snake {
        snake_mgr::create_snake(&self.snakes, socket_id)
    }

    pub fn spawn_snake(&mut self, socket_id: &str) {
        snake_mgr::spawn_snake(&mut self.snakes, &mut self.foods, socket_id);
    }

    pub fn respawn_snake(&mut self, socket_id: &str) {
        snake_mgr::respawn_snake(&mut self.snakes, &mut self.foods, socket_id);
    }

    pub fn remove_snake(&mut self, socket_id: &str) {
        snake_mgr::remove_snake(
            &mut self.snakes,
            &mut self.foods,
            &mut self.bonus_foods,
            &mut self.powerups,
            socket_id,
        );
    }

    pub fn activate_powerup(&mut self, socket_id: &str) {
        effects::activate_powerup(
            &mut self.snakes,
            &mut self.powerups,
            &mut self.explosions,
            socket_id,
        );
    }

    pub fn tick(&mut self) {
        tick::tick(self);
    }

    pub fn update_snake(&mut self, socket_id: &str, now: i64) {
        let mut snake = match self.snakes.remove(socket_id) {
            Some(s) => s,
            None => return,
        };

        snake.frame_count += 1;
        let effective_speed = handlers::calculate_effective_speed(&snake, now);

        let move_interval = (5 - effective_speed as i32).max(1) as u32;
        if snake.frame_count % move_interval != 0 {
            self.snakes.insert(socket_id.to_string(), snake);
            return;
        }

        snake.dir = snake.next_dir;
        let head = snake.head();
        let dir_point = snake.dir.to_point();
        let mut new_head = Point::new(head.x + dir_point.x, head.y + dir_point.y);

        let ghost_active = snake.active_effects.ghost.map(|t| t > now).unwrap_or(false);
        let shield_active = snake
            .active_effects
            .shield
            .map(|t| t > now)
            .unwrap_or(false);
        let super_active = snake
            .active_effects
            .super_mode
            .map(|t| t > now)
            .unwrap_or(false);

        if ghost_active || super_active {
            new_head.x = new_head.x.rem_euclid(CONFIG.cols as i32);
            new_head.y = new_head.y.rem_euclid(CONFIG.rows as i32);
        } else if collision::check_wall_collision(new_head.x, new_head.y) {
            if shield_active {
                let safe_dir = collision::get_shield_safe_direction(
                    new_head.x,
                    new_head.y,
                    CONFIG.cols as i32,
                    CONFIG.rows as i32,
                );
                snake.next_dir = safe_dir;
                snake.dir = safe_dir;
                tracing::info!("[SHIELD] {} auto-turned from wall", snake.color);
                self.snakes.insert(socket_id.to_string(), snake);
                return;
            }
            snake.alive = false;
            snake.death_reason = Some("WALL".to_string());
            tracing::info!("[DEATH] {} {} died: WALL", snake.color, snake.name);
            self.snakes.insert(socket_id.to_string(), snake);
            effects::drop_powerup_on_death(&self.snakes, &mut self.powerups, socket_id);
            return;
        }

        if !ghost_active
            && !shield_active
            && !super_active
            && collision::check_self_collision(new_head, &snake.segments)
        {
            snake.alive = false;
            snake.death_reason = Some("SELF".to_string());
            tracing::info!("[DEATH] {} {} died: SELF", snake.color, snake.name);
            self.snakes.insert(socket_id.to_string(), snake);
            effects::drop_powerup_on_death(&self.snakes, &mut self.powerups, socket_id);
            return;
        }

        let (kill_other, killed_by) = collision::check_snake_collisions(
            &self.snakes,
            socket_id,
            new_head,
            &snake,
            collision::SnakeCollisionStatus {
                effective_speed,
                shield_active,
                super_active,
                now,
            },
        );

        if let Some(killer_name) = killed_by {
            snake.alive = false;
            snake.death_reason = Some(format!("KILLED BY {}", killer_name));
            tracing::info!(
                "[DEATH] {} {} died: KILLED BY {}",
                snake.color,
                snake.name,
                killer_name
            );
            self.snakes.insert(socket_id.to_string(), snake);
            effects::drop_powerup_on_death(&self.snakes, &mut self.powerups, socket_id);
            return;
        }

        if let Some((other_id, _)) = kill_other {
            effects::kill_snake(
                &mut self.snakes,
                &other_id,
                &format!("RAMMED BY {}", snake.name),
            );
            effects::drop_powerup_on_death(&self.snakes, &mut self.powerups, &other_id);
        }

        handlers::apply_magnet_effect(
            &mut self.foods,
            &mut self.bonus_foods,
            new_head,
            &snake,
            now,
        );

        handlers::handle_food_pickup(
            socket_id,
            &mut snake,
            &mut self.foods,
            &self.snakes,
            new_head,
            now,
            shield_active,
        );

        handlers::handle_bonus_food_pickup(&mut snake, &mut self.bonus_foods, new_head);

        handlers::handle_powerup_pickup(&mut snake, &mut self.powerups, new_head, shield_active);

        snake.segments.insert(0, new_head);
        let grew = snake.segments.len() > 1 && snake.segments.len() % 3 == 0;
        if !grew {
            snake.segments.pop();
        }

        if super_active {
            if let Some(start) = snake.super_mode_start {
                let elapsed = (now - start) as f32 / 1000.0;
                snake.super_meter = (100.0 - elapsed * 20.0).max(0.0) as u32;
                if snake.super_meter == 0 {
                    snake.active_effects.super_mode = None;
                    snake.super_mode_start = None;
                    tracing::info!("[SUPER] {} SUPER MODE ended", snake.color);
                }
            }
        }

        self.snakes.insert(socket_id.to_string(), snake);
    }

    pub fn update_high_scores(&mut self) {
        self.high_scores = highscores::update_high_scores(&self.snakes);
    }

    pub fn broadcast_state(&self) -> GameBroadcast<'_> {
        GameBroadcast::new(
            &self.snakes,
            &self.foods,
            &self.bonus_foods,
            &self.powerups,
            &self.explosions,
            self.tick,
        )
    }
}
