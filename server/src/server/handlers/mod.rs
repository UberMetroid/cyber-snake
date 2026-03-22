//! Request handlers module.

pub mod http;
pub mod messages;
pub mod ws;

pub use http::{health_handler, highscores_handler, stats_handler};
pub use ws::{ws_handler, AppState};
