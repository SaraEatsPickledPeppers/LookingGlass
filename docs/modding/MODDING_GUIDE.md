# Looking Glass Modding Guide

Complete guide to creating, packaging, and distributing mods for Looking Glass.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Mod Manifest (`mod.json`/`mod.toml`)](#mod-manifest-modjsonmodtoml)
3. [Asset Override System](#asset-override-system)
4. [Config Injection](#config-injection)
5. [Runtime Hooks](#runtime-hooks)
6. [Creating a Mod from Scratch](#creating-a-mod-from-scratch)
7. [Button Glyphs Mod Example](#button-glyphs-mod-example)
8. [Packaging and Distribution](#packaging-and-distribution)
9. [Debugging Mods](#debugging-mods)

---

## Introduction

Looking Glass supports a flexible mod system that allows users to customize the game experience without modifying game files directly. Mods are loaded at runtime through an overlay filesystem and can:

- **Override assets** (textures, models, sounds, fonts, UI)
- **Inject configuration** (add new config options, modify defaults)
- **Hook into runtime events** (input, frame updates, menu open/close, level load/unload)
- **Patch UI** (add/remove/modify widgets)
- **Intercept input** (button prompts, keyboard, gamepad)

### Mod Directory Structure

```
mods/
└── Alice/                    # Game target directory
    └── my_mod/              # Mod ID directory
        ├── mod.toml         # Mod manifest (required)
        ├── config.json      # Mod configuration (optional)
        ├── assets/          # Asset overrides
        │   ├── textures/
        │   ├── models/
        │   ├── sounds/
        │   ├── fonts/
        │   └── ui/
        ├── scripts/         # Runtime hook scripts (optional)
        │   ├── hooks.rs
        │   ├── input.rs
        │   └── ui_patch.rs
        └── README.md        # Mod documentation (optional)
```

---

## Mod Manifest (`mod.json`/`mod.toml`)

The manifest is the entry point for every mod. It can be written in either JSON or TOML format (TOML recommended).

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique identifier (lowercase, alphanumeric, `_`, `-`, max 64 chars) |
| `name` | string | Human-readable name (max 128 chars) |
| `version` | string | Semantic version (e.g., `1.0.0`, `2.1.0-beta.1`) |
| `manifest_version` | integer | Must be `2` (current schema version) |
| `game_target` | enum | `"Alice"` or `"AliceDemo"` |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `author` | string | Author name (max 128 chars) |
| `description` | string | Mod description (max 4096 chars) |
| `homepage` | string (URI) | Mod homepage URL |
| `repository` | string (URI) | Source repository URL |
| `license` | string | SPDX license identifier (e.g., `MIT`, `Apache-2.0`) |
| `min_game_version` | string | Minimum game version (semver) |
| `max_game_version` | string | Maximum game version (semver) |
| `dependencies` | array | List of mod dependencies |
| `load_order` | object | Load ordering constraints (`before`, `after`) |
| `entry_points` | object | Paths to hook modules |
| `config_schema` | object | JSON Schema for mod configuration |
| `default_config` | object | Default configuration values |
| `assets` | array | Asset mappings for override system |
| `tags` | array | Searchable tags (lowercase, alphanumeric, `_`, `-`) |

### Complete Manifest Example (TOML)

```toml
id = "button_glyphs"
name = "Button Glyphs"
version = "1.2.0"
manifest_version = 2
author = "Looking Glass Team"
description = "Replaces keyboard button prompts with controller glyphs (PlayStation, Xbox, Switch, Generic)"
homepage = "https://github.com/looking-glass/button-glyphs-mod"
repository = "https://github.com/looking-glass/button-glyphs-mod"
license = "MIT"
game_target = "Alice"
min_game_version = "0.30.0"

dependencies = [
  { id = "core", version = ">=1.0.0", optional = false }
]

load_order = { after = ["core"] }

entry_points = {
  asset_override = "scripts/assets.rs",
  config_inject = "scripts/config.rs",
  input_hook = "scripts/input.rs",
  runtime_hook = "scripts/runtime.rs",
  ui_patch = "scripts/ui_patch.rs"
}

config_schema = {
  type = "object",
  properties = {
    glyph_style = {
      type = "string",
      enum = ["playstation", "xbox", "switch", "generic"],
      default = "playstation"
    },
    show_keyboard_fallback = { type = "boolean", default = true },
    scale = { type = "number", minimum = 0.5, maximum = 2.0, default = 1.0 }
  }
}

default_config = {
  glyph_style = "playstation",
  show_keyboard_fallback = true,
  scale = 1.0
}

assets = [
  { source = "glyphs/playstation/cross.png", target = "ui/glyphs/cross.png", type = "Texture" },
  { source = "glyphs/playstation/circle.png", target = "ui/glyphs/circle.png", type = "Texture" },
  { source = "glyphs/playstation/square.png", target = "ui/glyphs/square.png", type = "Texture" },
  { source = "glyphs/playstation/triangle.png", target = "ui/glyphs/triangle.png", type = "Texture" },
  { source = "glyphs/xbox/a.png", target = "ui/glyphs/cross.png", type = "Texture", condition = "config.glyph_style == 'xbox'" },
  { source = "glyphs/xbox/b.png", target = "ui/glyphs/circle.png", type = "Texture", condition = "config.glyph_style == 'xbox'" },
  { source = "glyphs/xbox/x.png", target = "ui/glyphs/square.png", type = "Texture", condition = "config.glyph_style == 'xbox'" },
  { source = "glyphs/xbox/y.png", target = "ui/glyphs/triangle.png", type = "Texture", condition = "config.glyph_style == 'xbox'" },
  { source = "glyphs/switch/a.png", target = "ui/glyphs/cross.png", type = "Texture", condition = "config.glyph_style == 'switch'" },
  { source = "glyphs/switch/b.png", target = "ui/glyphs/circle.png", type = "Texture", condition = "config.glyph_style == 'switch'" },
  { source = "glyphs/switch/x.png", target = "ui/glyphs/square.png", type = "Texture", condition = "config.glyph_style == 'switch'" },
  { source = "glyphs/switch/y.png", target = "ui/glyphs/triangle.png", type = "Texture", condition = "config.glyph_style == 'switch'" },
  { source = "glyphs/generic/a.png", target = "ui/glyphs/cross.png", type = "Texture", condition = "config.glyph_style == 'generic'" },
  { source = "glyphs/generic/b.png", target = "ui/glyphs/circle.png", type = "Texture", condition = "config.glyph_style == 'generic'" },
  { source = "glyphs/generic/x.png", target = "ui/glyphs/square.png", type = "Texture", condition = "config.glyph_style == 'generic'" },
  { source = "glyphs/generic/y.png", target = "ui/glyphs/triangle.png", type = "Texture", condition = "config.glyph_style == 'generic'" }
]

tags = ["ui", "accessibility", "controller", "glyphs"]
```

### Field Details

#### `id` - Unique Identifier

- Must match regex: `^[a-z0-9][a-z0-9_-]*$`
- Lowercase only
- Alphanumeric, underscore, hyphen
- 1-64 characters
- Used for dependencies, load ordering, and mod directory name

#### `version` - Semantic Versioning

Follows [SemVer](https://semver.org/) specification:
- `1.0.0` - Major.Minor.Patch
- `1.0.0-alpha.1` - Pre-release
- `1.0.0+build.123` - Build metadata

Used for dependency resolution and update checking.

#### `dependencies` - Mod Dependencies

```toml
dependencies = [
  { id = "core", version = ">=1.0.0", optional = false },
  { id = "optional_mod", version = "^2.0.0", optional = true }
]
```

- `id` - Dependency mod ID
- `version` - Semver requirement string
- `optional` - If `true`, mod loads even if dependency missing

#### `load_order` - Load Ordering

```toml
load_order = {
  before = ["other_mod"]    # This mod loads before other_mod
  after = ["base_mod"]      # This mod loads after base_mod
}
```

Combined with dependencies for topological sort. Circular dependencies are rejected.

#### `entry_points` - Hook Module Paths

Paths are relative to mod root directory. Each entry point is a Rust module that implements the corresponding trait:

```toml
entry_points = {
  asset_override = "scripts/assets.rs",      # implements AssetOverride
  config_inject = "scripts/config.rs",       # provides config schema/defaults
  runtime_hook = "scripts/runtime.rs",       # implements RuntimeHook
  ui_patch = "scripts/ui_patch.rs",          # provides UiPatch definitions
  input_hook = "scripts/input.rs"            # implements InputHook
}
```

#### `config_schema` / `default_config` - Configuration

- `config_schema`: JSON Schema (Draft 7) for validation
- `default_config`: Default values merged with user config

Configuration layers (highest to lowest priority):
1. CLI arguments (`--config.key=value`)
2. Project config (`private/config.toml`)
3. Global config (`~/.config/looking-glass/config.toml`)
4. Mod default_config
5. Built-in defaults

#### `assets` - Asset Mappings

```toml
assets = [
  { source = "textures/new_brick.png", target = "textures/brick.png", type = "Texture" },
  { source = "models/character.glb", target = "models/character.glb", type = "Model", condition = "config.high_quality == true" }
]
```

- `source` - Path within mod archive
- `target` - Target path in game asset namespace
- `type` - Asset type: `Texture`, `Model`, `Sound`, `Font`, `Ui`, `Other`
- `condition` - Optional expression (evaluated at runtime)

---

## Asset Override System

Looking Glass uses an **overlay filesystem** for asset overrides. When the game requests an asset, the mod system checks for overrides in load order (highest priority first).

### How It Works

1. **Registration**: Mods register asset overrides via `entry_points.asset_override` or manifest `assets` array
2. **Priority**: Overrides sorted by:
   - Explicit `priority` field (higher = first)
   - Load order (later mods override earlier)
   - Manifest declaration order
3. **Conditions**: Optional `condition` expression evaluated at runtime
4. **Resolution**: First matching override wins

### Asset Override Entry

```rust
pub struct AssetOverrideEntry {
    pub source: PathBuf,           // Path to replacement asset
    pub asset_type: AssetType,     // Texture, Model, Sound, Font, Ui, Other
    pub condition: Option<String>, // Runtime condition expression
    pub priority: i32,             // Override priority (default: 0)
}
```

### Asset Types

| Type | MIME Type | Description |
|------|-----------|-------------|
| `Texture` | `image/png` | PNG textures (recommended) |
| `Model` | `model/gltf-binary` | GLTF/GLB models |
| `Sound` | `audio/ogg` | OGG audio files |
| `Font` | `font/ttf` | TrueType fonts |
| `Ui` | `application/json` | UI definitions |
| `Other` | `application/octet-stream` | Any other file |

### Condition Expressions

Simple conditions evaluated at runtime:

```toml
# Config-based conditions
condition = "config.glyph_style == 'xbox'"
condition = "config.high_quality == true"

# Game state conditions (future)
condition = "game.level == 'menu'"
condition = "input.gamepad_connected == true"
```

### Creating Asset Overrides

1. Place replacement assets in mod's `assets/` directory
2. Declare in manifest `assets` array OR register via `AssetOverride` trait
3. Set appropriate `asset_type` for MIME detection
4. Use `condition` for config-dependent overrides

---

## Config Injection

Mods can inject configuration options that integrate with Looking Glass's layered config system.

### Configuration Layers

```
┌─────────────────────────────────────┐
│ 1. CLI Arguments (--config.key=val) │  ← Highest priority
├─────────────────────────────────────┤
│ 2. Project Config                   │  (private/config.toml)
├─────────────────────────────────────┤
│ 3. Global Config                    │  (~/.config/looking-glass/config.toml)
├─────────────────────────────────────┤
│ 4. Mod Default Config               │  (manifest.default_config)
├─────────────────────────────────────┤
│ 5. Built-in Defaults                │  ← Lowest priority
└─────────────────────────────────────┘
```

### Mod Config Structure

Mod configs are namespaced under `mods.<mod_id>`:

```toml
# In global config: ~/.config/looking-glass/config.toml
[mods.button_glyphs]
glyph_style = "xbox"
show_keyboard_fallback = false
scale = 1.2
```

### Defining Config Schema

In `mod.toml`:

```toml
config_schema = {
  type = "object",
  properties = {
    glyph_style = {
      type = "string",
      enum = ["playstation", "xbox", "switch", "generic"],
      default = "playstation",
      description = "Controller glyph style"
    },
    show_keyboard_fallback = {
      type = "boolean",
      default = true,
      description = "Show keyboard key names as fallback"
    },
    scale = {
      type = "number",
      minimum = 0.5,
      maximum = 2.0,
      default = 1.0,
      description = "Glyph display scale"
    }
  },
  additionalProperties = false
}

default_config = {
  glyph_style = "playstation",
  show_keyboard_fallback = true,
  scale = 1.0
}
```

### Config Validation

Mods can register custom validators via `ConfigValidator` trait:

```rust
pub struct GlyphStyleValidator;

#[async_trait]
impl ConfigValidator for GlyphStyleValidator {
    async fn validate(&self, value: &serde_json::Value) -> Result<()> {
        let style = value.as_str().ok_or_else(|| anyhow!("glyph_style must be a string"))?;
        match style {
            "playstation" | "xbox" | "switch" | "generic" => Ok(()),
            _ => Err(anyhow!("Invalid glyph_style: {}", style)),
        }
    }

    async fn apply_defaults(&self, value: &mut serde_json::Value) {
        if value.get("glyph_style").is_none() {
            value["glyph_style"] = serde_json::json!("playstation");
        }
    }
}
```

Register in `entry_points.config_inject` module:

```rust
pub fn register_validators(api: &ModApi) {
    api.register_config_validator("glyph_style".to_string(), Box::new(GlyphStyleValidator));
}
```

---

## Runtime Hooks

Five hook types allow mods to interact with the game at runtime.

### 1. InputHook

Intercept and handle input events.

```rust
#[async_trait]
pub trait InputHook: Send + Sync {
    /// Called when a button prompt is shown (e.g., "Press X to jump")
    /// Return Some(index) to auto-select, None to show prompt normally
    async fn on_button_prompt(&self, prompt_id: &str, options: &[ButtonOption]) -> Option<usize>;

    /// Called on key down. Return true to consume event.
    async fn on_key_down(&self, key: KeyCode, modifiers: KeyModifiers) -> bool;

    /// Called on key up. Return true to consume event.
    async fn on_key_up(&self, key: KeyCode, modifiers: KeyModifiers) -> bool;

    /// Called on gamepad button. Return true to consume event.
    async fn on_gamepad_button(&self, button: GamepadButton, state: ButtonState) -> bool;
}
```

**ButtonOption**:
```rust
pub struct ButtonOption {
    pub id: String,           // Internal action ID
    pub label: String,        // Display label
    pub description: Option<String>,
    pub icon: Option<String>, // Glyph texture path
}
```

**Use cases**: Auto-accept prompts, remap controls, log inputs, accessibility features.

### 2. AssetOverride

Dynamic asset overrides (beyond manifest declarations).

```rust
#[async_trait]
pub trait AssetOverride: Send + Sync {
    /// Try to provide override for target asset
    async fn try_override(&self, target: &str) -> Option<AssetOverrideResult>;

    /// List all overrides this module provides
    async fn list_overrides(&self) -> Vec<AssetMapping>;
}

pub struct AssetOverrideResult {
    pub data: Vec<u8>,      // Raw asset data
    pub mime_type: String,  // MIME type
}
```

**Use cases**: Procedural assets, runtime-generated textures, conditional overrides based on game state.

### 3. UiPatch

Modify UI at runtime using JSON Patch-like operations.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPatch {
    pub target: String,           // UI screen ID (e.g., "main_menu")
    pub operations: Vec<UiPatchOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum UiPatchOp {
    AddWidget { parent: String, widget: WidgetDef },
    RemoveWidget { id: String },
    ModifyWidget { id: String, changes: HashMap<String, serde_json::Value> },
    ReplaceWidget { id: String, widget: WidgetDef },
}

pub struct WidgetDef {
    pub id: String,
    pub widget_type: String,              // "Button", "Label", "Panel", etc.
    pub properties: HashMap<String, serde_json::Value>,
    pub children: Vec<WidgetDef>,
}
```

**Example**: Add a "Mods" button to main menu.

```json
{
  "target": "main_menu",
  "operations": [
    {
      "op": "add_widget",
      "parent": "main_menu_container",
      "widget": {
        "id": "mods_button",
        "widget_type": "Button",
        "properties": {
          "text": "Mods",
          "on_click": "open_mods_menu"
        },
        "children": []
      }
    }
  ]
}
```

### 4. RuntimeHook

Game lifecycle events.

```rust
#[async_trait]
pub trait RuntimeHook: Send + Sync {
    /// Called every frame
    async fn on_frame(&self, delta_time: f32);

    /// Menu opened
    async fn on_menu_open(&self, menu_id: &str);

    /// Menu closed
    async fn on_menu_close(&self, menu_id: &str);

    /// Level loaded
    async fn on_level_load(&self, level_id: &str);

    /// Level unloaded
    async fn on_level_unload(&self, level_id: &str);
}
```

**Use cases**: Background tasks, per-level logic, menu state tracking, performance monitoring.

### 5. ConfigValidator

Validate and apply defaults to config values (see [Config Injection](#config-injection)).

---

## Creating a Mod from Scratch

### Step 1: Create Mod Directory

```bash
mkdir -p ~/.local/share/looking-glass/mods/Alice/my_first_mod
cd ~/.local/share/looking-glass/mods/Alice/my_first_mod
```

### Step 2: Create Manifest (`mod.toml`)

```toml
id = "my_first_mod"
name = "My First Mod"
version = "1.0.0"
manifest_version = 2
author = "Your Name"
description = "A simple example mod"
game_target = "Alice"
```

### Step 3: Add Asset Override (Optional)

```bash
mkdir -p assets/textures
# Copy your replacement texture
cp /path/to/new_texture.png assets/textures/wall.png
```

Update manifest:
```toml
assets = [
  { source = "assets/textures/wall.png", target = "textures/wall.png", type = "Texture" }
]
```

### Step 4: Add Config (Optional)

```toml
config_schema = {
  type = "object",
  properties = {
    enabled = { type = "boolean", default = true }
  }
}

default_config = { enabled = true }
```

### Step 5: Add Runtime Hook (Optional)

Create `scripts/hooks.rs`:
```rust
use looking_glass_mod::loader::{RuntimeHook, ModContext, Result};

pub struct MyHook;

#[async_trait]
impl RuntimeHook for MyHook {
    async fn on_frame(&self, delta_time: f32) {
        // Your per-frame logic
    }

    async fn on_level_load(&self, level_id: &str) {
        log::info!("Level loaded: {}", level_id);
    }
}
```

Update manifest:
```toml
entry_points = { runtime_hook = "scripts/hooks.rs" }
```

### Step 6: Test the Mod

```bash
# Launch game with mod enabled
looking-glass-launcher --mod my_first_mod
```

---

## Button Glyphs Mod Example

A complete example mod that replaces keyboard button prompts with controller glyphs.

### Features

- **4 glyph styles**: PlayStation, Xbox, Switch, Generic
- **Configurable**: Style, fallback, scale via config
- **Conditional assets**: Different textures per style
- **Input hook**: Intercepts button prompts to inject glyphs
- **UI patch**: Adds glyph style selector to settings menu

### Directory Structure

```
button_glyphs/
├── mod.toml
├── config.json          # User config (created on first run)
├── assets/
│   └── glyphs/
│       ├── playstation/
│       │   ├── cross.png
│       │   ├── circle.png
│       │   ├── square.png
│       │   └── triangle.png
│       ├── xbox/
│       │   ├── a.png
│       │   ├── b.png
│       │   ├── x.png
│       │   └── y.png
│       ├── switch/
│       │   ├── a.png
│       │   ├── b.png
│       │   ├── x.png
│       │   └── y.png
│       └── generic/
│           ├── a.png
│           ├── b.png
│           ├── x.png
│           └── y.png
└── scripts/
    ├── assets.rs        # AssetOverride implementation
    ├── config.rs        # ConfigValidator registration
    ├── input.rs         # InputHook for button prompts
    ├── runtime.rs       # RuntimeHook for config changes
    └── ui_patch.rs      # UiPatch for settings menu
```

### Key Implementation Details

#### `scripts/input.rs` - InputHook

```rust
use looking_glass_mod::loader::{InputHook, ButtonOption, ModApi};
use std::sync::Arc;

pub struct GlyphInputHook {
    api: Arc<ModApi>,
    current_style: String,
}

#[async_trait]
impl InputHook for GlyphInputHook {
    async fn on_button_prompt(&self, prompt_id: &str, options: &[ButtonOption]) -> Option<usize> {
        // Don't auto-select, just enhance with glyphs
        None
    }

    async fn on_key_down(&self, _key: KeyCode, _mods: KeyModifiers) -> bool { false }
    async fn on_key_up(&self, _key: KeyCode, _mods: KeyModifiers) -> bool { false }
    async fn on_gamepad_button(&self, _btn: GamepadButton, _state: ButtonState) -> bool { false }
}

// Called by game to get glyph for button
impl GlyphInputHook {
    pub fn get_glyph_for_action(&self, action_id: &str) -> Option<String> {
        let style = &self.current_style;
        let glyph_map = match style.as_str() {
            "playstation" => ["cross", "circle", "square", "triangle"],
            "xbox" => ["a", "b", "x", "y"],
            "switch" => ["a", "b", "x", "y"],
            "generic" => ["a", "b", "x", "y"],
            _ => return None,
        };
        
        // Map action to glyph
        let glyph = match action_id {
            "jump" | "accept" => glyph_map[0],      // Cross/A
            "cancel" | "back" => glyph_map[1],      // Circle/B
            "attack" => glyph_map[2],               // Square/X
            "special" => glyph_map[3],              // Triangle/Y
            _ => return None,
        };
        
        Some(format!("ui/glyphs/{}.png", glyph))
    }
}
```

#### `scripts/assets.rs` - AssetOverride

```rust
use looking_glass_mod::loader::{AssetOverride, AssetOverrideResult, AssetMapping};
use std::path::PathBuf;

pub struct GlyphAssetOverride {
    mod_dir: PathBuf,
    config: Arc<ConfigRegistry>,
}

#[async_trait]
impl AssetOverride for GlyphAssetOverride {
    async fn try_override(&self, target: &str) -> Option<AssetOverrideResult> {
        // Check if target is a glyph texture
        if !target.starts_with("ui/glyphs/") {
            return None;
        }

        // Get current glyph style from config
        let style = self.config.get_value("mods.button_glyphs.glyph_style")
            .and_then(|v| v.as_str())
            .unwrap_or("playstation");

        // Map target to style-specific asset
        let glyph_name = target.strip_prefix("ui/glyphs/")?;
        let source_name = match (style, glyph_name) {
            ("playstation", "cross.png") => "cross.png",
            ("playstation", "circle.png") => "circle.png",
            // ... etc
            _ => return None,
        };

        let source_path = self.mod_dir.join("assets/glyphs").join(style).join(source_name);
        let data = std::fs::read(source_path).ok()?;
        
        Some(AssetOverrideResult {
            data,
            mime_type: "image/png".to_string(),
        })
    }

    async fn list_overrides(&self) -> Vec<AssetMapping> {
        // Return all possible glyph mappings
        vec![]
    }
}
```

---

## Packaging and Distribution

### Creating a Mod Archive

```bash
cd ~/.local/share/looking-glass/mods/Alice/my_mod
# Create zip archive
zip -r my_mod_v1.0.0.zip . -x "*.git*" "config.json"
```

Supported formats: `.zip`, `.tar.gz`, `.tgz`, `.tar`

### Archive Structure

```
my_mod_v1.0.0.zip
├── mod.toml           # Required at root
├── assets/
│   └── ...
├── scripts/
│   └── ...
└── README.md          # Optional
```

### Distribution Checklist

- [ ] Manifest validates against schema (`mod.schema.json`)
- [ ] All declared assets exist in archive
- [ ] Entry point scripts compile (if using Rust)
- [ ] Config schema matches default_config
- [ ] Dependencies listed correctly
- [ ] Version follows SemVer
- [ ] License specified
- [ ] README with installation instructions

### Publishing

1. **GitHub Releases**: Upload `.zip` to releases
2. **Mod Repository**: Submit to community mod index (future)
3. **Direct sharing**: Share `.zip` file directly

### Installation

Users can install via:
- **Launcher GUI**: Drag & drop `.zip` file
- **CLI**: `looking-glass-launcher mod install my_mod.zip`
- **Manual**: Extract to `~/.local/share/looking-glass/mods/Alice/`

---

## Debugging Mods

### Enable Debug Logging

```bash
# Environment variable
RUST_LOG=looking_glass_mod=debug looking-glash-launcher

# Or in config.toml
[launcher]
log_level = "debug"
```

### Common Issues

#### Mod Not Loading

1. Check manifest exists: `mod.toml` or `mod.json` in mod directory
2. Validate manifest: `looking-glass-launcher mod validate <mod_dir>`
3. Check logs for errors: `~/.local/share/looking-glass/logs/`

#### Asset Not Overriding

1. Verify `target` path matches game's asset path exactly
2. Check load order - another mod may override yours
3. Verify `condition` evaluates to true
4. Check asset checksum in logs

#### Config Not Applying

1. Verify config namespace: `mods.<mod_id>.<key>`
2. Check config schema validation errors in logs
3. Ensure `default_config` matches schema

#### Hook Not Called

1. Verify entry point path in manifest
2. Check script compiles without errors
3. Ensure trait implementation matches exactly

### Debugging Commands

```bash
# Validate manifest
looking-glass-launcher mod validate /path/to/mod

# List installed mods
looking-glass-launcher mod list

# Show mod info
looking-glass-launcher mod info <mod_id>

# Enable/disable mod
looking-glass-launcher mod enable <mod_id>
looking-glass-launcher mod disable <mod_id>

# Uninstall mod
looking-glass-launcher mod uninstall <mod_id>

# Check load order
looking-glash-launcher mod load-order
```

### Visual Debugging

For asset overrides, enable asset debug overlay:
```toml
# In config.toml
[mods]
debug_asset_overrides = true
```

This shows green borders on overridden assets in-game.

---

## API Reference

See [API_REFERENCE.md](API_REFERENCE.md) for complete Rust API documentation.

## CLI Reference

See [CLI_REFERENCE.md](CLI_REFERENCE.md) for launcher command reference.

## Example Mods

See [EXAMPLE_MODS/](EXAMPLE_MODS/) for complete working examples:

- [Simple Asset Override](EXAMPLE_MODS/simple-asset-override/) - Replace one texture
- [Config Injection](EXAMPLE_MODS/config-injection/) - Add config option
- [Input Hook](EXAMPLE_MODS/input-hook/) - Log button presses
- [UI Patch](EXAMPLE_MODS/ui-patch/) - Add menu item