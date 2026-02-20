use dioxus::prelude::*;

use crate::{
    components::version_list_item::{VersionListAddItem, VersionListFailed, VersionListItem},
    launcher,
};

#[component]
pub fn VersionListSidebar() -> Element {
    let installed = use_resource(move || async { launcher::instances::get_installed().await });
    let version_list = use_memo(move || match &*installed.read_unchecked() {
        Some(Ok(versions)) => versions
            .iter()
            .map(|v| rsx!(VersionListItem { version_id: v.clone() }))
            .chain([rsx!(VersionListAddItem {})])
            .collect::<Vec<_>>(),
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
            class: "w-1/3 flex flex-col px-8 py-4 bg-white/5 rounded-xl h-11/12",
            ul {
                class: "flex flex-col gap-2",
                {version_list().into_iter()}
            }
        }
    }
}
