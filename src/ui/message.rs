use crate::launcher::downloader::ManifestVersion;

#[derive(Clone, Debug)]
pub enum LogSource {
    NeliusLauncher,
    Minecraft,
    #[allow(dead_code)]
    Unknown, /* reserved for future use */
}

#[derive(Clone, Debug)]
pub enum Message {
    VersionChanged(String),
    MinecraftVersionsLoaded(Result<Vec<ManifestVersion>, String>),
    InstalledVersionsLoaded(Result<Vec<String>, String>),
    ShowReleasesUpdated(bool),
    ShowSnapshotsUpdated(bool),
    StartGameRequest,
    MinecraftLaunched(Option<String>),
    SubmitLogLine(String, LogSource),
    MinecraftClosed,
    ChangeJavaBinaryRequested,
    SetJavaBinary(Option<String>),
    AutoScrollLogsToggled(bool),
    PersistentStateSaved(Result<(), String>),
}
