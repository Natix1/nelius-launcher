#![windows_subsystem = "windows"]

use anyhow::anyhow;
use futures::SinkExt;
use iced::{
    Alignment::Center,
    Element,
    Length::{self, Fill},
    Task, Theme,
    alignment::Horizontal::{Left, Right},
    color, stream,
    widget::{self, button, checkbox, column, container, pick_list, row, scrollable, text},
};

use serde::{Deserialize, Serialize};

const MAX_LOGS_LENGTH: usize = 5000;

use crate::launcher::downloader::{GameInstance, ManifestVersion, VersionType};

mod launcher;

#[derive(Clone, Debug)]
enum LogSource {
    NeliusLauncher,
    Minecraft,
    #[allow(dead_code)]
    Unknown, /* reserved for future use */
}

#[derive(Serialize, Deserialize)]
pub struct PersistentAppState {
    pub java_binary: Option<String>,
    pub show_releases: bool,
    pub show_snapshots: bool,
    pub auto_scroll: bool,
    pub selected_version: Option<String>,
}

impl PersistentAppState {
    pub fn load() -> Self {
        let config_path = launcher::instances::get_project_dirs().config_dir().join("nelius-state.json");
        std::fs::read_to_string(config_path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let proj_dirs = launcher::instances::get_project_dirs();
        let config_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(&config_dir)?;
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(config_dir.join("nelius-state.json"), contents)?;

        Ok(())
    }
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

struct AppState {
    __persistent: PersistentAppState,

    all_versions: Result<Vec<ManifestVersion>, String>,
    filtered_versions: Vec<String>,
    installed_versions: Vec<String>,
    logs: String,
    game_locked: bool,
    scrollable_id: iced::widget::Id,
}

impl Default for AppState {
    fn default() -> Self {
        let config = PersistentAppState::load();

        AppState {
            __persistent: config,

            all_versions: Ok(Vec::with_capacity(1024)),
            filtered_versions: Vec::with_capacity(1024),
            installed_versions: Vec::new(),
            logs: String::with_capacity(MAX_LOGS_LENGTH),
            game_locked: false,
            scrollable_id: widget::Id::unique(),
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    VersionChanged(String),
    MinecraftVersionsLoaded(Result<Vec<ManifestVersion>, String>),
    InstalledVersionsLoaded(Result<Vec<String>, String>),
    ShowReleasesUpdated(bool),
    ShowSnapshotsUpdated(bool),
    StartGameRequest,
    MinecraftLaunched(Option<String>),
    SubmitLogLine(String, LogSource),
    MinecraftClosed,
    ChangeJavaBinaryRequested,
    SetJavaBinary(Option<String>),
    AutoScrollLogsToggled(bool),
}

fn log_error_task(contents: String, log_source: LogSource) -> Task<Message> {
    Task::perform(async move { (contents, log_source) }, |(contents, source)| Message::SubmitLogLine(contents, source))
}

impl AppState {
    fn init() -> (Self, Task<Message>) {
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

    fn refresh_filtered_versions(&mut self) {
        if let Ok(versions) = &self.all_versions {
            self.filtered_versions = versions
                .iter()
                .filter(|v| match v.version_type {
                    VersionType::Release => self.get_persistent_state().show_releases,
                    VersionType::Snapshot => self.get_persistent_state().show_snapshots,
                    VersionType::Unknown => true,
                })
                .map(|v| &v.version_id)
                .cloned()
                .collect()
        }
    }

    fn append_log(&mut self, line: String, source: LogSource) {
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

    fn update_persistent_state<F>(&mut self, f: F)
    where
        F: FnOnce(&mut PersistentAppState),
    {
        f(&mut self.__persistent);
        if let Err(e) = self.__persistent.save() {
            self.append_log(format!("Failed to save config: {}", e), LogSource::NeliusLauncher);
        }
    }

    fn get_persistent_state(&self) -> &PersistentAppState {
        &self.__persistent
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VersionChanged(version) => {
                self.update_persistent_state(|p| p.selected_version = Some(version));
                Task::none()
            }
            Message::MinecraftVersionsLoaded(versions) => {
                self.all_versions = versions;
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::ShowReleasesUpdated(show) => {
                self.update_persistent_state(|p| p.show_releases = show);
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::ShowSnapshotsUpdated(show) => {
                self.update_persistent_state(|p| p.show_snapshots = show);
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::StartGameRequest => {
                self.game_locked = true;

                let selected_version = match &self.get_persistent_state().selected_version {
                    Some(version) => version.clone(),
                    None => {
                        return log_error_task(
                            "No version selected - couldn't launch".to_string(),
                            LogSource::NeliusLauncher,
                        );
                    }
                };

                let binary = match &self.get_persistent_state().java_binary {
                    Some(binary) => binary.to_string(),
                    None => {
                        return log_error_task(
                            "No java binary selected - couldn't launch".to_string(),
                            LogSource::NeliusLauncher,
                        );
                    }
                };

                Task::run(
                    stream::channel(100, move |mut logger: futures::channel::mpsc::Sender<Message>| async move {
                        let mut instance = GameInstance::new(selected_version);
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
                    |v| v,
                )
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

                if self.get_persistent_state().auto_scroll {
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
                self.update_persistent_state(|p| p.java_binary = binary);
                Task::none()
            }
            Message::AutoScrollLogsToggled(enabled) => {
                self.update_persistent_state(|p| p.auto_scroll = enabled);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let status_text = match &self.all_versions {
            Ok(v) if v.is_empty() => text("Loading version information, hang tight!").color(color!(255, 255, 0)),
            Ok(_) => {
                if let Some(version) = &self.get_persistent_state().selected_version {
                    let is_installed = self.installed_versions.iter().any(|v| v == version);
                    if is_installed {
                        text("This version is installed on disk! Will launch immediately.").color(color!(0, 255, 0))
                    } else {
                        text("This version is not installed on disk. It will be downloaded upon pressing play.")
                            .color(color!(0, 172, 0))
                    }
                } else {
                    text("Versions loaded! Select one and play!").color(color!(0, 255, 0))
                }
            }
            Err(e) => text(format!("Error loading versions: {}", e)).color(color!(255, 0, 0)),
        };

        let dropdown = pick_list(
            self.filtered_versions.as_slice(),
            self.get_persistent_state().selected_version.clone(),
            Message::VersionChanged,
        )
        .placeholder("Select version");

        let mut play_button = button("Play!");
        if self.get_persistent_state().selected_version.is_some()
            && self.get_persistent_state().java_binary.is_some()
            && !self.game_locked
        {
            play_button = play_button.on_press(Message::StartGameRequest).style(button::success);
        } else {
            play_button = play_button.style(button::secondary)
        }

        container(
            column![
                dropdown,
                status_text,
                checkbox(self.get_persistent_state().show_releases)
                    .label("Show releases")
                    .on_toggle(Message::ShowReleasesUpdated),
                checkbox(self.get_persistent_state().show_snapshots)
                    .label("Show snapshots")
                    .on_toggle(Message::ShowSnapshotsUpdated),
                text(format!(
                    "Java binary: {}",
                    self.get_persistent_state().java_binary.clone().unwrap_or("None!".to_string())
                )),
                button("Change Java binary").style(button::primary).on_press(Message::ChangeJavaBinaryRequested),
                row![play_button].spacing(10),
                container(
                    checkbox(self.get_persistent_state().auto_scroll)
                        .label("Auto-scroll logs")
                        .width(Fill)
                        .on_toggle(Message::AutoScrollLogsToggled)
                )
                .align_x(Right),
                scrollable(text(&self.logs).width(Fill).align_x(Left))
                    .spacing(5)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .id(self.scrollable_id.clone())
            ]
            .spacing(10)
            .align_x(Center),
        )
        .padding(50)
        .width(Fill)
        .height(Fill)
        .align_x(Center)
        .align_y(Center)
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    let result = iced::application(AppState::init, AppState::update, AppState::view)
        .theme(Theme::GruvboxDark)
        .window_size((800.0, 600.0))
        .resizable(false)
        .run();

    println!("Until next time!");
    result.map_err(|e| anyhow!(e))
}
