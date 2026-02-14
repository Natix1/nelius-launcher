use std::process::Stdio;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    process::{Child, Command},
};
use uuid::Uuid;

use crate::{
    downloader::{self, ManifestVersion},
    installed_versions::{self, InstalledVersion},
};

pub const METADATA_FILENAME: &'static str = "nelius_metadata.lock";

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallationMetadata {
    pub main_class: String,
    pub version: String,
    pub asset_index_id: String,
    pub client_jar_relative: String,
    pub classpath_relative: Vec<String>,
}

async fn launch(installed_version: &InstalledVersion) -> anyhow::Result<Child, anyhow::Error> {
    let game_dir =
        installed_versions::get_project_dirs().data_local_dir().join("installations").join(&installed_version.id);

    fs::try_exists(&game_dir).await.context("the game directory was not found")?;

    let metdata_dir = game_dir.join(METADATA_FILENAME);
    let contents = fs::read(metdata_dir).await?;
    let metadata: InstallationMetadata = serde_json::from_slice(&contents)?;

    let mut classpath_entries: Vec<String> = metadata
        .classpath_relative
        .iter()
        .map(|relative| game_dir.join(relative).to_string_lossy().into_owned())
        .collect();

    classpath_entries.push(game_dir.join(metadata.client_jar_relative).to_string_lossy().into_owned());

    let seperator = if std::env::consts::OS == "windows" { ";" } else { ":" };
    let classpath = classpath_entries.join(&seperator);
    let mut cmd = Command::new("java");

    cmd.current_dir(&game_dir)
        .arg("-Xmx4G")
        .arg("-cp")
        .arg(classpath)
        .arg(metadata.main_class)
        .arg("--username")
        .arg("Nelius")
        .arg("--version")
        .arg(metadata.version)
        .arg("--gameDir")
        .arg(&game_dir)
        .arg("-assetsDir")
        .arg(&game_dir.join("assets"))
        .arg("--assetIndex")
        .arg(metadata.asset_index_id)
        .arg("--uuid")
        .arg("0")
        .arg("--accessToken")
        .arg("0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = cmd.spawn().context("failed to spawn the minecraft process")?;
    Ok(child)
}

pub struct PlayResult {
    pub child: Child,
    pub new_installation: Option<InstalledVersion>,
}

pub async fn play(selected_version: &ManifestVersion) -> anyhow::Result<PlayResult, anyhow::Error> {
    let installed = installed_versions::get_installed_versions_from_disk()
        .await?
        .iter()
        .find(|v| v.version == selected_version.version_id)
        .cloned();

    if let Some(installed) = installed {
        Ok(PlayResult { child: launch(&installed).await?, new_installation: None })
    } else {
        let installation_id = Uuid::new_v4().to_string();
        let installation_dir =
            installed_versions::get_project_dirs().data_local_dir().join("installations").join(&installation_id);

        fs::create_dir_all(&installation_dir).await?;
        let version_data = downloader::get_version_data(selected_version.version_id.clone()).await?;
        downloader::install_minecraft(&version_data, &installation_dir).await?;

        let new_installed_version =
            InstalledVersion { id: installation_id, version: selected_version.version_id.clone() };
        Ok(PlayResult { child: launch(&new_installed_version).await?, new_installation: Some(new_installed_version) })
    }
}
