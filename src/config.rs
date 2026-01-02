use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct Config {
    pub profiles_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub settings_file: PathBuf,
    pub current_profile_file: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let claude_dir = home.join(".claude");
        let profiles_dir = claude_dir.join("profiles");
        let backups_dir = claude_dir.join("backups");
        let settings_file = claude_dir.join("settings.json");
        let current_profile_file = profiles_dir.join(".current");

        Ok(Self {
            profiles_dir,
            backups_dir,
            settings_file,
            current_profile_file,
        })
    }
    
    pub fn profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{}.json", name))
    }
    
    pub fn backup_path(&self, name: &str) -> PathBuf {
        self.backups_dir.join(format!("{}.json", name))
    }
    
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.profiles_dir)
            .context("Failed to create profiles directory")?;
        std::fs::create_dir_all(&self.backups_dir)
            .context("Failed to create backups directory")?;
        Ok(())
    }
}
