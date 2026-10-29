use crate::manifest::{GameTarget, InstalledMod, ModManifest};
use anyhow::{Context, Result};
use dirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub mods: HashMap<String, ModConfig>,
    #[serde(default)]
    pub game_configs: HashMap<String, GameConfig>,
    #[serde(default)]
    pub settings: Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameConfig {
    pub game_target: GameTarget,
    pub mods_dir: Option<PathBuf>,
    #[serde(default)]
    pub enabled_mods: Vec<String>,
    #[serde(default)]
    pub disabled_mods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_auto_update")]
    pub auto_update: bool,
    #[serde(default = "default_check_updates")]
    pub check_updates_on_startup: bool,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub experimental_features: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_update: default_auto_update(),
            check_updates_on_startup: default_check_updates(),
            log_level: default_log_level(),
            experimental_features: false,
        }
    }
}

fn default_auto_update() -> bool {
    false
}

fn default_check_updates() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredConfig {
    pub defaults: GlobalConfig,
    pub global: GlobalConfig,
    pub project: Option<GlobalConfig>,
    pub cli: Option<GlobalConfig>,
}

impl LayeredConfig {
    pub fn new() -> Self {
        Self {
            defaults: GlobalConfig::default(),
            global: GlobalConfig::default(),
            project: None,
            cli: None,
        }
    }

    pub fn merge(&self) -> GlobalConfig {
        let mut merged = self.defaults.clone();
        merged.merge_from(&self.global);

        if let Some(project) = &self.project {
            merged.merge_from(project);
        }

        if let Some(cli) = &self.cli {
            merged.merge_from(cli);
        }

        merged
    }
}

impl GlobalConfig {
    pub fn merge_from(&mut self, other: &GlobalConfig) {
        // Merge mods
        for (id, config) in &other.mods {
            self.mods.insert(id.clone(), config.clone());
        }

        // Merge game configs
        for (id, config) in &other.game_configs {
            self.game_configs.insert(id.clone(), config.clone());
        }

        // Merge settings (other takes precedence)
        if other.settings.auto_update {
            self.settings.auto_update = other.settings.auto_update;
        }
        if other.settings.check_updates_on_startup {
            self.settings.check_updates_on_startup = other.settings.check_updates_on_startup;
        }
        if !other.settings.log_level.is_empty() {
            self.settings.log_level = other.settings.log_level.clone();
        }
        if other.settings.experimental_features {
            self.settings.experimental_features = other.settings.experimental_features;
        }
    }

    pub fn get_mod_config(&self, mod_id: &str) -> Option<&ModConfig> {
        self.mods.get(mod_id)
    }

    pub fn set_mod_config(&mut self, mod_id: String, config: ModConfig) {
        self.mods.insert(mod_id, config);
    }

    pub fn get_game_config(&self, game_id: &str) -> Option<&GameConfig> {
        self.game_configs.get(game_id)
    }

    pub fn set_game_config(&mut self, game_id: String, config: GameConfig) {
        self.game_configs.insert(game_id, config);
    }
}

pub struct ConfigManager {
    config_dir: PathBuf,
    config_file: PathBuf,
    layered: LayeredConfig,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_file = config_dir.join("mods.toml");

        let mut manager = Self {
            config_dir,
            config_file,
            layered: LayeredConfig::new(),
        };

        manager.load()?;
        Ok(manager)
    }

    fn get_config_dir() -> Result<PathBuf> {
        // XDG_CONFIG_HOME/looking-glass
        if let Some(config_home) = dirs::config_dir() {
            Ok(config_home.join("looking-glass"))
        } else {
            // Fallback to ~/.config/looking-glass
            let home = dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
            Ok(home.join(".config").join("looking-glass"))
        }
    }

    pub fn load(&mut self) -> Result<()> {
        // Load defaults
        self.layered.defaults = Self::default_config();

        // Load global config
        if self.config_file.exists() {
            let content = fs::read_to_string(&self.config_file)
                .with_context(|| format!("Failed to read config file: {}", self.config_file.display()))?;
            self.layered.global = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", self.config_file.display()))?;
        }

        // Load project config (if in a project directory)
        if let Some(project_config) = Self::find_project_config()? {
            let content = fs::read_to_string(&project_config)?;
            self.layered.project = Some(toml::from_str(&content)?);
        }

        Ok(())
    }

    fn find_project_config() -> Result<Option<PathBuf>> {
        let mut current = std::env::current_dir()?;

        loop {
            let config_path = current.join("looking-glass.toml");
            if config_path.exists() {
                return Ok(Some(config_path));
            }

            if !current.pop() {
                break;
            }
        }

        Ok(None)
    }

    pub fn apply_cli_overrides(&mut self, overrides: HashMap<String, serde_json::Value>) {
        let mut cli_config = GlobalConfig::default();

        for (key, value) in overrides {
            match key.as_str() {
                "auto_update" => cli_config.settings.auto_update = value.as_bool().unwrap_or(false),
                "check_updates" => cli_config.settings.check_updates_on_startup = value.as_bool().unwrap_or(true),
                "log_level" => cli_config.settings.log_level = value.as_str().unwrap_or("info").to_string(),
                "experimental" => cli_config.settings.experimental_features = value.as_bool().unwrap_or(false),
                _ => {
                    // Try to parse as mod config
                    if let Ok(mod_config) = serde_json::from_value::<ModConfig>(value) {
                        // Extract mod ID from key if it's in format "mod.<id>.<field>"
                        if let Some(mod_id) = key.strip_prefix("mod.") {
                            if let Some(field) = mod_id.strip_suffix(".enabled") {
                                let mut config = cli_config.mods.entry(field.to_string()).or_default();
                                config.enabled = mod_config.enabled;
                            }
                        }
                    }
                }
            }
        }

        self.layered.cli = Some(cli_config);
    }

    pub fn get_merged(&self) -> GlobalConfig {
        self.layered.merge()
    }

    pub fn save(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)
            .with_context(|| format!("Failed to create config directory: {}", self.config_dir.display()))?;

        let content = toml::to_string_pretty(&self.layered.global)
            .context("Failed to serialize config")?;

        fs::write(&self.config_file, content)
            .with_context(|| format!("Failed to write config file: {}", self.config_file.display()))?;

        Ok(())
    }

    pub fn set_mod_enabled(&mut self, mod_id: &str, enabled: bool) -> Result<()> {
        let mut config = self.layered.global.mods
            .entry(mod_id.to_string())
            .or_default();
        config.enabled = enabled;
        self.save()
    }

    pub fn set_mod_config(&mut self, mod_id: &str, config: serde_json::Value) -> Result<()> {
        let mut mod_config = self.layered.global.mods
            .entry(mod_id.to_string())
            .or_default();
        mod_config.config = Some(config);
        self.save()
    }

    pub fn get_mod_config_value(&self, mod_id: &str) -> Option<&serde_json::Value> {
        self.layered.global.mods.get(mod_id)?.config.as_ref()
    }

    pub fn get_game_mods_dir(&self, game_id: &str) -> PathBuf {
        self.layered.global.game_configs
            .get(game_id)
            .and_then(|g| g.mods_dir.clone())
            .unwrap_or_else(|| self.config_dir.join("mods").join(game_id))
    }

    pub fn set_game_mods_dir(&mut self, game_id: &str, path: PathBuf) -> Result<()> {
        let game_config = self.layered.global.game_configs
            .entry(game_id.to_string())
            .or_insert_with(|| GameConfig {
                game_target: GameTarget::Alice, // Default, will be updated
                ..Default::default()
            });
        game_config.mods_dir = Some(path);
        self.save()
    }

    fn default_config() -> GlobalConfig {
        let mut config = GlobalConfig::default();
        config.settings = Settings::default();
        config
    }

    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn config_file(&self) -> &PathBuf {
        &self.config_file
    }
}

pub fn resolve_xdg_paths() -> XdgPaths {
    XdgPaths {
        config_home: dirs::config_dir().unwrap_or_else(|| {
            dirs::home_dir().unwrap().join(".config")
        }),
        data_home: dirs::data_dir().unwrap_or_else(|| {
            dirs::home_dir().unwrap().join(".local").join("share")
        }),
        cache_home: dirs::cache_dir().unwrap_or_else(|| {
            dirs::home_dir().unwrap().join(".cache")
        }),
        state_home: dirs::home_dir().unwrap().join(".local").join("state"),
    }
}

#[derive(Debug, Clone)]
pub struct XdgPaths {
    pub config_home: PathBuf,
    pub data_home: PathBuf,
    pub cache_home: PathBuf,
    pub state_home: PathBuf,
}

impl XdgPaths {
    pub fn looking_glass_config(&self) -> PathBuf {
        self.config_home.join("looking-glass")
    }

    pub fn looking_glass_data(&self) -> PathBuf {
        self.data_home.join("looking-glass")
    }

    pub fn looking_glass_cache(&self) -> PathBuf {
        self.cache_home.join("looking-glass")
    }

    pub fn looking_glass_state(&self) -> PathBuf {
        self.state_home.join("looking-glass")
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.looking_glass_config())?;
        fs::create_dir_all(self.looking_glass_data())?;
        fs::create_dir_all(self.looking_glass_cache())?;
        fs::create_dir_all(self.looking_glass_state())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_global_config_default() {
        let config = GlobalConfig::default();
        assert!(!config.settings.auto_update);
        assert!(config.settings.check_updates_on_startup);
        assert_eq!(config.settings.log_level, "info");
    }

    #[test]
    fn test_config_merge() {
        let mut base = GlobalConfig::default();
        base.settings.auto_update = false;
        base.settings.log_level = "debug".to_string();

        let mut override_config = GlobalConfig::default();
        override_config.settings.auto_update = true;
        override_config.settings.log_level = "trace".to_string();

        base.merge_from(&override_config);

        assert!(base.settings.auto_update);
        assert_eq!(base.settings.log_level, "trace");
    }

    #[test]
    fn test_layered_config() {
        let mut layered = LayeredConfig::new();
        layered.defaults.settings.log_level = "warn".to_string();
        layered.global.settings.log_level = "info".to_string();
        layered.project = Some(GlobalConfig {
            settings: Settings {
                log_level: "debug".to_string(),
                ..Default::default()
            },
            ..Default::default()
        });

        let merged = layered.merge();
        assert_eq!(merged.settings.log_level, "debug");
    }

    #[test]
    fn test_config_manager() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join("config");
        fs::create_dir_all(&config_dir).unwrap();

        let config_file = config_dir.join("mods.toml");
        fs::write(&config_file, r#"
        [settings]
        auto_update = true
        log_level = "debug"
        "#).unwrap();

        let mut manager = ConfigManager {
            config_dir: config_dir.clone(),
            config_file: config_file.clone(),
            layered: LayeredConfig::new(),
        };

        manager.load().unwrap();
        let merged = manager.get_merged();

        assert!(merged.settings.auto_update);
        assert_eq!(merged.settings.log_level, "debug");
    }

    #[test]
    fn test_mod_config() {
        let mut config = GlobalConfig::default();
        config.set_mod_config("test_mod".to_string(), ModConfig {
            enabled: true,
            config: Some(serde_json::json!({"key": "value"})),
            load_priority: Some(10),
        });

        let mod_config = config.get_mod_config("test_mod").unwrap();
        assert!(mod_config.enabled);
        assert_eq!(mod_config.config.as_ref().unwrap()["key"], "value");
        assert_eq!(mod_config.load_priority, Some(10));
    }

    #[test]
    fn test_xdg_paths() {
        let paths = resolve_xdg_paths();
        assert!(paths.config_home.ends_with(".config") || paths.config_home.ends_with("config"));
        assert!(paths.data_home.ends_with(".local/share") || paths.data_home.ends_with("data"));
    }
}