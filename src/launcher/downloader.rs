use anyhow::{Context, Ok};
use futures::StreamExt;
use serde_json::Value;
use std::{env, fmt, path::PathBuf};
use tokio::{fs, io::AsyncWriteExt, sync::OnceCell};
use zip::ZipArchive;

use crate::launcher::{
    boot::{InstallationMetadata, METADATA_FILENAME},
    requests::reqwest_global_client::get_reqwest_client,
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
    pub is_native: bool,
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

impl Library {
    fn from_json(library: &Value) -> Result<Option<Library>, anyhow::Error> {
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
                "osx" => Some(MinecraftCompatibleOS::OSX),
                "windows" => Some(MinecraftCompatibleOS::Windows),

                _ => None,
            };
        }

        Ok(Some(Library {
            library_name: library["name"].as_str().context("bad json: couldn't parse library name")?.to_string(),
            specific_os: system,
            download_path: download_path,
            download_url: download_url,
            is_native: is_native,
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

    for raw_library in raw_libraries {
        let library = Library::from_json(raw_library)?;
        if let Some(library) = library {
            libraries.push(library);
        }
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

fn extract_natives(jar_path: &PathBuf, natives_dir: &PathBuf) -> anyhow::Result<()> {
    let file = std::fs::File::open(&jar_path)?;
    let mut archive = ZipArchive::new(file)?;

    if !natives_dir.exists() {
        std::fs::create_dir_all(&natives_dir)?;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();
        if name.ends_with(".dll") || name.ends_with(".so") || name.ends_with(".dylib") {
            if let Some(filename) = file.enclosed_name().and_then(|p| p.file_name().map(|f| f.to_owned())) {
                let out_path = natives_dir.join(filename);

                let mut out_file = std::fs::File::create(&out_path)?;
                std::io::copy(&mut file, &mut out_file)?;
            }
        }
    }

    Ok(())
}

struct ToDownload {
    download_uri: String,
    download_path: PathBuf,
    is_native: bool,
}

pub async fn install_minecraft(version: &VersionData, directory: &PathBuf) -> anyhow::Result<(), anyhow::Error> {
    fs::create_dir_all(directory.as_path()).await?;

    let mut classpath_relative: Vec<String> = Vec::with_capacity(256);

    let client_jar_directory =
        directory.join("versions").join(&version.version_id).join(format!("{}.jar", &version.version_id));
    let libraries_directory = directory.join("libraries");
    let assets_directory = directory.join("assets");

    let objects_directory = assets_directory.join("objects");
    let indexes_directory = assets_directory.join("indexes");
    let natives_directory = directory.join("natives");
    let mut asset_index_path = indexes_directory.join(&version.asset_index_id);
    asset_index_path.set_extension("json");

    fs::create_dir_all(&libraries_directory).await?;
    fs::create_dir_all(&objects_directory).await?;
    fs::create_dir_all(&indexes_directory).await?;
    fs::create_dir_all(client_jar_directory.parent().context("failed constructing client.jar directory")?).await?;
    fs::create_dir_all(&natives_directory).await?;

    let libraries = version.get_os_required_libraries();
    let asset_index = get_reqwest_client().get(&version.asset_index_download_url).send().await?.bytes().await?.to_vec();

    let mut file = fs::File::create(asset_index_path).await?;
    file.write_all(&asset_index).await?;

    let mut files_to_download: Vec<ToDownload> = Vec::with_capacity(1024);
    files_to_download.push(ToDownload {
        download_uri: version.client_jar_download_url.clone(),
        download_path: client_jar_directory.clone(),
        is_native: false,
    });

    for library in libraries {
        let full_download_path = libraries_directory.join(&library.download_path);
        files_to_download.push(ToDownload {
            download_uri: library.download_url.clone(),
            download_path: full_download_path.clone(),
            is_native: library.is_native,
        });

        if !library.is_native {
            classpath_relative.push(full_download_path.clone().to_string_lossy().to_owned().to_string());
        }
    }

    let decoded_asset_index: Value = serde_json::from_slice(&asset_index)?;
    let asset_index_objects = decoded_asset_index["objects"].as_object().context("bad json: couldn't parse objects")?;
    for (_, object_data) in asset_index_objects {
        let hash = object_data["hash"].as_str().context("bad json: couldn't parse hash")?;
        let hash_first_two = hash[0..2].to_string();
        let uri = format!("{}/{}/{}", RESOURCES_BASE_URL, hash_first_two, hash);
        let path = objects_directory.join(hash_first_two).join(hash);

        files_to_download.push(ToDownload { download_uri: uri, download_path: path, is_native: false });
    }

    let results = futures::stream::iter(files_to_download)
        .map(|task| {
            let client = get_reqwest_client();
            let natives_directory = natives_directory.clone();

            async move {
                let parent_path = task.download_path.parent().context("failed getting parent path")?;
                fs::create_dir_all(parent_path).await?;
                let response = client.get(task.download_uri).send().await?;
                let bytes = response.bytes().await?;
                fs::write(&task.download_path, bytes).await?;

                if task.is_native {
                    tokio::task::spawn_blocking(move || extract_natives(&task.download_path, &natives_directory))
                        .await?
                        .context("failed extracting natives")?;
                }

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
