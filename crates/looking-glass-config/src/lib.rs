mod migration;
pub mod paths;

pub use migration::{migrate_config, ConfigVersion, CONFIG_VERSION};
pub use paths::{
    cache_dir, config_dir, config_file_path, data_dir, local_config_file_path, log_dir, mod_dirs,
    save_dir,
};

use anyhow::{Context, Result};
use notify::{
    event::{ModifyKind, EventKind},
    Event, RecommendedWatcher, RecursiveMode, Watcher,
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

/// Configuration layer priority (higher = more specific, overrides lower)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ConfigLayer {
    /// Layer 0: Built-in defaults (lowest priority)
    Defaults = 0,
    /// Layer 1: Global XDG config (~/.config/looking-glass/config.toml)
    Global = 1,
    /// Layer 2: Project local (private/config.toml)
    Local = 2,
    /// Layer 3: CLI overrides (highest priority)
    Cli = 3,
}

impl ConfigLayer {
    pub fn all() -> [ConfigLayer; 4] {
        [
            ConfigLayer::Defaults,
            ConfigLayer::Global,
            ConfigLayer::Local,
            ConfigLayer::Cli,
        ]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "defaults" | "default" => Some(ConfigLayer::Defaults),
            "global" => Some(ConfigLayer::Global),
            "local" => Some(ConfigLayer::Local),
            "cli" => Some(ConfigLayer::Cli),
            _ => None,
        }
    }
}

/// Game configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GameConfig {
    /// Path to game data directory
    pub data_path: Option<PathBuf>,
    /// Automatically set up game on first run
    pub auto_setup: bool,
    /// Preferred language code (e.g., "en", "de", "fr")
    pub preferred_language: String,
}

/// Launcher configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LauncherConfig {
    /// Log level: trace, debug, info, warn, error
    pub log_level: String,
    /// Automatically check for updates
    pub auto_check_updates: bool,
    /// UI theme: light, dark, system
    pub theme: String,
    /// Whether this is the first run
    pub first_run: bool,
}

/// Platform configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PlatformConfig {
    /// Steam installation path
    pub steam_path: Option<PathBuf>,
    /// Heroic Games Launcher path
    pub heroic_path: Option<PathBuf>,
    /// Enable Flatpak support
    pub flatpak_enabled: bool,
    /// Wine prefix path
    pub wine_prefix: Option<PathBuf>,
    /// Proton version to use
    pub proton_version: String,
}

/// Per-mod configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ModEntryConfig {
    /// Whether this mod is enabled
    pub enabled: bool,
    /// Mod-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Load priority (higher = loaded first)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_priority: Option<i32>,
}

/// Mod configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ModConfig {
    /// Mod directories (in priority order)
    pub mod_dirs: Vec<PathBuf>,
    /// Automatically enable newly installed mods
    pub auto_enable_new: bool,
    /// Check for mod updates
    pub check_updates: bool,
    /// Default mod profile name
    pub default_profile: String,
    /// Per-mod configuration (enabled state, custom config, priority)
    #[serde(default)]
    pub mods: std::collections::HashMap<String, ModEntryConfig>,
}

/// Save configuration section
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SaveConfig {
    /// Backup directory path
    pub backup_dir: Option<PathBuf>,
    /// Enable automatic backups
    pub auto_backup: bool,
    /// Maximum number of backups to keep
    pub max_backups: usize,
    /// Enable save synchronization
    pub sync_enabled: bool,
    /// Sync provider: local, steam, gog, custom
    pub sync_provider: String,
}

/// Main configuration struct with all sections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Config file version for migration
    #[serde(flatten)]
    pub version: ConfigVersion,
    /// Game configuration
    pub game: GameConfig,
    /// Launcher configuration
    pub launcher: LauncherConfig,
    /// Platform configuration
    pub platform: PlatformConfig,
    /// Mod configuration
    pub mods: ModConfig,
    /// Save configuration
    pub saves: SaveConfig,
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self {
            version: ConfigVersion::default(),
            game: GameConfig {
                data_path: None,
                auto_setup: true,
                preferred_language: "en".to_string(),
            },
            launcher: LauncherConfig {
                log_level: "info".to_string(),
                auto_check_updates: true,
                theme: "system".to_string(),
                first_run: true,
            },
            platform: PlatformConfig {
                steam_path: None,
                heroic_path: None,
                flatpak_enabled: false,
                wine_prefix: None,
                proton_version: "GE-Proton-8".to_string(),
            },
            mods: ModConfig {
                mod_dirs: paths::mod_dirs(),
                auto_enable_new: false,
                check_updates: true,
                default_profile: "default".to_string(),
                mods: std::collections::HashMap::new(),
            },
            saves: SaveConfig {
                backup_dir: Some(paths::save_dir().join("backups")),
                auto_backup: true,
                max_backups: 10,
                sync_enabled: false,
                sync_provider: "local".to_string(),
            },
        }
    }

    /// Merge another config into this one (other takes precedence)
    pub fn merge(&mut self, other: Config) {
        // Merge version (take higher)
        self.version.version = self.version.version.max(other.version.version);

        // Merge game config
        if other.game.data_path.is_some() {
            self.game.data_path = other.game.data_path;
        }
        self.game.auto_setup = other.game.auto_setup;
        if !other.game.preferred_language.is_empty() {
            self.game.preferred_language = other.game.preferred_language;
        }

        // Merge launcher config
        if !other.launcher.log_level.is_empty() {
            self.launcher.log_level = other.launcher.log_level;
        }
        self.launcher.auto_check_updates = other.launcher.auto_check_updates;
        if !other.launcher.theme.is_empty() {
            self.launcher.theme = other.launcher.theme;
        }
        self.launcher.first_run = other.launcher.first_run;

        // Merge platform config
        if other.platform.steam_path.is_some() {
            self.platform.steam_path = other.platform.steam_path;
        }
        if other.platform.heroic_path.is_some() {
            self.platform.heroic_path = other.platform.heroic_path;
        }
        self.platform.flatpak_enabled = other.platform.flatpak_enabled;
        if other.platform.wine_prefix.is_some() {
            self.platform.wine_prefix = other.platform.wine_prefix;
        }
        if !other.platform.proton_version.is_empty() {
            self.platform.proton_version = other.platform.proton_version;
        }

        // Merge mod config
        if !other.mods.mod_dirs.is_empty() {
            self.mods.mod_dirs = other.mods.mod_dirs;
        }
        self.mods.auto_enable_new = other.mods.auto_enable_new;
        self.mods.check_updates = other.mods.check_updates;
        if !other.mods.default_profile.is_empty() {
            self.mods.default_profile = other.mods.default_profile;
        }
        // Merge per-mod config (other layer takes precedence)
        self.mods.mods.extend(other.mods.mods.into_iter());

        // Merge save config
        if other.saves.backup_dir.is_some() {
            self.saves.backup_dir = other.saves.backup_dir;
        }
        self.saves.auto_backup = other.saves.auto_backup;
        if other.saves.max_backups > 0 {
            self.saves.max_backups = other.saves.max_backups;
        }
        self.saves.sync_enabled = other.saves.sync_enabled;
        if !other.saves.sync_provider.is_empty() {
            self.saves.sync_provider = other.saves.sync_provider;
        }
    }
}

/// Layered configuration manager
#[derive(Debug, Clone)]
pub struct LayeredConfig {
    layers: std::collections::HashMap<ConfigLayer, Config>,
}

impl Default for LayeredConfig {
    fn default() -> Self {
        let mut layers = std::collections::HashMap::new();
        layers.insert(ConfigLayer::Defaults, Config::new());
        layers.insert(ConfigLayer::Global, Config::default());
        layers.insert(ConfigLayer::Local, Config::default());
        layers.insert(ConfigLayer::Cli, Config::default());
        Self { layers }
    }
}

impl LayeredConfig {
    /// Create a new layered config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a specific layer
    pub fn get_layer(&self, layer: ConfigLayer) -> &Config {
        self.layers
            .get(&layer)
            .unwrap_or_else(|| self.layers.get(&ConfigLayer::Defaults).unwrap())
    }

    /// Get a mutable reference to a specific layer
    pub fn get_layer_mut(&mut self, layer: ConfigLayer) -> &mut Config {
        self.layers
            .entry(layer)
            .or_insert_with(Config::default)
    }

    /// Resolve the final merged configuration
    pub fn resolve(&self) -> Config {
        let mut result = Config::new();

        // Apply layers in priority order (lowest to highest)
        for layer in ConfigLayer::all() {
            if let Some(config) = self.layers.get(&layer) {
                result.merge(config.clone());
            }
        }

        result
    }

    /// Load configuration from all layers
    pub fn load() -> Result<Self> {
        let mut layered = Self::new();

        // Layer 1: Global XDG config
        let global_path = paths::config_file_path();
        if global_path.exists() {
            let content = std::fs::read_to_string(&global_path)
                .with_context(|| format!("Failed to read global config: {:?}", global_path))?;
            let mut config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse global config: {:?}", global_path))?;
            config = migrate_config(serde_json::to_value(config)?)?
                .try_into()
                .context("Failed to convert migrated config")?;
            layered.layers.insert(ConfigLayer::Global, config);
            info!("Loaded global config from {:?}", global_path);
        }

        // Layer 2: Local project config
        if let Some(local_path) = paths::local_config_file_path() {
            if local_path.exists() {
                let content = std::fs::read_to_string(&local_path)
                    .with_context(|| format!("Failed to read local config: {:?}", local_path))?;
                let mut config: Config = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse local config: {:?}", local_path))?;
                config = migrate_config(serde_json::to_value(config)?)?
                    .try_into()
                    .context("Failed to convert migrated config")?;
                layered.layers.insert(ConfigLayer::Local, config);
                info!("Loaded local config from {:?}", local_path);
            }
        }

        Ok(layered)
    }

    /// Save a specific layer to disk
    pub fn save_layer(&self, layer: ConfigLayer) -> Result<()> {
        let config = self.get_layer(layer);
        let path = match layer {
            ConfigLayer::Global => paths::config_file_path(),
            ConfigLayer::Local => {
                paths::local_config_file_path()
                    .ok_or_else(|| anyhow::anyhow!("Cannot determine local config path"))?
            }
            ConfigLayer::Defaults => {
                return Err(anyhow::anyhow!("Cannot save defaults layer"));
            }
            ConfigLayer::Cli => {
                return Err(anyhow::anyhow!("Cannot save CLI layer"));
            }
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let content = toml::to_string_pretty(config)
            .with_context(|| format!("Failed to serialize config for layer {:?}", layer))?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {:?}", path))?;

        info!("Saved {:?} config to {:?}", layer, path);
        Ok(())
    }

    /// Get a value by dotted key path (e.g., "game.data_path")
    pub fn get(&self, key: &str) -> Option<Value> {
        let resolved = self.resolve();
        let value = serde_json::to_value(resolved).ok()?;
        get_value_by_path(&value, key)
    }

    /// Set a value by dotted key path in a specific layer
    pub fn set(&mut self, key: &str, value: Value, layer: ConfigLayer) -> Result<()> {
        let config = self.get_layer_mut(layer);
        let mut value_map = serde_json::to_value(config.clone())
            .context("Failed to serialize config")?
            .as_object_mut()
            .cloned()
            .unwrap_or_default();

        set_value_by_path(&mut value_map, key, value)?;

        let new_config: Config = serde_json::from_value(Value::Object(value_map))
            .context("Failed to deserialize modified config")?;
        *config = new_config;

        Ok(())
    }

    /// Reset a key to its default (remove from specific layer)
    pub fn reset(&mut self, key: &str, layer: ConfigLayer) -> Result<()> {
        let config = self.get_layer_mut(layer);
        let mut value_map = serde_json::to_value(config.clone())
            .context("Failed to serialize config")?
            .as_object_mut()
            .cloned()
            .unwrap_or_default();

        remove_value_by_path(&mut value_map, key)?;

        let new_config: Config = serde_json::from_value(Value::Object(value_map))
            .context("Failed to deserialize modified config")?;
        *config = new_config;

        Ok(())
    }
}

/// Watch handle for config file changes
#[derive(Debug)]
pub struct NotifyHandle {
    _watcher: RecommendedWatcher,
    // We don't need to keep the receiver, just the watcher
    // The watcher will run in the background and call our callback
}

impl NotifyHandle {
    /// Create a new file watcher for config files
    pub fn new<F>(callback: F) -> Result<Self>
    where
        F: Fn() + Send + 'static,
    {
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(ref event) = res {
                // Only trigger on modify events for config files
                if matches!(
                    event.kind,
                    EventKind::Modify(ModifyKind::Data(_)) | EventKind::Create(_)
                ) {
                    for path in &event.paths {
                        if path.file_name().map_or(false, |n| {
                            n == "config.toml" || n == "config.toml.tmp"
                        }) {
                            callback();
                            break;
                        }
                    }
                }
            }
            // Ignore send errors (channel closed)
            let _ = res;
        })?;

        // Watch global config directory
        let global_dir = paths::config_dir();
        if global_dir.exists() {
            watcher
                .watch(&global_dir, RecursiveMode::NonRecursive)
                .context("Failed to watch global config directory")?;
        }

        // Watch local config directory
        if let Some(local_path) = paths::local_config_file_path() {
            if let Some(parent) = local_path.parent() {
                if parent.exists() {
                    watcher
                        .watch(parent, RecursiveMode::NonRecursive)
                        .context("Failed to watch local config directory")?;
                }
            }
        }

        Ok(Self { _watcher: watcher })
    }
}

/// Thread-safe config manager with live reload support
#[derive(Debug, Clone)]
pub struct ConfigManager {
    layered: Arc<RwLock<LayeredConfig>>,
    _notify_handle: Option<Arc<NotifyHandle>>,
}

impl ConfigManager {
    /// Create a new config manager and load configuration
    pub fn new() -> Result<Self> {
        let layered = LayeredConfig::load()?;
        Ok(Self {
            layered: Arc::new(RwLock::new(layered)),
            _notify_handle: None,
        })
    }

    /// Create a new config manager with live reload
    pub fn new_with_watch<F>(on_change: F) -> Result<Self>
    where
        F: Fn() + Send + 'static,
    {
        let layered = LayeredConfig::load()?;
        let notify_handle = NotifyHandle::new(on_change)?;

        Ok(Self {
            layered: Arc::new(RwLock::new(layered)),
            _notify_handle: Some(Arc::new(notify_handle)),
        })
    }

    /// Get the current resolved configuration
    pub fn get_config(&self) -> Config {
        let mut config = self.layered.read().resolve();

        // Extract per-mod config from layered config dotted paths
        // and populate the ModConfig.mods HashMap
        let layered = self.layered.read();
        let mut mods_map = std::collections::HashMap::new();

        // Get all keys that match "mods.mods.*.enabled" or "mods.mods.*.config" or "mods.mods.*.load_priority"
        if let Some(value) = layered.get("mods.mods") {
            if let Some(obj) = value.as_object() {
                for (mod_id, mod_value) in obj {
                    if let Some(mod_obj) = mod_value.as_object() {
                        let mut entry = ModEntryConfig::default();
                        if let Some(enabled) = mod_obj.get("enabled").and_then(|v| v.as_bool()) {
                            entry.enabled = enabled;
                        }
                        if let Some(config_val) = mod_obj.get("config") {
                            entry.config = Some(config_val.clone());
                        }
                        if let Some(priority) = mod_obj.get("load_priority").and_then(|v| v.as_i64()) {
                            entry.load_priority = Some(priority as i32);
                        }
                        mods_map.insert(mod_id.clone(), entry);
                    }
                }
            }
        }

        config.mods.mods = mods_map;
        config
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<Value> {
        self.layered.read().get(key)
    }

    /// Set a value in a specific layer
    pub fn set(&self, key: &str, value: Value, layer: ConfigLayer) -> Result<()> {
        self.layered.write().set(key, value, layer)
    }

    /// Reset a key in a specific layer
    pub fn reset(&self, key: &str, layer: ConfigLayer) -> Result<()> {
        self.layered.write().reset(key, layer)
    }

    /// Save a specific layer to disk
    pub fn save_layer(&self, layer: ConfigLayer) -> Result<()> {
        self.layered.read().save_layer(layer)
    }

    /// Reload configuration from disk
    pub fn reload(&self) -> Result<()> {
        let new_layered = LayeredConfig::load()?;
        *self.layered.write() = new_layered;
        info!("Configuration reloaded");
        Ok(())
    }

    // Mod-specific config methods (for Alice-only scope)
    /// Get a mod-specific config value
    pub fn get_mod_config_value(&self, mod_id: &str) -> Option<Value> {
        self.get(&format!("mods.mods.{}.config", mod_id))
    }

    /// Set a mod-specific config value
    pub fn set_mod_config(&self, mod_id: &str, config: Value, layer: ConfigLayer) -> Result<()> {
        self.set(&format!("mods.mods.{}.config", mod_id), config, layer)
    }

    /// Enable or disable a mod
    pub fn set_mod_enabled(&self, mod_id: &str, enabled: bool, layer: ConfigLayer) -> Result<()> {
        self.set(&format!("mods.mods.{}.enabled", mod_id), Value::Bool(enabled), layer)
    }

    /// Set the mods directory for a game (simplified for Alice-only)
    pub fn set_game_mods_dir(&self, _game_id: &str, path: PathBuf, layer: ConfigLayer) -> Result<()> {
        self.set("mods.mod_dirs", Value::Array(vec![Value::String(path.to_string_lossy().to_string())]), layer)
    }
}

/// Global config manager instance
static GLOBAL_CONFIG: Lazy<ConfigManager> = Lazy::new(|| {
    ConfigManager::new().expect("Failed to initialize global config manager")
});

/// Get the global config manager
pub fn global_config() -> &'static ConfigManager {
    &GLOBAL_CONFIG
}

/// Helper to get a value by dotted path
fn get_value_by_path(value: &Value, path: &str) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        current = current.get(part)?;
    }

    Some(current.clone())
}

/// Helper to set a value by dotted path
fn set_value_by_path(map: &mut serde_json::Map<String, Value>, path: &str, value: Value) -> Result<()> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = map;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            current.insert(part.to_string(), value);
            return Ok(());
        }

        let next = current.entry(part.to_string()).or_insert_with(|| Value::Object(serde_json::Map::new()));
        current = next.as_object_mut().ok_or_else(|| anyhow::anyhow!("Path '{}' is not an object", path))?;
    }

    Ok(())
}

/// Helper to remove a value by dotted path
fn remove_value_by_path(map: &mut serde_json::Map<String, Value>, path: &str) -> Result<()> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = map;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            current.remove(*part);
            return Ok(());
        }

        current = match current.get_mut(*part) {
            Some(Value::Object(obj)) => obj,
            _ => return Ok(()), // Path doesn't exist
        };
    }

    Ok(())
}

// Implement TryFrom<Value> for Config to support migration
impl TryFrom<Value> for Config {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self> {
        serde_json::from_value(value).context("Failed to convert Value to Config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    fn test_config_defaults() {
        let config = Config::new();
        assert_eq!(config.launcher.log_level, "info");
        assert_eq!(config.launcher.theme, "system");
        assert!(config.launcher.auto_check_updates);
        assert!(config.game.auto_setup);
        assert_eq!(config.game.preferred_language, "en");
        assert!(!config.platform.flatpak_enabled);
        assert_eq!(config.platform.proton_version, "GE-Proton-8");
        assert!(!config.mods.auto_enable_new);
        assert!(config.mods.check_updates);
        assert_eq!(config.mods.default_profile, "default");
        assert!(config.saves.auto_backup);
        assert_eq!(config.saves.max_backups, 10);
        assert!(!config.saves.sync_enabled);
        assert_eq!(config.saves.sync_provider, "local");
    }

    #[test]
    fn test_config_merge() {
        let mut base = Config::new();
        let mut overlay = Config::new();

        overlay.game.data_path = Some(PathBuf::from("/custom/path"));
        overlay.launcher.log_level = "debug".to_string();
        overlay.platform.proton_version = "GE-Proton-9".to_string();

        base.merge(overlay);

        assert_eq!(base.game.data_path, Some(PathBuf::from("/custom/path")));
        assert_eq!(base.launcher.log_level, "debug");
        assert_eq!(base.platform.proton_version, "GE-Proton-9");
        // Unchanged values should remain
        assert_eq!(base.launcher.theme, "system");
    }

    #[test]
    fn test_layered_config_resolve() {
        let mut layered = LayeredConfig::new();

        // Set value in global layer
        layered
            .get_layer_mut(ConfigLayer::Global)
            .launcher
            .log_level = "warn".to_string();

        // Set value in local layer (higher priority)
        layered
            .get_layer_mut(ConfigLayer::Local)
            .launcher
            .log_level = "debug".to_string();

        let resolved = layered.resolve();
        assert_eq!(resolved.launcher.log_level, "debug");
    }

    #[test]
    fn test_layered_config_cli_override() {
        let mut layered = LayeredConfig::new();

        layered
            .get_layer_mut(ConfigLayer::Global)
            .launcher
            .log_level = "warn".to_string();
        layered
            .get_layer_mut(ConfigLayer::Local)
            .launcher
            .log_level = "debug".to_string();
        layered
            .get_layer_mut(ConfigLayer::Cli)
            .launcher
            .log_level = "trace".to_string();

        let resolved = layered.resolve();
        assert_eq!(resolved.launcher.log_level, "trace");
    }

    #[test]
    fn test_get_value_by_path() {
        let config = Config::new();
        let value = serde_json::to_value(config).unwrap();

        assert_eq!(
            get_value_by_path(&value, "launcher.log_level"),
            Some(Value::String("info".to_string()))
        );
        assert_eq!(
            get_value_by_path(&value, "game.auto_setup"),
            Some(Value::Bool(true))
        );
        assert_eq!(
            get_value_by_path(&value, "nonexistent.key"),
            None
        );
    }

    #[test]
    fn test_set_value_by_path() {
        let mut map = serde_json::Map::new();
        set_value_by_path(&mut map, "launcher.log_level", Value::String("debug".to_string())).unwrap();

        assert_eq!(map["launcher"]["log_level"], "debug");
    }

    #[test]
    fn test_layered_config_get_set() {
        let mut layered = LayeredConfig::new();

        // Set via layered config
        layered
            .set("launcher.log_level", Value::String("debug".to_string()), ConfigLayer::Global)
            .unwrap();

        let value = layered.get("launcher.log_level");
        assert_eq!(value, Some(Value::String("debug".to_string())));
    }

    #[test]
    fn test_layered_config_reset() {
        let mut layered = LayeredConfig::new();

        layered
            .set("launcher.log_level", Value::String("debug".to_string()), ConfigLayer::Global)
            .unwrap();
        assert_eq!(layered.get("launcher.log_level"), Some(Value::String("debug".to_string())));

        layered.reset("launcher.log_level", ConfigLayer::Global).unwrap();
        // Should fall back to defaults
        assert_eq!(layered.get("launcher.log_level"), Some(Value::String("info".to_string())));
    }

    #[test]
    #[serial]
    fn test_config_file_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config::new();
        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, content).unwrap();

        // Read it back
        let read_content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: Config = toml::from_str(&read_content).unwrap();

        assert_eq!(parsed.launcher.log_level, "info");
        assert_eq!(parsed.game.preferred_language, "en");
    }
}