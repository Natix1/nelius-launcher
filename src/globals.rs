use dioxus::prelude::*;

use crate::state::state::AppState;

pub const MAX_LOGS_LENGTH: usize = 5000;
pub static APP_STATE: GlobalSignal<AppState> = Signal::global(|| AppState::default());
