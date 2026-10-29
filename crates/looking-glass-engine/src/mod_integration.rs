//! Mod integration for Looking Glass game engine
//! Provides mod loading, asset overrides, input hooks, and runtime hooks

use crate::preferences::Preferences;
use looking_glass_mod::{
    config::{ConfigManager, GlobalConfig, GameConfig},
    discovery::ModDiscovery,
    loader::{AssetOverrideResult, ModApi, ModContext, ModLoader, ModRegistry, InputHook, RuntimeHook, ButtonOption, KeyCode, KeyModifiers, GamepadButton, ButtonState, UiPatch, WidgetDef},
    manifest::{GameTarget, AssetType, InstalledMod, ModManifest, ModEntryPoints, LoadOrder},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{Context, Result};

/// Mod integration state for the game engine
pub struct ModIntegration {
    registry: ModRegistry,
    config_manager: ConfigManager,
    discovery: ModDiscovery,
    game_version: String,
    enabled: bool,
    mods_dir: Option<PathBuf>,
}

impl ModIntegration {
    /// Create a new ModIntegration with default settings
    pub fn new(game_version: String) -> Result<Self> {
        let config_manager = ConfigManager::new()?;
        let merged_config = config_manager.get_merged();

        // Get mods directory from config or use default
        let game_config = merged_config.game_configs.get("Alice").cloned().unwrap_or_else(|| {
            GameConfig {
                game_target: GameTarget::Alice,
                mods_dir: None,
                enabled_mods: Vec::new(),
                disabled_mods: Vec::new(),
            }
        });

        let mods_dir = game_config.mods_dir.clone();

        // Use XDG data directory as fallback
        let xdg = looking_glass_mod::config::resolve_xdg_paths();

        // ModDiscovery expects the base directory (parent of "mods"), not the game-specific mods dir
        // If config provides mods_dir, use its parent. Otherwise use XDG data dir.
        let base_dir = mods_dir
            .clone()
            .map(|p| p.parent().unwrap_or(&p).to_path_buf())
            .unwrap_or_else(|| xdg.looking_glass_data());

        let discovery = ModDiscovery::new(base_dir, "Alice".to_string());

        let mut integration = Self {
            registry: ModRegistry::new(),
            config_manager,
            discovery,
            game_version,
            enabled: true,
            mods_dir,
        };

        // Load mods on creation
        integration.reload_mods()?;

        Ok(integration)
    }

    /// Create a new ModIntegration with custom mods directory
    /// The mods_dir parameter should be the game-specific mods directory (e.g., /path/to/mods/Alice)
    /// We pass its parent to ModDiscovery which expects the base directory containing "mods/"
    pub fn with_mods_dir(game_version: String, mods_dir: PathBuf) -> Result<Self> {
        let config_manager = ConfigManager::new()?;
        let base_dir = mods_dir.parent().unwrap_or(&mods_dir).to_path_buf();
        let discovery = ModDiscovery::new(base_dir, "Alice".to_string());

        let mut integration = Self {
            registry: ModRegistry::new(),
            config_manager,
            discovery,
            game_version,
            enabled: true,
            mods_dir: Some(mods_dir),
        };

        integration.reload_mods()?;
        Ok(integration)
    }

    /// Create ModIntegration without loading mods (for --no-mods flag)
    pub fn disabled(game_version: String) -> Result<Self> {
        let config_manager = ConfigManager::new()?;
        let xdg = looking_glass_mod::config::resolve_xdg_paths();
        let discovery = ModDiscovery::new(
            xdg.looking_glass_data(), // ModDiscovery will append "mods/Alice"
            "Alice".to_string(),
        );

        Ok(Self {
            registry: ModRegistry::new(),
            config_manager,
            discovery,
            game_version,
            enabled: false,
            mods_dir: None,
        })
    }

    /// Check if mod integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable mod integration
    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        if enabled != self.enabled {
            self.enabled = enabled;
            if enabled {
                self.reload_mods()?;
            } else {
                // Unload all mods
                for mod_id in self.registry.get_load_order() {
                    self.registry.unregister(&mod_id)?;
                }
            }
        }
        Ok(())
    }

    /// Reload all mods from disk
    pub fn reload_mods(&mut self) -> Result<()> {
        if !self.enabled {
            tracing::info!("Mod integration disabled, skipping mod load");
            return Ok(());
        }

        tracing::info!("Reloading mods...");

        // Unload existing mods
        for mod_id in self.registry.get_load_order() {
            self.registry.unregister(&mod_id)?;
        }

        // Check for mod injection via environment variables (from launcher)
        let env_mods = self.get_env_mods();

        // Discover installed mods
        let installed_mods = self.discovery.discover()?;
        tracing::info!("Discovered {} installed mods", installed_mods.len());
        for m in &installed_mods {
            tracing::info!("  Found mod: {} v{} (enabled: {})", m.manifest.id, m.manifest.version, m.enabled);
        }

        // Determine which mods to load
        let mods_to_load = if !env_mods.is_empty() {
            // Use environment variable injection (from launcher)
            tracing::info!("Using mod injection from environment variables");
            self.filter_mods_by_env(installed_mods, &env_mods)?
        } else {
            // Use config-based mod selection
            let merged_config = self.config_manager.get_merged();
            let game_config = merged_config.game_configs.get("Alice");
            let enabled_mods: Vec<String> = game_config
                .map(|gc| gc.enabled_mods.clone())
                .unwrap_or_default();
            let disabled_mods: Vec<String> = game_config
                .map(|gc| gc.disabled_mods.clone())
                .unwrap_or_default();

            tracing::info!("Config enabled_mods: {:?}", enabled_mods);
            tracing::info!("Config disabled_mods: {:?}", disabled_mods);

            installed_mods
                .into_iter()
                .filter(|m| {
                    if !enabled_mods.is_empty() && !enabled_mods.contains(&m.manifest.id) {
                        tracing::info!("  Skipping mod {} (not in enabled_mods)", m.manifest.id);
                        return false;
                    }
                    if disabled_mods.contains(&m.manifest.id) {
                        tracing::info!("  Skipping mod {} (in disabled_mods)", m.manifest.id);
                        return false;
                    }
                    m.enabled
                })
                .collect()
        };

        tracing::info!("Mods to load: {}", mods_to_load.len());
        for m in &mods_to_load {
            tracing::info!("  Will load: {}", m.manifest.id);
        }

        // Load mods
        for installed_mod in mods_to_load {
            // Create a simple mod wrapper that uses entry points
            let mod_wrapper = SimpleMod::new(installed_mod);
            self.registry.register(Box::new(mod_wrapper))?;
        }

        // Sort load order based on dependencies
        self.registry.sort_load_order()?;

        tracing::info!("Mods loaded successfully");
        Ok(())
    }

    /// Get mod paths from environment variables set by launcher
    fn get_env_mods(&self) -> Vec<(String, PathBuf)> {
        let mut mods = Vec::new();

        // Check LOOKING_GLASS_MODS (colon-separated paths)
        if let Ok(mods_str) = std::env::var("LOOKING_GLASS_MODS") {
            for path_str in mods_str.split(':') {
                let path = PathBuf::from(path_str);
                // Extract mod ID from path (last component)
                if let Some(mod_id) = path.file_name().and_then(|s| s.to_str()) {
                    mods.push((mod_id.to_string(), path));
                }
            }
            tracing::info!("Found {} mods from LOOKING_GLASS_MODS", mods.len());
            return mods;
        }

        // Check LOOKING_GLASS_MOD_{i} (individual mod paths)
        let mut i = 0;
        loop {
            let key = format!("LOOKING_GLASS_MOD_{}", i);
            if let Ok(path_str) = std::env::var(&key) {
                let path = PathBuf::from(path_str);
                if let Some(mod_id) = path.file_name().and_then(|s| s.to_str()) {
                    mods.push((mod_id.to_string(), path));
                }
                i += 1;
            } else {
                break;
            }
        }

        if !mods.is_empty() {
            tracing::info!("Found {} mods from LOOKING_GLASS_MOD_{{i}}", mods.len());
        }

        mods
    }

    /// Filter installed mods based on environment variable injection
    fn filter_mods_by_env(&self, installed_mods: Vec<InstalledMod>, env_mods: &[(String, PathBuf)]) -> Result<Vec<InstalledMod>> {
        // Build a set of mod IDs from environment variables
        let env_mod_ids: std::collections::HashSet<_> = env_mods.iter().map(|(id, _)| id.clone()).collect();

        // Also check LOOKING_GLASS_MOD_ORDER for load order
        let load_order: Vec<String> = std::env::var("LOOKING_GLASS_MOD_ORDER")
            .map(|s| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default();

        if !load_order.is_empty() {
            tracing::info!("Mod load order from env: {:?}", load_order);
        }

        // Filter installed mods to only those in env_mods
        let mut filtered: Vec<InstalledMod> = installed_mods
            .into_iter()
            .filter(|m| env_mod_ids.contains(&m.manifest.id))
            .collect();

        // Sort by load order if specified
        if !load_order.is_empty() {
            filtered.sort_by(|a, b| {
                let a_pos = load_order.iter().position(|id| id == &a.manifest.id).unwrap_or(usize::MAX);
                let b_pos = load_order.iter().position(|id| id == &b.manifest.id).unwrap_or(usize::MAX);
                a_pos.cmp(&b_pos)
            });
        }

        Ok(filtered)
    }

    /// Get the mod registry
    pub fn registry(&self) -> &ModRegistry {
        &self.registry
    }

    /// Try to override an asset path
    /// Returns asset data if a mod provides an override
    pub fn try_override_asset(&self, path: &str) -> Option<AssetOverrideResult> {
        if !self.enabled {
            return None;
        }
        self.registry.try_override_asset(path)
    }

    /// Try to override a button prompt through mod input hooks
    /// Returns the mod-provided label if any mod wants to override it
    pub fn try_override_button_prompt(&self, button: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }
        // Check if any mod has an input hook that wants to override this button
        // Note: This is a synchronous check - in a full implementation with async hooks,
        // we would need a different approach (e.g., a cached result from the async call)
        let hooks = self.registry.get_input_hooks();
        let hooks = hooks.lock().unwrap();
        if !hooks.is_empty() {
            // We have input hooks registered, but we can't call the async method here
            // For now, return None - a proper implementation would cache the result
            // from the async process_button_prompt call
        }
        None
    }

    /// Process a button prompt through input hooks
    /// Returns the selected option index if a mod handles it
    pub async fn process_button_prompt(
        &self,
        prompt_id: &str,
        options: &[ButtonOption],
    ) -> Option<usize> {
        if !self.enabled {
            return None;
        }
        self.registry.process_button_prompt(prompt_id, options).await
    }

    /// Update runtime hooks (called each frame)
    pub async fn update(&self, delta_time: f32) {
        if !self.enabled {
            return;
        }
        self.registry.update(delta_time).await;
    }

    /// Get mod config value for a specific mod
    pub fn get_mod_config(&self, mod_id: &str) -> Option<serde_json::Value> {
        self.config_manager.get_mod_config_value(mod_id).cloned()
    }

    /// Set mod config value
    pub fn set_mod_config(&mut self, mod_id: &str, config: serde_json::Value) -> Result<()> {
        self.config_manager.set_mod_config(mod_id, config)
    }

    /// Enable a mod
    pub fn enable_mod(&mut self, mod_id: &str) -> Result<()> {
        self.config_manager.set_mod_enabled(mod_id, true)?;
        self.reload_mods()
    }

    /// Disable a mod
    pub fn disable_mod(&mut self, mod_id: &str) -> Result<()> {
        self.config_manager.set_mod_enabled(mod_id, false)?;
        self.reload_mods()
    }

    /// Install a mod from an archive
    pub fn install_mod(&mut self, archive_path: &Path) -> Result<InstalledMod> {
        let installed = self.discovery.install_from_archive(archive_path)?;
        self.reload_mods()?;
        Ok(installed)
    }

    /// Uninstall a mod
    pub fn uninstall_mod(&mut self, mod_id: &str) -> Result<()> {
        self.discovery.uninstall(mod_id)?;
        self.reload_mods()
    }

    /// Get list of installed mods
    pub fn list_mods(&self) -> Result<Vec<InstalledMod>> {
        self.discovery.discover()
    }

    /// Set custom mods directory
    pub fn set_mods_dir(&mut self, path: PathBuf) -> Result<()> {
        self.mods_dir = Some(path.clone());
        self.discovery = ModDiscovery::new(path.clone(), "Alice".to_string());
        self.config_manager.set_game_mods_dir("Alice", path)?;
        self.reload_mods()
    }

    /// Get the mods directory
    pub fn mods_dir(&self) -> PathBuf {
        self.discovery.mods_dir()
    }
}

/// A simple mod wrapper that uses entry points from the manifest
struct SimpleMod {
    installed: InstalledMod,
}

impl SimpleMod {
    fn new(installed: InstalledMod) -> Self {
        Self { installed }
    }
}

#[async_trait::async_trait]
impl looking_glass_mod::loader::Mod for SimpleMod {
    async fn initialize(&mut self, context: &ModContext) -> Result<()> {
        // Register asset overrides from manifest
        for asset in &self.installed.manifest.assets {
            context.mod_api.register_asset_override(
                asset.target.clone(),
                self.installed.path.join(&asset.source),
                asset.asset_type.clone(),
                asset.condition.clone(),
                100, // default priority
            );
        }

        // Register input hook if specified
        if let Some(input_hook_path) = &self.installed.manifest.entry_points.input_hook {
            // In a real implementation, this would load a dynamic library or WASM module
            // For now, we'll skip dynamic loading
            log::info!("Input hook specified but not loaded: {}", input_hook_path);
        }

        // Register runtime hook if specified
        if let Some(runtime_hook_path) = &self.installed.manifest.entry_points.runtime_hook {
            log::info!("Runtime hook specified but not loaded: {}", runtime_hook_path);
        }

        // Register UI patch if specified
        if let Some(ui_patch_path) = &self.installed.manifest.entry_points.ui_patch {
            log::info!("UI patch specified but not loaded: {}", ui_patch_path);
        }

        // Register config validator if config schema exists
        if let Some(config_schema) = &self.installed.manifest.config_schema {
            let validator = ConfigSchemaValidator {
                schema: config_schema.clone(),
            };
            context.mod_api.register_config_validator(
                self.installed.manifest.id.clone(),
                Box::new(validator),
            );
        }

        // Apply default config if present
        if let Some(default_config) = &self.installed.manifest.default_config {
            context.mod_api.register_config_validator(
                format!("{}_defaults", self.installed.manifest.id),
                Box::new(DefaultConfigApplier {
                    defaults: default_config.clone(),
                }),
            );
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    async fn on_config_changed(&mut self, key: &str, value: &serde_json::Value) -> Result<()> {
        log::debug!("Mod {} config changed: {} = {}", self.installed.manifest.id, key, value);
        Ok(())
    }

    fn manifest(&self) -> &ModManifest {
        &self.installed.manifest
    }

    fn entry_points(&self) -> &ModEntryPoints {
        &self.installed.manifest.entry_points
    }
}

/// Config validator that validates against a JSON schema
struct ConfigSchemaValidator {
    schema: serde_json::Value,
}

#[async_trait::async_trait]
impl looking_glass_mod::loader::ConfigValidator for ConfigSchemaValidator {
    async fn validate(&self, value: &serde_json::Value) -> Result<()> {
        // Create validator using new jsonschema 0.60 API
        let validator = jsonschema::validator_for(&self.schema)
            .map_err(|e| anyhow::anyhow!("Failed to compile config schema: {}", e))?;

        if let Err(_) = validator.validate(value) {
            // Collect all validation errors using iter_errors
            let error_msgs: Vec<String> = validator.iter_errors(value)
                .map(|e| e.to_string())
                .collect();
            return Err(anyhow::anyhow!(
                "Config validation failed:\n{}",
                error_msgs.join("\n")
            ));
        }
        Ok(())
    }

    async fn apply_defaults(&self, _value: &mut serde_json::Value) {
        // Schema doesn't provide defaults, skip
    }
}

/// Config validator that applies default values
struct DefaultConfigApplier {
    defaults: serde_json::Value,
}

#[async_trait::async_trait]
impl looking_glass_mod::loader::ConfigValidator for DefaultConfigApplier {
    async fn validate(&self, _value: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    async fn apply_defaults(&self, value: &mut serde_json::Value) {
        apply_defaults_recursive(value, &self.defaults);
    }
}

fn apply_defaults_recursive(target: &mut serde_json::Value, defaults: &serde_json::Value) {
    match (target, defaults) {
        (serde_json::Value::Object(target_map), serde_json::Value::Object(defaults_map)) => {
            for (key, default_val) in defaults_map {
                if !target_map.contains_key(key) {
                    target_map.insert(key.clone(), default_val.clone());
                } else {
                    apply_defaults_recursive(&mut target_map[key], default_val);
                }
            }
        }
        _ => {}
    }
}

/// Global mod integration instance (initialized in main.rs)
static mut MOD_INTEGRATION: Option<ModIntegration> = None;

/// Initialize the global mod integration
pub fn init_mod_integration(game_version: String, mods_dir: Option<PathBuf>, no_mods: bool) -> Result<()> {
    let integration = if no_mods {
        ModIntegration::disabled(game_version)?
    } else if let Some(dir) = mods_dir {
        ModIntegration::with_mods_dir(game_version, dir)?
    } else {
        ModIntegration::new(game_version)?
    };

    unsafe {
        MOD_INTEGRATION = Some(integration);
    }
    Ok(())
}

/// Get the global mod integration instance
pub fn mod_integration() -> Option<&'static ModIntegration> {
    unsafe { MOD_INTEGRATION.as_ref() }
}

/// Get mutable reference to global mod integration
pub fn mod_integration_mut() -> Option<&'static mut ModIntegration> {
    unsafe { MOD_INTEGRATION.as_mut() }
}

/// Try to override an asset (convenience function)
pub fn try_override_asset(path: &str) -> Option<AssetOverrideResult> {
    mod_integration()?.try_override_asset(path)
}

/// Process a button prompt through mod hooks (convenience function)
pub async fn process_button_prompt(prompt_id: &str, options: &[ButtonOption]) -> Option<usize> {
    mod_integration()?.process_button_prompt(prompt_id, options).await
}

/// Update mod runtime hooks (convenience function)
pub async fn update_mods(delta_time: f32) {
    if let Some(integration) = mod_integration() {
        integration.update(delta_time).await;
    }
}

/// Try to override a button prompt through mod input hooks (convenience function)
/// Returns the mod-provided label if any mod wants to override it
pub fn try_override_button_prompt(button: &str) -> Option<String> {
    // For now, we'll check if any mod has an input hook that wants to override this button
    // This is a synchronous version - in a real implementation with async hooks, we'd need to handle this differently
    if let Some(integration) = mod_integration() {
        // Check if any mod has an input hook that wants to override this button
        // For now, we'll just check the registry for any input hooks
        // In a full implementation, this would call through to the async InputHook::on_button_prompt
        integration.try_override_button_prompt(button)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_mod_integration_creation() {
        let dir = tempdir().unwrap();
        let integration = ModIntegration::with_mods_dir("0.32.0".to_string(), dir.path().to_path_buf()).unwrap();
        assert!(integration.is_enabled());
        assert_eq!(integration.game_version, "0.32.0");
    }

    #[test]
    fn test_mod_integration_disabled() {
        let integration = ModIntegration::disabled("0.32.0".to_string()).unwrap();
        assert!(!integration.is_enabled());
    }

    #[test]
    fn test_asset_override_registration() {
        let dir = tempdir().unwrap();
        let mod_dir = dir.path().join("mods").join("Alice").join("test_mod").join("assets");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("test.png"), b"fake png").unwrap();

        let manifest_path = dir.path().join("mods").join("Alice").join("test_mod").join("mod.toml");
        fs::write(&manifest_path, r#"
            id = "test_mod"
            name = "Test Mod"
            version = "1.0.0"
            manifest_version = 2
            game_target = "Alice"
            assets = [
                {source = "assets/test.png", target = "textures/test.png", type = "Texture"}
            ]
        "#).unwrap();

        let mut integration = ModIntegration::with_mods_dir("0.32.0".to_string(), dir.path().join("mods")).unwrap();
        integration.reload_mods().unwrap();

        // Try to override the asset
        let result = integration.try_override_asset("textures/test.png");
        assert!(result.is_some());
        assert_eq!(result.unwrap().data, b"fake png");
    }

    #[test]
    fn test_mod_enable_disable() {
        let dir = tempdir().unwrap();
        let mod_dir = dir.path().join("mods").join("Alice").join("test_mod").join("assets");
        fs::create_dir_all(&mod_dir).unwrap();
        fs::write(mod_dir.join("test.png"), b"fake png").unwrap();

        let manifest_path = dir.path().join("mods").join("Alice").join("test_mod").join("mod.toml");
        fs::write(&manifest_path, r#"
            id = "test_mod"
            name = "Test Mod"
            version = "1.0.0"
            manifest_version = 2
            game_target = "Alice"
            assets = [
                {source = "assets/test.png", target = "textures/test.png", type = "Texture"}
            ]
        "#).unwrap();

        let mut integration = ModIntegration::with_mods_dir("0.32.0".to_string(), dir.path().join("mods")).unwrap();

        // Mod should be enabled by default
        assert!(integration.try_override_asset("textures/test.png").is_some());

        // Disable the mod
        integration.disable_mod("test_mod").unwrap();
        assert!(integration.try_override_asset("textures/test.png").is_none());

        // Re-enable the mod
        integration.enable_mod("test_mod").unwrap();
        assert!(integration.try_override_asset("textures/test.png").is_some());
    }
}