use dioxus::prelude::*;

use crate::components::nelius_button::{NeliusButton, NeliusButtonStyle};

const PLAY_ASSET: Asset = asset!("/assets/graphical/play_button.svg");
const KILL_ASSET: Asset = asset!("/assets/graphical/kill.svg");
const UNINSTALL_ASSET: Asset = asset!("/assets/graphical/uninstall.svg");

#[component]
pub fn ProfileManagementBox() -> Element {
    rsx! {
        div {
            class: "w-full h-full flex justify-center",
            div {
                class: "w-1/2 flex flex-row justify-center gap-4",
                NeliusButton {
                    text: "Play",
                    style: NeliusButtonStyle::Safe,
                    icon: PLAY_ASSET,
                    onclick: move |_| {},
                    disabled: false
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
