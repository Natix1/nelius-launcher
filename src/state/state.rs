use crate::{globals, state::persistent::PersistentAppState};

#[derive(Debug, Clone)]
pub struct AppState {
    pub persistent: PersistentAppState,
    pub logs: String,
    pub game_locked: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let config = PersistentAppState::load();

        AppState { persistent: config, logs: String::with_capacity(globals::MAX_LOGS_LENGTH), game_locked: false }
    }
}
