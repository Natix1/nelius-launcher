use iced::{
    Alignment::Center,
    Element,
    Length::{self, Fill},
    alignment::Horizontal::{Left, Right},
    color,
    widget::{button, checkbox, column, container, pick_list, row, scrollable, text},
};

use crate::{model::state::AppState, ui::message::Message};

impl AppState {
    pub fn view(&self) -> Element<'_, Message> {
        let status_text = match &self.all_versions {
            Ok(v) if v.is_empty() => text("Loading version information, hang tight!").color(color!(255, 255, 0)),
            Ok(_) => {
                if let Some(version) = &self.persistent.selected_version {
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
            self.persistent.selected_version.clone(),
            Message::VersionChanged,
        )
        .placeholder("Select version");

        let mut play_button = button("Play!");
        if self.persistent.selected_version.is_some() && self.persistent.java_binary.is_some() && !self.game_locked {
            play_button = play_button.on_press(Message::StartGameRequest).style(button::success);
        } else {
            play_button = play_button.style(button::secondary)
        }

        container(
            column![
                dropdown,
                status_text,
                checkbox(self.persistent.show_releases).label("Show releases").on_toggle(Message::ShowReleasesUpdated),
                checkbox(self.persistent.show_snapshots)
                    .label("Show snapshots")
                    .on_toggle(Message::ShowSnapshotsUpdated),
                text(format!("Java binary: {}", self.persistent.java_binary.clone().unwrap_or("None!".to_string()))),
                button("Change Java binary").style(button::primary).on_press(Message::ChangeJavaBinaryRequested),
                row![play_button].spacing(10),
                container(
                    checkbox(self.persistent.auto_scroll)
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
