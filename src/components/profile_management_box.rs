use dioxus::prelude::*;

use crate::{
    components::nelius_button::{NeliusButton, NeliusButtonStyle},
    profiles::store::ProfileStore,
};

const PLAY_ASSET: Asset = asset!("/assets/graphical/play_button.svg");
const KILL_ASSET: Asset = asset!("/assets/graphical/kill.svg");
const UNINSTALL_ASSET: Asset = asset!("/assets/graphical/uninstall.svg");

#[component]
pub fn ProfileManagementBox() -> Element {
    let profile_store = use_context::<ProfileStore>();
    let mut processing = use_signal(|| false);

    rsx! {
        div {
            class: "w-full h-full flex justify-center",
            div {
                class: "w-1/2 flex flex-row justify-center gap-4",
                NeliusButton {
                    text: "Play",
                    style: NeliusButtonStyle::Safe,
                    icon: PLAY_ASSET,
                    onclick: move |_| {
                        if processing() {
                            return;
                        }

                        processing.set(true);

                        if let Some(selected_profile) = profile_store.selected_profile_name.read().to_owned() {
                            let profile = profile_store.peek(selected_profile);
                            if let Some(profile) = profile {
                                let mut profile = profile.read().cloned();
                                spawn(async move {
                                   match profile.launch_or_install().await {
                                       Ok(_) => {},
                                       Err(e) => {
                                           eprintln!("{e}");
                                           return;
                                       }
                                   }
                                });
                            }
                        }
                    },
                    disabled: processing()
                }
                NeliusButton {
                    text: "Uninstall",
                    style: NeliusButtonStyle::Danger,
                    icon: UNINSTALL_ASSET,
                    onclick: move |_| {},
                    disabled: false
                }
                NeliusButton {
                    text: "Kill",
                    style: NeliusButtonStyle::Danger,
                    icon: KILL_ASSET,
                    onclick: move |_| {},
                    disabled: true
                }
            }
        }
    }
}
