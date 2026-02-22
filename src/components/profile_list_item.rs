use dioxus::prelude::*;

use crate::profiles::store::ProfileStore;

#[component]
fn BaseProfileListItem(text: String, onclick: EventHandler<Event<MouseData>>, selected: ReadSignal<bool>) -> Element {
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
pub fn ProfileListItem(profile_name: ReadSignal<String>) -> Element {
    let mut profile_store = use_context::<ProfileStore>();
    let is_selected =
        use_memo(move || profile_store.selected_profile_name.read().as_deref() == Some(profile_name.read().as_str()));

    rsx! {
        BaseProfileListItem {
            text: "{profile_name}",
            onclick: move |_| {
                profile_store.selected_profile_name.set(Some(profile_name.read().to_owned()));
            },
            selected: is_selected,
        }
    }
}

#[component]
pub fn ProfileListFailed() -> Element {
    let is_selected = use_signal(|| false);

    rsx! {
        BaseProfileListItem {
            text: "Failed getting version data!",
            onclick: |_| {},
            selected: is_selected,
        }
    }
}
