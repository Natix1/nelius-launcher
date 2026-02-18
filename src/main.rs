#![windows_subsystem = "windows"]

use anyhow::anyhow;
use iced::Theme;

use crate::{model::state::AppState, ui::message::Message};

mod launcher;
mod model;
mod ui;

fn main() -> anyhow::Result<()> {
    let result = iced::application(AppState::init, AppState::update, AppState::view)
        .theme(Theme::GruvboxDark)
        .window_size((800.0, 600.0))
        .resizable(false)
        .run();

    println!("Until next time!");
    result.map_err(|e| anyhow!(e))
}
