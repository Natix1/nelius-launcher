use std::process::Stdio;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    process::{Child, Command},
};

use crate::launcher::{self, downloader::ManifestVersion};

pub const METADATA_FILENAME: &'static str = "nelius_metadata.lock";

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallationMetadata {
    pub main_class: String,
    pub version: String,
    pub asset_index_id: String,
    pub client_jar_relative: String,
    pub classpath_relative: Vec<String>,
}

async fn launch(installed_version: &String) -> anyhow::Result<Child, anyhow::Error> {
    let game_dir =
        launcher::instances::get_project_dirs().data_local_dir().join("installations").join(&installed_version);

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
        .arg(format!("-Djava.library.path={}", game_dir.join("natives").display()))
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
    pub new_installation: Option<String>,
}

pub async fn play(selected_version: &ManifestVersion) -> anyhow::Result<PlayResult, anyhow::Error> {
    let installed =
        launcher::instances::get_installed().await?.iter().find(|v| *v == &selected_version.version_id).cloned();

    if let Some(installed) = installed {
        Ok(PlayResult { child: launch(&installed).await?, new_installation: None })
    } else {
        let installation_dir = launcher::instances::get_project_dirs()
            .data_local_dir()
            .join("installations")
            .join(&selected_version.version_id);

        fs::create_dir_all(&installation_dir).await?;
        let version_data = launcher::downloader::get_version_data(selected_version.version_id.clone()).await?;
        launcher::downloader::install_minecraft(&version_data, &installation_dir).await?;

        Ok(PlayResult {
            child: launch(&selected_version.version_id).await?,
            new_installation: Some(selected_version.version_id.clone()),
        })
    }
}
