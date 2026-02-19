use crate::{globals, launcher::downloader::ManifestVersion, state::persistent::PersistentAppState};

#[derive(Debug, Clone)]
pub struct AppState {
    pub persistent: PersistentAppState,
    pub all_versions: Result<Vec<ManifestVersion>, String>,
    pub installed_versions: Vec<String>,
    pub logs: String,
    pub game_locked: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let config = PersistentAppState::load();

        AppState {
            persistent: config,

            all_versions: Ok(Vec::with_capacity(1024)),
            installed_versions: Vec::new(),
            logs: String::with_capacity(globals::MAX_LOGS_LENGTH),
            game_locked: false,
        }
    }
}
