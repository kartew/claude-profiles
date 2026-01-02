use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;

fn create_test_home() -> (TempDir, assert_fs::NamedTempFile) {
    let temp_dir = TempDir::new().unwrap();
    let config_file = assert_fs::NamedTempFile::new(".claude/settings.json").unwrap();

    // Create the directory structure
    let _ = std::fs::create_dir_all(temp_dir.path().join(".claude/profiles"));
    let _ = std::fs::create_dir_all(temp_dir.path().join(".claude/backups"));

    // Create initial settings.json
    config_file.write_str(r#"{"$schema": "https://json.schemastore.org/claude-code-settings.json"}"#).unwrap();

    // Create .current file with default
    std::fs::write(temp_dir.path().join(".claude/profiles/.current"), "default").unwrap();

    // Create default profile
    let default_profile = temp_dir.path().join(".claude/profiles/default.json");
    std::fs::write(default_profile, r#"{"model": "sonnet-4"}"#).unwrap();

    (temp_dir, config_file)
}

#[test]
fn test_cli_list_profiles() {
    let (home_dir, _) = create_test_home();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("list")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("default"));
}

#[test]
fn test_cli_current() {
    let (home_dir, _) = create_test_home();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("current")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("default"));
}

#[test]
fn test_cli_create_profile() {
    let (home_dir, _) = create_test_home();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("create")
        .arg("test-profile")
        .output()
        .unwrap();

    assert!(output.status.success(), "stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));

    let profile_path = home_dir.path().join(".claude/profiles/test-profile.json");
    assert!(profile_path.exists());
}

#[test]
fn test_cli_create_profile_from_existing() {
    let (home_dir, _) = create_test_home();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("create")
        .arg("new-profile")
        .arg("--from")
        .arg("default")
        .output()
        .unwrap();

    assert!(output.status.success());

    let profile_path = home_dir.path().join(".claude/profiles/new-profile.json");
    assert!(profile_path.exists());

    let content = std::fs::read_to_string(profile_path).unwrap();
    let data: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(data["model"], "sonnet-4");
}

#[test]
fn test_cli_copy_profile() {
    let (home_dir, _) = create_test_home();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("copy")
        .arg("default")
        .arg("copied-profile")
        .output()
        .unwrap();

    assert!(output.status.success());

    let copied_path = home_dir.path().join(".claude/profiles/copied-profile.json");
    assert!(copied_path.exists());
}

#[test]
fn test_cli_rename_profile() {
    let (home_dir, _) = create_test_home();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("rename")
        .arg("default")
        .arg("renamed-profile")
        .output()
        .unwrap();

    assert!(output.status.success());

    let old_path = home_dir.path().join(".claude/profiles/default.json");
    let new_path = home_dir.path().join(".claude/profiles/renamed-profile.json");
    assert!(!old_path.exists());
    assert!(new_path.exists());
}

#[test]
fn test_cli_delete_profile() {
    let (home_dir, _) = create_test_home();

    // First create a profile to delete
    let profile_path = home_dir.path().join(".claude/profiles/to-delete.json");
    std::fs::write(&profile_path, r#"{"test": true}"#).unwrap();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("delete")
        .arg("to-delete")
        .arg("--force")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(!profile_path.exists());
}

#[test]
fn test_cli_use_profile() {
    let (home_dir, _) = create_test_home();

    // Create a profile to switch to
    let profile_path = home_dir.path().join(".claude/profiles/other.json");
    std::fs::write(&profile_path, r#"{"model": "haiku-3"}"#).unwrap();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("use")
        .arg("other")
        .output()
        .unwrap();

    assert!(output.status.success());

    let current_file = home_dir.path().join(".claude/profiles/.current");
    let current = std::fs::read_to_string(current_file).unwrap();
    assert_eq!(current.trim(), "other");
}

#[test]
fn test_cli_set_and_get() {
    let (home_dir, _) = create_test_home();

    // Set a value
    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("set")
        .arg("model")
        .arg("opus-4")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Get the value
    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("get")
        .arg("model")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("opus-4"));
}

#[test]
fn test_cli_import() {
    let (home_dir, _) = create_test_home();

    // Import a profile using echo
    let json_data = r#"{"model": "opus-4", "custom": "value"}"#;

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("echo '{}' | HOME={} cargo run --quiet --bin ccp -- import imported-profile",
            json_data, home_dir.path().display()))
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let imported_path = home_dir.path().join(".claude/profiles/imported-profile.json");
    assert!(imported_path.exists());

    let content = std::fs::read_to_string(imported_path).unwrap();
    let data: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(data["model"], "opus-4");
}

#[test]
fn test_cli_backup_restore() {
    let (home_dir, _) = create_test_home();

    // Modify settings.json first
    let settings_path = home_dir.path().join(".claude/settings.json");
    std::fs::write(&settings_path, r#"{"model": "modified", "custom": "value"}"#).unwrap();

    // Create backup
    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("backup")
        .arg("my-backup")
        .output()
        .unwrap();

    assert!(output.status.success());

    let backup_path = home_dir.path().join(".claude/backups/my-backup.json");
    assert!(backup_path.exists());

    // Modify settings again
    std::fs::write(&settings_path, r#"{"model": "changed"}"#).unwrap();

    // Restore from backup
    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("restore")
        .arg("my-backup")
        .output()
        .unwrap();

    assert!(output.status.success());

    let content = std::fs::read_to_string(&settings_path).unwrap();
    let data: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(data["model"], "modified");
    assert_eq!(data["custom"], "value");
}

#[test]
fn test_cli_diff() {
    let (home_dir, _) = create_test_home();

    // Create two different profiles
    let profile1_path = home_dir.path().join(".claude/profiles/profile1.json");
    let profile2_path = home_dir.path().join(".claude/profiles/profile2.json");

    std::fs::write(&profile1_path, r#"{"model": "sonnet-4"}"#).unwrap();
    std::fs::write(&profile2_path, r#"{"model": "haiku-3"}"#).unwrap();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("diff")
        .arg("profile1")
        .arg("profile2")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Diff should show the difference
    assert!(stdout.contains("sonnet-4") || stdout.contains("haiku-3"));
}

#[test]
fn test_cli_error_nonexistent_profile() {
    let (home_dir, _) = create_test_home();

    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("use")
        .arg("nonexistent")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist") || stderr.contains("error"));
}

#[test]
fn test_cli_unset() {
    let (home_dir, _) = create_test_home();

    // Set a value first
    let profile_path = home_dir.path().join(".claude/profiles/default.json");
    std::fs::write(&profile_path, r#"{"model": "sonnet-4", "custom": "value"}"#).unwrap();

    // Unset the value
    let mut cmd = Command::cargo_bin("ccp").unwrap();
    let output = cmd
        .env("HOME", home_dir.path())
        .arg("unset")
        .arg("custom")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify it's removed
    let content = std::fs::read_to_string(&profile_path).unwrap();
    let data: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(data.get("custom").is_none());
}
