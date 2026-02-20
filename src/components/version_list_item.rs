use dioxus::prelude::*;

use crate::globals::APP_STATE;

#[component]
fn BaseVersionListItem(
    text: String,
    onclick: EventHandler<Event<MouseData>>,
    selected: ReadSignal<bool>,
    show_bullet: bool,
) -> Element {
    rsx! {
        li {
            class: "hover:text-blue-400 cursor-pointer transition-colors font-semibold",
            id: if selected() { "selected-item" } else { "" },
            onclick: move |e| onclick.call(e),
            div {
                class: "flex flex-row gap-5 items-center",
                p {
                    class: "text-sm",
                    { if show_bullet { "•  " } else { "" } }
                }
                "{text}"
            }
        }
    }
}

#[component]
pub fn VersionListItem(version_id: String) -> Element {
    let id_for_memo = version_id.clone();
    let is_selected = use_memo(move || APP_STATE().persistent.selected_version.as_ref() == Some(&id_for_memo));

    rsx! {
        BaseVersionListItem {
            text: "{version_id}",
            onclick: move |_| {
                APP_STATE.write().persistent.selected_version = Some(version_id.clone());
            },
            selected: is_selected,
            show_bullet: true
        }
    }
}

#[component]
pub fn VersionListAddItem() -> Element {
    rsx! {
        li {
            class: "mt-4 mb-2 p-2 bg-blue-600 hover:bg-blue-500 text-white rounded-md
                    text-center cursor-pointer transition-all shadow-sm active:scale-95",
            onclick: move |_| {},
            "+ New Version"
        }
    }
}

#[component]
pub fn VersionListFailed() -> Element {
    let is_selected = use_signal(|| false);

    rsx! {
        BaseVersionListItem {
            text: "Failed getting version data!",
            onclick: |_| {},
            selected: is_selected,
            show_bullet: false
        }
    }
}
