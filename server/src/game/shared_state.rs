//! Thread-safe wrapper for game state.

use crate::game::state::GameState;

#[derive(Clone)]
pub struct SharedGameState(pub std::sync::Arc<parking_lot::RwLock<GameState>>);

impl Default for SharedGameState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedGameState {
    pub fn new() -> Self {
        Self(std::sync::Arc::new(parking_lot::RwLock::new(
            GameState::new(),
        )))
    }
}
