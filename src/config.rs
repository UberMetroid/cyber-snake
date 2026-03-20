use once_cell::sync::Lazy;
use std::env;
use std::path::PathBuf;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::from_env);

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub timezone: String,
    pub max_players: usize,
    pub tick_rate: u64,
    pub cols: u32,
    pub rows: u32,
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl Config {
    fn from_env() -> Self {
        let data_dir = env::var("DATA_DIR").unwrap_or_else(|_| "/app/data".into());
        let log_dir = env::var("LOG_DIR").unwrap_or_else(|_| "/app/logs".into());

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .unwrap_or(3000),
            timezone: env::var("TZ").unwrap_or_else(|_| "UTC".into()),
            max_players: env::var("MAX_PLAYERS")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10),
            tick_rate: env::var("TICK_RATE")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .unwrap_or(60),
            cols: env::var("COLS")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .unwrap_or(30),
            rows: env::var("ROWS")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .unwrap_or(30),
            data_dir: PathBuf::from(data_dir),
            log_dir: PathBuf::from(log_dir),
        }
    }
}
