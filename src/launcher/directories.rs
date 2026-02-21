use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use directories::ProjectDirs;

struct Directories {
    config: PathBuf,
    data: PathBuf,
    config_file: PathBuf,
}

static PROJECT_DIRS: LazyLock<Directories> = LazyLock::new(|| {
    let project_dirs =
        ProjectDirs::from("org", "nelius-launcher", "nelius-launcher").expect("failed finding project dirs");
    let config = project_dirs.config_dir().to_owned();
    let data = project_dirs.data_dir().to_owned();
    let config_file = config.join("config.json");

    Directories { config, data, config_file }
});

pub fn get_config_dir() -> &'static Path {
    &PROJECT_DIRS.config
}

pub fn get_data_dir() -> &'static Path {
    &PROJECT_DIRS.data
}

pub fn get_config_file_dir() -> &'static Path {
    &PROJECT_DIRS.config_file
}
