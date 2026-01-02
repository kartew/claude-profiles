use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

use crate::config::Config;

pub struct ProfileManager {
    pub config: Config,
}

impl ProfileManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: Config::new()?,
        })
    }
    
    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let mut profiles = Vec::new();
        
        if self.config.profiles_dir.exists() {
            for entry in fs::read_dir(&self.config.profiles_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Some(name) = path.file_stem() {
                        profiles.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        profiles.sort();
        Ok(profiles)
    }
    
    pub fn list_backups(&self) -> Result<Vec<String>> {
        let mut backups = Vec::new();
        
        if self.config.backups_dir.exists() {
            for entry in fs::read_dir(&self.config.backups_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Some(name) = path.file_stem() {
                        backups.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        backups.sort();
        Ok(backups)
    }
    
    pub fn get_current_profile(&self) -> Result<Option<String>> {
        if self.config.current_profile_file.exists() {
            let content = fs::read_to_string(&self.config.current_profile_file)?;
            Ok(Some(content.trim().to_string()))
        } else {
            Ok(None)
        }
    }
    
    pub fn set_current_profile(&self, name: &str) -> Result<()> {
        fs::write(&self.config.current_profile_file, name)?;
        Ok(())
    }
    
    pub fn profile_exists(&self, name: &str) -> bool {
        self.config.profile_path(name).exists()
    }
    
    pub fn load_profile(&self, name: &str) -> Result<Value> {
        let path = self.config.profile_path(name);
        self.load_json(&path)
    }
    
    pub fn save_profile(&self, name: &str, data: &Value) -> Result<()> {
        let path = self.config.profile_path(name);
        self.save_json(&path, data)
    }
    
    pub fn delete_profile(&self, name: &str) -> Result<()> {
        let path = self.config.profile_path(name);
        fs::remove_file(&path).context("Failed to delete profile")?;
        Ok(())
    }
    
    pub fn load_settings(&self) -> Result<Value> {
        self.load_json(&self.config.settings_file)
    }
    
    pub fn save_settings(&self, data: &Value) -> Result<()> {
        self.save_json(&self.config.settings_file, data)
    }
    
    pub fn load_backup(&self, name: &str) -> Result<Value> {
        let path = self.config.backup_path(name);
        self.load_json(&path)
    }
    
    pub fn save_backup(&self, name: &str, data: &Value) -> Result<()> {
        let path = self.config.backup_path(name);
        self.save_json(&path, data)
    }
    
    fn load_json(&self, path: &Path) -> Result<Value> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from {}", path.display()))
    }
    
    fn save_json(&self, path: &Path, data: &Value) -> Result<()> {
        let content = serde_json::to_string_pretty(data)?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }
    
    pub fn get_value(&self, data: &Value, key: &str) -> Option<Value> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = data;
        
        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                _ => return None,
            }
        }
        
        Some(current.clone())
    }
    
    pub fn set_value(&self, data: &mut Value, key: &str, value: Value) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = data;
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - set the value
                match current {
                    Value::Object(map) => {
                        map.insert(part.to_string(), value);
                        return Ok(());
                    }
                    _ => bail!("Cannot set value: path is not an object"),
                }
            } else {
                // Navigate deeper, create objects if needed
                match current {
                    Value::Object(map) => {
                        if !map.contains_key(*part) {
                            map.insert(part.to_string(), Value::Object(Map::new()));
                        }
                        current = map.get_mut(*part)
                            .with_context(|| format!("Key '{}' not found in path", part))?;
                    }
                    _ => bail!("Cannot navigate: path is not an object"),
                }
            }
        }
        
        Ok(())
    }
    
    pub fn unset_value(&self, data: &mut Value, key: &str) -> Result<bool> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = data;
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                match current {
                    Value::Object(map) => {
                        return Ok(map.remove(*part).is_some());
                    }
                    _ => return Ok(false),
                }
            } else {
                match current {
                    Value::Object(map) => {
                        if let Some(next) = map.get_mut(*part) {
                            current = next;
                        } else {
                            return Ok(false);
                        }
                    }
                    _ => return Ok(false),
                }
            }
        }
        
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_value_simple() {
        let data = json!({ "model": "sonnet-4" });
        let pm = ProfileManager::new().unwrap();
        assert_eq!(pm.get_value(&data, "model"), Some(json!("sonnet-4")));
    }

    #[test]
    fn test_get_value_nested() {
        let data = json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.example.com"
            }
        });
        let pm = ProfileManager::new().unwrap();
        assert_eq!(
            pm.get_value(&data, "env.ANTHROPIC_BASE_URL"),
            Some(json!("https://api.example.com"))
        );
    }

    #[test]
    fn test_get_value_missing() {
        let data = json!({ "model": "sonnet-4" });
        let pm = ProfileManager::new().unwrap();
        assert_eq!(pm.get_value(&data, "missing_key"), None);
    }

    #[test]
    fn test_get_value_missing_nested() {
        let data = json!({ "model": "sonnet-4" });
        let pm = ProfileManager::new().unwrap();
        assert_eq!(pm.get_value(&data, "env.MISSING"), None);
    }

    #[test]
    fn test_set_value_new_key() {
        let mut data = json!({});
        let pm = ProfileManager::new().unwrap();
        pm.set_value(&mut data, "model", json!("haiku-3")).unwrap();
        assert_eq!(data["model"], json!("haiku-3"));
    }

    #[test]
    fn test_set_value_nested() {
        let mut data = json!({});
        let pm = ProfileManager::new().unwrap();
        pm.set_value(&mut data, "env.ANTHROPIC_BASE_URL", json!("https://test.com")).unwrap();
        assert_eq!(data["env"]["ANTHROPIC_BASE_URL"], json!("https://test.com"));
    }

    #[test]
    fn test_set_value_overwrite() {
        let mut data = json!({ "model": "sonnet-4" });
        let pm = ProfileManager::new().unwrap();
        pm.set_value(&mut data, "model", json!("haiku-3")).unwrap();
        assert_eq!(data["model"], json!("haiku-3"));
    }

    #[test]
    fn test_unset_value_existing() {
        let mut data = json!({ "model": "sonnet-4", "other": "value" });
        let pm = ProfileManager::new().unwrap();
        assert!(pm.unset_value(&mut data, "model").unwrap());
        assert!(!data.get("model").is_some());
        assert_eq!(data["other"], json!("value"));
    }

    #[test]
    fn test_unset_value_missing() {
        let mut data = json!({ "model": "sonnet-4" });
        let pm = ProfileManager::new().unwrap();
        assert!(!pm.unset_value(&mut data, "missing").unwrap());
        assert_eq!(data["model"], json!("sonnet-4"));
    }

    #[test]
    fn test_unset_value_nested() {
        let mut data = json!({ "env": { "ANTHROPIC_BASE_URL": "https://test.com" } });
        let pm = ProfileManager::new().unwrap();
        assert!(pm.unset_value(&mut data, "env.ANTHROPIC_BASE_URL").unwrap());
        assert!(!data["env"].get("ANTHROPIC_BASE_URL").is_some());
    }

    #[test]
    fn test_profile_roundtrip() {
        let data = json!({
            "model": "sonnet-4",
            "env": {
                "ANTHROPIC_BASE_URL": "https://api.anthropic.com"
            },
            "alwaysThinkingEnabled": true
        });
        let _pm = ProfileManager::new().unwrap();
        let serialized = serde_json::to_string_pretty(&data).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(data, parsed);
    }
}
