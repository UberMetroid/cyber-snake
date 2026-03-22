//! Game state management for the WASM client.

use shared::GameBroadcast;

/// Application state for the game client.
#[derive(Default, Clone)]
pub struct AppState {
    pub my_id: String,
    pub grid_cols: u32,
    pub grid_rows: u32,
    pub game_state: Option<GameBroadcast>,
    pub connected: bool,
    pub spawned: bool,
    pub camera_x: f64,
    pub camera_y: f64,
    #[allow(dead_code)]
    pub last_server_update: i64,
    #[allow(dead_code)]
    pub server_tick_interval: f64,
}

#[cfg(test)]
impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn update_connection(&mut self, connected: bool) {
        self.connected = connected;
    }

    pub fn update_spawned(&mut self, spawned: bool) {
        self.spawned = spawned;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.my_id, "");
        assert_eq!(state.grid_cols, 0);
        assert_eq!(state.grid_rows, 0);
        assert!(state.game_state.is_none());
        assert!(!state.connected);
        assert!(!state.spawned);
        assert_eq!(state.camera_x, 0.0);
        assert_eq!(state.camera_y, 0.0);
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.my_id, "");
        assert!(!state.connected);
    }

    #[test]
    fn test_is_connected() {
        let state = AppState::default();
        assert!(!state.is_connected());
    }

    #[test]
    fn test_update_connection() {
        let mut state = AppState::default();
        assert!(!state.connected);
        state.update_connection(true);
        assert!(state.connected);
        state.update_connection(false);
        assert!(!state.connected);
    }

    #[test]
    fn test_update_spawned() {
        let mut state = AppState::default();
        assert!(!state.spawned);
        state.update_spawned(true);
        assert!(state.spawned);
        state.update_spawned(false);
        assert!(!state.spawned);
    }

    #[test]
    fn test_app_state_clone() {
        let mut state = AppState::default();
        state.my_id = "test_id".to_string();
        state.connected = true;
        state.spawned = true;
        state.camera_x = 100.0;
        state.camera_y = 200.0;

        let cloned = state.clone();
        assert_eq!(cloned.my_id, "test_id");
        assert_eq!(cloned.connected, true);
        assert_eq!(cloned.spawned, true);
        assert_eq!(cloned.camera_x, 100.0);
        assert_eq!(cloned.camera_y, 200.0);
    }
}
