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
            port: env::var("PORT")
                .unwrap_or_else(|_| "8300".into())
                .parse()
                .unwrap_or(8300),
            timezone: env::var("TZ").unwrap_or_else(|_| "UTC".into()),
            max_players: env::var("MAX_PLAYERS")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10),
            tick_rate: env::var("TICK_RATE")
                .unwrap_or_else(|_| "20".into())
                .parse()
                .unwrap_or(20),
            cols: env::var("COLS")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .unwrap_or(30),
            rows: env::var("ROWS")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .unwrap_or(30),
        }
    }
}
