use dioxus::prelude::*;

use crate::{
    components::{profile_logs_box::ProfileLogsBox, profile_management_box::ProfileManagementBox},
    profiles::store::ProfileStore,
};

#[component]
pub fn ProfileDetailsPane() -> Element {
    let profile_store = use_context::<ProfileStore>();
    if let Some(selected_name) = &*profile_store.selected_profile_name.read() {
        rsx! {
            div {
                class: "w-5/6 flex flex-col bg-white/5 rounded-xl h-full p-4 gap-3 ring-1 ring-white/15",
                h1 {
                    class: "w-full text-center text-4xl font-bold",
                    {format!("{}", selected_name)}
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
    } else {
        rsx! {
            div {
                class: "w-full h-full flex justify-center items-center p-10",
                p {
                    class: "text-3xl w-full text-center",
                    "Select a profile from your left to get started!"
                }
            }
        }
    }
}
