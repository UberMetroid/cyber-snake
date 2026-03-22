//! Game module containing all game logic components.
//! Organized into: state management, snake management, spawning, AI, effects, and collision.

pub mod bot_ai;
pub mod broadcast;
pub mod collision;
pub mod dominance;
pub mod effects;
pub mod food;
pub mod handlers;
pub mod highscores;
pub mod magnet;
pub mod pickup;
pub mod powerup;
pub mod shared_state;
pub mod snake;
pub mod snake_mgr;
pub mod snakes_collision;
pub mod spawner;
pub mod speed;
pub mod state;
pub mod tick;
pub mod wall;

#[allow(unused_imports)]
pub use shared::{
    ActiveEffects, BonusFood, Direction, Explosion, Food, HighScore, Point, Powerup, PowerupType,
    Snake,
};

#[allow(unused_imports)]
pub use shared_state::SharedGameState;

#[allow(unused_imports)]
pub use broadcast::GameBroadcast;

pub use food::get_bonus_color;

#[allow(unused_imports)]
pub use snake::{get_next_color, random_name, NEON_COLORS, PREFIXES, SUFFIXES};
