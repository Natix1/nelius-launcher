use std::path::{Path, PathBuf};

use anyhow::Context;
use futures::StreamExt;
use serde_json::Value;
use tokio::{fs, io::AsyncWriteExt, sync::OnceCell};
use zip::ZipArchive;

use crate::{
    launcher::{api_structures::*, directories, logging::log},
    profiles::ProfileInstallationData,
    reqwest_client::REQWEST_CLIENT,
};

const CONCURRENT_DOWNLOADS_LIMIT: usize = 32;
const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net";

static MANIFEST: OnceCell<Manifest> = OnceCell::const_new();

async fn get_manifest() -> anyhow::Result<&'static Manifest> {
    MANIFEST
        .get_or_try_init(async || -> anyhow::Result<Manifest, anyhow::Error> {
            println!("Fetching manifest");
            let response: Value = REQWEST_CLIENT.get(MANIFEST_URL.to_string()).send().await?.json().await?;

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
                    version_type,
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
                versions,
            })
        })
        .await?;

    Ok(MANIFEST.get().unwrap())
}

pub async fn get_versions() -> anyhow::Result<&'static [ManifestVersion]> {
    let manifest = get_manifest().await?;
    Ok(&manifest.versions)
}

pub async fn get_version_data(version_id: &String) -> anyhow::Result<VersionData> {
    let manifest = get_manifest().await?;
    let target = manifest
        .versions
        .iter()
        .find(|version| version.version_id.to_lowercase().trim() == version_id.to_lowercase().trim());

    let Some(target) = target else {
        anyhow::bail!("The version {} does not exist", version_id);
    };

    let response: Value = REQWEST_CLIENT
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
        libraries,
    })
}

fn extract_natives(jar_path: PathBuf, natives_dir: &Path) -> anyhow::Result<()> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = ZipArchive::new(file)?;

    if !natives_dir.exists() {
        std::fs::create_dir_all(natives_dir)?;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();

        if file.is_dir() {
            continue;
        }

        if !(name.ends_with(".dll") || name.ends_with(".so") || name.ends_with(".dylib")) {
            continue;
        }

        let out_path = natives_dir.join(name);
        let mut out_file = std::fs::File::create(&out_path)?;
        std::io::copy(&mut file, &mut out_file)?;
    }

    Ok(())
}

struct ToDownload {
    download_uri: String,
    download_path: PathBuf,
    is_native: bool,
}

pub async fn install_minecraft(
    version: &VersionData,
    profile_directory: &PathBuf,
) -> anyhow::Result<ProfileInstallationData, anyhow::Error> {
    fs::create_dir_all(profile_directory.as_path()).await?;

    let mut classpath_relative: Vec<String> = Vec::with_capacity(256);
    let directories = directories::get_directories();

    let client_jar_directory =
        directories.versions.join("versions").join(&version.version_id).join(format!("{}.jar", &version.version_id));
    let asset_index_path = directories.indexes.join(format!("{}.json", &version.asset_index_id));

    fs::create_dir_all(&directories.libraries).await?;
    fs::create_dir_all(&directories.objects).await?;
    fs::create_dir_all(&directories.indexes).await?;
    fs::create_dir_all(client_jar_directory.parent().context("failed constructing client.jar directory")?).await?;
    fs::create_dir_all(&directories.natives).await?;

    let libraries = version.get_os_required_libraries();
    let asset_index = REQWEST_CLIENT.get(&version.asset_index_download_url).send().await?.bytes().await?.to_vec();

    let mut file = fs::File::create(asset_index_path).await?;
    file.write_all(&asset_index).await?;

    let mut files_to_download: Vec<ToDownload> = Vec::with_capacity(1024);
    files_to_download.push(ToDownload {
        download_uri: version.client_jar_download_url.clone(),
        download_path: client_jar_directory.to_owned(),
        is_native: false,
    });

    for library in libraries {
        let full_download_path = directories.libraries.join(&library.download_path);
        files_to_download.push(ToDownload {
            download_uri: library.download_url.clone(),
            download_path: full_download_path.to_owned(),
            is_native: library.is_native,
        });

        if !library.is_native {
            classpath_relative.push(full_download_path.to_string_lossy().into_owned());
        }
    }

    let decoded_asset_index: Value = serde_json::from_slice(&asset_index)?;
    let asset_index_objects = decoded_asset_index["objects"].as_object().context("bad json: couldn't parse objects")?;
    for (_, object_data) in asset_index_objects {
        let hash = object_data["hash"].as_str().context("bad json: couldn't parse hash")?;
        let hash_first_two = hash[0..2].to_string();
        let uri = format!("{}/{}/{}", RESOURCES_BASE_URL, hash_first_two, hash);
        let path = directories.objects.join(hash_first_two).join(hash);

        files_to_download.push(ToDownload { download_uri: uri, download_path: path.to_owned(), is_native: false });
    }

    let results: Vec<anyhow::Result<()>> = futures::stream::iter(files_to_download)
        .map(|task| {
            let natives_directory = directories.natives.clone();

            async move {
                if fs::try_exists(&task.download_path).await? {
                    // it already exists so we don't download it again
                    return Ok(());
                }

                log(format!(
                    "Downloading {} to {}...",
                    task.download_uri,
                    task.download_path.to_str().context("invalid download path in task")?
                ));

                let parent_path = task.download_path.parent().context("failed getting parent path")?;
                fs::create_dir_all(parent_path).await?;
                let response = REQWEST_CLIENT.get(&task.download_uri).send().await?;
                let bytes = response.bytes().await?;
                fs::write(&task.download_path, bytes).await?;

                if task.is_native {
                    log(format!("Extracting {} to {}...", task.download_uri, natives_directory.display()));
                    tokio::task::spawn_blocking(move || extract_natives(task.download_path, &natives_directory))
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

    let installation_metadata = ProfileInstallationData {
        main_class: version.main_class.clone(),
        asset_index_id: version.asset_index_id.clone(),
        client_jar_relative: client_jar_directory
            .strip_prefix(&directories.minecraft_root)?
            .to_string_lossy()
            .into_owned(),
        classpath_relative,
    };

    Ok(installation_metadata)
}
