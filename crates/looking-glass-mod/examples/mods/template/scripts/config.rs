//! Template Mod - Config Injection Script
//! This is a placeholder for config injection. Copy and modify for your mod.

use looking_glass_mod::{ModApi, ModContext, ConfigValidator, Result, async_trait};
use serde::{Deserialize, Serialize};

/// Example mod configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConfig {
    pub enabled: bool,
    pub setting1: String,
    pub setting2: f64,
    pub setting3: bool,
}

impl Default for ModConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            setting1: "value1".to_string(),
            setting2: 42.0,
            setting3: false,
        }
    }
}

/// Config validator for this mod
struct ModConfigValidator;

#[async_trait]
impl ConfigValidator for ModConfigValidator {
    async fn validate(&self, value: &serde_json::Value) -> Result<()> {
        if let Some(obj) = value.as_object() {
            if let Some(v) = obj.get("enabled") {
                if !v.is_boolean() {
                    return Err(anyhow::anyhow!("'enabled' must be a boolean"));
                }
            }
            if let Some(v) = obj.get("setting1") {
                if !v.is_string() {
                    return Err(anyhow::anyhow!("'setting1' must be a string"));
                }
            }
            if let Some(v) = obj.get("setting2") {
                if !v.is_number() {
                    return Err(anyhow::anyhow!("'setting2' must be a number"));
                }
            }
            if let Some(v) = obj.get("setting3") {
                if !v.is_boolean() {
                    return Err(anyhow::anyhow!("'setting3' must be a boolean"));
                }
            }
        }
        Ok(())
    }

    async fn apply_defaults(&self, value: &mut serde_json::Value) {
        let defaults = ModConfig::default();
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

/// Register config validator and defaults
pub fn inject_config(api: &ModApi) {
    api.register_config_validator("template".to_string(), Box::new(ModConfigValidator));
}

/// Entry point called by the mod loader
#[no_mangle]
pub extern "C" fn inject_config_entry(api: &ModApi, _context: &ModContext) {
    inject_config(api);
}