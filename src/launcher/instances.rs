use directories::ProjectDirs;
use tokio::fs;
pub fn get_project_dirs() -> ProjectDirs {
    return ProjectDirs::from("dev", "natix", "nelius-launcher").expect("to find valid directories for storage");
}

pub async fn get_installed() -> anyhow::Result<Vec<String>> {
    let installations_path = get_project_dirs().data_local_dir().join("installations");
    fs::create_dir_all(&installations_path).await?;

    let mut versions: Vec<String> = Vec::with_capacity(16);
    let mut reader = fs::read_dir(&installations_path).await?;

    while let Some(entry) = reader.next_entry().await? {
        let file_type = entry.file_type().await?;
        if !file_type.is_dir() {
            continue;
        };

        versions.push(entry.file_name().to_string_lossy().to_string());
    }

    Ok(versions)
}
