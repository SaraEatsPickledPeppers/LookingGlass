// Config Injection Example
// This module demonstrates how to register custom config validators

use looking_glass_mod::loader::{ConfigValidator, ModApi};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// Validator for custom_difficulty config option
pub struct CustomDifficultyValidator;

#[async_trait]
impl ConfigValidator for CustomDifficultyValidator {
    async fn validate(&self, value: &Value) -> Result<()> {
        let difficulty = value.as_str().ok_or_else(|| anyhow::anyhow!("custom_difficulty must be a string"))?;

        match difficulty {
            "easy" | "normal" | "hard" | "custom" => Ok(()),
            _ => Err(anyhow::anyhow!("Invalid difficulty: {}. Must be one of: easy, normal, hard, custom", difficulty)),
        }
    }

    async fn apply_defaults(&self, value: &mut Value) {
        if value.get("custom_difficulty").is_none() {
            value["custom_difficulty"] = Value::String("normal".to_string());
        }
    }
}

/// Validator for custom_damage_multiplier config option
pub struct DamageMultiplierValidator;

#[async_trait]
impl ConfigValidator for DamageMultiplierValidator {
    async fn validate(&self, value: &Value) -> Result<()> {
        let multiplier = value.as_f64().ok_or_else(|| anyhow::anyhow!("custom_damage_multiplier must be a number"))?;

        if multiplier < 0.1 || multiplier > 5.0 {
            return Err(anyhow::anyhow!("custom_damage_multiplier must be between 0.1 and 5.0, got {}", multiplier));
        }

        Ok(())
    }

    async fn apply_defaults(&self, value: &mut Value) {
        if value.get("custom_damage_multiplier").is_none() {
            value["custom_damage_multiplier"] = Value::Number(serde_json::Number::from_f64(1.0).unwrap());
        }
    }
}

/// Register all config validators for this mod
pub fn register_validators(api: &ModApi) {
    api.register_config_validator(
        "mods.config_injection_example.custom_difficulty".to_string(),
        Box::new(CustomDifficultyValidator)
    );

    api.register_config_validator(
        "mods.config_injection_example.custom_damage_multiplier".to_string(),
        Box::new(DamageMultiplierValidator)
    );

    // Boolean validators are handled automatically by the schema
}

/// Example: Accessing config values in other parts of the mod
pub fn get_custom_difficulty(config: &looking_glass_mod::loader::ConfigRegistry) -> String {
    config.get_value("mods.config_injection_example.custom_difficulty")
        .and_then(|v| v.as_str())
        .unwrap_or("normal")
        .to_string()
}

pub fn get_damage_multiplier(config: &looking_glass_mod::loader::ConfigRegistry) -> f64 {
    config.get_value("mods.config_injection_example.custom_damage_multiplier")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0)
}

pub fn is_custom_feature_enabled(config: &looking_glass_mod::loader::ConfigRegistry) -> bool {
    config.get_value("mods.config_injection_example.enable_custom_feature")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}