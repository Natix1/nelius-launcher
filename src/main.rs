#![windows_subsystem = "windows"]

use dioxus::prelude::*;

use crate::{
    components::{
        profile_add::ProfileAdd, profile_details_pane::ProfileDetailsPane, profile_list_sidebar::ProfileListSidebar,
    },
    profiles::store::ProfileStore,
};

mod components;
mod launcher;
mod profiles;
mod reqwest_client;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const DIOXUS_COMPONENTS_CSS: Asset = asset!("/assets/dx-components-theme.css");

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Nelius launcher")
                .with_decorations(true)
                .with_transparent(true),
        )
        .with_menu(None);

    dioxus::LaunchBuilder::new().with_cfg(desktop!(config)).launch(App);
}

#[component]
fn App() -> Element {
    let store = ProfileStore::load();
    use_context_provider(move || store);

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: DIOXUS_COMPONENTS_CSS }

        div {
            class: "rounded-3xl flex flex-row items-center w-full h-screen space-x-3 items-center pr-3 pl-3 pb-3 pt-3",
            ProfileAdd {  },
            ProfileListSidebar {  }
            ProfileDetailsPane {  }
        }
    }
}
