use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct InstalledVersion {
    pub id: String,
    pub version: String,
}

pub const CONFIG_FILENAME: &'static str = "installed_versions.json";

pub fn get_project_dirs() -> ProjectDirs {
    return ProjectDirs::from("dev", "natix", "nelius-launcher").expect("to find valid directories for storage");
}

pub async fn get_installed() -> anyhow::Result<Vec<InstalledVersion>> {
    let config_path = get_project_dirs().data_local_dir().join(CONFIG_FILENAME);
    if !fs::try_exists(&config_path).await? {
        return Ok(Vec::new());
    }

    let contents = fs::read(config_path).await?;
    let versions: Vec<InstalledVersion> = serde_json::from_slice(&contents)?;

    Ok(versions)
}

pub async fn save_installed(versions: Vec<InstalledVersion>) -> anyhow::Result<()> {
    let config_path = get_project_dirs().data_local_dir().join(CONFIG_FILENAME);

    fs::create_dir_all(get_project_dirs().data_local_dir()).await?;
    let mut file = fs::File::create(config_path).await?;

    file.write_all(serde_json::to_string_pretty(&versions)?.as_str().as_bytes()).await?;

    Ok(())
}
