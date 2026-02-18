use crate::{LogSource, launcher::downloader::ManifestVersion};

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
