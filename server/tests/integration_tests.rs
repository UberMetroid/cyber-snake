//! Integration tests for game logic.
//! Tests the full game tick loop and entity interactions.

use server::game::collision::check_self_collision;
use server::game::state::GameState;
use shared::{BonusFood, Food, Point, Snake};

fn create_test_snake(id: &str, x: i32, y: i32) -> Snake {
    Snake::new(
        id.to_string(),
        "#ff0000".to_string(),
        format!("Test_{}", id),
        x,
        y,
    )
}

#[test]
fn test_game_state_initialization() {
    let state = GameState::new();
    assert!(state.snakes.is_empty());
    assert!(state.foods.is_empty());
    assert!(state.bonus_foods.is_empty());
    assert!(state.powerups.is_empty());
    assert_eq!(state.tick, 0);
}

#[test]
fn test_create_snake() {
    let mut state = GameState::new();
    let snake = state.create_snake("player1".to_string());

    // create_snake returns a snake but doesn't insert it - caller must insert
    assert!(snake.color.starts_with('#'));
    assert!(!snake.name.is_empty());
    assert!(snake.alive);
    assert!(!snake.spawned);
}

#[test]
fn test_insert_and_spawn_snake() {
    let mut state = GameState::new();
    let snake = state.create_snake("player1".to_string());
    state.snakes.insert("player1".to_string(), snake);

    // Verify snake exists but not spawned
    {
        let snake = state.snakes.get("player1").unwrap();
        assert!(!snake.spawned);
    }

    // Spawn the snake
    state.spawn_snake("player1");

    // Verify snake is now spawned
    {
        let snake = state.snakes.get("player1").unwrap();
        assert!(snake.spawned);
        assert!(!snake.segments.is_empty());
    }

    // Verify food was created for the snake
    assert!(!state.foods.is_empty());
}

#[test]
fn test_remove_snake() {
    let mut state = GameState::new();
    let snake = state.create_snake("player1".to_string());
    state.snakes.insert("player1".to_string(), snake);
    state.spawn_snake("player1");

    assert!(state.snakes.contains_key("player1"));

    state.remove_snake("player1");

    assert!(!state.snakes.contains_key("player1"));
}

#[test]
fn test_tick_increments() {
    let mut state = GameState::new();
    assert_eq!(state.tick, 0);

    state.tick();
    assert_eq!(state.tick, 1);

    state.tick();
    assert_eq!(state.tick, 2);
}

#[test]
fn test_self_collision_no_collision() {
    let segments = vec![Point::new(5, 5), Point::new(5, 6), Point::new(5, 7)];

    assert!(!check_self_collision(Point::new(10, 10), &segments));
}

#[test]
fn test_self_collision_with_body() {
    let segments = vec![Point::new(5, 5), Point::new(5, 6), Point::new(5, 7)];

    assert!(check_self_collision(Point::new(5, 6), &segments));
}

#[test]
fn test_bonus_food_creation() {
    let bonus_food = BonusFood::new(5, 5, "#ffffff".to_string(), false);
    assert_eq!(bonus_food.x, 5);
    assert_eq!(bonus_food.y, 5);
    assert!(!bonus_food.is_expired());
}

#[test]
fn test_food_expiration() {
    let food = Food::new("owner".to_string(), "#00ff00".to_string(), 5, 5, false);
    assert!(!food.is_expired());
}

#[test]
fn test_multiple_snakes() {
    let mut state = GameState::new();
    let snake1 = state.create_snake("player1".to_string());
    let snake2 = state.create_snake("player2".to_string());
    state.snakes.insert("player1".to_string(), snake1);
    state.snakes.insert("player2".to_string(), snake2);

    assert_eq!(state.snakes.len(), 2);
}

#[test]
fn test_respawn_dead_snake() {
    let mut state = GameState::new();
    let snake = state.create_snake("player1".to_string());
    state.snakes.insert("player1".to_string(), snake);
    state.spawn_snake("player1");

    // Kill the snake
    {
        let snake = state.snakes.get_mut("player1").unwrap();
        snake.alive = false;
    }

    // Respawn
    state.respawn_snake("player1");

    // Check snake is alive again
    {
        let snake = state.snakes.get("player1").unwrap();
        assert!(snake.alive);
        assert!(snake.spawned);
    }
}

#[test]
fn test_activate_powerup_no_powerup() {
    let mut state = GameState::new();
    let snake = state.create_snake("player1".to_string());
    state.snakes.insert("player1".to_string(), snake);
    state.spawn_snake("player1");

    // Try to activate with no held powerup
    state.activate_powerup("player1");

    // Snake should still have no active effects
    let snake = state.snakes.get("player1").unwrap();
    assert!(snake.active_effects.speed_boost.is_none());
    assert!(snake.active_effects.shield.is_none());
}
