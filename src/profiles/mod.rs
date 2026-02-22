use std::{path::PathBuf, process::Stdio};

use anyhow::anyhow;
use dioxus::core::SpawnIfAsync;
use serde::{Deserialize, Serialize};
use tokio::{fs, process::Command};

const PROFILE_INSTALLATION_DATA_FILENAME: &str = "profile.json";

use crate::launcher;
pub mod store;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileInstallationData {
    pub asset_index_id: String,
    pub client_jar_relative: String,
    pub classpath_relative: Vec<String>,
    pub main_class: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Profile {
    pub profile_name: String,
    pub version_id: String,
    pub java_binary_path: String,
}

impl Profile {
    pub fn get_profile_directory(&self) -> PathBuf {
        let profiles_directory = launcher::directories::get_directories().profiles.to_owned();
        let profile_directory = profiles_directory.join(&self.profile_name);

        return profile_directory;
    }

    pub async fn get_installation_data_or_install(&self) -> anyhow::Result<ProfileInstallationData> {
        let profile_directory = self.get_profile_directory();
        let profile_metadata_directory = profile_directory.join(PROFILE_INSTALLATION_DATA_FILENAME);

        let data_exists = fs::try_exists(&profile_metadata_directory).await?;
        if data_exists {
            let installation_data: ProfileInstallationData =
                serde_json::from_slice(fs::read(&profile_metadata_directory).await?.as_slice())?;

            Ok(installation_data)
        } else {
            let version_data = launcher::downloader::get_version_data(&self.version_id).await?;
            let installation_data = launcher::downloader::install_minecraft(&version_data, &profile_directory).await?;

            Ok(installation_data)
        }
    }

    fn make_launch_command(&self, installation_data: &ProfileInstallationData) -> Command {
        let directories = launcher::directories::get_directories();
        let mut cmd = Command::new(&self.java_binary_path);
        let mut classpath_entries: Vec<String> = installation_data
            .classpath_relative
            .iter()
            .map(|relative| directories.libraries.join(relative).to_string_lossy().into_owned())
            .collect();

        classpath_entries.push(
            directories.minecraft_root.join(&installation_data.client_jar_relative).to_string_lossy().into_owned(),
        );

        let seperator = if std::env::consts::OS == "windows" { ";" } else { ":" };
        let classpath = classpath_entries.join(seperator);

        cmd.current_dir(&self.get_profile_directory())
            .arg(format!("-Djava.library.path={}", directories.libraries.display()))
            .arg("-Xmx4G")
            .arg("-cp")
            .arg(classpath)
            .arg(&installation_data.main_class)
            .arg("--username")
            .arg("Nelius")
            .arg("--version")
            .arg(&self.version_id)
            .arg("--gameDir")
            .arg(&self.get_profile_directory())
            .arg("-assetsDir")
            .arg(&directories.assets)
            .arg("--assetIndex")
            .arg(&installation_data.asset_index_id)
            .arg("--uuid")
            .arg("0")
            .arg("--accessToken")
            .arg("0")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        return cmd;
    }

    pub async fn launch_or_install(&mut self) -> anyhow::Result<()> {
        let installation_data = self.get_installation_data_or_install().await?;
        let mut cmd = self.make_launch_command(&installation_data);

        let child = Command::spawn(&mut cmd)?;

        Ok(())
    }

    pub async fn uninstall(&self) -> anyhow::Result<()> {
        todo!()
    }
}
