use once_cell::sync::Lazy;
use std::env;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::from_env);

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub timezone: String,
    pub max_players: usize,
    pub tick_rate: u64,
    pub cols: u32,
    pub rows: u32,
}

impl Config {
    fn from_env() -> Self {
        Self {
            port: Self::parse_env_or_warn("PORT", 8300u16),
            timezone: env::var("TZ").unwrap_or_else(|_| "UTC".into()),
            max_players: Self::parse_env_or_warn("MAX_PLAYERS", 50usize),
            tick_rate: Self::parse_env_or_warn("TICK_RATE", 20u64),
            cols: Self::parse_env_or_warn("COLS", 100u32),
            rows: Self::parse_env_or_warn("ROWS", 100u32),
        }
    }

    fn parse_env_or_warn<T: std::str::FromStr>(key: &str, default: T) -> T {
        match env::var(key) {
            Ok(val) => match val.parse() {
                Ok(parsed) => parsed,
                Err(_) => {
                    tracing::warn!(
                        "[CONFIG] Failed to parse {}='{}', using default {}",
                        key,
                        val,
                        std::any::type_name::<T>()
                    );
                    default
                }
            },
            Err(_) => default,
        }
    }
}

pub const SPEED_BOOST_DURATION_MS: i64 = 5000;
pub const SHIELD_DURATION_MS: i64 = 5000;
pub const GHOST_DURATION_MS: i64 = 10000;
pub const MAGNET_DURATION_MS: i64 = 10000;
pub const SUPER_MODE_DURATION_MS: i64 = 5000;
pub const BOMB_RADIUS: i32 = 10;
pub const BOMB_EXPLOSION_DURATION_MS: i64 = 500;
pub const GROW_SEGMENTS: i32 = 5;
pub const GROW_BONUS_SCORE: u32 = 250;
#[allow(dead_code)]
pub const FOOD_EXPIRY_MS: i64 = 60000;
#[allow(dead_code)]
pub const POWERUP_EXPIRY_MS: i64 = 30000;
pub const SUPER_METER_INCREMENT: u32 = 20;
pub const FOOD_POINTS_MULTIPLIER: u32 = 50;
pub const BONUS_FOOD_POINTS_MULTIPLIER: u32 = 100;
pub const BONUS_FOOD_SPAWN_INTERVAL: u64 = 480;
pub const POWERUP_SPAWN_INTERVAL: u64 = 600;
pub const MAX_BONUS_FOOD_COUNT: usize = 2;
pub const MIN_BOT_COUNT: usize = 3;
