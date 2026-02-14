use anyhow::anyhow;
use futures::SinkExt;
use iced::{
    Alignment::Center,
    Element,
    Length::{self, Fill},
    Task, color, stream,
    widget::{button, checkbox, column, container, pick_list, scrollable, text},
};
use tokio::io::{AsyncBufReadExt, BufReader};

const MAX_LOGS_LINES: usize = 10_000;

use crate::{
    downloader::{ManifestVersion, VersionType},
    installed_versions::InstalledVersion,
};

mod downloader;
mod installed_versions;
mod launcher;
mod reqwest_global_client;

struct AppState {
    selected_version: Option<ManifestVersion>,
    all_versions: Result<Vec<ManifestVersion>, String>,
    filtered_versions: Vec<ManifestVersion>,
    installed_versions: Vec<InstalledVersion>,
    logs: Vec<String>,

    playing: bool,
    show_releases: bool,
    show_snapshots: bool,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            selected_version: None,
            all_versions: Ok(Vec::with_capacity(1024)),
            filtered_versions: Vec::with_capacity(1024),
            installed_versions: Vec::new(),
            logs: Vec::with_capacity(2 ^ 12),

            playing: false,
            show_releases: true,
            show_snapshots: false,
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    VersionChanged(ManifestVersion),
    MinecraftVersionsLoaded(Result<Vec<ManifestVersion>, String>),
    InstalledVersionsLoaded(Result<Vec<InstalledVersion>, String>),
    ShowReleasesUpdated(bool),
    ShowSnapshotsUpdated(bool),
    StartGameRequest,
    MinecraftLaunched(Result<Option<InstalledVersion>, String>),
    LogLineReceived(String),
    GameClosed,
    NoOp,
}

impl AppState {
    fn init() -> (Self, Task<Message>) {
        let fetch_versions = Task::perform(
            async move {
                let versions = downloader::get_versions().await.map(|v| v.to_vec()).map_err(|e| e.to_string())?;

                Ok(versions)
            },
            Message::MinecraftVersionsLoaded,
        );

        let load_installed_versions = Task::perform(
            async {
                let versions = installed_versions::get_installed_versions_from_disk()
                    .await
                    .map_err(|e| format!("failed getting versions from disk: {}", e));

                versions
            },
            Message::InstalledVersionsLoaded,
        );

        (AppState::default(), Task::batch([fetch_versions, load_installed_versions]))
    }

    pub async fn run_game_and_stream_logs(
        version: ManifestVersion,
        mut output: iced::futures::channel::mpsc::Sender<Message>,
    ) {
        match launcher::play(&version).await {
            Ok(mut play_result) => {
                let _ = output.send(Message::MinecraftLaunched(Ok(play_result.new_installation))).await;

                if let Some(stdout) = play_result.child.stdout.take() {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let _ = output.send(Message::LogLineReceived(line)).await;
                    }
                }

                let _ = play_result.child.wait().await;
                let _ = output.send(Message::GameClosed).await;
            }
            Err(e) => {
                let _ = output.send(Message::MinecraftLaunched(Err(e.to_string()))).await;
            }
        }
    }

    fn refresh_filtered_versions(&mut self) {
        if let Ok(versions) = &self.all_versions {
            self.filtered_versions = versions
                .iter()
                .filter(|v| match v.version_type {
                    VersionType::Release => self.show_releases,
                    VersionType::Snapshot => self.show_snapshots,
                    VersionType::Unknown => true,
                })
                .cloned()
                .collect()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VersionChanged(version) => {
                self.selected_version = Some(version);
                Task::none()
            }
            Message::MinecraftVersionsLoaded(versions) => {
                self.all_versions = versions;
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::ShowReleasesUpdated(show) => {
                self.show_releases = show;
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::ShowSnapshotsUpdated(show) => {
                self.show_snapshots = show;
                self.refresh_filtered_versions();
                Task::none()
            }
            Message::StartGameRequest => {
                let Some(version) = self.selected_version.clone() else {
                    return Task::none();
                };

                self.playing = true;
                Task::run(
                    stream::channel(100, move |output| AppState::run_game_and_stream_logs(version, output)),
                    |msg| msg,
                )
            }
            Message::InstalledVersionsLoaded(versions) => {
                if let Ok(versions) = versions {
                    self.installed_versions = versions;
                } else if let Err(e) = versions {
                    self.logs.push(format!("Failed getting installed versions: {}", e));
                }
                Task::none()
            }
            Message::MinecraftLaunched(installed_version) => match installed_version {
                Ok(Some(version)) => {
                    self.installed_versions.push(version);
                    let to_dump = self.installed_versions.clone();

                    Task::perform(
                        async move {
                            let _ = installed_versions::dump_installed_versions(to_dump).await;
                        },
                        |_| Message::NoOp,
                    )
                }
                _ => Task::none(),
            },
            Message::GameClosed => {
                self.playing = false;
                Task::none()
            }
            Message::LogLineReceived(line) => {
                if self.logs.len() > MAX_LOGS_LINES {
                    self.logs.clear();
                }
                self.logs.push(line);
                Task::none()
            }
            Message::NoOp => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let status_text = match &self.all_versions {
            Ok(v) if v.is_empty() => text("Loading version information, hang tight!").color(color!(255, 255, 0)),
            Ok(_) => {
                if let Some(version) = &self.selected_version {
                    let installed_version = &self.installed_versions.iter().find(|v| v.version == version.version_id);
                    if installed_version.is_some() {
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
            pick_list(self.filtered_versions.as_slice(), self.selected_version.clone(), Message::VersionChanged)
                .placeholder("Select version");

        let mut play_button = button("Play!");
        if self.selected_version.is_some() && !self.playing {
            play_button = play_button.on_press(Message::StartGameRequest);
        }

        container(
            column![
                dropdown,
                status_text,
                checkbox(self.show_releases).label("Show releases").on_toggle(Message::ShowReleasesUpdated),
                checkbox(self.show_snapshots).label("Show snapshots").on_toggle(Message::ShowSnapshotsUpdated),
                play_button,
                scrollable(column(self.logs.iter().map(|line| text(line).into())))
                    .spacing(5)
                    .height(Length::Fill)
                    .width(Length::Fill)
            ]
            .spacing(10)
            .align_x(Center),
        )
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
