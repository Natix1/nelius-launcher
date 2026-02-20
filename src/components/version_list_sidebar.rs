use dioxus::prelude::*;

use crate::{
    components::version_list_item::{VersionListFailed, VersionListItem},
    launcher,
};

#[component]
pub fn VersionListSidebar() -> Element {
    let installed = use_resource(move || async { launcher::instances::get_installed().await });
    let version_list = use_memo(move || match installed.read_unchecked().as_ref() {
        Some(Ok(versions)) => {
            versions.iter().map(|v| rsx!(VersionListItem { version_id: v.clone() })).collect::<Vec<_>>()
        }
        Some(Err(e)) => {
            eprintln!("Failed getting versions: {e}");
            vec![rsx!(VersionListFailed {})]
        }
        None => {
            vec![]
        }
    });

    rsx! {
        div {
            class: "w-1/6 px-3 py-4 bg-white/5 rounded-xl h-full",
            ul {
                class: "flex flex-col gap-1 w-full items-center",
                {version_list().into_iter()}
            }
        }
    }
}
