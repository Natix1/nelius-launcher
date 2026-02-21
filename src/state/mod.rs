#![allow(dead_code)]
use crate::state::persistent::PersistentAppState;

#[derive(Debug, Clone)]
pub struct AppState {
    pub persistent: PersistentAppState,
    pub game_locked: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let config = PersistentAppState::load();

        AppState { persistent: config, game_locked: false }
    }
}

pub mod persistent;
