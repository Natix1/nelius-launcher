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
    let mut profile_store = use_context::<ProfileStore>();
    let mut game_running = use_signal(|| false);
    let mut uninstalling = use_signal(|| false);
    let game_operations_safe = use_memo(move || !game_running() && !uninstalling());

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
                        if !game_operations_safe() {
                            return;
                        }

                        game_running.set(true);

                        if let Some(profile) = profile_store.read_selected() {
                            let mut profile = profile.read().cloned();
                            spawn(async move {
                               match profile.launch_or_install().await {
                                   Ok(_) => {},
                                   Err(e) => {
                                       eprintln!("{e}");
                                       return;
                                   }
                               }

                               game_running.set(false);
                            });
                        }
                    },
                    disabled: !game_operations_safe()
                }
                NeliusButton {
                    text: "Uninstall",
                    style: NeliusButtonStyle::Danger,
                    icon: UNINSTALL_ASSET,
                    onclick: move |_| {
                        if !game_operations_safe() {
                            return;
                        }

                        if let Some(profile) = profile_store.read_selected() {
                            let profile_name = profile.peek().profile_name.clone();
                            uninstalling.set(true);

                            spawn(async move {
                                if let Err(e) = profile_store.remove(&profile_name).await {
                                    eprintln!("Something went wrong while uninstalling the game: {e}");
                                }
                                uninstalling.set(false);
                            });
                        }
                    },
                    disabled: !game_operations_safe()
                }
                NeliusButton {
                    text: "Kill",
                    style: NeliusButtonStyle::Danger,
                    icon: KILL_ASSET,
                    onclick: move |_| {
                        if !game_running() {
                            return
                        }

                        if let Some(profile) = profile_store.read_selected() {
                            profile.peek().kill_notify.notify_one();
                        }
                    },
                    disabled: !game_running()
                }
            }
        }
    }
}
