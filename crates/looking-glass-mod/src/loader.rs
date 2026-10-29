use crate::manifest::{AssetMapping, AssetType, GameTarget, InstalledMod, ModManifest};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

// ModContext doesn't derive Serialize/Deserialize because it contains
// registry types with Arc<Mutex<...>> which can't be serialized.
// It's created fresh for each mod initialization.
// It doesn't derive Clone because ModApi and ConfigRegistry contain trait objects.
#[derive(Debug)]
pub struct ModContext {
    pub mod_dir: PathBuf,
    pub assets: AssetRegistry,
    pub menu: MenuRegistry,
    pub ui: UiRegistry,
    pub config: ConfigRegistry,
    pub game_version: String,
    pub mod_api: ModApi,
}

#[derive(Debug, Clone, Default)]
pub struct AssetRegistry {
    overrides: HashMap<String, Vec<AssetOverrideEntry>>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            overrides: HashMap::new(),
        }
    }

    pub fn register_override(&mut self, target: String, entry: AssetOverrideEntry) {
        self.overrides.entry(target).or_default().push(entry);
    }

    pub fn get_overrides(&self, target: &str) -> Option<&Vec<AssetOverrideEntry>> {
        self.overrides.get(target)
    }
}

#[derive(Debug, Clone)]
pub struct AssetOverrideEntry {
    pub source: PathBuf,
    pub asset_type: AssetType,
    pub condition: Option<String>,
    pub priority: i32,
}

#[derive(Debug, Clone, Default)]
pub struct MenuRegistry {
    entries: HashMap<String, MenuEntry>,
}

impl MenuRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: String, entry: MenuEntry) {
        self.entries.insert(id, entry);
    }

    pub fn get(&self, id: &str) -> Option<&MenuEntry> {
        self.entries.get(id)
    }
}

// MenuEntry doesn't derive Clone/Debug because it contains a callback.
// We use a wrapper that can be cloned for registration purposes.
pub struct MenuEntry {
    pub id: String,
    pub label: String,
    pub callback: MenuCallback,
}

// Type alias for menu callbacks
pub type MenuCallback = Arc<dyn Fn() + Send + Sync>;

impl std::fmt::Debug for MenuEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuEntry")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("callback", &"<callback>")
            .finish()
    }
}

impl Clone for MenuEntry {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            label: self.label.clone(),
            callback: Arc::clone(&self.callback),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UiRegistry {
    patches: Vec<UiPatch>,
}

impl UiRegistry {
    pub fn new() -> Self {
        Self { patches: Vec::new() }
    }

    pub fn register_patch(&mut self, patch: UiPatch) {
        self.patches.push(patch);
    }

    pub fn get_patches(&self) -> &[UiPatch] {
        &self.patches
    }
}

#[derive(Default)]
pub struct ConfigRegistry {
    validators: HashMap<String, Box<dyn ConfigValidator + Send + Sync>>,
    values: HashMap<String, serde_json::Value>,
}

impl std::fmt::Debug for ConfigRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigRegistry")
            .field("validators_count", &self.validators.len())
            .field("values", &self.values)
            .finish()
    }
}

impl ConfigRegistry {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn register_validator(&mut self, key: String, validator: Box<dyn ConfigValidator + Send + Sync>) {
        self.validators.insert(key, validator);
    }

    pub fn set_value(&mut self, key: String, value: serde_json::Value) {
        self.values.insert(key, value);
    }

    pub fn get_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.values.get(key)
    }

    pub async fn validate_all(&self) -> Result<()> {
        for (key, validator) in &self.validators {
            if let Some(value) = self.values.get(key) {
                validator.validate(value).await?;
            }
        }
        Ok(())
    }
}

pub struct ModApi {
    asset_registry: Arc<std::sync::Mutex<AssetRegistry>>,
    ui_registry: Arc<std::sync::Mutex<UiRegistry>>,
    config_registry: Arc<std::sync::Mutex<ConfigRegistry>>,
    input_hooks: Arc<std::sync::Mutex<Vec<Box<dyn InputHook + Send + Sync>>>>,
    runtime_hooks: Arc<std::sync::Mutex<Vec<Box<dyn RuntimeHook + Send + Sync>>>>,
}

impl std::fmt::Debug for ModApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModApi")
            .field("asset_registry", &"<mutex>")
            .field("ui_registry", &"<mutex>")
            .field("config_registry", &"<mutex>")
            .field("input_hooks_count", &self.input_hooks.lock().unwrap().len())
            .field("runtime_hooks_count", &self.runtime_hooks.lock().unwrap().len())
            .finish()
    }
}

impl ModApi {
    pub fn new(
        asset_registry: Arc<std::sync::Mutex<AssetRegistry>>,
        ui_registry: Arc<std::sync::Mutex<UiRegistry>>,
        config_registry: Arc<std::sync::Mutex<ConfigRegistry>>,
        input_hooks: Arc<std::sync::Mutex<Vec<Box<dyn InputHook + Send + Sync>>>>,
        runtime_hooks: Arc<std::sync::Mutex<Vec<Box<dyn RuntimeHook + Send + Sync>>>>,
    ) -> Self {
        Self {
            asset_registry,
            ui_registry,
            config_registry,
            input_hooks,
            runtime_hooks,
        }
    }

    pub fn register_asset_override(
        &self,
        target: String,
        source: PathBuf,
        asset_type: AssetType,
        condition: Option<String>,
        priority: i32,
    ) {
        let entry = AssetOverrideEntry {
            source,
            asset_type,
            condition,
            priority,
        };
        self.asset_registry
            .lock()
            .unwrap()
            .register_override(target, entry);
    }

    pub fn register_ui_patch(&self, patch: UiPatch) {
        self.ui_registry.lock().unwrap().register_patch(patch);
    }

    pub fn register_input_hook(&self, hook: Box<dyn InputHook + Send + Sync>) {
        self.input_hooks.lock().unwrap().push(hook);
    }

    pub fn register_config_validator(&self, key: String, validator: Box<dyn ConfigValidator + Send + Sync>) {
        self.config_registry.lock().unwrap().register_validator(key, validator);
    }

    pub fn log(&self, level: log::Level, message: &str) {
        log::log!(level, "[ModAPI] {}", message);
    }

    pub fn get_input_hooks(&self) -> Vec<Box<dyn InputHook + Send + Sync>> {
        self.input_hooks.lock().unwrap().drain(..).collect()
    }

    pub fn get_runtime_hooks(&self) -> Vec<Box<dyn RuntimeHook + Send + Sync>> {
        self.runtime_hooks.lock().unwrap().drain(..).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPatch {
    pub target: String,
    pub operations: Vec<UiPatchOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum UiPatchOp {
    AddWidget {
        parent: String,
        widget: WidgetDef,
    },
    RemoveWidget {
        id: String,
    },
    ModifyWidget {
        id: String,
        changes: HashMap<String, serde_json::Value>,
    },
    ReplaceWidget {
        id: String,
        widget: WidgetDef,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetDef {
    pub id: String,
    pub widget_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub children: Vec<WidgetDef>,
}

#[async_trait]
pub trait InputHook: Send + Sync {
    async fn on_button_prompt(
        &self,
        prompt_id: &str,
        options: &[ButtonOption],
    ) -> Option<usize>;

    async fn on_key_down(&self, key: KeyCode, modifiers: KeyModifiers) -> bool;

    async fn on_key_up(&self, key: KeyCode, modifiers: KeyModifiers) -> bool;

    async fn on_gamepad_button(
        &self,
        button: GamepadButton,
        state: ButtonState,
    ) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Digit0, Digit1, Digit2, Digit3, Digit4,
    Digit5, Digit6, Digit7, Digit8, Digit9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Escape, Tab, CapsLock, ShiftLeft, ShiftRight,
    ControlLeft, ControlRight, AltLeft, AltRight,
    Space, Enter, Backspace, Delete, Insert,
    Home, End, PageUp, PageDown,
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    Unknown(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamepadButton {
    South, East, West, North,
    C, Z,
    LeftTrigger, RightTrigger,
    LeftTrigger2, RightTrigger2,
    Select, Start, Mode,
    LeftStick, RightStick,
    DPUp, DPDown, DPLeft, DPRight,
    Unknown(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[async_trait]
pub trait AssetOverride: Send + Sync {
    async fn try_override(&self, target: &str) -> Option<AssetOverrideResult>;

    async fn list_overrides(&self) -> Vec<AssetMapping>;
}

#[derive(Debug, Clone)]
pub struct AssetOverrideResult {
    pub data: Vec<u8>,
    pub mime_type: String,
}

#[async_trait]
pub trait ConfigValidator: Send + Sync {
    async fn validate(&self, value: &serde_json::Value) -> Result<()>;

    async fn apply_defaults(&self, value: &mut serde_json::Value);
}

#[async_trait]
pub trait RuntimeHook: Send + Sync {
    async fn on_frame(&self, delta_time: f32);

    async fn on_menu_open(&self, menu_id: &str);

    async fn on_menu_close(&self, menu_id: &str);

    async fn on_level_load(&self, level_id: &str);

    async fn on_level_unload(&self, level_id: &str);
}

#[async_trait]
pub trait Mod: Send + Sync {
    async fn initialize(&mut self, context: &ModContext) -> Result<()>;

    async fn shutdown(&mut self) -> Result<()>;

    async fn on_config_changed(&mut self, key: &str, value: &serde_json::Value) -> Result<()>;

    fn manifest(&self) -> &ModManifest;

    fn entry_points(&self) -> &crate::manifest::ModEntryPoints;
}

#[async_trait]
pub trait ModLoader: Send + Sync {
    async fn load_mod(&self, mod_dir: &PathBuf) -> Result<Box<dyn Mod>>;

    async fn unload_mod(&self, mod_id: &str) -> Result<()>;

    fn supported_game_targets(&self) -> Vec<GameTarget>;
}

pub struct ModRegistry {
    mods: HashMap<String, Box<dyn Mod>>,
    load_order: Vec<String>,
    asset_registry: Arc<std::sync::Mutex<AssetRegistry>>,
    ui_registry: Arc<std::sync::Mutex<UiRegistry>>,
    config_registry: Arc<std::sync::Mutex<ConfigRegistry>>,
    input_hooks: Arc<std::sync::Mutex<Vec<Box<dyn InputHook + Send + Sync>>>>,
    runtime_hooks: Arc<std::sync::Mutex<Vec<Box<dyn RuntimeHook + Send + Sync>>>>,
}

impl ModRegistry {
    pub fn new() -> Self {
        let asset_registry = Arc::new(std::sync::Mutex::new(AssetRegistry::new()));
        let ui_registry = Arc::new(std::sync::Mutex::new(UiRegistry::new()));
        let config_registry = Arc::new(std::sync::Mutex::new(ConfigRegistry::new()));
        let input_hooks = Arc::new(std::sync::Mutex::new(Vec::new()));
        let runtime_hooks = Arc::new(std::sync::Mutex::new(Vec::new()));

        Self {
            mods: HashMap::new(),
            load_order: Vec::new(),
            asset_registry,
            ui_registry,
            config_registry,
            input_hooks,
            runtime_hooks,
        }
    }

    pub fn register(&mut self, mut mod_instance: Box<dyn Mod>) -> Result<()> {
        let mod_id = mod_instance.manifest().id.clone();

        // Initialize the mod
        let context = self.create_context(&mod_id)?;
        futures::executor::block_on(mod_instance.initialize(&context))?;

        // Register entry points
        self.register_entry_points(&mut *mod_instance)?;

        self.mods.insert(mod_id.clone(), mod_instance);
        self.load_order.push(mod_id);

        Ok(())
    }

    fn create_context(&self, _mod_id: &str) -> Result<ModContext> {
        Ok(ModContext {
            mod_dir: PathBuf::new(), // Will be set by loader
            assets: AssetRegistry::new(),
            menu: MenuRegistry::new(),
            ui: UiRegistry::new(),
            config: ConfigRegistry::new(),
            game_version: String::new(), // Will be set by loader
            mod_api: ModApi::new(
                self.asset_registry.clone(),
                self.ui_registry.clone(),
                self.config_registry.clone(),
                self.input_hooks.clone(),
                self.runtime_hooks.clone(),
            ),
        })
    }

    fn register_entry_points(&self, mod_instance: &mut dyn Mod) -> Result<()> {
        let entry_points = mod_instance.entry_points();

        if let Some(asset_override_path) = &entry_points.asset_override {
            // Load and register asset override module
            // This would typically involve dynamic loading
        }

        if let Some(ui_patch_path) = &entry_points.ui_patch {
            // Load and register UI patch module
        }

        if let Some(input_hook_path) = &entry_points.input_hook {
            // Load and register input hook module
        }

        if let Some(runtime_hook_path) = &entry_points.runtime_hook {
            // Load and register runtime hook module
        }

        Ok(())
    }

    pub fn unregister(&mut self, mod_id: &str) -> Result<()> {
        if let Some(mut mod_instance) = self.mods.remove(mod_id) {
            futures::executor::block_on(mod_instance.shutdown())?;
            self.load_order.retain(|id| id != mod_id);
        }
        Ok(())
    }

    pub fn get(&self, mod_id: &str) -> Option<&dyn Mod> {
        self.mods.get(mod_id).map(|m| m.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Mod> {
        self.load_order.iter().filter_map(|id| self.mods.get(id).map(|m| m.as_ref()))
    }

    pub fn try_override_asset(&self, target: &str) -> Option<AssetOverrideResult> {
        let registry = self.asset_registry.lock().unwrap();
        let overrides = registry.get_overrides(target)?;

        // Sort by priority and try each
        let mut sorted = overrides.clone();
        sorted.sort_by_key(|e| -e.priority);

        for entry in sorted {
            // Check condition if present
            if let Some(condition) = &entry.condition {
                if !evaluate_condition(condition) {
                    continue;
                }
            }

            // Read asset data
            if let Ok(data) = std::fs::read(&entry.source) {
                let mime_type = guess_mime_type(&entry.source, entry.asset_type.clone());
                return Some(AssetOverrideResult { data, mime_type });
            }
        }

        None
    }

    pub async fn process_button_prompt(
        &self,
        prompt_id: &str,
        options: &[ButtonOption],
    ) -> Option<usize> {
        let hooks = self.input_hooks.lock().unwrap();

        for hook in hooks.iter() {
            if let Some(choice) = hook.on_button_prompt(prompt_id, options).await {
                return Some(choice);
            }
        }

        None
    }

    pub async fn update(&self, delta_time: f32) {
        let hooks = self.runtime_hooks.lock().unwrap();

        for hook in hooks.iter() {
            hook.on_frame(delta_time).await;
        }
    }

    pub fn sort_load_order(&mut self) -> Result<()> {
        // Topological sort based on dependencies and load_order constraints
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();

        fn visit(
            mod_id: &str,
            mods: &HashMap<String, Box<dyn Mod>>,
            sorted: &mut Vec<String>,
            visited: &mut std::collections::HashSet<String>,
            visiting: &mut std::collections::HashSet<String>,
        ) -> Result<()> {
            if visiting.contains(mod_id) {
                return Err(anyhow::anyhow!("Circular dependency detected: {}", mod_id));
            }
            if visited.contains(mod_id) {
                return Ok(());
            }

            visiting.insert(mod_id.to_string());

            if let Some(mod_instance) = mods.get(mod_id) {
                let manifest = mod_instance.manifest();

                // Visit dependencies first
                for dep in &manifest.dependencies {
                    if !dep.optional {
                        visit(&dep.id, mods, sorted, visited, visiting)?;
                    }
                }

                // Visit load_order.after mods
                for after_id in &manifest.load_order.after {
                    if mods.contains_key(after_id) {
                        visit(after_id, mods, sorted, visited, visiting)?;
                    }
                }
            }

            visiting.remove(mod_id);
            visited.insert(mod_id.to_string());
            sorted.push(mod_id.to_string());

            Ok(())
        }

        for mod_id in self.load_order.clone() {
            visit(&mod_id, &self.mods, &mut sorted, &mut visited, &mut visiting)?;
        }

        // Apply load_order.before constraints (reverse iteration)
        let mut final_order = Vec::new();
        for mod_id in sorted {
            let manifest = self.mods.get(&mod_id).unwrap().manifest();
            let mut insert_pos = final_order.len();

            for before_id in &manifest.load_order.before {
                if let Some(pos) = final_order.iter().position(|id| id == before_id) {
                    insert_pos = insert_pos.min(pos);
                }
            }

            final_order.insert(insert_pos, mod_id);
        }

        self.load_order = final_order;
        Ok(())
    }

    pub fn get_asset_registry(&self) -> Arc<std::sync::Mutex<AssetRegistry>> {
        self.asset_registry.clone()
    }

    pub fn get_ui_registry(&self) -> Arc<std::sync::Mutex<UiRegistry>> {
        self.ui_registry.clone()
    }

    pub fn get_config_registry(&self) -> Arc<std::sync::Mutex<ConfigRegistry>> {
        self.config_registry.clone()
    }

    pub fn get_input_hooks(&self) -> Arc<std::sync::Mutex<Vec<Box<dyn InputHook + Send + Sync>>>> {
        self.input_hooks.clone()
    }

    pub fn get_runtime_hooks(&self) -> Arc<std::sync::Mutex<Vec<Box<dyn RuntimeHook + Send + Sync>>>> {
        self.runtime_hooks.clone()
    }

    pub fn get_load_order(&self) -> Vec<String> {
        self.load_order.clone()
    }
}

fn evaluate_condition(condition: &str) -> bool {
    // Simple condition evaluation - in practice would use a proper expression evaluator
    // For now, just check for simple true/false
    condition.trim().eq_ignore_ascii_case("true")
}

fn guess_mime_type(path: &PathBuf, asset_type: AssetType) -> String {
    match asset_type {
        AssetType::Texture => "image/png".to_string(),
        AssetType::Model => "model/gltf-binary".to_string(),
        AssetType::Sound => "audio/ogg".to_string(),
        AssetType::Font => "font/ttf".to_string(),
        AssetType::Ui => "application/json".to_string(),
        AssetType::Other => "application/octet-stream".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ModManifest, GameTarget, ModEntryPoints, LoadOrder};
    use std::sync::Arc;

    #[test]
    fn test_mod_registry_creation() {
        let registry = ModRegistry::new();
        assert!(registry.mods.is_empty());
        assert!(registry.load_order.is_empty());
    }

    #[test]
    fn test_asset_registry() {
        let mut registry = AssetRegistry::new();
        let entry = AssetOverrideEntry {
            source: PathBuf::from("test.png"),
            asset_type: AssetType::Texture,
            condition: None,
            priority: 10,
        };
        registry.register_override("textures/test.png".to_string(), entry);
        assert!(registry.get_overrides("textures/test.png").is_some());
    }

    #[test]
    fn test_ui_patch_serialization() {
        let patch = UiPatch {
            target: "main_menu".to_string(),
            operations: vec![
                UiPatchOp::AddWidget {
                    parent: "root".to_string(),
                    widget: WidgetDef {
                        id: "test_button".to_string(),
                        widget_type: "Button".to_string(),
                        properties: HashMap::new(),
                        children: Vec::new(),
                    },
                },
            ],
        };

        let json = serde_json::to_string(&patch).unwrap();
        let parsed: UiPatch = serde_json::from_str(&json).unwrap();
        assert_eq!(patch.target, parsed.target);
        assert_eq!(patch.operations.len(), parsed.operations.len());
    }

    #[test]
    fn test_key_modifiers() {
        let mods = KeyModifiers {
            shift: true,
            control: false,
            alt: true,
            meta: false,
        };
        assert!(mods.shift);
        assert!(mods.alt);
        assert!(!mods.control);
        assert!(!mods.meta);
    }
}