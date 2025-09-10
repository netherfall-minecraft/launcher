use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub last_profile: Option<String>,
    pub ram_mb: i32,
    pub java_path: Option<String>,
    pub game_dir: Option<String>,
    pub compat_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: "Player".into(),
            last_profile: None,
            ram_mb: 4096,
            java_path: Some("/usr/local/bin/java".into()),
            game_dir: Some("~/.netherfall".into()),
            compat_mode: false,
        }
    }
}

impl Config {
    pub fn path(base: PathBuf) -> PathBuf { base.join("config.toml") }

    pub fn load(base: PathBuf) -> Result<Self> {
        let p = Self::path(base);
        if !p.exists() { return Ok(Self::default()); }
        let s = fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
        Ok(toml::from_str(&s)?)
    }

    pub fn save(&self, base: PathBuf) -> Result<()> {
        let p = Self::path(base);
        if let Some(parent) = p.parent() { fs::create_dir_all(parent)?; }
        let s = toml::to_string_pretty(self)?;
        fs::write(&p, s)?;
        Ok(())
    }
}
