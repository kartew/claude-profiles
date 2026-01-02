use anyhow::{bail, Context, Result};
use chrono::Local;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use similar::{ChangeTag, TextDiff};
use std::io::{self, Read};

use crate::cli::Cli;
use crate::profile::ProfileManager;

pub fn interactive() -> Result<()> {
    let pm = ProfileManager::new()?;
    let profiles = pm.list_profiles()?;
    
    if profiles.is_empty() {
        println!("{}", "No profiles found. Run 'ccp init' to initialize.".yellow());
        return Ok(());
    }
    
    let current = pm.get_current_profile()?;
    let current_idx = current
        .as_ref()
        .and_then(|c| profiles.iter().position(|p| p == c))
        .unwrap_or(0);
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select profile")
        .items(&profiles)
        .default(current_idx)
        .interact_opt()?;
    
    match selection {
        Some(idx) => {
            let selected = &profiles[idx];
            if Some(selected) != current.as_ref() {
                use_profile(selected)?;
            } else {
                println!("{} Already on '{}'", "·".dimmed(), selected.cyan());
            }
        }
        None => {
            println!("Cancelled");
        }
    }
    
    Ok(())
}

pub fn init() -> Result<()> {
    let pm = ProfileManager::new()?;
    pm.config.ensure_dirs()?;
    
    // If settings.json exists but no profiles, create default profile
    if pm.config.settings_file.exists() && pm.list_profiles()?.is_empty() {
        let settings = pm.load_settings()?;
        pm.save_profile("default", &settings)?;
        pm.set_current_profile("default")?;
        println!("{} Created default profile from existing settings", "✓".green());
    } else if !pm.config.settings_file.exists() {
        // Create empty default settings
        let default_settings = serde_json::json!({
            "$schema": "https://json.schemastore.org/claude-code-settings.json"
        });
        pm.save_settings(&default_settings)?;
        pm.save_profile("default", &default_settings)?;
        pm.set_current_profile("default")?;
        println!("{} Initialized with empty default profile", "✓".green());
    } else {
        println!("{} Profiles directory already initialized", "✓".green());
    }
    
    println!("  Profiles dir: {}", pm.config.profiles_dir.display());
    println!("  Backups dir: {}", pm.config.backups_dir.display());
    
    Ok(())
}

pub fn list() -> Result<()> {
    let pm = ProfileManager::new()?;
    let profiles = pm.list_profiles()?;
    let current = pm.get_current_profile()?;
    
    if profiles.is_empty() {
        println!("{}", "No profiles found. Run 'ccp init' to initialize.".yellow());
        return Ok(());
    }
    
    println!("{}", "Available profiles:".bold());
    for profile in profiles {
        let marker = if Some(&profile) == current.as_ref() {
            "→".green()
        } else {
            " ".normal()
        };
        let name = if Some(&profile) == current.as_ref() {
            profile.green().bold()
        } else {
            profile.normal()
        };
        println!("  {} {}", marker, name);
    }
    
    Ok(())
}

pub fn current() -> Result<()> {
    let pm = ProfileManager::new()?;
    
    match pm.get_current_profile()? {
        Some(name) => {
            println!("{}", name.green().bold());
        }
        None => {
            println!("{}", "No profile selected. Run 'ccp init' or 'ccp use <profile>'".yellow());
        }
    }
    
    Ok(())
}

pub fn use_profile(name: &str) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    if !pm.profile_exists(name) {
        bail!("Profile '{}' does not exist. Use 'ccp list' to see available profiles.", name);
    }
    
    // Load profile and apply to settings.json
    let profile_data = pm.load_profile(name)?;
    pm.save_settings(&profile_data)?;
    pm.set_current_profile(name)?;
    
    println!("{} Switched to profile '{}'", "✓".green(), name.cyan());
    Ok(())
}

pub fn create(name: &str, from: Option<&str>) -> Result<()> {
    let pm = ProfileManager::new()?;
    pm.config.ensure_dirs()?;
    
    if pm.profile_exists(name) {
        bail!("Profile '{}' already exists", name);
    }
    
    let data = match from {
        Some(source) => {
            if !pm.profile_exists(source) {
                bail!("Source profile '{}' does not exist", source);
            }
            pm.load_profile(source)?
        }
        None => {
            // Try to copy from current settings or create empty
            if pm.config.settings_file.exists() {
                pm.load_settings()?
            } else {
                serde_json::json!({
                    "$schema": "https://json.schemastore.org/claude-code-settings.json"
                })
            }
        }
    };
    
    pm.save_profile(name, &data)?;
    
    let source_msg = from.map_or("current settings".to_string(), |s| format!("'{}'", s));
    println!("{} Created profile '{}' from {}", "✓".green(), name.cyan(), source_msg);
    Ok(())
}

pub fn delete(name: &str, force: bool) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    if !pm.profile_exists(name) {
        bail!("Profile '{}' does not exist", name);
    }
    
    if name == "default" && !force {
        bail!("Cannot delete 'default' profile. Use --force to override.");
    }
    
    if !force {
        let confirm = Confirm::new()
            .with_prompt(format!("Delete profile '{}'?", name))
            .default(false)
            .interact()?;
        
        if !confirm {
            println!("Cancelled");
            return Ok(());
        }
    }
    
    // If deleting current profile, switch to default
    if let Some(current) = pm.get_current_profile()? {
        if current == name {
            if pm.profile_exists("default") && name != "default" {
                use_profile("default")?;
            }
        }
    }
    
    pm.delete_profile(name)?;
    println!("{} Deleted profile '{}'", "✓".green(), name);
    Ok(())
}

pub fn copy(src: &str, dst: &str) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    if !pm.profile_exists(src) {
        bail!("Source profile '{}' does not exist", src);
    }
    
    if pm.profile_exists(dst) {
        bail!("Destination profile '{}' already exists", dst);
    }
    
    let data = pm.load_profile(src)?;
    pm.save_profile(dst, &data)?;
    
    println!("{} Copied '{}' to '{}'", "✓".green(), src, dst.cyan());
    Ok(())
}

pub fn rename(old: &str, new: &str) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    if !pm.profile_exists(old) {
        bail!("Profile '{}' does not exist", old);
    }
    
    if pm.profile_exists(new) {
        bail!("Profile '{}' already exists", new);
    }
    
    let data = pm.load_profile(old)?;
    pm.save_profile(new, &data)?;
    pm.delete_profile(old)?;
    
    // Update current if renamed
    if let Some(current) = pm.get_current_profile()? {
        if current == old {
            pm.set_current_profile(new)?;
        }
    }
    
    println!("{} Renamed '{}' to '{}'", "✓".green(), old, new.cyan());
    Ok(())
}

pub fn configure(profile: Option<&str>) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    let profile_name = match profile {
        Some(p) => p.to_string(),
        None => pm.get_current_profile()?.unwrap_or_else(|| "default".to_string()),
    };
    
    if !pm.profile_exists(&profile_name) {
        bail!("Profile '{}' does not exist", profile_name);
    }
    
    let mut data = pm.load_profile(&profile_name)?;
    
    println!("{}", format!("Configuring profile '{}'", profile_name).bold());
    println!("Press Enter to keep current value, or enter new value.\n");
    
    // Model
    let current_model = data.get("model").and_then(|v| v.as_str()).unwrap_or("");
    let model: String = Input::new()
        .with_prompt("Model")
        .default(current_model.to_string())
        .allow_empty(true)
        .interact_text()?;
    if !model.is_empty() {
        data["model"] = serde_json::Value::String(model);
    }
    
    // API Base URL
    let current_url = data.get("env")
        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let url: String = Input::new()
        .with_prompt("API Base URL (env.ANTHROPIC_BASE_URL)")
        .default(current_url.to_string())
        .allow_empty(true)
        .interact_text()?;
    if !url.is_empty() {
        if data.get("env").is_none() {
            data["env"] = serde_json::json!({});
        }
        data["env"]["ANTHROPIC_BASE_URL"] = serde_json::Value::String(url);
    }
    
    // API Token
    let current_token = data.get("env")
        .and_then(|e| e.get("ANTHROPIC_AUTH_TOKEN"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let token_display = if current_token.len() > 10 {
        format!("{}...{}", &current_token[..6], &current_token[current_token.len()-4..])
    } else {
        current_token.to_string()
    };
    let token: String = Input::new()
        .with_prompt(format!("API Token [current: {}]", token_display))
        .default(current_token.to_string())
        .allow_empty(true)
        .interact_text()?;
    if !token.is_empty() {
        if data.get("env").is_none() {
            data["env"] = serde_json::json!({});
        }
        data["env"]["ANTHROPIC_AUTH_TOKEN"] = serde_json::Value::String(token);
    }
    
    // Always thinking
    let current_thinking = data.get("alwaysThinkingEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let thinking = Confirm::new()
        .with_prompt("Always thinking enabled?")
        .default(current_thinking)
        .interact()?;
    data["alwaysThinkingEnabled"] = serde_json::Value::Bool(thinking);
    
    pm.save_profile(&profile_name, &data)?;
    
    // Apply if current profile
    if Some(&profile_name) == pm.get_current_profile()?.as_ref() {
        pm.save_settings(&data)?;
        println!("\n{} Configuration saved and applied", "✓".green());
    } else {
        println!("\n{} Configuration saved", "✓".green());
    }
    
    Ok(())
}

pub fn set(key: &str, value: &str, profile: Option<&str>) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    let profile_name = match profile {
        Some(p) => p.to_string(),
        None => pm.get_current_profile()?.unwrap_or_else(|| "default".to_string()),
    };
    
    if !pm.profile_exists(&profile_name) {
        bail!("Profile '{}' does not exist", profile_name);
    }
    
    let mut data = pm.load_profile(&profile_name)?;
    
    // Parse value - try as JSON first, then as string
    let json_value: serde_json::Value = serde_json::from_str(value)
        .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
    
    pm.set_value(&mut data, key, json_value)?;
    pm.save_profile(&profile_name, &data)?;
    
    // Apply if current profile
    if Some(&profile_name) == pm.get_current_profile()?.as_ref() {
        pm.save_settings(&data)?;
    }
    
    println!("{} Set {}={} in '{}'", "✓".green(), key.cyan(), value, profile_name);
    Ok(())
}

pub fn get(key: &str, profile: Option<&str>) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    let profile_name = match profile {
        Some(p) => p.to_string(),
        None => pm.get_current_profile()?.unwrap_or_else(|| "default".to_string()),
    };
    
    if !pm.profile_exists(&profile_name) {
        bail!("Profile '{}' does not exist", profile_name);
    }
    
    let data = pm.load_profile(&profile_name)?;
    
    match pm.get_value(&data, key) {
        Some(value) => {
            let output = serde_json::to_string_pretty(&value)?;
            println!("{}", output);
        }
        None => {
            println!("{}", "(not set)".dimmed());
        }
    }
    
    Ok(())
}

pub fn unset(key: &str, profile: Option<&str>) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    let profile_name = match profile {
        Some(p) => p.to_string(),
        None => pm.get_current_profile()?.unwrap_or_else(|| "default".to_string()),
    };
    
    if !pm.profile_exists(&profile_name) {
        bail!("Profile '{}' does not exist", profile_name);
    }
    
    let mut data = pm.load_profile(&profile_name)?;
    
    if pm.unset_value(&mut data, key)? {
        pm.save_profile(&profile_name, &data)?;
        
        // Apply if current profile
        if Some(&profile_name) == pm.get_current_profile()?.as_ref() {
            pm.save_settings(&data)?;
        }
        
        println!("{} Removed '{}' from '{}'", "✓".green(), key.cyan(), profile_name);
    } else {
        println!("{} Key '{}' not found in '{}'", "!".yellow(), key, profile_name);
    }
    
    Ok(())
}

pub fn export(name: Option<&str>) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    let profile_name = match name {
        Some(p) => p.to_string(),
        None => pm.get_current_profile()?.unwrap_or_else(|| "default".to_string()),
    };
    
    if !pm.profile_exists(&profile_name) {
        bail!("Profile '{}' does not exist", profile_name);
    }
    
    let data = pm.load_profile(&profile_name)?;
    let output = serde_json::to_string_pretty(&data)?;
    println!("{}", output);
    
    Ok(())
}

pub fn import(name: &str) -> Result<()> {
    let pm = ProfileManager::new()?;
    pm.config.ensure_dirs()?;
    
    if pm.profile_exists(name) {
        bail!("Profile '{}' already exists. Delete it first or use a different name.", name);
    }
    
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)
        .context("Failed to read from stdin")?;
    
    let data: serde_json::Value = serde_json::from_str(&input)
        .context("Failed to parse JSON from stdin")?;
    
    pm.save_profile(name, &data)?;
    
    eprintln!("{} Imported profile '{}'", "✓".green(), name.cyan());
    Ok(())
}

pub fn diff(profile1: &str, profile2: &str) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    if !pm.profile_exists(profile1) {
        bail!("Profile '{}' does not exist", profile1);
    }
    if !pm.profile_exists(profile2) {
        bail!("Profile '{}' does not exist", profile2);
    }
    
    let data1 = pm.load_profile(profile1)?;
    let data2 = pm.load_profile(profile2)?;
    
    let json1 = serde_json::to_string_pretty(&data1)?;
    let json2 = serde_json::to_string_pretty(&data2)?;
    
    if json1 == json2 {
        println!("{} Profiles are identical", "=".green());
        return Ok(());
    }
    
    println!("{} {} vs {}", "Diff:".bold(), profile1.red(), profile2.green());
    println!();
    
    let diff = TextDiff::from_lines(&json1, &json2);
    
    for change in diff.iter_all_changes() {
        let line = change.value();
        match change.tag() {
            ChangeTag::Delete => print!("{}", format!("- {}", line).red()),
            ChangeTag::Insert => print!("{}", format!("+ {}", line).green()),
            ChangeTag::Equal => print!("  {}", line),
        }
    }
    
    Ok(())
}

pub fn backup(name: Option<&str>) -> Result<()> {
    let pm = ProfileManager::new()?;
    pm.config.ensure_dirs()?;
    
    if !pm.config.settings_file.exists() {
        bail!("No settings.json found to backup");
    }
    
    let backup_name = match name {
        Some(n) => n.to_string(),
        None => Local::now().format("backup-%Y%m%d-%H%M%S").to_string(),
    };
    
    let data = pm.load_settings()?;
    pm.save_backup(&backup_name, &data)?;
    
    println!("{} Created backup '{}'", "✓".green(), backup_name.cyan());
    println!("  Path: {}", pm.config.backup_path(&backup_name).display());
    Ok(())
}

pub fn restore(backup: &str) -> Result<()> {
    let pm = ProfileManager::new()?;
    
    // Check if it's a backup or profile
    let data = if pm.config.backup_path(backup).exists() {
        pm.load_backup(backup)?
    } else if pm.profile_exists(backup) {
        pm.load_profile(backup)?
    } else {
        // List available backups
        let backups = pm.list_backups()?;
        if backups.is_empty() {
            bail!("Backup '{}' not found and no backups available", backup);
        } else {
            println!("{}", "Available backups:".bold());
            for b in &backups {
                println!("  {}", b);
            }
            bail!("Backup '{}' not found", backup);
        }
    };
    
    // Create backup of current before restoring
    if pm.config.settings_file.exists() {
        let auto_backup = Local::now().format("pre-restore-%Y%m%d-%H%M%S").to_string();
        let current = pm.load_settings()?;
        pm.save_backup(&auto_backup, &current)?;
        eprintln!("{} Created auto-backup '{}'", "ℹ".blue(), auto_backup);
    }
    
    pm.save_settings(&data)?;
    
    println!("{} Restored from '{}'", "✓".green(), backup.cyan());
    Ok(())
}

pub fn completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
    Ok(())
}
