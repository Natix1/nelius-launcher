use std::{fs, path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;

pub struct Directories {
    pub config: PathBuf,
    pub data: PathBuf,
    pub minecraft_root: PathBuf,
    pub config_file: PathBuf,
    pub profiles: PathBuf,
    pub natives: PathBuf,
    pub objects: PathBuf,
    pub versions: PathBuf,
    pub libraries: PathBuf,
    pub indexes: PathBuf,
    pub assets: PathBuf,
}

static DIRECTORIES: LazyLock<Directories> = LazyLock::new(|| {
    let project_dirs =
        ProjectDirs::from("org", "nelius-launcher", "nelius-launcher").expect("failed finding project dirs");
    let config = project_dirs.config_dir().to_owned();
    let data = project_dirs.data_dir().to_owned();
    let minecraft_root = data.join("minecraft-root");
    let config_file = config.join("config.json");

    let profiles = data.join("profiles");
    let natives = minecraft_root.join("natives");
    let versions = minecraft_root.join("versions");
    let libraries = minecraft_root.join("libraries");

    let assets = minecraft_root.join("assets");
    let objects = assets.join("objects");
    let indexes = assets.join("indexes");

    Directories {
        config,
        data,
        config_file,
        profiles,
        natives,
        objects,
        versions,
        libraries,
        minecraft_root,
        indexes,
        assets,
    }
});

pub fn get_directories() -> &'static Directories {
    &DIRECTORIES
}
