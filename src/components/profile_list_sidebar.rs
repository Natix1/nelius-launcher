use dioxus::prelude::*;

use crate::{
    components::{
        nelius_button::{NeliusButton, NeliusButtonStyle},
        profile_list_item::ProfileListItem,
    },
    profiles::store::ProfileStore,
};

const NEW_ASSET: Asset = asset!("/assets/graphical/new.svg");

#[component]
pub fn ProfileListSidebar(onaddbuttonpressed: EventHandler<()>) -> Element {
    let profile_store = use_context::<ProfileStore>();
    let profiles = profile_store.profiles.read();

    rsx! {
        div {
            class: "w-1/6 px-3 ring-1 ring-white/15 py-4 bg-white/5 rounded-xl h-full space-y-5 flex flex-col",
            p {
                class: "opacity-50 font-ligh text-sm text-center",
                "Profiles"
            }
            ul {
                class: "flex flex-col gap-2 w-full items-center",
                {
                    profiles.iter().map(|(_, profile)| {
                        rsx! {
                            ProfileListItem {
                                profile_name: profile.read().profile_name.clone()
                            }
                        }
                    })
                }
            }
            div {
                class: "mt-auto",
                NeliusButton {
                    text: "Add profile...",
                    style: NeliusButtonStyle::Safe,
                    icon: NEW_ASSET,
                    disabled: false,
                    onclick: move |_| {
                        onaddbuttonpressed.call(());
                    }
                }
            }
        }
    }
}
