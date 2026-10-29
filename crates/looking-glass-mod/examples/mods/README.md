# Example Mods for Looking Glass

This directory contains example mods for American McGee's Alice (2011) using the `looking-glass-mod` crate.

## Mods Included

### 1. Button Glyphs Mod (`button-glyphs/`)
Replaces controller button prompts with proper platform-specific glyphs (PS4/Xbox/Switch icons).

**Features:**
- Platform-specific button glyphs (PS4, Xbox, Switch)
- Uses `AssetRegistry` to override UI textures
- Conditional asset loading based on platform
- High priority overrides for button textures

**Structure:**
```
button-glyphs/
├── mod.toml              # Mod manifest with asset mappings
├── scripts/
│   ├── mod.rs           # Main mod implementation
│   └── assets.rs        # Asset override registration
└── assets/
    └── textures/
        └── ui/          # Placeholder button glyph images
```

**Key API Usage:**
- `ModManifest.assets` - Array of `AssetMapping` for texture overrides
- `AssetMapping.condition` - Platform-specific conditions
- `ModApi.register_asset_override()` - Register overrides programmatically
- `AssetType::Texture` - Texture asset type

---

### 2. Cheats Mod (`cheats/`)
Gameplay modifications including infinite health, infinite will/ammo, god mode, no-target mode, and all weapons unlocked.

**Features:**
- Infinite health (sanity)
- Infinite will (ammo/mana)
- God mode (complete invincibility)
- No-target mode (enemies ignore player)
- All weapons unlocked
- Max weapon copies (Demon Dice)
- Keyboard shortcuts to toggle cheats (Ctrl+Shift+F1-F5)
- Configurable via `mod.toml` config schema

**Structure:**
```
cheats/
├── mod.toml              # Mod manifest with config schema
├── scripts/
│   ├── mod.rs           # Main mod implementation
│   ├── cheats.rs        # Runtime hook & input hook
│   └── config.rs        # Config injection & validation
```

**Key API Usage:**
- `RuntimeHook` trait - `on_frame`, `on_menu_open`, `on_level_load`
- `InputHook` trait - Keyboard shortcuts for toggling cheats
- `ConfigValidator` trait - Validate and apply default config
- `ModApi.register_runtime_hook()` - Register frame callbacks
- `ModApi.register_input_hook()` - Register input handlers
- `ModApi.register_config_validator()` - Config validation
- Integration with `crate::inventory::Stats` for game state modification

**Keyboard Shortcuts:**
- `Ctrl+Shift+F1` - Toggle God Mode
- `Ctrl+Shift+F2` - Toggle Infinite Health
- `Ctrl+Shift+F3` - Toggle Infinite Will
- `Ctrl+Shift+F4` - Toggle No-Target
- `Ctrl+Shift+F5` - Toggle All Weapons

---

### 3. Simple Mod Template (`template/`)
A blank mod template for users to copy and customize.

**Structure:**
```
template/
├── mod.toml              # Template manifest with all fields documented
├── scripts/
│   ├── mod.rs           # Main mod implementation with all entry points
│   ├── assets.rs        # Asset override template
│   ├── runtime.rs       # Runtime hook template
│   ├── config.rs        # Config injection template
│   ├── ui.rs            # UI patch template
│   └── input.rs         # Input hook template
```

**Entry Points Available:**
- `asset_override` - Override game assets (textures, models, sounds)
- `runtime_hook` - Execute code every frame, on menu/level events
- `config_inject` - Register config schema and validators
- `ui_patch` - Modify UI widgets and layouts
- `input_hook` - Handle keyboard/gamepad input

---

## How to Install Mods

1. Copy the mod directory to your Looking Glass mods folder:
   ```
   ~/.local/share/looking-glass/mods/
   ```
   Or on Windows:
   ```
   %APPDATA%\looking-glass\mods\
   ```

2. Enable the mod in the Looking Glass launcher or mod manager.

3. Configure the mod via the mod settings UI (for mods with config).

---

## Creating Your Own Mod

1. Copy the `template/` directory to a new folder with your mod's ID:
   ```bash
   cp -r template/ ~/.local/share/looking-glass/mods/my-awesome-mod/
   ```

2. Edit `mod.toml` with your mod's metadata:
   ```toml
   id = "my-awesome-mod"
   name = "My Awesome Mod"
   version = "1.0.0"
   author = "Your Name"
   description = "What this mod does"
   ```

3. Implement your mod logic in `scripts/mod.rs` and the entry point scripts.

4. Add assets to `assets/` directory and reference them in `mod.toml`.

5. Test your mod by running the game with the mod enabled.

---

## Mod Manifest Reference

### Required Fields
- `id` - Unique mod identifier (lowercase, alphanumeric, hyphens)
- `name` - Display name
- `version` - Semantic version (e.g., "1.0.0")
- `manifest_version` - Always 2
- `game_target` - "Alice" or "AliceDemo"

### Optional Fields
- `author` - Mod author name
- `description` - Mod description
- `homepage` - Mod website URL
- `repository` - Source code URL
- `license` - License identifier (e.g., "MIT")
- `min_game_version` - Minimum game version
- `max_game_version` - Maximum game version
- `dependencies` - Array of mod dependencies
- `load_order` - `before` and `after` arrays for load ordering
- `entry_points` - Script entry points (see below)
- `config_schema` - JSON Schema for mod configuration
- `default_config` - Default configuration values
- `assets` - Array of asset mappings
- `tags` - Array of tags for categorization

### Entry Points
```toml
entry_points = {
    asset_override = "scripts/assets.rs",  # Asset override registration
    runtime_hook = "scripts/runtime.rs",   # Frame/menu/level callbacks
    config_inject = "scripts/config.rs",   # Config validation & defaults
    ui_patch = "scripts/ui.rs",            # UI modifications
    input_hook = "scripts/input.rs"        # Input handling
}
```

### Asset Mappings
```toml
assets = [
    { source = "assets/textures/my_texture.png", target = "textures/original.png", type = "Texture" },
    { source = "assets/models/my_model.gltf", target = "models/original.gltf", type = "Model" },
    { source = "assets/sounds/my_sound.ogg", target = "sounds/original.ogg", type = "Sound" },
    # condition is optional for platform-specific overrides
    { source = "assets/textures/button_ps.png", target = "textures/button.png", type = "Texture", condition = "platform == \"ps4\"" },
]
```

### Asset Types
- `Texture` - Image files (PNG, etc.)
- `Model` - 3D models (GLTF, etc.)
- `Sound` - Audio files (OGG, WAV)
- `Font` - Font files (TTF, OTF)
- `Ui` - UI definitions (JSON)
- `Other` - Any other file type

---

## API Reference

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

### InputHook Trait
```rust
#[async_trait]
pub trait InputHook: Send + Sync {
    async fn on_button_prompt(&self, prompt_id: &str, options: &[ButtonOption]) -> Option<usize>;
    async fn on_key_down(&self, key: KeyCode, modifiers: KeyModifiers) -> bool;
    async fn on_key_up(&self, key: KeyCode, modifiers: KeyModifiers) -> bool;
    async fn on_gamepad_button(&self, button: GamepadButton, state: ButtonState) -> bool;
}
```

### ConfigValidator Trait
```rust
#[async_trait]
pub trait ConfigValidator: Send + Sync {
    async fn validate(&self, value: &serde_json::Value) -> Result<()>;
    async fn apply_defaults(&self, value: &mut serde_json::Value);
}
```

### ModApi Methods
```rust
impl ModApi {
    pub fn register_asset_override(&self, target: String, source: PathBuf, asset_type: AssetType, condition: Option<String>, priority: i32);
    pub fn register_ui_patch(&self, patch: UiPatch);
    pub fn register_input_hook(&self, hook: Box<dyn InputHook + Send + Sync>);
    pub fn register_runtime_hook(&self, hook: Box<dyn RuntimeHook + Send + Sync>);
    pub fn register_config_validator(&self, key: String, validator: Box<dyn ConfigValidator + Send + Sync>);
    pub fn log(&self, level: log::Level, message: &str);
}
```

---

## Building Mod Scripts

Mod scripts are Rust code that gets compiled dynamically. The mod loader expects:

1. A `create_mod` function exported as `#[no_mangle] pub extern "C" fn create_mod(...)`
2. Entry point functions for each entry point type

The scripts in `scripts/` are examples - in a real mod, you would compile them as a dynamic library (.so/.dll) and the mod loader would load them.

For development, you can also implement everything directly in `scripts/mod.rs` as shown in the examples.

---

## Game Integration Notes

The cheats mod demonstrates integration with the game's `Stats` struct from `crate::inventory`:

- `stats.sanity()` - Current health (0-100, up to 1,000,000 with cheats)
- `stats.will()` - Current will/ammo (0-100)
- `stats.god` - God mode flag
- `stats.notarget` - No-target flag
- `stats.owned` - Array of weapon ownership (10 weapons)
- `stats.selected` - Currently selected weapon
- `stats.set_health(value)` - Set health directly
- `stats.grant_weapons()` - Unlock all weapons
- `stats.spend_will(cost)` - Spend will (returns false if infinite)

Weapon indices:
- 0: Vorpal Blade (Knife)
- 1: Cards
- 2: Croquet Mallet
- 3: Jackbomb
- 4: Ice Wand
- 5: Jacks
- 6: Demon Dice (max 3 copies)
- 7: Jabberwock's Eye Staff
- 8: Blunderbuss
- 9: Deadtime Watch

---

## License

These example mods are provided under the MIT license. Feel free to use them as a starting point for your own mods.