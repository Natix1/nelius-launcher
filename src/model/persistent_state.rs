use iced::Task;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{launcher, ui::message::Message};

/// Do not mutate directly.
/// Use [update()].
#[derive(Serialize, Deserialize)]
pub struct PersistentAppState {
    pub java_binary: Option<String>,
    pub show_releases: bool,
    pub show_snapshots: bool,
    pub auto_scroll: bool,
    pub selected_version: Option<String>,
}

impl Default for PersistentAppState {
    fn default() -> Self {
        PersistentAppState {
            java_binary: None,
            show_releases: true,
            show_snapshots: false,
            auto_scroll: true,
            selected_version: None,
        }
    }
}

impl PersistentAppState {
    pub fn load() -> Self {
        let config_path = launcher::instances::get_project_dirs().config_dir().join("nelius-state.json");
        std::fs::read_to_string(config_path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save_task(&self) -> iced::Task<Message> {
        let project_dir = launcher::instances::get_project_dirs();
        let config_dir = project_dir.config_dir().to_owned();
        let to_save = serde_json::to_string_pretty(self);

        Task::perform(
            async move {
                fs::create_dir_all(&config_dir)
                    .await
                    .map_err(|e| format!("Failed creating / ensuring config directory: {e}"))?;
                fs::write(
                    &config_dir.join("nelius-state.json"),
                    to_save.map_err(|e| format!("Failed serializing app state: {e}"))?,
                )
                .await
                .map_err(|e| format!("Failed writing configuration: {e}"))?;

                Ok(())
            },
            |result: Result<(), String>| Message::PersistentStateSaved(result),
        )
    }
}
