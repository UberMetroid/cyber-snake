use crate::config::CONFIG;
use crate::game::{
    get_bonus_color, get_next_color, random_name, BonusFood, Direction, Food, HighScore, Point,
    Powerup, PowerupType, Snake,
};
use chrono::Utc;
use parking_lot::RwLock;
use rand::Rng;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct GameState {
    pub snakes: HashMap<String, Snake>,
    pub foods: HashMap<String, Food>,
    pub bonus_foods: Vec<BonusFood>,
    pub powerups: Vec<Powerup>,
    pub high_scores: Vec<HighScore>,
    pub tick: u64,
    pub start_time: i64,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            snakes: HashMap::new(),
            foods: HashMap::new(),
            bonus_foods: Vec::new(),
            powerups: Vec::new(),
            high_scores: Vec::new(),
            tick: 0,
            start_time: Utc::now().timestamp_millis(),
        }
    }

    pub fn create_snake(&mut self, socket_id: String) -> Snake {
        let used_colors: Vec<String> = self.snakes.values().map(|s| s.color.clone()).collect();
        let color = get_next_color(&used_colors);
        let name = random_name();
        Snake::new(socket_id, color, name)
    }

    pub fn spawn_snake(&mut self, socket_id: &str) {
        if let Some(snake) = self.snakes.get_mut(socket_id) {
            if !snake.spawned {
                snake.spawned = true;
                let food = Food::new(
                    socket_id.to_string(),
                    snake.color.clone(),
                    snake.head().x,
                    snake.head().y,
                    false,
                );
                self.foods.insert(socket_id.to_string(), food);
                tracing::info!("[SPAWN] {} {} SPAWNED", snake.color, snake.name);
            }
        }
    }

    pub fn respawn_snake(&mut self, socket_id: &str) {
        let (old_color, old_name, old_score) = if let Some(old_snake) = self.snakes.get(socket_id) {
            (
                old_snake.color.clone(),
                old_snake.name.clone(),
                old_snake.score,
            )
        } else {
            return;
        };

        let mut new_snake = self.create_snake(socket_id.to_string());
        new_snake.color = old_color;
        new_snake.name = old_name;
        new_snake.score = old_score;
        new_snake.spawned = true;

        self.snakes.insert(socket_id.to_string(), new_snake.clone());
        let food = Food::new(
            socket_id.to_string(),
            new_snake.color.clone(),
            new_snake.head().x,
            new_snake.head().y,
            false,
        );
        self.foods.insert(socket_id.to_string(), food);
        tracing::info!("[RESPAWN] {} {} respawned", new_snake.color, new_snake.name);
    }

    pub fn remove_snake(&mut self, socket_id: &str) {
        self.snakes.remove(socket_id);
        self.foods.remove(socket_id);
    }

    pub fn occupied_positions(&self) -> HashSet<Point> {
        let mut occupied = HashSet::new();

        for snake in self.snakes.values() {
            for seg in &snake.segments {
                occupied.insert(Point::new(seg.x, seg.y));
            }
        }

        for food in self.foods.values() {
            occupied.insert(Point::new(food.x, food.y));
        }

        for bf in &self.bonus_foods {
            occupied.insert(Point::new(bf.x, bf.y));
        }

        for pu in &self.powerups {
            occupied.insert(Point::new(pu.x, pu.y));
        }

        occupied
    }

    pub fn spawn_food(&mut self, owner_id: &str, color: &str, is_super: bool) -> Option<Food> {
        let occupied = self.occupied_positions();
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let x = rng.gen_range(0..CONFIG.cols as i32);
            let y = rng.gen_range(0..CONFIG.rows as i32);
            let pos = Point::new(x, y);
            if !occupied.contains(&pos) {
                let food = Food::new(owner_id.to_string(), color.to_string(), x, y, is_super);
                return Some(food);
            }
        }
        None
    }

    pub fn spawn_bonus_food(&mut self) {
        if self.bonus_foods.len() >= 3 {
            return;
        }

        let occupied = self.occupied_positions();
        let used_colors: Vec<String> = self.snakes.values().map(|s| s.color.clone()).collect();
        let color = get_bonus_color(&used_colors);
        let is_ring = rand::thread_rng().gen_bool(0.5);
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let x = rng.gen_range(0..CONFIG.cols as i32);
            let y = rng.gen_range(0..CONFIG.rows as i32);
            let pos = Point::new(x, y);
            if !occupied.contains(&pos) {
                self.bonus_foods.push(BonusFood::new(x, y, color, is_ring));
                tracing::info!("[BONUS] spawned at {},{}", x, y);
                return;
            }
        }
    }

    pub fn spawn_powerup(&mut self) {
        if self.powerups.len() >= 3 {
            return;
        }

        let occupied = self.occupied_positions();
        let powerup_type = PowerupType::random();
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let x = rng.gen_range(0..CONFIG.cols as i32);
            let y = rng.gen_range(0..CONFIG.rows as i32);
            let pos = Point::new(x, y);
            if !occupied.contains(&pos) {
                self.powerups.push(Powerup::new(x, y, powerup_type));
                tracing::info!("[POWERUP] {} spawned at {},{}", powerup_type.color(), x, y);
                return;
            }
        }
    }

    pub fn tick(&mut self) {
        let now = Utc::now().timestamp_millis();
        self.tick += 1;

        self.foods.retain(|_, food| !food.is_expired());
        self.bonus_foods.retain(|bf| !bf.is_expired());
        self.powerups.retain(|pu| !pu.is_expired());

        if self.bonus_foods.len() < 2 && self.tick.is_multiple_of(480) {
            self.spawn_bonus_food();
        }

        if self.powerups.len() < 3 && self.tick.is_multiple_of(600) {
            self.spawn_powerup();
        }

        let spawned_player_ids: Vec<String> = self
            .snakes
            .values()
            .filter(|p| p.alive && p.spawned)
            .map(|p| p.id.clone())
            .collect();

        for id in spawned_player_ids {
            self.update_snake(&id, now);
        }
    }

    pub fn activate_powerup(&mut self, socket_id: &str) {
        let now = Utc::now().timestamp_millis();
        let (head, powerup_type) = {
            let snake = match self.snakes.get_mut(socket_id) {
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
                if let Some(snake) = self.snakes.get_mut(socket_id) {
                    snake.active_effects.speed_boost = Some(now + 5000);
                }
            }
            "SHIELD" => {
                if let Some(snake) = self.snakes.get_mut(socket_id) {
                    snake.active_effects.shield = Some(now + 5000);
                }
            }
            "GHOST" => {
                if let Some(snake) = self.snakes.get_mut(socket_id) {
                    snake.active_effects.ghost = Some(now + 10000);
                }
            }
            "MAGNET" => {
                if let Some(snake) = self.snakes.get_mut(socket_id) {
                    snake.active_effects.magnet = Some(now + 10000);
                }
            }
            "GROW" => {
                if let Some(snake) = self.snakes.get_mut(socket_id) {
                    if let Some(&last) = snake.segments.last() {
                        for _ in 0..5 {
                            snake.segments.push(last);
                        }
                    }
                    snake.score += 250;
                }
            }
            "SHRINK" => {
                if let Some(snake) = self.snakes.get_mut(socket_id) {
                    let new_len = (snake.segments.len() / 2).max(3);
                    snake.segments.truncate(new_len);
                }
            }
            "BOMB" => {
                let radius = 10;
                let mut to_kill = Vec::new();
                for (other_id, other_snake) in &self.snakes {
                    if other_id != socket_id && other_snake.alive && other_snake.spawned {
                        let other_head = other_snake.head();
                        let dist = (head.x - other_head.x).abs() + (head.y - other_head.y).abs();
                        if dist <= radius {
                            to_kill.push(other_id.clone());
                        }
                    }
                }
                for id in to_kill {
                    self.kill_snake(&id, "BOMB");
                    self.drop_powerup(&id);
                }
            }
            _ => {}
        }
    }

    fn update_snake(&mut self, socket_id: &str, now: i64) {
        let mut snake = match self.snakes.remove(socket_id) {
            Some(s) => s,
            None => return,
        };

        snake.frame_count += 1;

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

        let mut effective_speed = if speed_boost {
            snake.speed + 4
        } else {
            snake.speed
        };
        if slowed {
            effective_speed = (effective_speed as f32 * 0.5).max(1.0) as u32;
        }

        let move_interval = (15 - effective_speed as i32).max(3) as u32;
        if snake.frame_count % move_interval != 0 {
            self.snakes.insert(socket_id.to_string(), snake);
            return;
        }

        snake.dir = snake.next_dir;
        let head = snake.head();
        let dir_point = snake.dir.to_point();
        let mut new_head = Point::new(head.x + dir_point.x, head.y + dir_point.y);

        if ghost_active || super_active {
            new_head.x = (new_head.x + CONFIG.cols as i32) % (CONFIG.cols as i32);
            new_head.y = (new_head.y + CONFIG.rows as i32) % (CONFIG.rows as i32);
        } else if new_head.x < 0
            || new_head.x >= CONFIG.cols as i32
            || new_head.y < 0
            || new_head.y >= CONFIG.rows as i32
        {
            if shield_active {
                let safe_dir = if new_head.x < 0 {
                    Direction::Right
                } else if new_head.x >= CONFIG.cols as i32 {
                    Direction::Left
                } else if new_head.y < 0 {
                    Direction::Down
                } else {
                    Direction::Up
                };
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
            self.drop_powerup(socket_id);
            return;
        }

        if !ghost_active && !shield_active && !super_active {
            for (_i, seg) in snake.segments.iter().enumerate().skip(1) {
                if new_head.x == seg.x && new_head.y == seg.y {
                    snake.alive = false;
                    snake.death_reason = Some("SELF".to_string());
                    tracing::info!("[DEATH] {} {} died: SELF", snake.color, snake.name);
                    self.snakes.insert(socket_id.to_string(), snake);
                    self.drop_powerup(socket_id);
                    return;
                }
            }
        }

        let mut kill_other: Option<String> = None;
        let mut killed_by_other = false;

        if !ghost_active {
            for (other_id, other) in &self.snakes {
                if !other.alive {
                    continue;
                }
                for (_i, seg) in other.segments.iter().enumerate().skip(1) {
                    if new_head.x == seg.x && new_head.y == seg.y {
                        if shield_active || super_active {
                            tracing::info!(
                                "[SHIELD/STAR] {} killed by {} ram",
                                other.color,
                                snake.color
                            );
                            kill_other = Some(other_id.clone());
                        } else {
                            killed_by_other = true;
                        }
                        break;
                    }
                }
                if kill_other.is_some() || killed_by_other {
                    break;
                }
            }
        }

        if killed_by_other {
            snake.alive = false;
            snake.death_reason = Some("SNAKE".to_string());
            tracing::info!("[DEATH] {} {} died: SNAKE", snake.color, snake.name);
            self.snakes.insert(socket_id.to_string(), snake);
            self.drop_powerup(socket_id);
            return;
        }

        if let Some(other_id) = kill_other {
            self.kill_snake(&other_id, "RAM");
            self.drop_powerup(&other_id);
        }

        let magnet_active = snake
            .active_effects
            .magnet
            .map(|t| t > now)
            .unwrap_or(false)
            || super_active;
        if magnet_active {
            let pull_radius = 5;
            for food in self.foods.values_mut() {
                let dx = new_head.x - food.x;
                let dy = new_head.y - food.y;
                if dx.abs() + dy.abs() <= pull_radius && dx.abs() + dy.abs() > 0 {
                    if dx.abs() > dy.abs() {
                        food.x += dx.signum();
                    } else {
                        food.y += dy.signum();
                    }
                }
            }
            for bf in &mut self.bonus_foods {
                let dx = new_head.x - bf.x;
                let dy = new_head.y - bf.y;
                if dx.abs() + dy.abs() <= pull_radius && dx.abs() + dy.abs() > 0 {
                    if dx.abs() > dy.abs() {
                        bf.x += dx.signum();
                    } else {
                        bf.y += dy.signum();
                    }
                }
            }
        }

        let mut grew = false;
        let mut spawn_new_food = false;
        let mut new_food_super = false;

        if let Some(food) = self.foods.get_mut(socket_id) {
            if new_head.x == food.x && new_head.y == food.y {
                if shield_active {
                    tracing::info!("[SHIELD] {} deflected food", snake.color);
                } else {
                    let is_own_food = food.color == snake.color;
                    if is_own_food {
                        if food.is_super || snake.super_meter >= 100 {
                            tracing::info!(
                                "[SUPER] {} {} activated SUPER MODE!",
                                snake.color,
                                snake.name
                            );
                            snake.active_effects.super_mode = Some(now + 5000);
                            snake.active_effects.speed_boost = Some(now + 5000);
                            snake.active_effects.shield = Some(now + 5000);
                            snake.active_effects.magnet = Some(now + 5000);
                            snake.super_meter = 100;
                            snake.super_mode_start = Some(now);
                            snake.own_food_count = 0;
                        } else {
                            grew = true;
                            snake.super_meter = (snake.super_meter + 20).min(100);
                            snake.own_food_count += 1;
                            tracing::info!(
                                "[FOOD] {} ate OWN food (+grow, meter: {}%)",
                                snake.color,
                                snake.super_meter
                            );
                        }
                    } else {
                        let points = 50 * snake.segments.len() as u32;
                        snake.score += points;
                        snake.super_meter = (snake.super_meter + 10).min(100);
                        tracing::info!(
                            "[FOOD] {} ate ENEMY food (+{} points)",
                            snake.color,
                            points
                        );
                    }
                    spawn_new_food = true;
                    new_food_super = snake.own_food_count >= 5;
                }
            }
        }

        if spawn_new_food {
            let food_color = snake.color.clone();
            if let Some(new_food) = self.spawn_food(socket_id, &food_color, new_food_super) {
                self.foods.insert(socket_id.to_string(), new_food);
            }
        }

        let mut remove_super_food = None;
        for (other_id, food) in &self.foods {
            if other_id == socket_id || !food.is_super {
                continue;
            }
            if new_head.x == food.x && new_head.y == food.y {
                let points = 50 * snake.segments.len() as u32;
                snake.score += points;
                tracing::info!(
                    "[SUPER FOOD] {} ate SUPER food (+{} points)",
                    snake.color,
                    points
                );
                remove_super_food = Some(other_id.clone());
                break;
            }
        }
        if let Some(id) = remove_super_food {
            self.foods.remove(&id);
        }

        let mut remove_bonus_food = None;
        for (i, bf) in self.bonus_foods.iter().enumerate() {
            if new_head.x == bf.x && new_head.y == bf.y {
                let points = 100 * snake.segments.len() as u32;
                snake.score += points;
                snake.super_meter = (snake.super_meter + 10).min(100);
                tracing::info!(
                    "[BONUS] {} ate BONUS food (+{} points)",
                    snake.color,
                    points
                );
                remove_bonus_food = Some(i);
                break;
            }
        }
        if let Some(i) = remove_bonus_food {
            self.bonus_foods.remove(i);
            self.spawn_bonus_food();
        }

        let mut remove_powerup = None;
        for (i, pu) in self.powerups.iter().enumerate() {
            if new_head.x == pu.x && new_head.y == pu.y {
                if shield_active {
                    tracing::info!("[SHIELD] {} deflected powerup", snake.color);
                } else if snake.held_powerup.is_none() {
                    snake.held_powerup = Some(pu.powerup_type.clone());
                    tracing::info!("[POWERUP] {} picked up {}", snake.color, pu.powerup_type);
                    remove_powerup = Some(i);
                }
                break;
            }
        }
        if let Some(i) = remove_powerup {
            self.powerups.remove(i);
        }

        snake.segments.insert(0, new_head);

        if grew {
            let multiplier = if super_active { 3 } else { 1 };
            snake.score += 100 * multiplier;
        } else {
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

    fn kill_snake(&mut self, socket_id: &str, reason: &str) {
        if let Some(snake) = self.snakes.get_mut(socket_id) {
            snake.alive = false;
            snake.death_reason = Some(reason.to_string());
            tracing::info!("[DEATH] {} {} died: {}", snake.color, snake.name, reason);
        }
    }

    fn drop_powerup(&mut self, socket_id: &str) {
        if self.powerups.len() >= 3 {
            return;
        }
        if let Some(snake) = self.snakes.get(socket_id) {
            let powerup_type = PowerupType::random();
            self.powerups
                .push(Powerup::new(snake.head().x, snake.head().y, powerup_type));
            tracing::info!(
                "[POWERUP] {} dropped at {},{}",
                powerup_type.color(),
                snake.head().x,
                snake.head().y
            );
        }
    }

    pub fn update_high_scores(&mut self) {
        let mut all_scores: Vec<(&str, u32)> = self
            .snakes
            .values()
            .filter(|s| s.spawned)
            .map(|s| (s.name.as_str(), s.score))
            .collect();

        all_scores.sort_by(|a, b| b.1.cmp(&a.1));

        let top_10: Vec<HighScore> = all_scores
            .into_iter()
            .take(10)
            .map(|(name, score)| HighScore::new(name.to_string(), score))
            .collect();

        self.high_scores = top_10;
    }

    pub fn broadcast_state(&self) -> GameBroadcast<'_> {
        GameBroadcast {
            msg_type: "gameState",
            snakes: &self.snakes,
            foods: &self.foods,
            bonus_foods: &self.bonus_foods,
            powerups: &self.powerups,
            tick: self.tick,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameBroadcast<'a> {
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    #[serde(borrow)]
    pub snakes: &'a HashMap<String, Snake>,
    #[serde(borrow)]
    pub foods: &'a HashMap<String, Food>,
    #[serde(borrow)]
    pub bonus_foods: &'a Vec<BonusFood>,
    #[serde(borrow)]
    pub powerups: &'a Vec<Powerup>,
    pub tick: u64,
}

#[derive(Clone)]
pub struct SharedGameState(pub Arc<RwLock<GameState>>);

impl SharedGameState {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(GameState::new())))
    }
}
