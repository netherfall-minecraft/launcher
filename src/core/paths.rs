use directories::ProjectDirs;
use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    ProjectDirs::from("org", "Netherfall", "Launcher")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap())
}
