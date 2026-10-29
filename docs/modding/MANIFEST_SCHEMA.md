# Mod Manifest JSON Schema Documentation

Complete reference for the Looking Glass mod manifest schema (`mod.schema.json`).

---

## Schema Overview

- **Schema Version**: Draft 7
- **Current Manifest Version**: 2
- **File Names**: `mod.json` or `mod.toml`
- **Validation**: Automatic on mod load

---

## Root Object

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Looking Glass Mod Manifest",
  "type": "object",
  "required": ["id", "name", "version", "manifest_version", "game_target"],
  "additionalProperties": false,
  "properties": { ... }
}
```

---

## Properties Reference

### `id` (Required)

```json
"id": {
  "type": "string",
  "pattern": "^[a-z0-9][a-z0-9_-]*$",
  "minLength": 1,
  "maxLength": 64,
  "description": "Unique mod identifier (lowercase, alphanumeric, underscore, hyphen)"
}
```

**Examples**: `"my_mod"`, `"button-glyphs"`, `"enhanced_graphics_v2"`

**Invalid**: `"MyMod"`, `"123mod"`, `"mod@name"`, `"mod name"`

---

### `name` (Required)

```json
"name": {
  "type": "string",
  "minLength": 1,
  "maxLength": 128,
  "description": "Human-readable mod name"
}
```

**Examples**: `"Button Glyphs"`, `"HD Textures"`, `"My Awesome Mod"`

---

### `version` (Required)

```json
"version": {
  "type": "string",
  "pattern": "^\\d+\\.\\d+\\.\\d+(?:-[0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*)?(?:\\+[0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*)?$",
  "description": "Semantic version string"
}
```

**SemVer Format**: `MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]`

**Examples**:
- `"1.0.0"`
- `"2.1.0-beta.1"`
- `"1.0.0+build.123"`
- `"0.9.0-alpha"`

---

### `manifest_version` (Required)

```json
"manifest_version": {
  "type": "integer",
  "const": 2,
  "description": "Manifest schema version"
}
```

**Must be**: `2`

---

### `author` (Optional)

```json
"author": {
  "type": "string",
  "maxLength": 128,
  "description": "Mod author name"
}
```

---

### `description` (Optional)

```json
"description": {
  "type": "string",
  "maxLength": 4096,
  "description": "Mod description"
}
```

---

### `homepage` (Optional)

```json
"homepage": {
  "type": "string",
  "format": "uri",
  "description": "Mod homepage URL"
}
```

**Example**: `"https://github.com/user/my-mod"`

---

### `repository` (Optional)

```json
"repository": {
  "type": "string",
  "format": "uri",
  "description": "Mod repository URL"
}
```

**Example**: `"https://github.com/user/my-mod.git"`

---

### `license` (Optional)

```json
"license": {
  "type": "string",
  "maxLength": 64,
  "description": "SPDX license identifier"
}
```

**Common Values**: `"MIT"`, `"Apache-2.0"`, `"GPL-3.0-only"`, `"BSD-3-Clause"`, `"Unlicense"`

**Full List**: https://spdx.org/licenses/

---

### `game_target` (Required)

```json
"game_target": {
  "type": "string",
  "enum": ["Alice", "AliceDemo"],
  "description": "Target game"
}
```

**Values**:
- `"Alice"` - Full game
- `"AliceDemo"` - Demo version

---

### `min_game_version` (Optional)

```json
"min_game_version": {
  "type": "string",
  "pattern": "^\\d+\\.\\d+\\.\\d+.*$",
  "description": "Minimum game version (semver)"
}
```

**Examples**: `"0.30.0"`, `"1.0.0"`, `"0.32.0-preview.1"`

---

### `max_game_version` (Optional)

```json
"max_game_version": {
  "type": "string",
  "pattern": "^\\d+\\.\\d+\\.\\d+.*$",
  "description": "Maximum game version (semver)"
}
```

**Examples**: `"1.0.0"`, `"0.35.0"`

---

### `dependencies` (Optional)

```json
"dependencies": {
  "type": "array",
  "items": {
    "type": "object",
    "required": ["id", "version"],
    "properties": {
      "id": {
        "type": "string",
        "pattern": "^[a-z0-9][a-z0-9_-]*$"
      },
      "version": {
        "type": "string",
        "description": "Semver version requirement"
      },
      "optional": {
        "type": "boolean",
        "default": false
      }
    }
  }
}
```

**Dependency Object Fields**:

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Dependency mod ID (same format as root `id`) |
| `version` | Yes | Semver requirement (e.g., `">=1.0.0"`, `"^2.0.0"`, `"1.0.0"`) |
| `optional` | No | If `true`, mod loads even if dependency missing (default: `false`) |

**Version Requirement Examples**:
- `">=1.0.0"` - Version 1.0.0 or higher
- `"^2.0.0"` - Compatible with 2.0.0 (>=2.0.0 <3.0.0)
- `"~1.2.3"` - Patch compatible (>=1.2.3 <1.3.0)
- `"1.0.0"` - Exactly 1.0.0
- `">1.0.0 <2.0.0"` - Range

**Example**:
```json
"dependencies": [
  { "id": "core", "version": ">=1.0.0", "optional": false },
  { "id": "optional_feature", "version": "^1.0.0", "optional": true }
]
```

---

### `load_order` (Optional)

```json
"load_order": {
  "type": "object",
  "properties": {
    "before": {
      "type": "array",
      "items": { "type": "string" }
    },
    "after": {
      "type": "array",
      "items": { "type": "string" }
    }
  }
}
```

**Fields**:
- `before` - Array of mod IDs this mod should load before
- `after` - Array of mod IDs this mod should load after

**Example**:
```json
"load_order": {
  "before": ["ui_overhaul", "graphics_mod"],
  "after": ["core", "base_mod"]
}
```

**Resolution**: Combined with dependencies for topological sort. Circular dependencies cause load failure.

---

### `entry_points` (Optional)

```json
"entry_points": {
  "type": "object",
  "properties": {
    "asset_override": {
      "type": "string",
      "description": "Path to asset override module"
    },
    "config_inject": {
      "type": "string",
      "description": "Path to config injection module"
    },
    "runtime_hook": {
      "type": "string",
      "description": "Path to runtime hook module"
    },
    "ui_patch": {
      "type": "string",
      "description": "Path to UI patch module"
    },
    "input_hook": {
      "type": "string",
      "description": "Path to input hook module"
    }
  }
}
```

**Fields**: Paths relative to mod root directory. Each implements a specific trait.

| Entry Point | Trait | Use Case |
|-------------|-------|----------|
| `asset_override` | `AssetOverride` | Dynamic asset overrides |
| `config_inject` | Config registration | Validators, default values |
| `runtime_hook` | `RuntimeHook` | Frame, menu, level events |
| `ui_patch` | `UiPatch` | UI modifications |
| `input_hook` | `InputHook` | Input interception |

**Example**:
```json
"entry_points": {
  "asset_override": "scripts/assets.rs",
  "config_inject": "scripts/config.rs",
  "runtime_hook": "scripts/runtime.rs",
  "ui_patch": "scripts/ui_patch.rs",
  "input_hook": "scripts/input.rs"
}
```

---

### `config_schema` (Optional)

```json
"config_schema": {
  "type": "object",
  "description": "JSON Schema for mod configuration"
}
```

**Format**: JSON Schema Draft 7 object. Validates mod configuration under `mods.<mod_id>`.

**Example**:
```json
"config_schema": {
  "type": "object",
  "properties": {
    "glyph_style": {
      "type": "string",
      "enum": ["playstation", "xbox", "switch", "generic"],
      "default": "playstation"
    },
    "scale": {
      "type": "number",
      "minimum": 0.5,
      "maximum": 2.0,
      "default": 1.0
    },
    "enabled": {
      "type": "boolean",
      "default": true
    }
  },
  "additionalProperties": false
}
```

---

### `default_config` (Optional)

```json
"default_config": {
  "type": "object",
  "description": "Default configuration values"
}
```

**Format**: JSON object with default values for config keys. Merged with user config (user values take precedence).

**Example**:
```json
"default_config": {
  "glyph_style": "playstation",
  "scale": 1.0,
  "enabled": true
}
```

---

### `assets` (Optional)

```json
"assets": {
  "type": "array",
  "items": {
    "type": "object",
    "required": ["source", "target", "type"],
    "properties": {
      "source": {
        "type": "string",
        "description": "Source path within mod archive"
      },
      "target": {
        "type": "string",
        "description": "Target path in game assets"
      },
      "type": {
        "type": "string",
        "enum": ["Texture", "Model", "Sound", "Font", "Ui", "Other"]
      },
      "condition": {
        "type": "string",
        "description": "Optional condition expression"
      }
    }
  }
}
```

**Asset Object Fields**:

| Field | Required | Description |
|-------|----------|-------------|
| `source` | Yes | Path to asset file within mod (relative to mod root) |
| `target` | Yes | Target path in game's asset namespace |
| `type` | Yes | Asset type enum |
| `condition` | No | Runtime condition expression |

**Asset Types**:

| Type | MIME Type | Typical Extensions |
|------|-----------|-------------------|
| `Texture` | `image/png` | `.png`, `.jpg`, `.tga` |
| `Model` | `model/gltf-binary` | `.glb`, `.gltf` |
| `Sound` | `audio/ogg` | `.ogg`, `.wav` |
| `Font` | `font/ttf` | `.ttf`, `.otf` |
| `Ui` | `application/json` | `.json` |
| `Other` | `application/octet-stream` | Any |

**Condition Expression Examples**:
- `"config.glyph_style == 'xbox'"`
- `"config.high_quality == true"`
- `"game.level != 'menu'"`

**Example**:
```json
"assets": [
  {
    "source": "assets/textures/wall_hd.png",
    "target": "textures/wall.png",
    "type": "Texture"
  },
  {
    "source": "glyphs/xbox/a.png",
    "target": "ui/glyphs/cross.png",
    "type": "Texture",
    "condition": "config.glyph_style == 'xbox'"
  }
]
```

---

### `tags` (Optional)

```json
"tags": {
  "type": "array",
  "items": {
    "type": "string",
    "pattern": "^[a-z0-9][a-z0-9_-]*$"
  }
}
```

**Format**: Lowercase, alphanumeric, underscore, hyphen (same as `id` format).

**Examples**: `["ui", "graphics", "accessibility", "controller", "texture"]`

---

## Complete Example (JSON)

```json
{
  "id": "button_glyphs",
  "name": "Button Glyphs",
  "version": "1.2.0",
  "manifest_version": 2,
  "author": "Looking Glass Team",
  "description": "Replaces keyboard button prompts with controller glyphs",
  "homepage": "https://github.com/looking-glass/button-glyphs-mod",
  "repository": "https://github.com/looking-glass/button-glyphs-mod",
  "license": "MIT",
  "game_target": "Alice",
  "min_game_version": "0.30.0",
  "dependencies": [
    { "id": "core", "version": ">=1.0.0", "optional": false }
  ],
  "load_order": {
    "after": ["core"]
  },
  "entry_points": {
    "asset_override": "scripts/assets.rs",
    "config_inject": "scripts/config.rs",
    "input_hook": "scripts/input.rs",
    "runtime_hook": "scripts/runtime.rs",
    "ui_patch": "scripts/ui_patch.rs"
  },
  "config_schema": {
    "type": "object",
    "properties": {
      "glyph_style": {
        "type": "string",
        "enum": ["playstation", "xbox", "switch", "generic"],
        "default": "playstation"
      },
      "show_keyboard_fallback": { "type": "boolean", "default": true },
      "scale": { "type": "number", "minimum": 0.5, "maximum": 2.0, "default": 1.0 }
    },
    "additionalProperties": false
  },
  "default_config": {
    "glyph_style": "playstation",
    "show_keyboard_fallback": true,
    "scale": 1.0
  },
  "assets": [
    { "source": "glyphs/playstation/cross.png", "target": "ui/glyphs/cross.png", "type": "Texture" },
    { "source": "glyphs/xbox/a.png", "target": "ui/glyphs/cross.png", "type": "Texture", "condition": "config.glyph_style == 'xbox'" }
  ],
  "tags": ["ui", "accessibility", "controller", "glyphs"]
}
```

---

## Validation Rules

1. **Required fields** must be present
2. **`id`** must match pattern and be unique
3. **`version`** must be valid SemVer
4. **`manifest_version`** must be `2`
5. **`game_target`** must be `Alice` or `AliceDemo`
6. **Dependencies** cannot reference self
7. **Version requirements** must be valid semver ranges
8. **`min_game_version`** must be <= `max_game_version` (if both set)
9. **Assets** must have valid source/target/type
10. **Conditions** are validated at runtime (not schema level)
11. **No additional properties** allowed at root level

---

## Migration from v1

Version 1 manifests are automatically migrated to v2 on load. Key changes:

| v1 Field | v2 Equivalent |
|----------|---------------|
| `mod_id` | `id` |
| `mod_name` | `name` |
| `mod_version` | `version` |
| `target_game` | `game_target` |
| `requires` | `dependencies` |
| `replaces` | `assets` (with type) |
| `config` | `config_schema` + `default_config` |

---

## Schema File Location

The schema is embedded in the `looking-glass-mod` crate at:
- Source: `crates/looking-glass-mod/mod.schema.json`
- Compiled: Included in binary via `include_str!`

---

## Validation Tools

```bash
# Validate manifest file
looking-glass-launcher mod validate /path/to/mod

# Validate with JSON Schema directly
python -m jsonschema -i mod.json mod.schema.json
```

---

## See Also

- [MODDING_GUIDE.md](MODDING_GUIDE.md) - Complete modding guide
- [API_REFERENCE.md](API_REFERENCE.md) - Rust API documentation
- [EXAMPLE_MODS/](EXAMPLE_MODS/) - Working examples