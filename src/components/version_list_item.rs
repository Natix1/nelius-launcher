use dioxus::prelude::*;

#[component]
fn BaseVersionListItem(text: String, onclick: EventHandler<Event<MouseData>>, selected: ReadSignal<bool>) -> Element {
    rsx! {
        li {
            class: "cursor-pointer transition-transform w-full rounded-sm bg-white/10 hover:bg-white/20 ring-1 ring-white/15",
            id: if selected() { "selected-item" } else { "" },
            onclick: move |e| onclick.call(e),
            div {
                class: "flex flex-row gap-5 items-center p-1",
                p {
                    class: "text-center w-full",
                    "{text}"
                }
            }
        }
    }
}

#[component]
pub fn VersionListItem(version_id: String) -> Element {
    rsx! {
        BaseVersionListItem {
            text: "{version_id}",
            onclick: move |_| {},
            selected: false,
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
        }
    }
}
