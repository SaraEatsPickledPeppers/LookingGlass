//! Cheats Mod - Config Injection Script
//! This script registers default config and config validation for the cheats mod.

use looking_glass_mod::{
    ModApi, ModContext, ConfigValidator, Result,
    async_trait,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Cheats configuration matching the schema in mod.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CheatsConfig {
    pub infinite_health: bool,
    pub infinite_will: bool,
    pub god_mode: bool,
    pub no_target: bool,
    pub infinite_ammo: bool,
    pub all_weapons: bool,
    pub max_weapon_copies: bool,
    pub one_hit_kill: bool,
    pub no_clip: bool,
}

/// Config injector - registers default config and validator
pub struct CheatsConfigInjector;

impl CheatsConfigInjector {
    /// Register default config and validator with the config registry
    pub fn inject(api: &ModApi) {
        let default_config = CheatsConfig::default();
        let config_json = serde_json::to_value(&default_config).unwrap();

        // Set default values in config registry
        api.register_config_validator("cheats".to_string(), Box::new(CheatsConfigValidator));

        // The actual default config is set by the mod loader from mod.toml
        // This function ensures the validator is registered
    }
}

/// Config validator for cheats configuration
struct CheatsConfigValidator;

#[async_trait]
impl ConfigValidator for CheatsConfigValidator {
    async fn validate(&self, value: &serde_json::Value) -> Result<()> {
        // Validate that all values are booleans
        if let Some(obj) = value.as_object() {
            for (k, v) in obj {
                if !v.is_boolean() {
                    return Err(anyhow::anyhow!(
                        "Cheats config '{}' must be a boolean, got {}",
                        k,
                        v
                    ));
                }
            }
        }
        Ok(())
    }

    async fn apply_defaults(&self, value: &mut serde_json::Value) {
        let defaults = CheatsConfig::default();
        let default_json = serde_json::to_value(defaults).unwrap();

        if let Some(obj) = value.as_object_mut() {
            if let Some(default_obj) = default_json.as_object() {
                for (k, v) in default_obj {
                    if !obj.contains_key(k) {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
        }
    }
}

/// Entry point for config injection
#[no_mangle]
pub extern "C" fn inject_config(api: &ModApi, _context: &ModContext) {
    CheatsConfigInjector::inject(api);
}