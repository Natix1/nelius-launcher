use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use directories::ProjectDirs;

struct Directories {
    config: PathBuf,
    data: PathBuf,
    config_file: PathBuf,
    profiles: PathBuf,
}

static DIRECTORIES: LazyLock<Directories> = LazyLock::new(|| {
    let project_dirs =
        ProjectDirs::from("org", "nelius-launcher", "nelius-launcher").expect("failed finding project dirs");
    let config = project_dirs.config_dir().to_owned();
    let data = project_dirs.data_dir().to_owned();
    let config_file = config.join("config.json");
    let profiles = config.join("profiles");

    Directories { config, data, config_file, profiles }
});

pub fn get_config_dir() -> &'static Path {
    &DIRECTORIES.config
}

pub fn get_data_dir() -> &'static Path {
    &DIRECTORIES.data
}

pub fn get_config_file_dir() -> &'static Path {
    &DIRECTORIES.config_file
}

pub fn get_profiles_dir() -> &'static Path {
    &DIRECTORIES.profiles
}
