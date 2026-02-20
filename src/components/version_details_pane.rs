use dioxus::prelude::*;

use crate::globals::APP_STATE;

#[component]
pub fn VersionDetailsPane() -> Element {
    rsx! {
        div {
            class: "w-5/6 flex flex-col bg-white/5 rounded-xl h-full p-4",
            h1 {
                class: "w-full text-center text-4xl font-bold",
                {APP_STATE().persistent.selected_version}
            }
        }
    }
}
