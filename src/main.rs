use std::sync::Arc;

use iced::{
    Element,
    Length::Fill,
    Task, Theme,
    widget::{button, column, container, row, text, text_input},
};

mod downloader;
mod reqwest_global_client;

#[derive(Default)]
struct NeliusLauncher {
    current_selected_version: String,
    current_install_dir: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    VersionChanged(String),
    PromptSetInstallationDirectory,
    InstallationDirectorySelected(Option<String>),
    InstallMinecraft,
    MinecraftInstallFinished(Arc<anyhow::Result<(), anyhow::Error>>),
}

fn new() -> NeliusLauncher {
    NeliusLauncher {
        current_selected_version: String::new(),
        current_install_dir: String::new(),
    }
}

fn theme(_state: &NeliusLauncher) -> Theme {
    Theme::CatppuccinFrappe
}

impl NeliusLauncher {
    pub fn view(&self) -> Element<'_, Message> {
        container(
            column![
                row![
                    text("Enter minecraft version, or leave blank for latest release:"),
                    text_input("latest", &self.current_selected_version)
                        .on_input(Message::VersionChanged),
                ]
                .spacing(10),
                row![
                    text(format!(
                        "Current instalation directory: {}",
                        if &self.current_install_dir == "" {
                            "None"
                        } else {
                            &self.current_install_dir
                        }
                    )),
                    button("Change").on_press(Message::PromptSetInstallationDirectory)
                ]
                .spacing(10),
                container(button("Install!").on_press(Message::InstallMinecraft))
                    .center_x(Fill)
                    .height(60)
            ]
            .spacing(10),
        )
        .padding(10)
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::VersionChanged(version) => {
                self.current_selected_version = version;
                Task::none()
            }

            Message::PromptSetInstallationDirectory => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|handle| handle.path().to_string_lossy().to_string())
                },
                Message::InstallationDirectorySelected,
            ),

            Message::InstallationDirectorySelected(Some(path)) => {
                self.current_install_dir = path;
                Task::none()
            }

            Message::InstallationDirectorySelected(None) => Task::none(),
            Message::InstallMinecraft => Task::perform(async {}, Message::MinecraftInstallFinished),
        }
    }
}

pub fn main() -> iced::Result {
    iced::application(new, NeliusLauncher::update, NeliusLauncher::view)
        .theme(theme)
        .run()
}
