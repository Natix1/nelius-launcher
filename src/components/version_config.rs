use dioxus::prelude::*;

use crate::{
    globals::APP_STATE,
    launcher::{self, downloader::VersionType},
};

#[component]
pub fn VersionConfig() -> Element {
    let mut versions = use_resource(|| async {
        return launcher::downloader::get_versions().await;
    });

    let filtered_versions: Memo<Vec<String>> = use_memo(move || {
        let versions = &*versions.read_unchecked();
        match versions {
            Some(Ok(versions)) => versions
                .iter()
                .filter(|v| {
                    if v.version_type == VersionType::Release && !APP_STATE().persistent.show_releases {
                        return false;
                    }

                    if v.version_type == VersionType::Snapshot && !APP_STATE().persistent.show_snapshots {
                        return false;
                    }

                    return true;
                })
                .map(|v| v.version_id.clone())
                .collect(),
            Some(Err(e)) => {
                eprintln!("Something went wrong while fetching available versions: {e}");
                vec![format!("Couldn't get available versions").to_string()]
            }
            None => vec!["Loading...".to_string()],
        }
    });

    rsx! {
        div {
            class: "flex flex-col items-center space-y-4 p-5",
            div {
                class: "flex flex-col items-center p-2",
                label {
                    input {
                        class: "mr-4",
                        type: "checkbox",
                        checked: APP_STATE().persistent.show_releases,
                        onchange: move |_| {
                            APP_STATE.write().persistent.show_releases = !APP_STATE()
                                .persistent
                                .show_releases
                        },
                    },
                "Show releases"
            }
            label {
                    input {
                        class: "mr-4",
                        type: "checkbox",
                        checked: APP_STATE().persistent.show_snapshots,
                        onchange: move |_| {
                        APP_STATE.write().persistent.show_snapshots = !APP_STATE()
                            .persistent
                            .show_snapshots
                        },
                    },
                    "Show snapshots"
                }
            }
            select {
                class: "w-1/4 text-black",
                {
                    {filtered_versions().iter().map(|v| rsx!(option { key: "{v}", "{v}" }))}
                }
            }
        }
    }
}
