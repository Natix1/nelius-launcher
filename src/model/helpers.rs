use futures::SinkExt;
use iced::{Task, stream};

use crate::{
    MAX_LOGS_LENGTH,
    launcher::downloader::{GameInstance, VersionType},
    model::state::AppState,
    ui::message::{LogSource, Message},
};

impl AppState {
    pub fn refresh_filtered_versions(&mut self) {
        if let Ok(versions) = &self.all_versions {
            self.filtered_versions = versions
                .iter()
                .filter(|v| match v.version_type {
                    VersionType::Release => self.persistent.show_releases,
                    VersionType::Snapshot => self.persistent.show_snapshots,
                    VersionType::Unknown => true,
                })
                .map(|v| &v.version_id)
                .cloned()
                .collect()
        }
    }

    pub fn make_log_error_task(&self, contents: String, log_source: LogSource) -> Task<Message> {
        Task::perform(async move { (contents, log_source) }, |(contents, source)| {
            Message::SubmitLogLine(contents, source)
        })
    }

    pub fn append_log(&mut self, line: String, source: LogSource) {
        let formatted_line = match source {
            LogSource::NeliusLauncher => format!("[NELIUS LAUNCHER] {}\n", line),
            LogSource::Minecraft => format!("[MINECRAFT] {}\n", line),
            LogSource::Unknown => format!("{}\n", line),
        };

        self.logs.push_str(&formatted_line);
        if self.logs.len() > MAX_LOGS_LENGTH {
            let to_remove = self.logs.len() - MAX_LOGS_LENGTH;
            self.logs.replace_range(..to_remove, "");
        }
    }

    pub fn make_game_runner_task(&self, mut instance: GameInstance, binary: String) -> Task<Message> {
        Task::run(
            stream::channel(100, move |mut logger: futures::channel::mpsc::Sender<Message>| async move {
                let was_installed = instance.is_installed().await;
                match instance.ensure_installed(&mut logger).await {
                    Ok(_) => {}
                    Err(e) => {
                        let _ = logger
                            .send(Message::SubmitLogLine(
                                format!("Couldn't check installation status: {}", e),
                                LogSource::NeliusLauncher,
                            ))
                            .await;
                        return;
                    }
                };

                if was_installed {
                    let _ = logger.send(Message::MinecraftLaunched(None)).await;
                } else {
                    let _ = logger.send(Message::MinecraftLaunched(Some(instance.version_id.clone()))).await;
                }

                match instance.run(&binary, &mut logger).await {
                    Ok(_) => {}
                    Err(e) => {
                        let _ = logger
                            .send(Message::SubmitLogLine(
                                format!("Couldn't start the game: {}", e),
                                LogSource::NeliusLauncher,
                            ))
                            .await;
                        return;
                    }
                };
                let _ = logger.send(Message::MinecraftClosed).await;
            }),
            |msg| msg,
        )
    }
}
