use anyhow::anyhow;
use futures::SinkExt;
use iced::{
    Alignment::Center,
    Element,
    Length::{self, Fill},
    Task, color, stream,
    widget::{self, button, checkbox, column, container, pick_list, row, scrollable, text},
};

use serde::{Deserialize, Serialize};

const MAX_LOGS_LENGTH: usize = 1000;

use crate::launcher::downloader::{GameInstance, ManifestVersion, VersionType};

mod launcher;

#[derive(Clone, Debug)]
enum LogSource {
    NeliusLauncher,
    Minecraft,
    #[allow(dead_code)]
    Unknown, /* reserved for future use */
}

#[derive(Serialize, Deserialize, Default)]
pub struct LauncherConfig {
    pub java_binary: Option<String>,
    pub show_releases: bool,
    pub show_snapshots: bool,
    pub selected_version: Option<String>,
}

impl LauncherConfig {
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

struct AppState {
    config: LauncherConfig,

    all_versions: Result<Vec<ManifestVersion>, String>,
    filtered_versions: Vec<String>,
    installed_versions: Vec<String>,
    logs: String,
    game_locked: bool,
    scrollable_id: iced::widget::Id,
}

impl Default for AppState {
    fn default() -> Self {
        let config = LauncherConfig::load();

        AppState {
            config: config,

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
                    VersionType::Release => self.config.show_releases,
                    VersionType::Snapshot => self.config.show_snapshots,
                    VersionType::Unknown => true,
                })
                .map(|v| &v.version_id)
                .cloned()
                .collect()
        }
    }

    fn append_log(&mut self, line: String, source: LogSource) {
        let formatted_line = match source {
            LogSource::NeliusLauncher => format!("[NELIUS-LAUNCHER] {}\n", line),
            LogSource::Minecraft => format!("[MINECRAFT] {}\n", line),
            LogSource::Unknown => format!("{}\n", line),
        };

        self.logs.push_str(&formatted_line);
        if self.logs.len() > MAX_LOGS_LENGTH {
            let to_remove = self.logs.len() - MAX_LOGS_LENGTH;
            self.logs.replace_range(..to_remove, "");
        }
    }

    fn update_config<F>(&mut self, f: F)
    where
        F: FnOnce(&mut LauncherConfig),
    {
        f(&mut self.config);
        if let Err(e) = self.config.save() {
            self.append_log(format!("Failed to save config: {}", e), LogSource::NeliusLauncher);
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VersionChanged(version) => {
                self.config.selected_version = Some(version);
                Task::none()
            }
            Message::MinecraftVersionsLoaded(versions) => {
                self.all_versions = versions;
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::ShowReleasesUpdated(show) => {
                self.update_config(|c| c.show_releases = show);
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::ShowSnapshotsUpdated(show) => {
                self.update_config(|c| c.show_snapshots = show);
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::StartGameRequest => {
                self.game_locked = true;

                let selected_version = match &self.config.selected_version {
                    Some(version) => version.clone(),
                    None => {
                        return log_error_task(
                            "No version selected - couldn't launch".to_string(),
                            LogSource::NeliusLauncher,
                        );
                    }
                };

                let binary = match &self.config.java_binary {
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
                iced::widget::operation::snap_to(self.scrollable_id.clone(), scrollable::RelativeOffset::END)
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
                self.update_config(|c| c.java_binary = binary);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let status_text = match &self.all_versions {
            Ok(v) if v.is_empty() => text("Loading version information, hang tight!").color(color!(255, 255, 0)),
            Ok(_) => {
                if let Some(version) = &self.config.selected_version {
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

        let dropdown =
            pick_list(self.filtered_versions.as_slice(), self.config.selected_version.clone(), Message::VersionChanged)
                .placeholder("Select version");

        let mut play_button = button("Play!");
        if self.config.selected_version.is_some() && self.config.java_binary.is_some() && !self.game_locked {
            play_button = play_button.on_press(Message::StartGameRequest).style(button::success);
        } else {
            play_button = play_button.style(button::secondary)
        }

        container(
            column![
                dropdown,
                status_text,
                checkbox(self.config.show_releases).label("Show releases").on_toggle(Message::ShowReleasesUpdated),
                checkbox(self.config.show_snapshots).label("Show snapshots").on_toggle(Message::ShowSnapshotsUpdated),
                text(format!("Java binary: {}", self.config.java_binary.clone().unwrap_or("None!".to_string()))),
                button("Change Java binary").style(button::primary).on_press(Message::ChangeJavaBinaryRequested),
                row![play_button].spacing(10),
                scrollable(text(&self.logs).width(Fill).align_x(Center))
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
    let result = iced::application(AppState::init, AppState::update, AppState::view).window_size((800.0, 400.0)).run();

    println!("Until next time!");
    result.map_err(|e| anyhow!(e))
}
