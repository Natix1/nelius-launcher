use dioxus::prelude::*;

use crate::{
    components::nelius_button::{NeliusButton, NeliusButtonStyle},
    launcher::{self, downloader},
    profiles::store::ProfileStore,
};

const DONE_ASSET: Asset = asset!("assets/graphical/done.svg");

fn validate_profile_name(name: String, profiles: &ProfileStore) -> bool {
    let len = name.len() as i32;

    if profiles.exists_with_name(&name) {
        return false;
    }

    if len < 1 {
        return false;
    }

    return true;
}

#[component]
fn ProfileNameInput(name: Signal<String>) -> Element {
    let store = use_context::<ProfileStore>();

    rsx! {
        div {
            class: "flex flex-col w-full items-center gap-1",
            input {
                class: "w-1/2 text-center ring-1 ring-white/15 rounded-sm p-1",
                r#type: "text",
                placeholder: "Give it a cool name",
                value: name(),
                oninput: move |e| {
                    name.set(e.value());
                }
            }
            {
                if validate_profile_name(name(), &store) {
                    rsx! {
                        p {
                            class: "text-green-600 text-md opacity-75",
                            "Name available. Nice!"
                        }
                    }
                } else {
                    rsx! {
                        p {
                            class: "text-red-600 text-md opacity-75",
                            "This name is invalid. Try something longer or more unique."
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProfileNevermind(onclick: EventHandler<Event<MouseData>>) -> Element {
    rsx! {
        button {
            class: "w-full",
            onclick: move |e| {
                onclick.call(e);
            },
            p {
                class: "w-full text-right cursor-pointer",
                "Nevermind ->"
            }
        }
    }
}

#[component]
fn ProfileGameVersionDropdownOption(value: String, #[props(default = false)] disabled: bool) -> Element {
    rsx! {
        option {
            disabled: disabled,
            class: "text-white bg-black text-center",
            value: "{value}",
            "{value}"
        }
    }
}

#[component]
fn ProfileGameVersionDropdown(game_version: Signal<String>) -> Element {
    let versions = use_resource(|| async { launcher::downloader::get_versions().await });

    rsx! {
        select {
            class: "w-1/2 bg-black text-white rounded-lg",
            onchange: move |evt| {
                game_version.set(evt.value());
            },

            match &*versions.read() {
                Some(Ok(list)) => {
                    rsx! {
                        ProfileGameVersionDropdownOption { disabled: true, value: "Select a version..." }
                        for v in list {
                            ProfileGameVersionDropdownOption { value: v.version_id.clone() }
                        }
                    }
                }
                Some(Err(_)) => {
                    rsx! { ProfileGameVersionDropdownOption { value: "Error loading versions" } }
                }
                None => {
                    rsx! { ProfileGameVersionDropdownOption { value: "Loading..." } }
                }
            }
        }
    }
}

#[component]
pub fn JavaBinarySelector(java_binary: Signal<String>) -> Element {
    rsx! {
        div {
            class: "flex flex-1 flex-col justify-center items-center gap-3 bg-white/10 rounded-lg p-2 text-sm",
            button {
                class: "w-full h-full cursor-pointer",
                onclick: move |_| async move {
                    let path = rfd::AsyncFileDialog::new()
                        .pick_file()
                        .await;

                    if let Some(path) = path {
                        java_binary.set(path.path().to_string_lossy().to_string());
                    }
                },
                {
                    if java_binary() == "" {
                        rsx! {
                            p {
                                "Select java binary"
                            }
                        }
                    } else {
                        rsx! {
                            p {
                                "Java binary: {java_binary()}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ProfileAdd() -> Element {
    let store = use_context::<ProfileStore>();
    let name = use_signal(|| String::new());
    let game_version = use_signal(|| String::from(""));
    let java_binary = use_signal(|| String::from(""));

    let all_valid = use_memo(move || {
        if !validate_profile_name(name(), &store) {
            return false;
        }

        if game_version() == "" {
            return false;
        }

        if java_binary() == "" {
            return false;
        }

        return true;
    });

    rsx! {
        div {
            class: "fixed top-0 left-0 bg-black/80",
            style: "width: 100vw; height: 100vh; z-index: 9999;",

            div {
                class: "w-full h-full flex justify-center items-center",
                onclick: move |e| {
                    e.stop_propagation();
                },
                div {
                    class: "w-1/2 h-auto max-h-[90vh] rounded-lg ring-2 ring-white/15 p-2",
                    id: "card",

                    ProfileNevermind {
                        onclick: |_| {
                            // TODO: Exit
                        }
                    },
                    div {
                        class: "flex flex-col items-center justify-between w-full h-full p-4 gap-4",

                        h1 {
                            class: "w-full text-center font-bold text-3xl",
                            "Add a profile..."
                        }
                        hr {
                            class: "w-3/4 opacity-50 rounded-lg border-1"
                        }

                        ProfileNameInput { name: name }
                        ProfileGameVersionDropdown { game_version: game_version }
                        JavaBinarySelector { java_binary: java_binary }

                        div {
                            class: "w-1/2 h-1/6 flex items-center justify-center",
                            NeliusButton {
                                text: "Submit",
                                disabled: !all_valid(),
                                icon: DONE_ASSET,
                                onclick: |_| {},
                                style: NeliusButtonStyle::Safe
                            }
                        }
                    }
                }
            }
        }
    }
}
