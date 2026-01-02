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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> Config {
        let profiles_dir = temp_dir.path().join("profiles");
        let backups_dir = temp_dir.path().join("backups");
        let settings_file = temp_dir.path().join("settings.json");
        let current_profile_file = profiles_dir.join(".current");

        Config {
            profiles_dir,
            backups_dir,
            settings_file,
            current_profile_file,
        }
    }

    #[test]
    fn test_profile_path_format() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        let path = config.profile_path("my-profile");
        assert_eq!(path, temp_dir.path().join("profiles/my-profile.json"));
    }

    #[test]
    fn test_backup_path_format() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        let path = config.backup_path("backup-20240101");
        assert_eq!(path, temp_dir.path().join("backups/backup-20240101.json"));
    }

    #[test]
    fn test_ensure_dirs_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        assert!(!config.profiles_dir.exists());
        assert!(!config.backups_dir.exists());

        config.ensure_dirs().unwrap();

        assert!(config.profiles_dir.exists());
        assert!(config.backups_dir.exists());
    }

    #[test]
    fn test_ensure_dirs_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        config.ensure_dirs().unwrap();
        // Second call should not fail
        config.ensure_dirs().unwrap();

        assert!(config.profiles_dir.exists());
        assert!(config.backups_dir.exists());
    }
}
