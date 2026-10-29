//! Cheats Mod - Main Mod Implementation
//! This mod provides gameplay cheats: infinite health, infinite will/ammo, god mode, no-target, all weapons, etc.

use looking_glass_mod::{
    ModApi, ModContext, ModManifest, ModEntryPoints, RuntimeHook, InputHook, ConfigValidator,
    Result, async_trait,
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

/// Cheats runtime hook - applies cheat effects every frame
pub struct CheatsRuntimeHook {
    config: Arc<Mutex<CheatsConfig>>,
    weapons_granted: Arc<Mutex<bool>>,
}

impl CheatsRuntimeHook {
    pub fn new(config: CheatsConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            weapons_granted: Arc::new(Mutex::new(false)),
        }
    }

    pub fn update_config(&self, new_config: CheatsConfig) {
        *self.config.lock().unwrap() = new_config;
    }

    /// Apply cheat effects to player stats
    /// Note: This needs integration with the game's Stats object.
    /// The actual implementation would be called from the game's main loop
    /// with a reference to the player's Stats.
    pub fn apply_cheats(&self, stats: &mut crate::inventory::Stats) {
        let config = self.config.lock().unwrap();

        // Infinite health (sanity)
        if config.infinite_health || config.god_mode {
            if stats.sanity() < 100.0 {
                stats.set_health(100.0).ok();
            }
        }

        // God mode - complete invincibility
        if config.god_mode {
            stats.god = true;
            stats.scripted_immunity = 1000.0;
        } else {
            stats.god = false;
        }

        // No target - enemies ignore player
        if config.no_target {
            stats.notarget = true;
        } else {
            stats.notarget = false;
        }

        // Infinite will (ammo/mana)
        if config.infinite_will || config.infinite_ammo {
            if stats.will() < 100.0 {
                stats.will = 100.0;
            }
            stats.regen_delay = 0.0;
        }

        // Unlock all weapons
        if config.all_weapons && !*self.weapons_granted.lock().unwrap() {
            stats.grant_weapons();
            *self.weapons_granted.lock().unwrap() = true;
        }

        // Max weapon copies (for Demon Dice)
        if config.max_weapon_copies {
            stats.owned[6] = 3; // Demon Dice max copies
        }
    }
}

#[async_trait]
impl RuntimeHook for CheatsRuntimeHook {
    async fn on_frame(&self, _delta_time: f32) {
        // Called every frame - actual stats modification would happen
        // when the game passes the Stats object to this hook
    }

    async fn on_menu_open(&self, menu_id: &str) {
        if menu_id == "pause" || menu_id == "main" {
            // Reset weapons granted flag so they can be re-granted if needed
            *self.weapons_granted.lock().unwrap() = false;
        }
    }

    async fn on_menu_close(&self, _menu_id: &str) {}

    async fn on_level_load(&self, _level_id: &str) {
        // Re-apply cheats on level load
        *self.weapons_granted.lock().unwrap() = false;
    }

    async fn on_level_unload(&self, _level_id: &str) {}
}

/// Input hook for cheat toggles via keyboard shortcuts
pub struct CheatsInputHook {
    runtime_hook: Arc<CheatsRuntimeHook>,
}

impl CheatsInputHook {
    pub fn new(runtime_hook: Arc<CheatsRuntimeHook>) -> Self {
        Self { runtime_hook }
    }
}

#[async_trait]
impl InputHook for CheatsInputHook {
    async fn on_button_prompt(
        &self,
        _prompt_id: &str,
        _options: &[looking_glass_mod::ButtonOption],
    ) -> Option<usize> {
        None
    }

    async fn on_key_down(&self, key: looking_glass_mod::KeyCode, modifiers: looking_glass_mod::KeyModifiers) -> bool {
        // Toggle cheats with Ctrl+Shift+Function keys
        if modifiers.control && modifiers.shift {
            let mut config = self.runtime_hook.config.lock().unwrap();
            match key {
                looking_glass_mod::KeyCode::F1 => {
                    config.god_mode = !config.god_mode;
                    true
                }
                looking_glass_mod::KeyCode::F2 => {
                    config.infinite_health = !config.infinite_health;
                    true
                }
                looking_glass_mod::KeyCode::F3 => {
                    config.infinite_will = !config.infinite_will;
                    true
                }
                looking_glass_mod::KeyCode::F4 => {
                    config.no_target = !config.no_target;
                    true
                }
                looking_glass_mod::KeyCode::F5 => {
                    config.all_weapons = !config.all_weapons;
                    *self.runtime_hook.weapons_granted.lock().unwrap() = false;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    async fn on_key_up(&self, _key: looking_glass_mod::KeyCode, _modifiers: looking_glass_mod::KeyModifiers) -> bool {
        false
    }

    async fn on_gamepad_button(
        &self,
        _button: looking_glass_mod::GamepadButton,
        _state: looking_glass_mod::ButtonState,
    ) -> bool {
        false
    }
}

/// Config validator for cheats configuration
struct CheatsConfigValidator;

#[async_trait]
impl ConfigValidator for CheatsConfigValidator {
    async fn validate(&self, value: &serde_json::Value) -> Result<()> {
        if let Some(obj) = value.as_object() {
            for (k, v) in obj {
                if !v.is_boolean() {
                    return Err(anyhow::anyhow!(
                        "Cheats config '{}' must be a boolean",
                        k
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

/// Main mod struct
pub struct CheatsMod {
    manifest: ModManifest,
    entry_points: ModEntryPoints,
    runtime_hook: Arc<CheatsRuntimeHook>,
}

impl CheatsMod {
    pub fn new(mod_dir: &std::path::Path) -> Result<Self> {
        let manifest_path = mod_dir.join("mod.toml");
        let manifest = ModManifest::load_from_file(&manifest_path)?;
        let entry_points = manifest.entry_points.clone();

        // Load config from mod config
        let config = manifest.default_config
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let runtime_hook = Arc::new(CheatsRuntimeHook::new(config));

        Ok(Self {
            manifest,
            entry_points,
            runtime_hook,
        })
    }
}

#[async_trait]
impl looking_glass_mod::Mod for CheatsMod {
    async fn initialize(&mut self, context: &ModContext) -> Result<()> {
        // Register the runtime hook
        context.mod_api.register_runtime_hook(Box::new(Arc::clone(&self.runtime_hook)));

        // Register input hook for keyboard shortcuts
        context.mod_api.register_input_hook(Box::new(CheatsInputHook::new(Arc::clone(&self.runtime_hook))));

        // Register config validator
        context.mod_api.register_config_validator(
            "cheats".to_string(),
            Box::new(CheatsConfigValidator),
        );

        context.mod_api.log(
            log::Level::Info,
            "Cheats mod initialized - Runtime hook and input hook registered",
        );

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    async fn on_config_changed(&mut self, key: &str, value: &serde_json::Value) -> Result<()> {
        if key == "cheats" {
            if let Ok(new_config) = serde_json::from_value::<CheatsConfig>(value.clone()) {
                self.runtime_hook.update_config(new_config);
                // Reset weapons granted when config changes
                if new_config.all_weapons {
                    *self.runtime_hook.weapons_granted.lock().unwrap() = false;
                }
            }
        }
        Ok(())
    }

    fn manifest(&self) -> &ModManifest {
        &self.manifest
    }

    fn entry_points(&self) -> &ModEntryPoints {
        &self.entry_points
    }
}

/// Module entry point - called by the mod loader
#[no_mangle]
pub extern "C" fn create_mod(mod_dir: &std::path::Path) -> Result<Box<dyn looking_glass_mod::Mod>> {
    Ok(Box::new(CheatsMod::new(mod_dir)?))
}