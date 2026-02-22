use dioxus::prelude::*;

use crate::components::{profile_logs_box::ProfileLogsBox, profile_management_box::ProfileManagementBox};

#[component]
pub fn ProfileDetailsPane() -> Element {
    rsx! {
        div {
            class: "w-5/6 flex flex-col bg-white/5 rounded-xl h-full p-4 gap-3 ring-1 ring-white/15",
            h1 {
                class: "w-full text-center text-4xl font-bold",
            }
            div {
                class: "w-full flex-shrink-0",
                ProfileManagementBox {  }
            }
            div {
                class: "w-full flex-1 min-h-0 overflow-auto",
                ProfileLogsBox { }
            }
        }
    }
}
