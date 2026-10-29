# Looking Glass Mod API Reference

Complete Rust API documentation for the `looking-glass-mod` crate.

---

## Crate Overview

```toml
[dependencies]
looking-glass-mod = { path = "../crates/looking-glass-mod" }
```

**Version**: 0.1.0  
**Minimum Rust**: 1.86  
**Edition**: 2021

---

## Module Structure

```
looking_glass_mod
├── manifest          # Mod manifest types and validation
│   ├── GameTarget
│   ├── ModDependency
│   ├── ModEntryPoints
│   ├── AssetType
│   ├── AssetMapping
│   ├── LoadOrder
│   ├── ModManifest
│   └── InstalledMod
├── loader            # Mod loading and runtime APIs
│   ├── ModContext
│   ├── AssetRegistry
│   ├── MenuRegistry
│   ├── UiRegistry
│   ├── ConfigRegistry
│   ├── ModApi
│   ├── UiPatch
│   ├── UiPatchOp
│   ├── WidgetDef
│   ├── InputHook
│   ├── KeyCode
│   ├── KeyModifiers
│   ├── GamepadButton
│   ├── ButtonState
│   ├── ButtonOption
│   ├── AssetOverride
│   ├── AssetOverrideResult
│   ├── ConfigValidator
│   ├── RuntimeHook
│   ├── Mod
│   ├── ModLoader
│   └── ModRegistry
└── discovery         # Mod discovery and installation
    ├── ModDiscovery
    └── compute_directory_checksum
```

---

## manifest Module

### GameTarget

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum GameTarget {
    Alice,
    AliceDemo,
}

impl GameTarget {
    pub fn as_str(&self) -> &'static str;
}
```

**Variants**:
- `Alice` - Full game
- `AliceDemo` - Demo version

---

### ModDependency

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub id: String,
    #[serde(with = "semver::VersionReq")]
    pub version: VersionReq,
    #[serde(default)]
    pub optional: bool,
}
```

**Fields**:
- `id` - Dependency mod ID
- `version` - Semver requirement
- `optional` - If true, mod loads without this dependency

---

### ModEntryPoints

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModEntryPoints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_override: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_inject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_hook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_patch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_hook: Option<String>,
}
```

Paths to hook modules relative to mod root.

---

### AssetType

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AssetType {
    Texture,
    Model,
    Sound,
    Font,
    Ui,
    Other,
}
```

---

### AssetMapping

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMapping {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}
```

---

### LoadOrder

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoadOrder {
    #[serde(default)]
    pub before: Vec<String>,
    #[serde(default)]
    pub after: Vec<String>,
}
```

---

### ModManifest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    pub game_target: GameTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_game_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_game_version: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
    #[serde(default)]
    pub load_order: LoadOrder,
    #[serde(default)]
    pub entry_points: ModEntryPoints,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_config: Option<serde_json::Value>,
    #[serde(default)]
    pub assets: Vec<AssetMapping>,
    #[serde(default)]
    pub tags: Vec<String>,
}
```

#### Methods

```rust
impl ModManifest {
    /// Load manifest from file (JSON or TOML)
    pub fn load_from_file(path: &Path) -> Result<Self>;

    /// Validate manifest against schema and semantic rules
    pub fn validate(&self) -> Result<()>;

    /// Compute SHA256 checksum of mod directory
    pub fn compute_checksum(&self, mod_dir: &Path) -> Result<String>;

    /// Check if mod satisfies game version constraints
    pub fn satisfies_game_version(&self, game_version: &str) -> Result<bool>;
}
```

---

### InstalledMod

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMod {
    pub manifest: ModManifest,
    pub path: PathBuf,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    pub installed_at: DateTime<Utc>,
    pub checksum: String,
}
```

#### Methods

```rust
impl InstalledMod {
    /// Load installed mod from directory
    pub fn load(mod_dir: &Path) -> Result<Self>;

    /// Verify mod files match checksum
    pub fn verify_checksum(&self) -> Result<bool>;
}
```

---

## loader Module

### ModContext

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModContext {
    pub mod_dir: PathBuf,
    pub assets: AssetRegistry,
    pub menu: MenuRegistry,
    pub ui: UiRegistry,
    pub config: ConfigRegistry,
    pub game_version: String,
    pub mod_api: ModApi,
}
```

Provided to mods during initialization.

---

### AssetRegistry

```rust
pub struct AssetRegistry {
    overrides: HashMap<String, Vec<AssetOverrideEntry>>,
}

impl AssetRegistry {
    pub fn new() -> Self;
    pub fn register_override(&mut self, target: String, entry: AssetOverrideEntry);
    pub fn get_overrides(&self, target: &str) -> Option<&Vec<AssetOverrideEntry>>;
}
```

---

### AssetOverrideEntry

```rust
#[derive(Debug, Clone)]
pub struct AssetOverrideEntry {
    pub source: PathBuf,
    pub asset_type: AssetType,
    pub condition: Option<String>,
    pub priority: i32,
}
```

---

### MenuRegistry

```rust
pub struct MenuRegistry {
    entries: HashMap<String, MenuEntry>,
}

impl MenuRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, id: String, entry: MenuEntry);
    pub fn get(&self, id: &str) -> Option<&MenuEntry>;
}
```

---

### MenuEntry

```rust
#[derive(Debug, Clone)]
pub struct MenuEntry {
    pub id: String,
    pub label: String,
    pub callback: Box<dyn Fn() + Send + Sync>,
}
```

---

### UiRegistry

```rust
pub struct UiRegistry {
    patches: Vec<UiPatch>,
}

impl UiRegistry {
    pub fn new() -> Self;
    pub fn register_patch(&mut self, patch: UiPatch);
    pub fn get_patches(&self) -> &[UiPatch];
}
```

---

### ConfigRegistry

```rust
pub struct ConfigRegistry {
    validators: HashMap<String, Box<dyn ConfigValidator>>,
    values: HashMap<String, serde_json::Value>,
}

impl ConfigRegistry {
    pub fn new() -> Self;
    pub fn register_validator(&mut self, key: String, validator: Box<dyn ConfigValidator>);
    pub fn set_value(&mut self, key: String, value: serde_json::Value);
    pub fn get_value(&self, key: &str) -> Option<&serde_json::Value>;
    pub fn validate_all(&self) -> Result<()>;
}
```

---

### ModApi

```rust
pub struct ModApi {
    asset_registry: Arc<Mutex<AssetRegistry>>,
    ui_registry: Arc<Mutex<UiRegistry>>,
    config_registry: Arc<Mutex<ConfigRegistry>>,
    input_hooks: Arc<Mutex<Vec<Box<dyn InputHook>>>>,
    runtime_hooks: Arc<Mutex<Vec<Box<dyn RuntimeHook>>>>,
}

impl ModApi {
    pub fn new(
        asset_registry: Arc<Mutex<AssetRegistry>>,
        ui_registry: Arc<Mutex<UiRegistry>>,
        config_registry: Arc<Mutex<ConfigRegistry>>,
        input_hooks: Arc<Mutex<Vec<Box<dyn InputHook>>>>,
        runtime_hooks: Arc<Mutex<Vec<Box<dyn RuntimeHook>>>>,
    ) -> Self;

    pub fn register_asset_override(
        &self,
        target: String,
        source: PathBuf,
        asset_type: AssetType,
        condition: Option<String>,
        priority: i32,
    );

    pub fn register_ui_patch(&self, patch: UiPatch);

    pub fn register_input_hook(&self, hook: Box<dyn InputHook>);

    pub fn register_config_validator(&self, key: String, validator: Box<dyn ConfigValidator>);

    pub fn log(&self, level: log::Level, message: &str);

    pub fn get_input_hooks(&self) -> Vec<Box<dyn InputHook>>;

    pub fn get_runtime_hooks(&self) -> Vec<Box<dyn RuntimeHook>>;
}
```

---

### UiPatch

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPatch {
    pub target: String,
    pub operations: Vec<UiPatchOp>,
}
```

---

### UiPatchOp

```rust
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
```

---

### WidgetDef

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetDef {
    pub id: String,
    pub widget_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub children: Vec<WidgetDef>,
}
```

---

### InputHook Trait

```rust
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
```

**Methods**:

| Method | Description | Return |
|--------|-------------|--------|
| `on_button_prompt` | Button prompt shown (e.g., "Press X") | `Some(index)` to auto-select, `None` for normal |
| `on_key_down` | Keyboard key pressed | `true` to consume event |
| `on_key_up` | Keyboard key released | `true` to consume event |
| `on_gamepad_button` | Gamepad button pressed/released | `true` to consume event |

---

### KeyCode

```rust
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
```

---

### KeyModifiers

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}
```

---

### GamepadButton

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamepadButton {
    South, East, West, North,  // Cross, Circle, Square, Triangle / A, B, X, Y
    C, Z,
    LeftTrigger, RightTrigger,
    LeftTrigger2, RightTrigger2,
    Select, Start, Mode,
    LeftStick, RightStick,
    DPUp, DPDown, DPLeft, DPRight,
    Unknown(u32),
}
```

---

### ButtonState

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonState {
    Pressed,
    Released,
}
```

---

### ButtonOption

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}
```

---

### AssetOverride Trait

```rust
#[async_trait]
pub trait AssetOverride: Send + Sync {
    async fn try_override(&self, target: &str) -> Option<AssetOverrideResult>;
    async fn list_overrides(&self) -> Vec<AssetMapping>;
}
```

---

### AssetOverrideResult

```rust
#[derive(Debug, Clone)]
pub struct AssetOverrideResult {
    pub data: Vec<u8>,
    pub mime_type: String,
}
```

---

### ConfigValidator Trait

```rust
#[async_trait]
pub trait ConfigValidator: Send + Sync {
    async fn validate(&self, value: &serde_json::Value) -> Result<()>;
    async fn apply_defaults(&self, value: &mut serde_json::Value);
}
```

---

### RuntimeHook Trait

```rust
#[async_trait]
pub trait RuntimeHook: Send + Sync {
    async fn on_frame(&self, delta_time: f32);
    async fn on_menu_open(&self, menu_id: &str);
    async fn on_menu_close(&self, menu_id: &str);
    async fn on_level_load(&self, level_id: &str);
    async fn on_level_unload(&self, level_id: &str);
}
```

**Events**:

| Method | When Called |
|--------|-------------|
| `on_frame` | Every frame (delta_time in seconds) |
| `on_menu_open` | Menu opened |
| `on_menu_close` | Menu closed |
| `on_level_load` | Level loaded |
| `on_level_unload` | Level unloaded |

---

### Mod Trait

```rust
#[async_trait]
pub trait Mod: Send + Sync {
    async fn initialize(&mut self, context: &ModContext) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    async fn on_config_changed(&mut self, key: &str, value: &serde_json::Value) -> Result<()>;
    fn manifest(&self) -> &ModManifest;
    fn entry_points(&self) -> &ModEntryPoints;
}
```

---

### ModLoader Trait

```rust
#[async_trait]
pub trait ModLoader: Send + Sync {
    async fn load_mod(&self, mod_dir: &PathBuf) -> Result<Box<dyn Mod>>;
    async fn unload_mod(&self, mod_id: &str) -> Result<()>;
    fn supported_game_targets(&self) -> Vec<GameTarget>;
}
```

---

### ModRegistry

```rust
pub struct ModRegistry {
    mods: HashMap<String, Box<dyn Mod>>,
    load_order: Vec<String>,
    asset_registry: Arc<Mutex<AssetRegistry>>,
    ui_registry: Arc<Mutex<UiRegistry>>,
    config_registry: Arc<Mutex<ConfigRegistry>>,
    input_hooks: Arc<Mutex<Vec<Box<dyn InputHook>>>>,
    runtime_hooks: Arc<Mutex<Vec<Box<dyn RuntimeHook>>>>,
}

impl ModRegistry {
    pub fn new() -> Self;

    pub fn register(&mut self, mut mod_instance: Box<dyn Mod>) -> Result<()>;

    pub fn unregister(&mut self, mod_id: &str) -> Result<()>;

    pub fn get(&self, mod_id: &str) -> Option<&dyn Mod>;

    pub fn iter(&self) -> impl Iterator<Item = &dyn Mod>;

    pub fn try_override_asset(&self, target: &str) -> Option<AssetOverrideResult>;

    pub async fn process_button_prompt(
        &self,
        prompt_id: &str,
        options: &[ButtonOption],
    ) -> Option<usize>;

    pub async fn update(&self, delta_time: f32);

    pub fn sort_load_order(&mut self) -> Result<()>;

    pub fn get_asset_registry(&self) -> Arc<Mutex<AssetRegistry>>;

    pub fn get_ui_registry(&self) -> Arc<Mutex<UiRegistry>>;

    pub fn get_config_registry(&self) -> Arc<Mutex<ConfigRegistry>>;
}
```

---

## discovery Module

### ModDiscovery

```rust
pub struct ModDiscovery {
    pub base_dir: PathBuf,
    pub game_id: String,
}

impl ModDiscovery {
    pub fn new(base_dir: PathBuf, game_id: String) -> Self;
    pub fn mods_dir(&self) -> PathBuf;
    pub fn discover(&self) -> Result<Vec<InstalledMod>>;
    pub fn install_from_archive(&self, archive_path: &Path) -> Result<InstalledMod>;
    pub fn uninstall(&self, mod_id: &str) -> Result<()>;
    pub fn get_mod_info(&self, mod_id: &str) -> Result<InstalledMod>;
    pub fn enable_mod(&self, mod_id: &str) -> Result<()>;
    pub fn disable_mod(&self, mod_id: &str) -> Result<()>;
    pub fn compute_archive_checksum(&self, archive_path: &Path) -> Result<String>;
}
```

---

### compute_directory_checksum

```rust
pub fn compute_directory_checksum(dir: &Path) -> Result<String>;
```

Compute SHA256 checksum of directory contents (used for mod integrity verification).

---

## Error Handling

All functions return `Result<T, anyhow::Error>`. Use `?` operator or `.context()` for error chaining.

```rust
use anyhow::{Context, Result};

fn example() -> Result<()> {
    let manifest = ModManifest::load_from_file(&path)
        .context("Failed to load manifest")?;
    manifest.validate()
        .context("Manifest validation failed")?;
    Ok(())
}
```

---

## Async Runtime

The mod system uses `async-trait` and requires an async runtime (tokio or async-std). The `ModRegistry` uses `futures::executor::block_on` for synchronous initialization.

---

## Feature Flags

```toml
looking-glass-mod = { path = "../crates/looking-glass-mod", features = [] }
```

No optional features currently.

---

## See Also

- [MODDING_GUIDE.md](MODDING_GUIDE.md) - Complete modding guide
- [MANIFEST_SCHEMA.md](MANIFEST_SCHEMA.md) - JSON Schema reference
- [CLI_REFERENCE.md](CLI_REFERENCE.md) - Launcher CLI reference
- [EXAMPLE_MODS/](EXAMPLE_MODS/) - Working examples