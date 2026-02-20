#![windows_subsystem = "windows"]

use dioxus::prelude::*;

use crate::{
    components::{version_details_pane::VersionDetailsPane, version_list_sidebar::VersionListSidebar},
    globals::APP_STATE,
};

mod components;
mod globals;
mod launcher;
mod state;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const DIOXUS_COMPONENTS_CSS: Asset = asset!("/assets/dx-components-theme.css");

fn main() {
    // So we don't get a big bulky title bar. Makes the app feel more native which is what I'm going for
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("GDK_BACKEND", "x11");
        std::env::set_var("GTK_CSD", "0");
    }

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
    use_effect(move || {
        let data = APP_STATE.read().clone();
        spawn(async move {
            match data.persistent.save().await {
                Ok(_) => {
                    println!("Persistent state saved")
                }
                Err(e) => eprintln!("Auto-save failed: {e}"),
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: DIOXUS_COMPONENTS_CSS }

        div {
            class: "rounded-3xl flex flex-row items-center w-full h-screen space-x-3 items-center pr-3 pl-3 pb-3 pt-3",
            VersionListSidebar {  }
            VersionDetailsPane {  }
        }
    }
}
