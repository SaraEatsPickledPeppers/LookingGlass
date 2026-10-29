//! Template Mod - Main Mod Implementation
//! This is the main entry point for the template mod.
//! Copy this file and modify it for your mod.

use looking_glass_mod::{
    ModApi, ModContext, ModManifest, ModEntryPoints, RuntimeHook, InputHook, ConfigValidator,
    Result, async_trait,
};
use std::sync::{Arc, Mutex};

/// Template mod configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TemplateModConfig {
    pub enabled: bool,
    pub setting1: String,
    pub setting2: f64,
    pub setting3: bool,
}

/// Main mod struct
pub struct TemplateMod {
    manifest: ModManifest,
    entry_points: ModEntryPoints,
    config: Arc<Mutex<TemplateModConfig>>,
}

impl TemplateMod {
    pub fn new(mod_dir: &std::path::Path) -> Result<Self> {
        let manifest_path = mod_dir.join("mod.toml");
        let manifest = ModManifest::load_from_file(&manifest_path)?;
        let entry_points = manifest.entry_points.clone();

        // Load config from manifest default_config
        let config = manifest.default_config
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(Self {
            manifest,
            entry_points,
            config: Arc::new(Mutex::new(config)),
        })
    }
}

#[async_trait]
impl looking_glass_mod::Mod for TemplateMod {
    async fn initialize(&mut self, context: &ModContext) -> Result<()> {
        // Register entry points based on manifest
        let entry_points = self.entry_points.clone();

        // Register asset override if specified
        if let Some(_asset_override_path) = &entry_points.asset_override {
            // The asset override module would be loaded dynamically
            // For now, we can register directly:
            // crate::scripts::assets::register_asset_overrides_entry(&context.mod_api, context);
        }

        // Register runtime hook if specified
        if let Some(_runtime_hook_path) = &entry_points.runtime_hook {
            context.mod_api.register_runtime_hook(Box::new(TemplateRuntimeHook::new()));
        }

        // Register UI patch if specified
        if let Some(_ui_patch_path) = &entry_points.ui_patch {
            // crate::scripts::ui::register_ui_patches_entry(&context.mod_api, context);
        }

        // Register input hook if specified
        if let Some(_input_hook_path) = &entry_points.input_hook {
            context.mod_api.register_input_hook(Box::new(TemplateInputHook::new()));
        }

        // Register config validator
        if let Some(_config_inject_path) = &entry_points.config_inject {
            context.mod_api.register_config_validator(
                "template".to_string(),
                Box::new(TemplateConfigValidator),
            );
        }

        context.mod_api.log(
            log::Level::Info,
            "Template mod initialized",
        );

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // Cleanup resources if needed
        Ok(())
    }

    async fn on_config_changed(&mut self, key: &str, value: &serde_json::Value) -> Result<()> {
        if key == "template" {
            if let Ok(new_config) = serde_json::from_value::<TemplateModConfig>(value.clone()) {
                *self.config.lock().unwrap() = new_config;
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

/// Runtime hook implementation
pub struct TemplateRuntimeHook {
    counter: Arc<Mutex<u64>>,
}

impl TemplateRuntimeHook {
    pub fn new() -> Self {
        Self { counter: Arc::new(Mutex::new(0)) }
    }
}

#[async_trait]
impl RuntimeHook for TemplateRuntimeHook {
    async fn on_frame(&self, delta_time: f32) {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        // Example: Log every 60 frames (once per second at 60 FPS)
        // if *counter % 60 == 0 {
        //     println!("Template mod: {} seconds elapsed", *counter / 60);
        // }
    }

    async fn on_menu_open(&self, menu_id: &str) {
        // println!("Menu opened: {}", menu_id);
    }

    async fn on_menu_close(&self, menu_id: &str) {
        // println!("Menu closed: {}", menu_id);
    }

    async fn on_level_load(&self, level_id: &str) {
        // println!("Level loaded: {}", level_id);
    }

    async fn on_level_unload(&self, level_id: &str) {
        // println!("Level unloaded: {}", level_id);
    }
}

/// Input hook implementation
pub struct TemplateInputHook;

impl TemplateInputHook {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl InputHook for TemplateInputHook {
    async fn on_button_prompt(
        &self,
        _prompt_id: &str,
        _options: &[looking_glass_mod::ButtonOption],
    ) -> Option<usize> {
        None
    }

    async fn on_key_down(&self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        // Example: Ctrl+M to toggle something
        if modifiers.control && key == KeyCode::M {
            // println!("Template mod: Ctrl+M pressed");
            return true;
        }
        false
    }

    async fn on_key_up(&self, _key: KeyCode, _modifiers: KeyModifiers) -> bool {
        false
    }

    async fn on_gamepad_button(
        &self,
        _button: GamepadButton,
        _state: ButtonState,
    ) -> bool {
        false
    }
}

/// Config validator
struct TemplateConfigValidator;

#[async_trait]
impl ConfigValidator for TemplateConfigValidator {
    async fn validate(&self, value: &serde_json::Value) -> Result<()> {
        if let Some(obj) = value.as_object() {
            for (k, v) in obj {
                match k.as_str() {
                    "enabled" | "setting3" => {
                        if !v.is_boolean() {
                            return Err(anyhow::anyhow!("'{}' must be a boolean", k));
                        }
                    }
                    "setting1" => {
                        if !v.is_string() {
                            return Err(anyhow::anyhow!("'{}' must be a string", k));
                        }
                    }
                    "setting2" => {
                        if !v.is_number() {
                            return Err(anyhow::anyhow!("'{}' must be a number", k));
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn apply_defaults(&self, value: &mut serde_json::Value) {
        let defaults = TemplateModConfig::default();
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

/// Module entry point - called by the mod loader
#[no_mangle]
pub extern "C" fn create_mod(mod_dir: &std::path::Path) -> Result<Box<dyn looking_glass_mod::Mod>> {
    Ok(Box::new(TemplateMod::new(mod_dir)?))
}