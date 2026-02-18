use iced::{Task, widget::scrollable};

use crate::{
    launcher::downloader::GameInstance,
    model::state::AppState,
    ui::message::{LogSource, Message},
};

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VersionChanged(version) => {
                self.persistent.selected_version = Some(version);
                self.persistent.save_task()
            }
            Message::MinecraftVersionsLoaded(versions) => {
                self.all_versions = versions;
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::ShowReleasesUpdated(show) => {
                self.persistent.show_releases = show;
                self.refresh_filtered_versions();
                self.persistent.save_task()
            }
            Message::ShowSnapshotsUpdated(show) => {
                self.persistent.show_snapshots = show;
                self.refresh_filtered_versions();
                self.persistent.save_task()
            }
            Message::StartGameRequest => {
                self.game_locked = true;
                let selected_version = match &self.persistent.selected_version {
                    Some(version) => version.clone(),
                    None => {
                        return self.make_log_error_task(
                            "No version selected - couldn't launch".to_string(),
                            LogSource::NeliusLauncher,
                        );
                    }
                };

                let binary = match &self.persistent.java_binary {
                    Some(binary) => binary.to_string(),
                    None => {
                        return self.make_log_error_task(
                            "No java binary selected - couldn't launch".to_string(),
                            LogSource::NeliusLauncher,
                        );
                    }
                };

                self.make_game_runner_task(GameInstance::new(selected_version), binary)
            }
            Message::InstalledVersionsLoaded(versions) => {
                if let Ok(versions) = versions {
                    self.installed_versions = versions;
                } else if let Err(e) = versions {
                    self.append_log(format!("Failed getting installed versions: {}", e), LogSource::NeliusLauncher);
                }
                Task::none()
            }
            Message::MinecraftLaunched(installed_version) => match installed_version {
                Some(version) => {
                    self.installed_versions.push(version);
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::MinecraftClosed => {
                self.game_locked = false;
                Task::none()
            }
            Message::SubmitLogLine(line, source) => {
                println!("Received log line: {}", line);
                self.append_log(line, source);

                if self.persistent.auto_scroll {
                    return iced::widget::operation::snap_to(
                        self.scrollable_id.clone(),
                        scrollable::RelativeOffset::END,
                    );
                }

                Task::none()
            }
            Message::ChangeJavaBinaryRequested => Task::perform(
                async {
                    let mut builder = rfd::AsyncFileDialog::new();
                    if std::env::consts::OS == "windows" {
                        builder = builder.add_filter("Executable files", &["exe"]);
                    }
                    let file = builder.pick_file().await;
                    if let Some(file) = file { Some(file.path().to_string_lossy().to_string()) } else { None }
                },
                Message::SetJavaBinary,
            ),
            Message::SetJavaBinary(binary) => {
                self.persistent.java_binary = binary;
                self.persistent.save_task()
            }
            Message::AutoScrollLogsToggled(enabled) => {
                self.persistent.auto_scroll = enabled;
                self.persistent.save_task()
            }
            Message::PersistentStateSaved(result) => {
                match result {
                    Ok(_) => {
                        println!("Persistent state saved.");
                    }
                    Err(e) => {
                        self.append_log(format!("Failed saving persistent app state: {}", e), LogSource::NeliusLauncher)
                    }
                }

                Task::none()
            }
        }
    }
}
