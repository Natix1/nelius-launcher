use iced::Task;

use crate::{
    launcher,
    model::state::AppState,
    ui::message::{LogSource, Message},
};

impl AppState {
    pub fn init() -> (Self, Task<Message>) {
        let fetch_versions = Task::perform(
            async move {
                let versions =
                    launcher::downloader::get_versions().await.map(|v| v.to_vec()).map_err(|e| e.to_string())?;

                Ok(versions)
            },
            Message::MinecraftVersionsLoaded,
        );

        let load_installed_versions = Task::perform(
            async {
                let versions = launcher::instances::get_installed()
                    .await
                    .map_err(|e| format!("failed getting versions from disk: {}", e));

                versions
            },
            Message::InstalledVersionsLoaded,
        );

        let greet = Task::perform(
            async { ("Welcome to nelius launcher!".to_string(), LogSource::NeliusLauncher) },
            |(message, source)| Message::SubmitLogLine(message, source),
        );

        (AppState::default(), Task::batch([fetch_versions, load_installed_versions, greet]))
    }
}
