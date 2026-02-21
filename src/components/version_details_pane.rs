use dioxus::prelude::*;

use crate::{
    components::{game_logs_box::GameLogsBox, version_management_box::VersionManagementBox},
    globals::APP_STATE,
};

#[component]
pub fn VersionDetailsPane() -> Element {
    rsx! {
        div {
            class: "w-5/6 flex flex-col bg-white/5 rounded-xl h-full p-4 gap-3 ring-1 ring-white/15",
            h1 {
                class: "w-full text-center text-4xl font-bold",
                {APP_STATE().persistent.selected_version}
            }
            div {
                class: "w-full flex-shrink-0",
                VersionManagementBox {  }
            }
            div {
                class: "w-full flex-1 min-h-0 overflow-auto",
                GameLogsBox { logs: "".to_string(), onlogsclear: |_| {} }
            }
        }
    }
}
