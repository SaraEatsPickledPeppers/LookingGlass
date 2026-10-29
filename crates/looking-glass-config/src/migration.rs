use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current config version
pub const CONFIG_VERSION: u32 = 2;

/// Config version stored in the config file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub version: u32,
}

impl Default for ConfigVersion {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
        }
    }
}

/// Migration trait for version upgrades
pub trait Migrate {
    fn migrate(value: Value) -> Result<Value>;
}

/// Migration from v1 to v2
pub struct V1ToV2;

impl Migrate for V1ToV2 {
    fn migrate(mut value: Value) -> Result<Value> {
        // v1 structure had flat keys, v2 uses nested sections
        // Map old flat keys to new nested structure

        let mut new_config = serde_json::Map::new();

        // GameConfig section
        let mut game_config = serde_json::Map::new();
        if let Some(data_path) = value.as_object_mut().and_then(|o| o.remove("data_path")) {
            game_config.insert("data_path".to_string(), data_path);
        }
        if let Some(auto_setup) = value.as_object_mut().and_then(|o| o.remove("auto_setup")) {
            game_config.insert("auto_setup".to_string(), auto_setup);
        }
        if let Some(preferred_language) = value.as_object_mut().and_then(|o| o.remove("preferred_language")) {
            game_config.insert("preferred_language".to_string(), preferred_language);
        }
        if !game_config.is_empty() {
            new_config.insert("game".to_string(), Value::Object(game_config));
        }

        // LauncherConfig section
        let mut launcher_config = serde_json::Map::new();
        if let Some(log_level) = value.as_object_mut().and_then(|o| o.remove("log_level")) {
            launcher_config.insert("log_level".to_string(), log_level);
        }
        if let Some(auto_check_updates) = value.as_object_mut().and_then(|o| o.remove("auto_check_updates")) {
            launcher_config.insert("auto_check_updates".to_string(), auto_check_updates);
        }
        if let Some(theme) = value.as_object_mut().and_then(|o| o.remove("theme")) {
            launcher_config.insert("theme".to_string(), theme);
        }
        if let Some(first_run) = value.as_object_mut().and_then(|o| o.remove("first_run")) {
            launcher_config.insert("first_run".to_string(), first_run);
        }
        if !launcher_config.is_empty() {
            new_config.insert("launcher".to_string(), Value::Object(launcher_config));
        }

        // PlatformConfig section
        let mut platform_config = serde_json::Map::new();
        if let Some(steam_path) = value.as_object_mut().and_then(|o| o.remove("steam_path")) {
            platform_config.insert("steam_path".to_string(), steam_path);
        }
        if let Some(heroic_path) = value.as_object_mut().and_then(|o| o.remove("heroic_path")) {
            platform_config.insert("heroic_path".to_string(), heroic_path);
        }
        if let Some(flatpak_enabled) = value.as_object_mut().and_then(|o| o.remove("flatpak_enabled")) {
            platform_config.insert("flatpak_enabled".to_string(), flatpak_enabled);
        }
        if let Some(wine_prefix) = value.as_object_mut().and_then(|o| o.remove("wine_prefix")) {
            platform_config.insert("wine_prefix".to_string(), wine_prefix);
        }
        if let Some(proton_version) = value.as_object_mut().and_then(|o| o.remove("proton_version")) {
            platform_config.insert("proton_version".to_string(), proton_version);
        }
        if !platform_config.is_empty() {
            new_config.insert("platform".to_string(), Value::Object(platform_config));
        }

        // ModConfig section
        let mut mod_config = serde_json::Map::new();
        if let Some(mod_dirs) = value.as_object_mut().and_then(|o| o.remove("mod_dirs")) {
            mod_config.insert("mod_dirs".to_string(), mod_dirs);
        }
        if let Some(auto_enable_new) = value.as_object_mut().and_then(|o| o.remove("auto_enable_new")) {
            mod_config.insert("auto_enable_new".to_string(), auto_enable_new);
        }
        if let Some(check_updates) = value.as_object_mut().and_then(|o| o.remove("check_updates")) {
            mod_config.insert("check_updates".to_string(), check_updates);
        }
        if let Some(default_profile) = value.as_object_mut().and_then(|o| o.remove("default_profile")) {
            mod_config.insert("default_profile".to_string(), default_profile);
        }
        if !mod_config.is_empty() {
            new_config.insert("mods".to_string(), Value::Object(mod_config));
        }

        // SaveConfig section
        let mut save_config = serde_json::Map::new();
        if let Some(backup_dir) = value.as_object_mut().and_then(|o| o.remove("backup_dir")) {
            save_config.insert("backup_dir".to_string(), backup_dir);
        }
        if let Some(auto_backup) = value.as_object_mut().and_then(|o| o.remove("auto_backup")) {
            save_config.insert("auto_backup".to_string(), auto_backup);
        }
        if let Some(max_backups) = value.as_object_mut().and_then(|o| o.remove("max_backups")) {
            save_config.insert("max_backups".to_string(), max_backups);
        }
        if let Some(sync_enabled) = value.as_object_mut().and_then(|o| o.remove("sync_enabled")) {
            save_config.insert("sync_enabled".to_string(), sync_enabled);
        }
        if let Some(sync_provider) = value.as_object_mut().and_then(|o| o.remove("sync_provider")) {
            save_config.insert("sync_provider".to_string(), sync_provider);
        }
        if !save_config.is_empty() {
            new_config.insert("saves".to_string(), Value::Object(save_config));
        }

        // Preserve any unknown keys at root level
        if let Value::Object(ref mut remaining) = value {
            let keys: Vec<String> = remaining.keys().cloned().collect();
            for k in keys {
                if let Some(v) = remaining.remove(&k) {
                    new_config.insert(k, v);
                }
            }
        }

        // Add version
        new_config.insert("version".to_string(), Value::Number(CONFIG_VERSION.into()));

        Ok(Value::Object(new_config))
    }
}

/// Run all necessary migrations to bring config to current version
pub fn migrate_config(mut value: Value) -> Result<Value> {
    let current_version = value
        .get("version")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    if current_version >= CONFIG_VERSION {
        return Ok(value);
    }

    // Apply migrations sequentially
    for version in current_version..CONFIG_VERSION {
        value = match version {
            1 => V1ToV2::migrate(value).context("Failed to migrate from v1 to v2")?,
            _ => {
                anyhow::bail!("Unknown config version: {}", version);
            }
        };
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migrate_v1_to_v2() {
        let v1_config = json!({
            "data_path": "/path/to/game",
            "auto_setup": true,
            "preferred_language": "en",
            "log_level": "info",
            "auto_check_updates": true,
            "theme": "dark",
            "first_run": true,
            "steam_path": "/path/to/steam",
            "heroic_path": "/path/to/heroic",
            "flatpak_enabled": false,
            "wine_prefix": "/path/to/wine",
            "proton_version": "GE-Proton-8",
            "mod_dirs": ["/mods"],
            "auto_enable_new": true,
            "check_updates": true,
            "default_profile": "default",
            "backup_dir": "/backups",
            "auto_backup": true,
            "max_backups": 10,
            "sync_enabled": false,
            "sync_provider": "local"
        });

        let migrated = V1ToV2::migrate(v1_config).unwrap();

        assert_eq!(migrated["version"], 2);
        assert_eq!(migrated["game"]["data_path"], "/path/to/game");
        assert_eq!(migrated["game"]["auto_setup"], true);
        assert_eq!(migrated["launcher"]["log_level"], "info");
        assert_eq!(migrated["platform"]["steam_path"], "/path/to/steam");
        assert_eq!(migrated["mods"]["mod_dirs"][0], "/mods");
        assert_eq!(migrated["saves"]["backup_dir"], "/backups");
    }

    #[test]
    fn test_migrate_already_current() {
        let v2_config = json!({
            "version": 2,
            "game": {
                "data_path": "/path/to/game"
            }
        });

        let result = migrate_config(v2_config).unwrap();
        assert_eq!(result["version"], 2);
    }
}