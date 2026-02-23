use std::env;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionType {
    Snapshot,
    Release,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinecraftCompatibleOS {
    Linux,
    Osx,
    Windows,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub latest_release: String,
    pub latest_snapshot: String,

    pub versions: Vec<ManifestVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestVersion {
    pub version_id: String,
    pub version_type: VersionType,
    pub details_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub library_name: String,
    pub specific_os: Option<MinecraftCompatibleOS>,
    pub download_path: String,
    pub download_url: String,
    pub is_native: bool,
}

impl Library {
    pub fn is_needed_for_this_os(&self) -> bool {
        match self.specific_os {
            Some(MinecraftCompatibleOS::Linux) => env::consts::OS == "linux",
            Some(MinecraftCompatibleOS::Osx) => env::consts::OS == "macos",
            Some(MinecraftCompatibleOS::Windows) => env::consts::OS == "windows",

            _ => true,
        }
    }

    pub fn from_json(library: &Value) -> Result<Option<Library>, anyhow::Error> {
        let mut system: Option<MinecraftCompatibleOS> = None;
        let raw_rules = match library["rules"].as_array() {
            Some(v) => v,
            None => &vec![],
        };
        let raw_classifiers = library["downloads"]["classifiers"].as_object();
        let download_url: String;
        let download_path: String;
        let mut is_native = false;

        if let Some(classifiers) = raw_classifiers {
            let native_key = match std::env::consts::OS {
                "windows" => "natives-windows",
                "macos" => "natives-osx",
                "linux" => "natives-linux",
                _ => {
                    anyhow::bail!("we're on an unsupported os");
                }
            };

            if let Some(native) = classifiers.get(native_key) {
                download_url =
                    native["url"].as_str().context("bad json: download url not listed in classifier")?.to_string();
                download_path =
                    native["path"].as_str().context("bad json: download path not listed in classifier")?.to_string();

                is_native = true;
            } else {
                return Ok(None);
            }
        } else {
            download_url = library["downloads"]["artifact"]["url"]
                .as_str()
                .context("couldn't find download url in both classifier and artifact")?
                .to_string();

            download_path = library["downloads"]["artifact"]["path"]
                .as_str()
                .context("couldn't find download path in both classifier and artifact")?
                .to_string();
        }

        for rule in raw_rules {
            let raw_system = rule["os"]["name"].as_str().unwrap_or("unknown");
            let action = rule["action"].as_str().context("couldn't find action for rule")?;
            if action != "allow" {
                continue;
            }

            system = match raw_system {
                "linux" => Some(MinecraftCompatibleOS::Linux),
                "osx" => Some(MinecraftCompatibleOS::Osx),
                "windows" => Some(MinecraftCompatibleOS::Windows),

                _ => None,
            };
        }

        Ok(Some(Library {
            library_name: library["name"].as_str().context("bad json: couldn't parse library name")?.to_string(),
            specific_os: system,
            download_path,
            download_url,
            is_native,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionData {
    pub version_id: String,
    pub asset_index_download_url: String,
    pub asset_index_id: String,
    pub client_jar_download_url: String,
    pub main_class: String,
    pub libraries: Vec<Library>,
}

impl VersionData {
    pub fn get_os_required_libraries(&self) -> Vec<&Library> {
        self.libraries.iter().filter(|lib| lib.is_needed_for_this_os()).collect()
    }
}
