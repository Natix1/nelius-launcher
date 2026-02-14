use anyhow::{Context, Ok};
use futures::StreamExt;
use serde_json::Value;
use std::{env, fmt, path::PathBuf};
use tokio::{fs, io::AsyncWriteExt, sync::OnceCell};

use crate::{
    launcher::{InstallationMetadata, METADATA_FILENAME},
    reqwest_global_client::get_reqwest_client,
};

const CONCURRENT_DOWNLOADS_LIMIT: usize = 32;
const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net";

static MANIFEST: OnceCell<Manifest> = OnceCell::const_new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionType {
    Snapshot,
    Release,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MinecraftCompatibleOS {
    Linux,
    OSX,
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

impl fmt::Display for ManifestVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub library_name: String,
    pub specific_os: Option<MinecraftCompatibleOS>,
    pub download_path: String,
    pub download_url: String,
}

impl Library {
    pub fn is_needed_for_this_os(&self) -> bool {
        match self.specific_os {
            Some(MinecraftCompatibleOS::Linux) => env::consts::OS == "linux",
            Some(MinecraftCompatibleOS::OSX) => env::consts::OS == "macos",
            Some(MinecraftCompatibleOS::Windows) => env::consts::OS == "windows",

            _ => true,
        }
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

async fn get_manifest() -> anyhow::Result<&'static Manifest> {
    MANIFEST
        .get_or_try_init(async || {
            let response: Value = get_reqwest_client().get(MANIFEST_URL.to_string()).send().await?.json().await?;

            let raw_versions =
                response["versions"].as_array().ok_or_else(|| anyhow::anyhow!("Versions field is not an array"))?;

            let mut versions = Vec::with_capacity(raw_versions.len());

            for version in raw_versions {
                let version_type = match version["type"].as_str().context("bad json: couldn't parse version type")? {
                    "release" => VersionType::Release,
                    "snapshot" => VersionType::Snapshot,

                    _ => VersionType::Unknown,
                };

                versions.push(ManifestVersion {
                    version_id: version["id"].as_str().context("bad json: couldn't parse version id")?.to_string(),
                    version_type: version_type,
                    details_url: version["url"]
                        .as_str()
                        .context("bad json: couldn't parse version download url")?
                        .to_string(),
                });
            }

            Ok(Manifest {
                latest_release: response["latest"]["release"]
                    .as_str()
                    .context("bad json: couldn't find latest release")?
                    .to_string(),
                latest_snapshot: response["latest"]["snapshot"]
                    .as_str()
                    .context("bad json: couldn't find latest snapshot")?
                    .to_string(),
                versions: versions,
            })
        })
        .await?;

    Ok(MANIFEST.get().unwrap())
}

pub async fn get_versions() -> anyhow::Result<&'static [ManifestVersion]> {
    let manifest = get_manifest().await?;
    return Ok(&manifest.versions);
}

pub async fn get_version_data(version_id: String) -> anyhow::Result<VersionData> {
    let manifest = get_manifest().await?;
    let target = manifest
        .versions
        .iter()
        .find(|version| version.version_id.to_lowercase().trim() == version_id.to_lowercase().trim());

    let Some(target) = target else {
        anyhow::bail!("The version {} does not exist", version_id);
    };

    let response: Value = get_reqwest_client()
        .get(&target.details_url)
        .send()
        .await
        .context("network error")?
        .json()
        .await
        .context("bad json: couldn't parse details response into json")?;

    let raw_libraries = response["libraries"].as_array().context("bad json: couldn't parse libraries into json")?;
    let mut libraries = Vec::with_capacity(raw_libraries.len());

    for library in raw_libraries {
        let mut system: Option<MinecraftCompatibleOS> = None;
        let raw_rules = match library["rules"].as_array() {
            Some(v) => v,
            None => &vec![],
        };

        for rule in raw_rules {
            let raw_system = rule["os"]["name"].as_str().unwrap_or("unknown");
            system = match raw_system {
                "linux" => Some(MinecraftCompatibleOS::Linux),
                "osx" => Some(MinecraftCompatibleOS::OSX),
                "windows" => Some(MinecraftCompatibleOS::Windows),

                _ => None,
            };

            if system.is_some() {
                // we dont need to keep iterating via rules; we got what we needed
                break;
            }
        }

        libraries.push(Library {
            library_name: library["name"].as_str().context("bad json: couldn't parse library name")?.to_string(),
            specific_os: system,
            download_path: library["downloads"]["artifact"]["path"]
                .as_str()
                .context("bad json: couldn't parse download path")?
                .to_string(),
            download_url: library["downloads"]["artifact"]["url"]
                .as_str()
                .context("bad json: couldn't parse download url")?
                .to_string(),
        });
    }

    Ok(VersionData {
        version_id: response["id"].as_str().context("bad json: couldnt't parse version id")?.to_string(),
        asset_index_download_url: response["assetIndex"]["url"]
            .as_str()
            .context("bad json: couldn't parse asset index download url")?
            .to_string(),
        asset_index_id: response["assetIndex"]["id"]
            .as_str()
            .context("bad json: couldn't parse asset index id")?
            .to_string(),
        client_jar_download_url: response["downloads"]["client"]["url"]
            .as_str()
            .context("bad json: couldn't parse asset index client download url")?
            .to_string(),
        main_class: response["mainClass"]
            .as_str()
            .context("bad json: couldn't parse asset indexx mainclass")?
            .to_string(),
        libraries: libraries,
    })
}

pub async fn install_minecraft(version: &VersionData, directory: &PathBuf) -> anyhow::Result<(), anyhow::Error> {
    fs::create_dir_all(directory.as_path()).await?;

    let mut classpath_relative: Vec<String> = Vec::with_capacity(256);
    let client_jar_directory = directory.join("client.jar");
    let libraries_directory = directory.join("libraries");
    let assets_directory = directory.join("assets");

    let objects_directory = assets_directory.join("objects");
    let indexes_directory = assets_directory.join("indexes");
    let mut asset_index_path = indexes_directory.join(&version.asset_index_id);
    asset_index_path.set_extension("json");

    fs::create_dir_all(&libraries_directory).await?;
    fs::create_dir_all(&objects_directory).await?;
    fs::create_dir_all(&indexes_directory).await?;

    let libraries = version.get_os_required_libraries();
    let asset_index = get_reqwest_client().get(&version.asset_index_download_url).send().await?.bytes().await?.to_vec();

    let mut file = fs::File::create(asset_index_path).await?;
    file.write_all(&asset_index).await?;

    let mut files_to_download: Vec<(String, PathBuf)> = Vec::with_capacity(1024);
    files_to_download.push((version.client_jar_download_url.clone(), client_jar_directory.clone()));

    for library in libraries {
        let full_download_path = libraries_directory.join(&library.download_path);
        files_to_download.push((library.download_url.clone(), full_download_path.clone()));
        classpath_relative.push(full_download_path.clone().to_string_lossy().to_owned().to_string());
    }

    let decoded_asset_index: Value = serde_json::from_slice(&asset_index)?;
    let asset_index_objects = decoded_asset_index["objects"].as_object().context("bad json: couldn't parse objects")?;
    for (_, object_data) in asset_index_objects {
        let hash = object_data["hash"].as_str().context("bad json: couldn't parse hash")?;
        let hash_first_two = hash[0..2].to_string();
        let uri = format!("{}/{}/{}", RESOURCES_BASE_URL, hash_first_two, hash);
        let path = objects_directory.join(hash_first_two).join(hash);

        files_to_download.push((uri, path));
    }

    let results = futures::stream::iter(files_to_download)
        .map(|(url, path)| {
            let client = get_reqwest_client();
            async move {
                let parent_path = path.parent().context("failed getting parent path")?;
                fs::create_dir_all(parent_path).await?;
                let response = client.get(url).send().await?;
                let bytes = response.bytes().await?;
                fs::write(path, bytes).await?;
                Ok(())
            }
        })
        .buffer_unordered(CONCURRENT_DOWNLOADS_LIMIT)
        .collect::<Vec<_>>()
        .await;

    for res in results {
        res?;
    }

    let installation_metadata = InstallationMetadata {
        main_class: version.main_class.clone(),
        version: version.version_id.clone(),
        asset_index_id: version.asset_index_id.clone(),
        client_jar_relative: client_jar_directory.strip_prefix(directory)?.to_string_lossy().into_owned(),
        classpath_relative: classpath_relative,
    };

    let encoded_installation_metadata = serde_json::to_string(&installation_metadata)?;
    let mut file = fs::File::create(directory.join(METADATA_FILENAME)).await?;
    file.write_all(encoded_installation_metadata.as_bytes()).await?;

    Ok(())
}
