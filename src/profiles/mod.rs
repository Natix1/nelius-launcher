use std::{path::PathBuf, process::Stdio, sync::Arc};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

const PROFILE_INSTALLATION_DATA_FILENAME: &str = "profile.json";

use crate::launcher::{self, logging::log};
pub mod store;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileInstallationData {
    pub asset_index_id: String,
    pub client_jar_relative: String,
    pub classpath_relative: Vec<String>,
    pub main_class: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Profile {
    pub profile_name: String,
    pub version_id: String,
    pub java_binary_path: String,

    #[serde(skip)]
    pub kill_notify: Arc<tokio::sync::Notify>,
}

impl Profile {
    pub fn get_profile_directory(&self) -> PathBuf {
        let profiles_directory = launcher::directories::get_directories().profiles.to_owned();
        profiles_directory.join(&self.profile_name)
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

        cmd.current_dir(self.get_profile_directory())
            .arg(format!("-Djava.library.path={}", directories.natives.display()))
            .arg("-Xmx4G")
            .arg("-cp")
            .arg(classpath)
            .arg(&installation_data.main_class)
            .arg("--username")
            .arg("Nelius")
            .arg("--version")
            .arg(&self.version_id)
            .arg("--gameDir")
            .arg(self.get_profile_directory())
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

        cmd
    }

    // This future does not resolve until the game process exits.
    pub async fn launch_or_install(&mut self) -> anyhow::Result<()> {
        let installation_data = self.get_installation_data_or_install().await?;
        let mut cmd = self.make_launch_command(&installation_data);

        let mut child = Command::spawn(&mut cmd)?;
        let stdout = child.stdout.take().context("no stdin attached to process")?;
        let stderr = child.stderr.take().context("no stderr attached to process")?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        loop {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            log(line);
                        },
                        Err(e) => {
                            eprintln!("Failed reading stdout: {e}");
                        },
                        _ => {
                            eprintln!("Failed reading stdout, got empty value. Considering the program as exited.");
                            break Ok(())
                        }
                    }
                },
                line = stderr_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            log(line);
                        },
                        Err(e) => {
                            eprintln!("Failed reading stderr: {e}");
                        },
                        _ => {
                            eprintln!("Failed reading stderr, got empty value. Considering the program as exited.");
                            break Ok(())
                        }
                    }
                },
                _ = self.kill_notify.notified() => {
                    match child.kill().await {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Failed killing process. Exiting loop. Error: {e}");
                            break Ok(())
                        }
                    }
                }
            }
        }
    }

    // TODO:
    // For now, this doesn't actually uninstall any natives, libraries, assets or game binaries. Just worlds, and other artifacts **generated by the game**,
    // **not** by our downloader. In the future, this could initiate some "disk space re-claim" process which enumerates all installed profiles and their versions,
    // creates a dependency map and checks if removing this profile leaves any unused dependencies.
    // With this being said, this method will have to do for now.
    pub async fn uninstall(&self) -> anyhow::Result<()> {
        let profile_dir = self.get_profile_directory();
        fs::remove_dir_all(&profile_dir).await?;

        Ok(())
    }
}
