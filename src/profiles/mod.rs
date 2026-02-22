use serde::{Deserialize, Serialize};

pub mod store;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Profile {
    pub profile_name: String,
    pub version_id: String,
    pub java_jar_path: String,

    pub asset_index_id: String,
    pub client_jar_relative: String,
    pub classpath_relative: String,
    pub main_class: String,
}

impl Profile {
    pub async fn ensure_installed(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn launch(&self) -> anyhow::Result<tokio::process::Child> {
        todo!()
    }

    pub async fn uninstall(&self) -> anyhow::Result<()> {
        todo!()
    }
}
