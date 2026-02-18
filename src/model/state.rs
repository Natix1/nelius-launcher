use iced::widget;

use crate::{
    launcher::downloader::ManifestVersion,
    model::{MAX_LOGS_LENGTH, persistent_state::PersistentAppState},
};

pub struct AppState {
    pub persistent: PersistentAppState,
    pub all_versions: Result<Vec<ManifestVersion>, String>,
    pub filtered_versions: Vec<String>,
    pub installed_versions: Vec<String>,
    pub logs: String,
    pub game_locked: bool,
    pub scrollable_id: iced::widget::Id,
}

impl Default for AppState {
    fn default() -> Self {
        let config = PersistentAppState::load();

        AppState {
            persistent: config,

            all_versions: Ok(Vec::with_capacity(1024)),
            filtered_versions: Vec::with_capacity(1024),
            installed_versions: Vec::new(),
            logs: String::with_capacity(MAX_LOGS_LENGTH),
            game_locked: false,
            scrollable_id: widget::Id::unique(),
        }
    }
}
